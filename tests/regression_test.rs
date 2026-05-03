//! Test di regressione per confrontare output tra versioni del motore
//!
//! Questi test verificano che le modifiche non alterino il comportamento
//! di ricerca in modo significativo, confrontando score e bestmove
//! su un set di posizioni di test.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Risultato di un singolo test di ricerca
#[derive(Debug, Clone)]
struct SearchResult {
    depth: u8,
    score: i16,
    bestmove: String,
}

/// Esegue una ricerca su una posizione FEN e ritorna il risultato
fn search_position(engine_path: &str, fen: &str, depth: u8) -> Option<SearchResult> {
    let mut child = Command::new(engine_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    let stdin = child.stdin.as_mut()?;
    let stdout = BufReader::new(child.stdout.take()?);

    // Setup UCI
    writeln!(stdin, "uci").ok()?;
    let mut lines = stdout.lines();
    while let Some(Ok(line)) = lines.next() {
        if line.trim() == "uciok" {
            break;
        }
    }

    writeln!(stdin, "isready").ok()?;
    while let Some(Ok(line)) = lines.next() {
        if line.trim() == "readyok" {
            break;
        }
    }

    // Posizione e ricerca
    writeln!(stdin, "ucinewgame").ok()?;
    writeln!(stdin, "position fen {}", fen).ok()?;
    writeln!(stdin, "go depth {}", depth).ok()?;
    stdin.flush().ok()?;

    let mut depth_found = 0;
    let mut score_found = 0;
    let mut bestmove_found = String::new();

    for line in lines {
        let line = line.ok()?;
        let trimmed = line.trim();

        if trimmed.starts_with("info") && trimmed.contains("score cp") {
            // Parse info line
            let parts: Vec<_> = trimmed.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "depth" && i + 1 < parts.len() {
                    depth_found = parts[i + 1].parse().unwrap_or(0);
                }
                if *part == "score" && i + 2 < parts.len() && parts[i + 1] == "cp" {
                    score_found = parts[i + 2].parse().unwrap_or(0);
                }
            }
        }

        if trimmed.starts_with("bestmove") {
            let parts: Vec<_> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                bestmove_found = parts[1].to_string();
            }
            break;
        }
    }

    let _ = child.kill();

    if bestmove_found.is_empty() {
        None
    } else {
        Some(SearchResult {
            depth: depth_found,
            score: score_found,
            bestmove: bestmove_found,
        })
    }
}

/// Confronta due risultati di ricerca e ritorna true se sono compatibili
#[test]
fn test_regression_depth_progression() {
    // Verifica che l'engine dia risultati consistenti tra profondità diverse
    // e che lo score sia stabile
    let engine = "./target/release/scacchista";

    if !std::path::Path::new(engine).exists() {
        eprintln!("Engine non trovato: {}", engine);
        return;
    }

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    println!("\n=== Test Progressione Profondità ===");

    let mut prev_score = 0i16;
    for depth in [4, 5, 6] {
        let result = search_position(engine, fen, depth);
        assert!(result.is_some(), "Ricerca fallita a depth {}", depth);

        let res = result.unwrap();
        println!(
            "Depth {}: score={}, bestmove={}",
            res.depth, res.score, res.bestmove
        );

        // Lo score non dovrebbe cambiare troppo tra profondità consecutive
        if depth > 4 {
            let diff = (res.score - prev_score).abs();
            assert!(
                diff < 100,
                "Cambio di score troppo grande tra depth {} e {}: {} vs {} (diff={})",
                depth - 1,
                depth,
                prev_score,
                res.score,
                diff
            );
        }
        prev_score = res.score;
    }
}

#[test]
fn test_self_consistency() {
    // Verifica che lo stesso engine dia risultati consistenti
    let engine = "./target/release/scacchista";

    if !std::path::Path::new(engine).exists() {
        eprintln!("Engine non trovato: {}", engine);
        return;
    }

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let depth = 6;

    let r1 = search_position(engine, fen, depth);
    let r2 = search_position(engine, fen, depth);

    assert!(r1.is_some(), "Prima ricerca fallita");
    assert!(r2.is_some(), "Seconda ricerca fallita");

    let res1 = r1.unwrap();
    let res2 = r2.unwrap();

    // Stesso engine dovrebbe dare risultati identici
    assert_eq!(
        res1.score, res2.score,
        "Score diverso nello stesso engine: {} vs {}",
        res1.score, res2.score
    );
    assert_eq!(
        res1.bestmove, res2.bestmove,
        "Bestmove diverso nello stesso engine: {} vs {}",
        res1.bestmove, res2.bestmove
    );
}
