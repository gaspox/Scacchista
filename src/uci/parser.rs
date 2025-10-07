use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum UciCommand {
    Uci,
    IsReady,
    SetOption { name: String, value: Option<String> },
    UciNewGame,
    Position { fen: Option<String>, moves: Vec<String> },
    Go { wtime: Option<u64>, btime: Option<u64>, movetime: Option<u64> },
    Stop,
    Quit,
    Unknown(String),
}

pub fn parse_uci_command(line: &str) -> UciCommand {
    let mut parts = line.trim().split_whitespace();
    if let Some(cmd) = parts.next() {
        match cmd {
            "uci" => UciCommand::Uci,
            "isready" => UciCommand::IsReady,
            "setoption" => {
                // setoption name <name> value <value>
                let mut name = None;
                let mut value = None;
                let mut iter = parts;
                while let Some(token) = iter.next() {
                    match token {
                        "name" => {
                            // collect until we find "value" or end
                            let mut collected = Vec::new();
                            while let Some(t) = iter.next() {
                                if t == "value" {
                                    break;
                                }
                                collected.push(t);
                            }
                            name = Some(collected.join(" "));
                        }
                        "value" => {
                            // rest of tokens is value
                            let rest: Vec<&str> = iter.collect();
                            if !rest.is_empty() {
                                value = Some(rest.join(" "));
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                UciCommand::SetOption { name: name.unwrap_or_default(), value }
            }
            "ucinewgame" => UciCommand::UciNewGame,
            "position" => {
                // position [fen <fenstring> | startpos ] moves <move1> ...
                let mut fen: Option<String> = None;
                let mut moves: Vec<String> = Vec::new();
                let mut iter = parts;
                if let Some(token) = iter.next() {
                    if token == "startpos" {
                        fen = None;
                    } else if token == "fen" {
                        // collect 6 fields of fen
                        let mut fen_parts = Vec::new();
                        for _ in 0..6 {
                            if let Some(f) = iter.next() {
                                fen_parts.push(f);
                            }
                        }
                        fen = Some(fen_parts.join(" "));
                    }
                }

                // if next token is "moves", collect moves
                if let Some(token) = iter.next() {
                    if token == "moves" {
                        for m in iter {
                            moves.push(m.to_string());
                        }
                    }
                }

                UciCommand::Position { fen, moves }
            }
            "go" => {
                let mut wtime = None;
                let mut btime = None;
                let mut movetime = None;
                let mut iter = parts;
                while let Some(token) = iter.next() {
                    match token {
                        "wtime" => {
                            if let Some(v) = iter.next() {
                                if let Ok(val) = v.parse::<u64>() {
                                    wtime = Some(val);
                                }
                            }
                        }
                        "btime" => {
                            if let Some(v) = iter.next() {
                                if let Ok(val) = v.parse::<u64>() {
                                    btime = Some(val);
                                }
                            }
                        }
                        "movetime" => {
                            if let Some(v) = iter.next() {
                                if let Ok(val) = v.parse::<u64>() {
                                    movetime = Some(val);
                                }
                            }
                        }
                        _ => {
                            // ignore other go params for now
                            let _ = iter.next();
                        }
                    }
                }
                UciCommand::Go { wtime, btime, movetime }
            }
            "stop" => UciCommand::Stop,
            "quit" => UciCommand::Quit,
            other => UciCommand::Unknown(other.to_string()),
        }
    } else {
        UciCommand::Unknown("".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_commands() {
        assert_eq!(parse_uci_command("uci"), UciCommand::Uci);
        assert_eq!(parse_uci_command("isready"), UciCommand::IsReady);
        assert_eq!(parse_uci_command("ucinewgame"), UciCommand::UciNewGame);
        assert_eq!(parse_uci_command("stop"), UciCommand::Stop);
        assert_eq!(parse_uci_command("quit"), UciCommand::Quit);
    }

    #[test]
    fn test_parse_setoption() {
        let cmd = "setoption name Hash value 128";
        if let UciCommand::SetOption { name, value } = parse_uci_command(cmd) {
            assert_eq!(name, "Hash");
            assert_eq!(value.unwrap(), "128");
        } else {
            panic!("SetOption parse failed");
        }

        let cmd2 = "setoption name UCI_EngineOptions";
        if let UciCommand::SetOption { name, value } = parse_uci_command(cmd2) {
            assert_eq!(name, "UCI_EngineOptions");
            assert!(value.is_none());
        } else {
            panic!("SetOption parse failed");
        }
    }

    #[test]
    fn test_parse_position_startpos_moves() {
        let cmd = "position startpos moves e2e4 e7e5";
        if let UciCommand::Position { fen, moves } = parse_uci_command(cmd) {
            assert!(fen.is_none());
            assert_eq!(moves, vec!["e2e4".to_string(), "e7e5".to_string()]);
        } else {
            panic!("Position parse failed");
        }
    }

    #[test]
    fn test_parse_position_fen() {
        let cmd = "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4";
        if let UciCommand::Position { fen, moves } = parse_uci_command(cmd) {
            assert!(fen.is_some());
            assert_eq!(moves, vec!["e2e4".to_string()]);
        } else {
            panic!("Position fen parse failed");
        }
    }

    #[test]
    fn test_parse_go() {
        let cmd = "go wtime 300000 btime 300000 movetime 1000";
        if let UciCommand::Go { wtime, btime, movetime } = parse_uci_command(cmd) {
            assert_eq!(wtime.unwrap(), 300000);
            assert_eq!(btime.unwrap(), 300000);
            assert_eq!(movetime.unwrap(), 1000);
        } else {
            panic!("Go parse failed");
        }
    }
}
