//! Torneo multi-versione: round-robin tra tutte le versioni

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

const VERSIONS: &[&str] = &[
    "./scacchista_v0.4",
    "./scacchista_v0.5",
    "./scacchista_v0.5.1",
    "./scacchista_v0.5.2",
    "./scacchista_v0.5.3",
];

const VERSION_NAMES: &[&str] = &["v0.4", "v0.5", "v0.5.1", "v0.5.2", "v0.5.3"];

const ROUNDS_PER_PAIR: usize = 2; // Each pair plays 2 games (swap colors)
const TIME_MS: u64 = 10000; // 10 seconds per move
const INC_MS: u64 = 50;

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
    Error,
}

#[derive(Clone)]
struct TournamentResult {
    wins: u32,
    losses: u32,
    draws: u32,
}

impl TournamentResult {
    fn new() -> Self {
        Self {
            wins: 0,
            losses: 0,
            draws: 0,
        }
    }

    fn score(&self) -> f32 {
        self.wins as f32 + 0.5 * self.draws as f32
    }

    fn games(&self) -> u32 {
        self.wins + self.losses + self.draws
    }
}

#[allow(clippy::needless_range_loop)]
fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           TORNEO MULTI-VERSIONE SCACCHISTA                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Versioni partecipanti:");
    for (i, name) in VERSION_NAMES.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    println!();
    println!("Configurazione:");
    println!("  - Round per coppia: {}", ROUNDS_PER_PAIR);
    println!("  - Tempo: {}ms + {}ms incremento", TIME_MS, INC_MS);
    println!();

    let n = VERSIONS.len();
    let mut results: Vec<TournamentResult> = vec![TournamentResult::new(); n];
    let mut total_games = 0;

    // Round-robin: ogni versione contro ogni altra
    for i in 0..n {
        for j in (i + 1)..n {
            println!("\n┌─────────────────────────────────────────────────────────────┐");
            println!("│ Match: {} vs {}", VERSION_NAMES[i], VERSION_NAMES[j]);
            println!("└─────────────────────────────────────────────────────────────┘");

            for round in 0..ROUNDS_PER_PAIR {
                let (white_idx, black_idx, white_name, black_name) = if round % 2 == 0 {
                    (i, j, VERSION_NAMES[i], VERSION_NAMES[j])
                } else {
                    (j, i, VERSION_NAMES[j], VERSION_NAMES[i])
                };

                print!(
                    "  Round {}/{}: {} (W) vs {} (B)... ",
                    round + 1,
                    ROUNDS_PER_PAIR,
                    white_name,
                    black_name
                );

                match play_game(VERSIONS[white_idx], VERSIONS[black_idx]) {
                    GameResult::WhiteWin => {
                        println!("1-0");
                        results[white_idx].wins += 1;
                        results[black_idx].losses += 1;
                    }
                    GameResult::BlackWin => {
                        println!("0-1");
                        results[black_idx].wins += 1;
                        results[white_idx].losses += 1;
                    }
                    GameResult::Draw => {
                        println!("1/2-1/2");
                        results[white_idx].draws += 1;
                        results[black_idx].draws += 1;
                    }
                    GameResult::Error => {
                        println!("ERROR");
                    }
                }
                total_games += 1;
            }
        }
    }

    // Stampa risultati finali
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    RISULTATI FINALI                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "{:10} {:>6} {:>6} {:>6} {:>6} {:>8}",
        "Versione", "Vinte", "Perse", "Patte", "Punti", "Partite"
    );
    println!("{:-<60}", "");

    // Ordina per punteggio
    let mut ranked: Vec<(usize, &str, f32)> = results
        .iter()
        .enumerate()
        .map(|(i, r)| (i, VERSION_NAMES[i], r.score()))
        .collect();
    ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (idx, name, score) in ranked.iter() {
        let r = &results[*idx];
        println!(
            "{:10} {:>6} {:>6} {:>6} {:>6.1} {:>8}",
            name,
            r.wins,
            r.losses,
            r.draws,
            score,
            r.games()
        );
    }

    println!();
    println!("Classifica:");
    for (rank, (_idx, name, score)) in ranked.iter().enumerate() {
        println!("  {}. {} - {:.1} punti", rank + 1, name, score);
    }

    // Calcola ELO relativi (semplificato)
    println!();
    println!("Performance relative (approssimative):");
    let baseline_idx = ranked
        .iter()
        .position(|(_, name, _)| *name == "v0.4")
        .unwrap_or(0);
    let baseline_score = ranked[baseline_idx].2;

    for (_, name, score) in &ranked {
        let diff = score - baseline_score;
        let elo = diff * 20.0; // Approssimazione: 1 punto = 20 ELO
        println!("  {}: {:+.0} ELO", name, elo);
    }

    println!();
    println!("Totale partite giocate: {}", total_games);
}

