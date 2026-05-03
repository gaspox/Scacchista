use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

fn main() {
    let engine1_path = "./scacchista_v0.5";
    let engine2_path = "./scacchista_v0.4";
    let rounds = 10;
    let time_ms = 3000; // 3 seconds (quick test)
    let inc_ms = 50; // 0.05 seconds

    println!("Starting tournament: {} vs {}", engine1_path, engine2_path);
    println!("Rounds: {}, TC: {}ms + {}ms", rounds, time_ms, inc_ms);

    let mut score1 = 0.0;
    let mut score2 = 0.0;

    for i in 1..=rounds {
        let (white_path, black_path, white_name, black_name) = if i % 2 != 0 {
            (engine1_path, engine2_path, "v0.5", "v0.4")
        } else {
            (engine2_path, engine1_path, "v0.4", "v0.5")
        };

        print!(
            "Game {}/{} ({} vs {})... ",
            i, rounds, white_name, black_name
        );
        std::io::stdout().flush().unwrap();

        match play_game(white_path, black_path, time_ms, inc_ms) {
            GameResult::WhiteWin => {
                println!("1-0 (White wins)");
                if white_name == "v0.5" {
                    score1 += 1.0;
                } else {
                    score2 += 1.0;
                }
            }
            GameResult::BlackWin => {
                println!("0-1 (Black wins)");
                if black_name == "v0.5" {
                    score1 += 1.0;
                } else {
                    score2 += 1.0;
                }
            }
            GameResult::Draw => {
                println!("1/2-1/2 (Draw)");
                score1 += 0.5;
                score2 += 0.5;
            }
            GameResult::Error(e) => {
                println!("Error: {}", e);
            }
        }
    }

    println!("\n--- Final Results ---");
    println!("v0.5 Score: {}", score1);
    println!("v0.4 Score: {}", score2);

    let total = score1 + score2;
    if total > 0.0 {
        let fraction = score1 / total;
        let elo_diff = -400.0 * (1.0f64 / fraction - 1.0f64).log10();
        println!("Elo Difference (v0.5 - v0.4): {:+.1}", elo_diff);
    }
}

enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Error(String),
}

fn play_game(white_path: &str, black_path: &str, time_ms: u64, inc_ms: u64) -> GameResult {
    let mut white_proc = match Command::new(white_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(p) => p,
        Err(e) => return GameResult::Error(format!("Failed to start white: {}", e)),
    };

    let mut black_proc = match Command::new(black_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(p) => p,
        Err(e) => return GameResult::Error(format!("Failed to start black: {}", e)),
    };

    let mut w_in = white_proc.stdin.take().unwrap();
    let mut w_out = BufReader::new(white_proc.stdout.take().unwrap());
    let mut b_in = black_proc.stdin.take().unwrap();
    let mut b_out = BufReader::new(black_proc.stdout.take().unwrap());

    // Init UCI
    writeln!(w_in, "uci").unwrap();
    writeln!(b_in, "uci").unwrap();

    // Wait for "uciok"
    wait_for_uciok(&mut w_out);
    wait_for_uciok(&mut b_out);

    writeln!(w_in, "isready").unwrap();
    writeln!(b_in, "isready").unwrap();
    wait_for_readyok(&mut w_out);
    wait_for_readyok(&mut b_out);

    writeln!(w_in, "ucinewgame").unwrap();
    writeln!(b_in, "ucinewgame").unwrap();

    let mut moves = Vec::new();
    let mut turn = 0; // 0=White, 1=Black
    let mut wtime = time_ms;
    let mut btime = time_ms;

    // Game loop (limited to 200 moves to prevent infinite games)
    for _ in 0..200 {
        let position_cmd = if moves.is_empty() {
            "position startpos".to_string()
        } else {
            format!("position startpos moves {}", moves.join(" "))
        };

        let go_cmd = format!(
            "go wtime {} btime {} winc {} binc {}",
            wtime, btime, inc_ms, inc_ms
        );

        let start = Instant::now();
        if turn == 0 {
            // White
            writeln!(w_in, "{}", position_cmd).unwrap();
            writeln!(w_in, "{}", go_cmd).unwrap();
            w_in.flush().unwrap();

            let (mv, score) = read_bestmove(&mut w_out);
            print!(" W:{} ({})", mv, score);
            std::io::stdout().flush().unwrap();

            let real_elapsed = start.elapsed().as_millis() as u64;
            // Adjust time tracking roughly
            if wtime > real_elapsed {
                wtime -= real_elapsed;
            } else {
                wtime = 0;
            }
            wtime += inc_ms;

            if mv == "0000" || mv == "(none)" {
                return GameResult::BlackWin;
            }
            // Check for mate announcement
            if score.abs() >= 29000 {
                println!(" [Mate detected]");
                if score > 0 {
                    return GameResult::WhiteWin;
                } else {
                    return GameResult::BlackWin;
                }
            }
            moves.push(mv.clone());
            turn = 1;
        } else {
            // Black
            writeln!(b_in, "{}", position_cmd).unwrap();
            writeln!(b_in, "{}", go_cmd).unwrap();
            b_in.flush().unwrap();

            let (mv, score) = read_bestmove(&mut b_out);
            print!(" B:{} ({})", mv, score);
            std::io::stdout().flush().unwrap();

            let real_elapsed = start.elapsed().as_millis() as u64;
            if btime > real_elapsed {
                btime -= real_elapsed;
            } else {
                btime = 0;
            }
            btime += inc_ms;

            if mv == "0000" || mv == "(none)" {
                return GameResult::WhiteWin;
            }
            if score.abs() >= 29000 {
                println!(" [Mate detected]");
                if score > 0 {
                    return GameResult::BlackWin;
                } else {
                    return GameResult::WhiteWin;
                }
            }
            moves.push(mv.clone());
            turn = 0;
        }
    }

    // Kill processes
    let _ = white_proc.kill();
    let _ = black_proc.kill();

    GameResult::Draw // Limit reached
}

fn wait_for_uciok<R: BufRead>(reader: &mut R) {
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == "uciok" {
            return;
        }
        line.clear();
    }
}

fn wait_for_readyok<R: BufRead>(reader: &mut R) {
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == "readyok" {
            return;
        }
        line.clear();
    }
}

fn read_bestmove<R: BufRead>(reader: &mut R) -> (String, i32) {
    let mut line = String::new();
    let mut score_cp = 0;
    while reader.read_line(&mut line).unwrap() > 0 {
        let trimmed = line.trim();
        if trimmed.starts_with("info") && trimmed.contains("score") {
            // Parse score if possible "score cp 50" or "score mate 5"
            if let Some(idx) = trimmed.find("cp") {
                if let Some(val_str) = trimmed[idx + 2..].split_whitespace().next() {
                    if let Ok(val) = val_str.parse::<i32>() {
                        score_cp = val;
                    }
                }
            } else if let Some(idx) = trimmed.find("mate") {
                if let Some(val_str) = trimmed[idx + 4..].split_whitespace().next() {
                    if let Ok(val) = val_str.parse::<i32>() {
                        // Mate score: extremely high
                        score_cp = if val > 0 { 30000 } else { -30000 };
                    }
                }
            }
        }
        if trimmed.starts_with("bestmove") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return (parts[1].to_string(), score_cp);
            }
        }
        line.clear();
    }
    ("0000".to_string(), 0)
}
