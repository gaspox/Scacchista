//! UCI state machine and main loop implementation for Scacchista
//!
//! This module provides the complete UCI protocol implementation including:
//! - UCI state machine with states: Init, Ready, Thinking, Pondering
//!
//! This module provides the complete UCI protocol implementation including:
//! - UCI state machine with states: Init, Ready, Thinking, Pondering
//! - Main event loop for stdin/stdout communication
//! - Integration with search engine components

use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::search::{SearchContext, SearchResult};
use crate::board::Board;
use super::parser::{UciCommand, parse_uci_command};

/// States of the UCI engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciState {
    /// Initial state before UCI handshake
    Init,
    /// Engine is ready to receive commands
    Ready,
    /// Engine is actively searching for a move
    Thinking,
    /// Engine is pondering (thinking while opponent's turn)
    Pondering,
}

/// UCI Engine state machine
pub struct UciEngine {
    state: UciState,
    board: Board,
    search_context: Option<Arc<Mutex<SearchContext>>>,
    running: bool,
}

impl UciEngine {
    /// Create a new UCI engine instance
    pub fn new() -> Self {
        UciEngine {
            state: UciState::Init,
            board: Board::default(),
            search_context: None,
            running: true,
        }
    }

    /// Get the current engine state
    pub fn state(&self) -> &UciState {
        &self.state
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: UciState) {
        self.state = new_state;
    }

    /// Handle UCI command and return response
    pub fn handle_command(&mut self, command: UciCommand) -> Vec<String> {
        let mut responses = Vec::new();

        match command {
            UciCommand::Uci => {
                responses.push("id name Scacchista".to_string());
                responses.push("id author Claude Code".to_string());
                responses.push("uciok".to_string());
            }

            UciCommand::IsReady => {
                responses.push("readyok".to_string());
            }

            UciCommand::Position { fen, moves } => {
                // Parse FEN or use default starting position
                if let Some(fen_str) = fen {
                    // TODO: Implement FEN parsing when available
                    log::info!("Position command with FEN: {}", fen_str);
                } else {
                    self.board = Board::default();
                }

                // Apply moves if provided
                for mv_str in moves {
                    // TODO: Implement move application when available
                    log::info!("Applying move: {}", mv_str);
                }

                // Transition to ready state
                self.transition_to(UciState::Ready);
            }

            UciCommand::Go {
                wtime,
                btime,
                movetime,
                depth,
                nodes,
                mate,
                infinite,
                ponder,
            } => {
                responses.push("info string Starting search".to_string());

                // For initial implementation, use a mock search
                let mock_move = self.mock_search(depth);

                responses.push(format!("bestmove {}", mock_move));
                self.transition_to(UciState::Ready);
            }

            UciCommand::Stop => {
                // Stop any ongoing search
                if self.state == UciState::Thinking || self.state == UciState::Pondering {
                    self.transition_to(UciState::Ready);
                }
            }

            UciCommand::Quit => {
                self.running = false;
            }

            UciCommand::UciNewGame => {
                // Reset board and clear search
                self.board = Board::default();
                self.search_context = None;
                self.transition_to(UciState::Ready);
            }

            UciCommand::SetOption { name, value } => {
                responses.push(format!("info string Option {} = {:?}", name, value));
            }

            UciCommand::PonderHit => {
                // Convert ponder search to normal search
                if self.state == UciState::Pondering {
                    self.transition_to(UciState::Thinking));
            }

            _ => {
                responses.push("info string Unknown or unsupported command".to_string());
            }
        }

        responses
    }

    /// Simple mock search for initial testing
    fn mock_search(&self, depth: Option<u8>) -> String {
        // For testing, always return a simple move
        // TODO: Replace with actual search integration
        "e2e4".to_string()
    }

    /// Check if the engine should continue running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

/// Main UCI event loop
pub fn run_uci_loop() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let mut engine = UciEngine::new();
    let mut buffer = String::new();

    log::info!("Scacchista UCI engine starting");

    while engine.is_running() {
        buffer.clear();

        // Read input line by line
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // EOF reached
                log::info!("EOF reached, shutting down");
        break;
            }
            Ok(_) => {
                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }

                log::info!("Received command: {}", line);

                // Parse the UCI command
                let command = parse_uci_command(line);

                // Handle the command
                let responses = engine.handle_command(command);

                // Write responses
                for response in responses {
                    writeln!(writer, "{}", response)?;
                writer.flush()?;
            }
            Err(e) => {
                log::error!("Error reading input: {}", e);
                return Err(e);
            }
        }
    }

    log::info!("Scacchista UCI engine shutting down");
    Ok(())
}

/// Parse and execute a single UCI command line
pub fn process_uci_line(line: &str, engine: &mut UciEngine) -> Vec<String> {
    let command = parse_uci_command(line);
    engine.handle_command(command)
}