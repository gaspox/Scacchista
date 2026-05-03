//! Minimal UCI event loop and state machine for Scacchista

use super::parser::{parse_uci_command, UciCommand};
use crate::board::{move_to_uci, parse_uci_move, Board};
use crate::search::search::{MATE, MATE_THRESHOLD};
use std::io::{self, BufRead, Write};
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use std::time::Instant;

use crate::uci::options::UciOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciState {
    Init,
    Ready,
    Thinking,
    Pondering,
}

pub struct UciEngine {
    state: UciState,
    board: Board,
    running: bool,
    options: UciOptions,
    thread_mgr: Option<crate::search::ThreadManager>,
    /// Flag indicating if an async search (go infinite) is currently active
    async_search_active: bool,
    /// Cancel flag for an active ponder timer thread
    ponder_timer_cancel: Option<Arc<AtomicBool>>,
    /// Last clock parameters from go command (used by ponderhit)
    last_wtime: Option<u64>,
    last_btime: Option<u64>,
    last_winc: Option<u64>,
    last_binc: Option<u64>,
    last_movetime: Option<u64>,
    last_movestogo: Option<u64>,
}

impl Default for UciEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UciEngine {
    pub fn new() -> Self {
        let opts = UciOptions::default();
        let tm = crate::search::ThreadManager::new(opts.threads as usize, 16);
        Self {
            state: UciState::Init,
            board: Board::new(),
            running: true,
            options: opts,
            thread_mgr: Some(tm),
            async_search_active: false,
            ponder_timer_cancel: None,
            last_wtime: None,
            last_btime: None,
            last_winc: None,
            last_binc: None,
            last_movetime: None,
            last_movestogo: None,
        }
    }

