//! Minimal UCI event loop and state machine for Scacchista

use super::parser::{parse_uci_command, UciCommand};
use crate::board::Board;
use std::io::{self, BufRead, Write};

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
        }
    }

    pub fn handle_command(&mut self, cmd: UciCommand) -> Vec<String> {
        let mut res = Vec::new();
        match cmd {
            UciCommand::Uci => {
                res.push("id name Scacchista".to_string());
                res.push("id author Claude Code".to_string());
                res.push("uciok".to_string());
                self.state = UciState::Ready;
            }
            UciCommand::IsReady => {
                res.push("readyok".to_string());
            }
            UciCommand::Position { fen, moves } => {
                if let Some(_f) = fen {
                    // FEN parsing not implemented yet
                } else {
                    self.board = Board::new();
                }
                for _m in moves { /* apply moves when move parsing is available */ }
                self.state = UciState::Ready;
            }
            UciCommand::Go {
                wtime,
                btime,
                movetime,
                depth,
                nodes: _nodes,
                mate: _mate,
                movestogo: _movestogo,
                infinite: _infinite,
                ponder: _ponder,
            } => {
                // Compute time budget (simple allocation for now)
                let side_white = true; // TODO: derive from board.side when available
                let time_alloc = crate::time::TimeManager::allocate_time(
                    &crate::search::params::TimeManagement::new(),
                    wtime,
                    btime,
                    movetime,
                    side_white,
                );

                // Build search params (map depth if provided)
                let params = crate::search::SearchParams::new().max_depth(depth.unwrap_or(4));

                // Submit job to persistent thread manager
                if let Some(tm) = &self.thread_mgr {
                    let job = crate::search::thread_mgr::SearchJob {
                        board: self.board.clone(),
                        params,
                    };
                    let (mv, score) = tm.submit_job(job);
                    res.push(format!(
                        "info string search done score {} time_alloc_ms {}",
                        score, time_alloc
                    ));
                    res.push(format!("bestmove {}", mv));
                } else {
                    res.push("info string no thread manager available".to_string());
                    res.push("bestmove 0000".to_string());
                }

                self.state = UciState::Ready;
            }
            UciCommand::Stop => {
                // Current ThreadManager does not support preemption of a running search yet.
                // For now, mark engine Ready. Future work: add stop flag API to ThreadManager.
                self.state = UciState::Ready;
            }
            UciCommand::UciNewGame => {
                self.board = Board::new();
                self.state = UciState::Ready;
            }
            UciCommand::SetOption { name, value } => {
                // Basic setoption: update stored options (no dynamic reconfigure for thread count yet)
                let _ = self.options.set_option(&name, value.clone().as_deref());
                res.push(format!("info string setoption {} = {:?}", name, value));
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
            break;
        }
        let line = buf.trim();
        if line.is_empty() {
            continue;
        }
        let cmd = parse_uci_command(line);
        let responses = engine.handle_command(cmd);
        for r in responses {
            writeln!(writer, "{}", r)?;
        }
        writer.flush()?;
    }

    Ok(())
}

pub fn process_uci_line(line: &str, engine: &mut UciEngine) -> Vec<String> {
    let cmd = parse_uci_command(line);
    engine.handle_command(cmd)
}
