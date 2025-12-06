//! Minimal UCI event loop and state machine for Scacchista

use super::parser::{parse_uci_command, UciCommand};
use crate::board::{move_to_uci, parse_uci_move, Board};
use std::io::{self, BufRead, Write};
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
                winc,  // FIX Bug #4
                binc,  // FIX Bug #4
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
                    winc,     // FIX Bug #4: Pass increment
                    binc,     // FIX Bug #4: Pass increment
                    movetime,
                    side_white,
                );

                if infinite {
                    // ASYNC MODE: go infinite - start search in background, don't send bestmove yet
                    let params = crate::search::SearchParams::new()
                        .max_depth(10) // Reasonable depth for infinite mode
                        .time_limit(600_000); // 10 minutes timeout

                    if let Some(ref tm) = self.thread_mgr {
                        let job = crate::search::thread_mgr::SearchJob {
                            board: self.board.clone(),
                            params,
                        };
                        tm.start_async_search(job);
                        self.async_search_active = true;
                        self.state = UciState::Thinking;
                        // No bestmove sent here - will be sent when stop command is received
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
                        0  // 0 = no time limit, depth controls search
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
                        // FIX Bug #3: submit_job now returns (mv, score, completed_depth)
                        let (mv, score, completed_depth) = tm.submit_job(job);
                        let search_time_ms = search_start.elapsed().as_millis() as u64;
                        res.push(format!(
                            "info depth {} score cp {} time {}",
                            completed_depth,  // FIX Bug #3: Use actual completed depth
                            score,
                            search_time_ms
                        ));

                        // Check if position is terminal (no legal moves)
                        if mv == 0 {
                            res.push(
                                "info string position is terminal (checkmate or stalemate)"
                                    .to_string(),
                            );
                        }

                        res.push(format!("bestmove {}", move_to_uci(mv)));
                    } else {
                        res.push("info string no thread manager available".to_string());
                        res.push("bestmove 0000".to_string());
                    }

                    self.state = UciState::Ready;
                }
            }
            UciCommand::Stop => {
                if self.async_search_active {
                    // Stop async search (go infinite mode) and send bestmove
                    if let Some(ref tm) = self.thread_mgr {
                        tm.stop_current_job();

                        // Wait for result with timeout (500ms should be enough for graceful stop)
                        // FIX Bug #3: wait_async_result now returns (mv, score, completed_depth)
                        if let Some((mv, score, completed_depth)) = tm.wait_async_result(500) {
                            res.push(format!("info depth {} score cp {}", completed_depth, score));
                            res.push(format!("bestmove {}", move_to_uci(mv)));
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
                                if mb >= 1 && mb <= 4096 {
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
