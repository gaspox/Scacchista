//! UCI command parser for Scacchista
//!
//! This module provides a complete parser for the Universal Chess Interface protocol.

#[derive(Debug, PartialEq, Eq)]
pub enum UciCommand {
    Uci,
    IsReady,
    SetOption { name: String, value: Option<String> },
    UciNewGame,
    Position { fen: Option<String>, moves: Vec<String> },
    Go {
        wtime: Option<u64>,
        btime: Option<u64>,
        movetime: Option<u64>,
        depth: Option<u8>,
        nodes: Option<u64>,
        mate: Option<u8>,
        movestogo: Option<u8>,
        infinite: bool,
        ponder: bool,
    },
    Stop,
    PonderHit,
    Quit,
    Unknown(String),
}

/// Parse a UCI command from a string
pub fn parse_uci_command(line: &str) -> UciCommand {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return UciCommand::Unknown("".to_string());
    }

    match trimmed {
        "uci" => UciCommand::Uci,
        "isready" => UciCommand::IsReady,
        "ucinewgame" => UciCommand::UciNewGame,
        "stop" => UciCommand::Stop,
        "ponderhit" => UciCommand::PonderHit,
        "quit" => UciCommand::Quit,
        _ => UciCommand::Unknown(trimmed.to_string()),
    }
}