//! Simple test for the search engine
//!
//! Tests basic search functionality on a few positions

use scacchista::search::Search;
use scacchista::{init, Board};

fn main() {
    // Initialize the engine
    init();

    println!("=== Scacchista Search Engine Test ===\n");

    // Test 1: Starting position - shallow search
    println!("Test 1: Starting Position");
    let mut board = Board::new();
    board
        .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    let mut search = Search::with_board(board);

    for depth in 1..=3 {
        let (best_move, score) = search.search(Some(depth));
        println!("Depth {}: best_move={}, score={}", depth, best_move, score);
    }

    search.print_stats();
    println!();

    // Test 2: Simple tactical position
    println!("Test 2: Tactical Position (Queen takes pawn)");
    let mut board2 = Board::new();
    board2
        .set_from_fen("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2")
        .unwrap();

    let mut search2 = Search::with_board(board2);

    for depth in 1..=3 {
        let (best_move, score) = search2.search(Some(depth));
        println!("Depth {}: best_move={}, score={}", depth, best_move, score);
    }

    search2.print_stats();
    println!();

    // Test 3: Endgame position
    println!("Test 3: Endgame Position");
    let mut board3 = Board::new();
    board3
        .set_from_fen("8/8/8/5k2/8/8/4K3/8 w - - 0 1")
        .unwrap();

    let mut search3 = Search::with_board(board3);

    for depth in 1..=3 {
        let (best_move, score) = search3.search(Some(depth));
        println!("Depth {}: best_move={}, score={}", depth, best_move, score);
    }

    search3.print_stats();
    println!();

    println!("=== All Tests Completed ===");
}