fn play_game(white_path: &str, black_path: &str) -> GameResult {
    let mut white_proc = match Command::new(white_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(p) => p,
        Err(_) => return GameResult::Error,
    };

    let mut black_proc = match Command::new(black_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(p) => p,
        Err(_) => return GameResult::Error,
    };

    let mut w_in = white_proc.stdin.take().unwrap();
    let mut w_out = BufReader::new(white_proc.stdout.take().unwrap());
    let mut b_in = black_proc.stdin.take().unwrap();
    let mut b_out = BufReader::new(black_proc.stdout.take().unwrap());

    // Init UCI
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

    let mut moves: Vec<String> = Vec::new();
    let mut turn = 0; // 0=White, 1=Black
    let mut wtime = TIME_MS;
    let mut btime = TIME_MS;

    // Game loop (max 150 moves)
    for _ in 0..150 {
        let position_cmd = if moves.is_empty() {
            "position startpos".to_string()
        } else {
            format!("position startpos moves {}", moves.join(" "))
        };

        let go_cmd = format!(
            "go wtime {} btime {} winc {} binc {}",
            wtime, btime, INC_MS, INC_MS
        );

        let start = Instant::now();

        if turn == 0 {
            writeln!(w_in, "{}", position_cmd).unwrap();
            writeln!(w_in, "{}", go_cmd).unwrap();
            w_in.flush().unwrap();

            let (mv, score) = read_bestmove(&mut w_out);
            let elapsed = start.elapsed().as_millis() as u64;
            if wtime > elapsed {
                wtime -= elapsed;
            } else {
                wtime = 0;
            }
            wtime += INC_MS;

            if mv == "0000" || mv == "(none)" || score.abs() >= 29000 && score < 0 {
                let _ = white_proc.kill();
                let _ = black_proc.kill();
                return GameResult::BlackWin;
            }
            if score.abs() >= 29000 && score > 0 {
                let _ = white_proc.kill();
                let _ = black_proc.kill();
                return GameResult::WhiteWin;
            }
            moves.push(mv);
            turn = 1;
        } else {
            writeln!(b_in, "{}", position_cmd).unwrap();
            writeln!(b_in, "{}", go_cmd).unwrap();
            b_in.flush().unwrap();

            let (mv, score) = read_bestmove(&mut b_out);
            let elapsed = start.elapsed().as_millis() as u64;
            if btime > elapsed {
                btime -= elapsed;
            } else {
                btime = 0;
            }
            btime += INC_MS;

            if mv == "0000" || mv == "(none)" || score.abs() >= 29000 && score > 0 {
                let _ = white_proc.kill();
                let _ = black_proc.kill();
                return GameResult::WhiteWin;
            }
            if score.abs() >= 29000 && score < 0 {
                let _ = white_proc.kill();
                let _ = black_proc.kill();
                return GameResult::BlackWin;
            }
            moves.push(mv);
            turn = 0;
        }
    }

    let _ = white_proc.kill();
    let _ = black_proc.kill();
    GameResult::Draw
}

fn wait_for<R: BufRead>(reader: &mut R, target: &str) {
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        if line.trim() == target {
            return;
        }
        line.clear();
    }
}

fn read_bestmove<R: BufRead>(reader: &mut R) -> (String, i32) {
    let mut line = String::new();
    let mut score = 0i32;

    while reader.read_line(&mut line).unwrap() > 0 {
        let trimmed = line.trim();
        if trimmed.starts_with("info") && trimmed.contains("score cp") {
            if let Some(idx) = trimmed.find("cp") {
                if let Some(val_str) = trimmed[idx + 2..].split_whitespace().next() {
                    if let Ok(val) = val_str.parse::<i32>() {
                        score = val;
                    }
                }
            }
        }
        if trimmed.starts_with("bestmove") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return (parts[1].to_string(), score);
            }
        }
        line.clear();
    }
    ("0000".to_string(), 0)
}
