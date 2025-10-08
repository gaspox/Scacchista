//! Minimal UCI event loop and state machine for Scacchista

use std::io::{self, BufRead, Write};
use crate::board::Board;
use super::parser::{UciCommand, parse_uci_command};

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
}

impl UciEngine {
    pub fn new() -> Self {
        Self { state: UciState::Init, board: Board::new(), running: true }
    }

    pub fn handle_command(&mut self, cmd: UciCommand) -> Vec<String> {
        let mut res = Vec::new();
        match cmd {
            UciCommand::Uci => {
                res.push("id name Scacchista".to_string());
                res.push("id author Claude Code".to_string());
                res.push("uciok".to_string());
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
            UciCommand::Go { .. } => {
                // Mock search
                res.push("info string starting search".to_string());
                res.push("bestmove e2e4".to_string());
                self.state = UciState::Ready;
            }
            UciCommand::Stop => {
                self.state = UciState::Ready;
            }
            UciCommand::UciNewGame => {
                self.board = Board::new();
                self.state = UciState::Ready;
            }
            UciCommand::SetOption { name, value } => {
                res.push(format!("info string setoption {} = {:?}", name, value));
            }
            UciCommand::PonderHit => {
                if self.state == UciState::Pondering {
                    self.state = UciState::Thinking;
                }
            }
            UciCommand::Quit => {
                self.running = false;
            }
            UciCommand::Unknown(s) => {
                res.push(format!("info string unknown command: {}", s));
            }
        }
        res
    }

    pub fn is_running(&self) -> bool { self.running }
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
        if n == 0 { break; }
        let line = buf.trim();
        if line.is_empty() { continue; }
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
