//! Quick match test: v0.4 vs v0.5.2 head-to-head

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    let engine1 = "./scacchista_v0.4";
    let engine2 = "./scacchista_v0.5.2";
    let rounds = 10;
    let time_ms = 5000u64; // 5s per move
    
    println!("Quick Match: v0.4 vs v0.5.2 ({} rounds, {}ms/move)", rounds, time_ms);
    println!();
    
    let mut score1 = 0.0f32;
    let mut score2 = 0.0f32;
    
    for i in 1..=rounds {
        let (white, black, w_name, b_name) = if i % 2 == 1 {
            (engine1, engine2, "v0.4", "v0.5.2")
        } else {
            (engine2, engine1, "v0.5.2", "v0.4")
        };
        
        print!("Game {}/{}: {} (W) vs {} (B)... ", i, rounds, w_name, b_name);
        
        match play_game(white, black, time_ms) {
            GameResult::WhiteWin => {
                if w_name == "v0.4" { score1 += 1.0; } else { score2 += 1.0; }
                println!("1-0 ({:+})", if w_name == "v0.4" { score1 - score2 } else { score2 - score1 });
            }
            GameResult::BlackWin => {
                if b_name == "v0.4" { score1 += 1.0; } else { score2 += 1.0; }
                println!("0-1 ({:+})", score1 - score2);
            }
            GameResult::Draw => {
                score1 += 0.5; score2 += 0.5;
                println!("1/2-1/2 ({:+})", score1 - score2);
            }
            GameResult::Error => {
                println!("ERROR");
            }
        }
    }
    
    println!();
    println!("Final Score:");
    println!("  v0.4:   {:.1} points", score1);
    println!("  v0.5.2: {:.1} points", score2);
    println!("  Diff:   {:+.1}", score1 - score2);
    
    if score1 > score2 {
        println!("  Result: v0.4 wins by {:.0} points", score1 - score2);
    } else if score2 > score1 {
        println!("  Result: v0.5.2 wins by {:.0} points", score2 - score1);
    } else {
        println!("  Result: Draw");
    }
}

#[derive(Clone, Copy)]
enum GameResult {
    WhiteWin, BlackWin, Draw, Error
}

fn play_game(white_path: &str, black_path: &str, time_ms: u64) -> GameResult {
    let mut w = Command::new(white_path).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut b = Command::new(black_path).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    
    let mut w_in = w.stdin.take().unwrap();
    let mut w_out = BufReader::new(w.stdout.take().unwrap());
    let mut b_in = b.stdin.take().unwrap();
    let mut b_out = BufReader::new(b.stdout.take().unwrap());
    
    // Init
    writeln!(w_in, "uci").unwrap();
    writeln!(b_in, "uci").unwrap();
    wait_for(&mut w_out, "uciok");
    wait_for(&mut b_out, "uciok");
    writeln!(w_in, "isready").unwrap();
    writeln!(b_in, "isready").unwrap();
    wait_for(&mut w_out, "readyok");
    wait_for(&mut b_out, "readyok");
    writeln!(w_in, "ucinewgame").unwrap();
    writeln!(b_in, "ucinewgame").unwrap();
    
    let mut moves = vec![];
    let mut turn = 0;
    
    for _ in 0..100 {
        let pos = if moves.is_empty() { "position startpos".to_string() } 
                  else { format!("position startpos moves {}", moves.join(" ")) };
        let go = format!("go wtime {} btime {} winc 50 binc 50", time_ms, time_ms);
        
        if turn == 0 {
            writeln!(w_in, "{}", pos).unwrap();
            writeln!(w_in, "{}", go).unwrap();
            w_in.flush().unwrap();
            let mv = read_move(&mut w_out);
            if mv == "0000" || mv == "(none)" { return GameResult::BlackWin; }
            moves.push(mv);
            turn = 1;
        } else {
            writeln!(b_in, "{}", pos).unwrap();
            writeln!(b_in, "{}", go).unwrap();
            b_in.flush().unwrap();
            let mv = read_move(&mut b_out);
            if mv == "0000" || mv == "(none)" { return GameResult::WhiteWin; }
            moves.push(mv);
            turn = 0;
        }
    }
    
    let _ = w.kill();
    let _ = b.kill();
    GameResult::Draw
}

fn wait_for<R: BufRead>(reader: &mut R, target: &str) {
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == target { return; }
        line.clear();
    }
}

fn read_move<R: BufRead>(reader: &mut R) -> String {
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        let trimmed = line.trim();
        if trimmed.starts_with("bestmove") {
            let parts: Vec<_> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 { return parts[1].to_string(); }
        }
        line.clear();
    }
    "0000".to_string()
}
