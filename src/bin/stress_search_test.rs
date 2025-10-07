//! Stress test for the search engine
//!
//! Tests deeper searches and more complex positions

use scacchista::search::{Search, SearchParams};
use scacchista::{init, Board};

fn main() {
    // Initialize the engine
    init();

    println!("=== Scacchista Search Stress Test ===\n");

    // Test 1: Deeper search on starting position
    println!("Test 1: Deep search on Starting Position");
    let mut board = Board::new();
    board
        .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    let mut search = Search::with_board(board);

    println!("Searching to depth 6...");
    let start = std::time::Instant::now();
    let (best_move, score) = search.search(Some(6));
    let duration = start.elapsed();

    println!("Depth 6: best_move={}, score={}", best_move, score);
    println!("Time: {}ms", duration.as_millis());

    search.print_stats();
    println!();

    // Test 2: Tactical position - should find captures
    println!("Test 2: Tactical Position");
    let mut board2 = Board::new();
    board2
        .set_from_fen("r1bqkbnr/ppp1pppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4")
        .unwrap();

    let mut search2 = Search::with_board(board2);

    for depth in 1..=5 {
        let (best_move, score) = search2.search(Some(depth));
        println!("Depth {}: best_move={}, score={}", depth, best_move, score);
    }

    search2.print_stats();
    println!();

    // Test 3: Complex middlegame position
    println!("Test 3: Middlegame Position");
    let mut board3 = Board::new();
    board3
        .set_from_fen("r2q1rk1/ppp2ppp/2nP1n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQR1K1 w - - 0 8")
        .unwrap();

    let mut search3 = Search::with_board(board3);

    for depth in 1..=4 {
        let (best_move, score) = search3.search(Some(depth));
        println!("Depth {}: best_move={}, score={}", depth, best_move, score);
    }

    search3.print_stats();
    println!();

    // Test 4: Endgame with tactical opportunities
    println!("Test 4: Endgame Position");
    let mut board4 = Board::new();
    board4
        .set_from_fen("8/5pk1/3p1p1p/4P3/8/4K3/8/8 w - - 0 1")
        .unwrap();

    let mut search4 = Search::with_board(board4);

    for depth in 1..=5 {
        let (best_move, score) = search4.search(Some(depth));
        println!("Depth {}: best_move={}, score={}", depth, best_move, score);
    }

    search4.print_stats();
    println!();

    println!("=== Stress Test Completed ===");
}
