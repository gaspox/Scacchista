//! Benchmark suite for Scacchista
//!
//! Runs perft and fixed-depth search on representative positions
//! and prints performance metrics.

use std::time::Instant;

use scacchista::board::Board;
use scacchista::search::params::SearchParams;
use scacchista::search::Search;

fn main() {
    scacchista::init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              SCACCHISTA BENCHMARK SUITE v0.6.0               ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ------------------------------------------------------------------
    // Perft benchmarks
    // ------------------------------------------------------------------
    println!("── Perft Benchmarks ───────────────────────────────────────────");
    let perft_positions = [
        (
            "startpos",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ),
        (
            "pos4",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        ),
    ];

    for (name, fen) in &perft_positions {
        let mut board = Board::new();
        board.set_from_fen(fen).unwrap();

        let start = Instant::now();
        let nodes = perft(&mut board, 5);
        let elapsed = start.elapsed();
        let nps = nodes as f64 / elapsed.as_secs_f64();

        println!(
            "  {:12} depth 5 | {:>10} nodes | {:>6.2?} | {:>8.2} Mnps",
            name,
            nodes,
            elapsed,
            nps / 1_000_000.0
        );
    }

    println!();

    // ------------------------------------------------------------------
    // Search benchmarks
    // ------------------------------------------------------------------
    println!("── Search Benchmarks ──────────────────────────────────────────");
    let search_positions = [
        (
            "startpos",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        (
            "midgame",
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        ),
        ("tactical", "2k1r3/ppp5/8/8/8/8/PPP5/2K1R3 w - - 0 1"),
        ("endgame", "8/3k4/8/8/8/8/3K4/8 w - - 0 1"),
        (
            "complex",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ),
    ];

    let search_depth = 8;

    for (name, fen) in &search_positions {
        let mut board = Board::new();
        board.set_from_fen(fen).unwrap();

        let mut search = Search::new(board, 16, SearchParams::new());

        let start = Instant::now();
        let (_best_move, _score) = search.search(Some(search_depth));
        let elapsed = start.elapsed();

        let stats = search.stats();
        let nodes = stats.nodes;
        let nps = nodes as f64 / elapsed.as_secs_f64();
        let tt_rate = if nodes > 0 {
            (stats.tt_hits as f64 / nodes as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "  {:12} depth {} | {:>10} nodes | {:>6.2?} | {:>8.2} knps | TT {:>5.1}%",
            name,
            search_depth,
            nodes,
            elapsed,
            nps / 1_000.0,
            tt_rate
        );
    }

    println!();
    println!("Benchmark complete.");
}

/// Recursive perft counter.
fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = board.generate_moves();
    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for mv in moves {
        let undo = board.make_move(mv);
        nodes += perft(board, depth - 1);
        board.unmake_move(undo);
    }
    nodes
}
