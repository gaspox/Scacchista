//! UCI command parser for Scacchista

//! Minimal but practical UCI parser covering the commands used in tests.

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UciCommand {
    Uci,
    IsReady,
    SetOption {
        name: String,
        value: Option<String>,
    },
    UciNewGame,
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
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

/// Parse a UCI command from a string (simple tokenizer)
pub fn parse_uci_command(line: &str) -> UciCommand {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return UciCommand::Unknown("".to_string());
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    match parts[0] {
        "uci" => UciCommand::Uci,
        "isready" => UciCommand::IsReady,
        "ucinewgame" => UciCommand::UciNewGame,
        "stop" => UciCommand::Stop,
        "ponderhit" => UciCommand::PonderHit,
        "quit" => UciCommand::Quit,
        "setoption" => {
            // expected: setoption name <name> [value <val>]
            let mut name = String::new();
            let mut value: Option<String> = None;
            let mut i = 1usize;
            while i < parts.len() {
                match parts[i] {
                    "name" => {
                        i += 1;
                        let mut vals = Vec::new();
                        while i < parts.len() && parts[i] != "value" {
                            vals.push(parts[i]);
                            i += 1;
                        }
                        name = vals.join(" ");
                    }
                    "value" => {
                        i += 1;
                        let vals = parts[i..].join(" ");
                        value = Some(vals);
                        break;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            UciCommand::SetOption { name, value }
        }
        "position" => {
            // position [fen <fenstring> | startpos ]  moves <move1> ...
            let mut fen: Option<String> = None;
            let mut moves: Vec<String> = Vec::new();
            if parts.len() >= 2 && parts[1] == "startpos" {
                fen = None;
                // find "moves"
                if let Some(pos) = parts.iter().position(|&s| s == "moves") {
                    for &m in &parts[pos + 1..] {
                        moves.push(m.to_string());
                    }
                }
            } else if parts.len() >= 2 && parts[1] == "fen" {
                // collect until "moves" or end
                let mut i = 2usize;
                let mut fen_parts = Vec::new();
                while i < parts.len() && parts[i] != "moves" {
                    fen_parts.push(parts[i]);
                    i += 1;
                }
                fen = Some(fen_parts.join(" "));
                if i < parts.len() && parts[i] == "moves" {
                    for &m in &parts[i + 1..] {
                        moves.push(m.to_string());
                    }
                }
            }
            UciCommand::Position { fen, moves }
        }
        "go" => {
            let mut wtime: Option<u64> = None;
            let mut btime: Option<u64> = None;
            let mut movetime: Option<u64> = None;
            let mut depth: Option<u8> = None;
            let mut nodes: Option<u64> = None;
            let mut mate: Option<u8> = None;
            let mut movestogo: Option<u8> = None;
            let mut infinite = false;
            let mut ponder = false;

            let mut i = 1usize;
            while i < parts.len() {
                match parts[i] {
                    "wtime" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u64>() {
                                wtime = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "btime" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u64>() {
                                btime = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "movetime" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u64>() {
                                movetime = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "depth" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u8>() {
                                depth = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "nodes" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u64>() {
                                nodes = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "mate" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u8>() {
                                mate = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "movestogo" => {
                        if let Some(v) = parts.get(i + 1) {
                            if let Ok(x) = v.parse::<u8>() {
                                movestogo = Some(x);
                            }
                        }
                        i += 2;
                    }
                    "infinite" => {
                        infinite = true;
                        i += 1;
                    }
                    "ponder" => {
                        ponder = true;
                        i += 1;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }

            UciCommand::Go {
                wtime,
                btime,
                movetime,
                depth,
                nodes,
                mate,
                movestogo,
                infinite,
                ponder,
            }
        }
        other => UciCommand::Unknown(other.to_string()),
    }
}