    pub fn handle_command(&mut self, cmd: UciCommand) -> Vec<String> {
        let mut res = Vec::new();
        match cmd {
            UciCommand::Uci => {
                res.push("id name Scacchista".to_string());
                res.push("id author Gaspox (AI co-author: Claude Code)".to_string());

                // Send UCI options
                res.push("option name Hash type spin default 16 min 1 max 4096".to_string());
                res.push("option name Threads type spin default 1 min 1 max 256".to_string());
                res.push("option name SyzygyPath type string default <empty>".to_string());
                res.push("option name UseExperienceBook type check default true".to_string());
                res.push(
                    "option name Style type combo default Normal var Normal var Tal var Petrosian"
                        .to_string(),
                );

                res.push("uciok".to_string());
                self.state = UciState::Ready;
            }
            UciCommand::IsReady => {
                res.push("readyok".to_string());
            }
            UciCommand::Position { fen, moves } => {
                // Create temporary board to validate all moves atomically
                let mut temp_board = Board::new();
                let fen_str = if let Some(f) = fen {
                    f
                } else {
                    // Default starting position FEN
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
                };

                // Set position from FEN
                if let Err(e) = temp_board.set_from_fen(&fen_str) {
                    res.push(format!("info string FEN parse error: {}", e));
                    // Don't update self.board if FEN is invalid
                    self.state = UciState::Ready;
                    return res;
                }

                // Apply all moves to temporary board first
                for move_str in &moves {
                    match parse_uci_move(&mut temp_board, move_str) {
                        Ok(mv) => {
                            let _undo = temp_board.make_move(mv);
                        }
                        Err(e) => {
                            res.push(format!("info string invalid move {}: {}", move_str, e));
                            // Don't update self.board if any move is invalid
                            self.state = UciState::Ready;
                            return res;
                        }
                    }
                }

                // All moves valid: commit the new position
                self.board = temp_board;
                self.state = UciState::Ready;
            }
            UciCommand::Go {
                wtime,
                btime,
                winc, // FIX Bug #4
                binc, // FIX Bug #4
                movetime,
                depth,
                nodes: _nodes,
                mate: _mate,
                movestogo: _movestogo,
                infinite,
                ponder: _ponder,
            } => {
                // Compute time budget
                let side_white = self.board.side == crate::board::Color::White;
                let time_alloc = crate::time::TimeManager::allocate_time(
                    &crate::search::params::TimeManagement::new(),
                    wtime,
                    btime,
                    winc,
                    binc,
                    movetime,
                    _movestogo.map(|x| x as u64),
                    side_white,
                    self.options.move_overhead_ms,
                );

                // Cancel any pending ponder timer before starting a new search
                if let Some(cancel) = self.ponder_timer_cancel.take() {
                    cancel.store(true, Ordering::Relaxed);
                }

                // Save clock parameters for potential ponderhit later
                self.last_wtime = wtime;
                self.last_btime = btime;
                self.last_winc = winc;
                self.last_binc = binc;
                self.last_movetime = movetime;
                self.last_movestogo = _movestogo.map(|x| x as u64);

                if infinite || _ponder {
                    // ASYNC MODE: go infinite / ponder - start search in background
                    let params = crate::search::SearchParams::new()
                        .max_depth(99)
                        .time_limit(0); // No time limit; wait for stop/ponderhit

                    if let Some(ref tm) = self.thread_mgr {
                        let job = crate::search::thread_mgr::SearchJob {
                            board: self.board.clone(),
                            params,
                        };
                        tm.start_async_search(job);
                        self.async_search_active = true;
                        self.state = if _ponder {
                            UciState::Pondering
                        } else {
                            UciState::Thinking
                        };
                        // No bestmove sent here - will be sent when stop/ponderhit arrives
                    } else {
                        res.push("info string no thread manager available".to_string());
                        res.push("bestmove 0000".to_string());
                        self.state = UciState::Ready;
                    }
                } else {
                    // SYNC MODE: go depth/movetime - traditional blocking search
                    // Build search params

                    // UCI semantics: depth is always a maximum depth limit.
                    // Time limits (wtime/btime/movetime) are additional constraints.
                    // When only depth is specified, search runs until depth is reached.
                    let max_search_depth = depth.map(|d| d.clamp(1, 99)).unwrap_or(99);

                    // If depth is specified WITHOUT time limits, use unlimited time
                    // (depth will control the search). Otherwise, use time allocation.
                    let effective_time = if depth.is_some()
                        && movetime.is_none()
                        && wtime.is_none()
                        && btime.is_none()
                    {
                        0 // 0 = no time limit, depth controls search
                    } else {
                        time_alloc
                    };

                    let params = crate::search::SearchParams::new()
                        .max_depth(max_search_depth)
                        .time_limit(effective_time);

                    // Submit job to persistent thread manager
                    if let Some(tm) = &self.thread_mgr {
                        let job = crate::search::thread_mgr::SearchJob {
                            board: self.board.clone(),
                            params,
                        };
                        let search_start = Instant::now();
                        let result = tm.submit_job(job);
                        let search_time_ms = search_start.elapsed().as_millis() as u64;

                        // Build UCI info line with full search data
                        let mut info_parts = vec![
                            format!("depth {}", result.completed_depth),
                            format!("seldepth {}", result.seldepth),
                        ];

                        // Score: cp or mate
                        if result.score >= MATE_THRESHOLD {
                            let mate_plies = MATE - result.score;
                            let mate_moves = (mate_plies + 1) / 2;
                            info_parts.push(format!("score mate {}", mate_moves));
                        } else if result.score <= -MATE_THRESHOLD {
                            let mate_plies = MATE + result.score;
                            let mate_moves = -(mate_plies / 2);
                            info_parts.push(format!("score mate {}", mate_moves));
                        } else {
                            info_parts.push(format!("score cp {}", result.score));
                        }

                        info_parts.push(format!("nodes {}", result.nodes));
                        if search_time_ms > 0 {
                            info_parts.push(format!(
                                "nps {}",
                                result.nodes * 1000 / search_time_ms
                            ));
                        }
                        info_parts.push(format!("time {}", search_time_ms));
                        info_parts.push(format!("hashfull {}", result.hashfull));

                        if !result.pv.is_empty() {
                            let pv_str = result
                                .pv
                                .iter()
                                .map(|&m| move_to_uci(m))
                                .collect::<Vec<_>>()
                                .join(" ");
                            info_parts.push(format!("pv {}", pv_str));
                        }

                        res.push(format!("info {}", info_parts.join(" ")));

                        if result.best_move == 0 {
                            res.push(
                                "info string position is terminal (checkmate or stalemate)"
                                    .to_string(),
                            );
                        }

                        res.push(format!("bestmove {}", move_to_uci(result.best_move)));
                    } else {
                        res.push("info string no thread manager available".to_string());
                        res.push("bestmove 0000".to_string());
                    }

                    self.state = UciState::Ready;
                }
            }
            UciCommand::Stop => {
                // Cancel any pending ponder timer
                if let Some(cancel) = self.ponder_timer_cancel.take() {
                    cancel.store(true, Ordering::Relaxed);
                }

                if self.async_search_active {
                    // Stop async search (go infinite mode) and send bestmove
                    if let Some(ref tm) = self.thread_mgr {
                        tm.stop_current_job();

                        // Wait for result with timeout (500ms should be enough for graceful stop)
                        if let Some(result) = tm.wait_async_result(500) {
                            let mut info_parts = vec![
                                format!("depth {}", result.completed_depth),
                                format!("seldepth {}", result.seldepth),
                            ];
                            if result.score >= MATE_THRESHOLD {
                                let mate_plies = MATE - result.score;
                                let mate_moves = (mate_plies + 1) / 2;
                                info_parts.push(format!("score mate {}", mate_moves));
                            } else if result.score <= -MATE_THRESHOLD {
                                let mate_plies = MATE + result.score;
                                let mate_moves = -(mate_plies / 2);
                                info_parts.push(format!("score mate {}", mate_moves));
                            } else {
                                info_parts.push(format!("score cp {}", result.score));
                            }
                            info_parts.push(format!("nodes {}", result.nodes));
                            if result.nps > 0 {
                                info_parts.push(format!("nps {}", result.nps));
                            }
                            info_parts.push(format!("hashfull {}", result.hashfull));
                            if !result.pv.is_empty() {
                                let pv_str = result
                                    .pv
                                    .iter()
                                    .map(|&m| move_to_uci(m))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                info_parts.push(format!("pv {}", pv_str));
                            }
                            res.push(format!("info {}", info_parts.join(" ")));
                            res.push(format!("bestmove {}", move_to_uci(result.best_move)));
                        } else {
                            // Timeout: search didn't complete in time, return null move
                            res.push("info string search timeout on stop".to_string());
                            res.push("bestmove 0000".to_string());
                        }
                    }

                    self.async_search_active = false;
                } else {
                    // Stop command during normal search (already completed or no search active)
                    if let Some(ref tm) = self.thread_mgr {
                        tm.stop_current_job();
                    }
                }

                self.state = UciState::Ready;
            }
            UciCommand::UciNewGame => {
                // Reset to starting position
                self.board = Board::new();
                let _ = self
                    .board
                    .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                self.state = UciState::Ready;
            }
            UciCommand::SetOption { name, value } => {
                // Dynamic setoption: reconfigure thread manager if Threads/Hash changed
                match name.as_str() {
                    "Threads" => {
                        if let Some(v) = value {
                            if let Ok(n) = v.parse::<usize>() {
                                if n > 0 && n <= 256 {
                                    // Stop old thread manager and create new one with updated thread count
                                    if let Some(old_tm) = self.thread_mgr.take() {
                                        old_tm.stop();
                                    }
                                    let hash_mb = self.options.hash as usize;
                                    self.thread_mgr =
                                        Some(crate::search::ThreadManager::new(n, hash_mb));
                                    self.options.threads = n as u8;
                                    res.push(format!("info string Threads set to {}", n));
                                } else {
                                    res.push(
                                        "info string error: Threads must be between 1 and 256"
                                            .to_string(),
                                    );
                                }
                            } else {
                                res.push("info string error: invalid Threads value".to_string());
                            }
                        }
                    }
                    "Hash" => {
                        if let Some(v) = value {
                            if let Ok(mb) = v.parse::<usize>() {
                                if (1..=4096).contains(&mb) {
                                    // Stop old thread manager and create new one with updated TT size
                                    if let Some(old_tm) = self.thread_mgr.take() {
                                        old_tm.stop();
                                    }
                                    let threads = self.options.threads as usize;
                                    self.thread_mgr =
                                        Some(crate::search::ThreadManager::new(threads, mb));
                                    self.options.hash = mb as u64;
                                    res.push(format!("info string Hash set to {} MB", mb));
                                } else {
                                    res.push(
                                        "info string error: Hash must be between 1 and 4096 MB"
                                            .to_string(),
                                    );
                                }
                            } else {
                                res.push("info string error: invalid Hash value".to_string());
                            }
                        }
                    }
                    "MoveOverhead" => {
                        if let Some(v) = value {
                            if let Ok(ms) = v.parse::<u64>() {
                                self.options.move_overhead_ms = ms;
                                res.push(format!("info string MoveOverhead set to {} ms", ms));
                            } else {
                                res.push("info string error: invalid MoveOverhead value".to_string());
                            }
                        }
                    }
                    _ => {
                        // Other options: use existing set_option method
                        let _ = self.options.set_option(&name, value.clone().as_deref());
                        res.push(format!("info string setoption {} = {:?}", name, value));
                    }
                }
            }
            UciCommand::PonderHit => {
                if self.state == UciState::Pondering {
                    self.state = UciState::Thinking;

                    // Schedule a stop after the allocated thinking time.
                    // Use the same clock parameters from the preceding go command.
                    if let Some(ref tm) = self.thread_mgr {
                        let side_white = self.board.side == crate::board::Color::White;
                        let time_alloc = crate::time::TimeManager::allocate_time(
                            &crate::search::params::TimeManagement::new(),
                            self.last_wtime,
                            self.last_btime,
                            self.last_winc,
                            self.last_binc,
                            self.last_movetime,
                            self.last_movestogo,
                            side_white,
                            self.options.move_overhead_ms,
                        );

                        // Cancel any previous timer
                        if let Some(cancel) = self.ponder_timer_cancel.take() {
                            cancel.store(true, Ordering::Relaxed);
                        }

                        let cancel = Arc::new(AtomicBool::new(false));
                        self.ponder_timer_cancel = Some(cancel.clone());
                        let flag = tm.get_stop_flag();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(time_alloc));
                            if !cancel.load(Ordering::Relaxed) {
                                flag.store(true, Ordering::Relaxed);
                            }
                        });
                    }
                }
            }
            UciCommand::Quit => {
                // Stop and join worker threads if present
                if let Some(tm) = self.thread_mgr.take() {
                    tm.stop();
                }
                self.running = false;
            }
            UciCommand::Unknown(s) => {
                res.push(format!("info string unknown command: {}", s));
            }
        }
        res
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

pub fn run_uci_loop() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let mut engine = UciEngine::new();
    let mut buf = String::new();

    while engine.is_running() {
        buf.clear();
        let n = reader.read_line(&mut buf)?;
        if n == 0 {
            // EOF reached
            break;
        }
        let line = buf.trim();
        if line.is_empty() {
            continue;
        }

        let cmd = parse_uci_command(line);
        let responses = engine.handle_command(cmd);

        // Write all responses
        for r in responses {
            writeln!(writer, "{}", r)?;
        }

        // CRITICAL: Always flush after each command to ensure GUI receives output immediately
        writer.flush()?;
    }

    Ok(())
}

pub fn process_uci_line(line: &str, engine: &mut UciEngine) -> Vec<String> {
    let cmd = parse_uci_command(line);
    engine.handle_command(cmd)
}
