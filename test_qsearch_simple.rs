use scacchista::search::{Search, SearchParams};
use scacchista::board::Board;

fn main() {
    println!("Testing quiescence search impact...\n");

    // Test tactical position with captures
    let mut board = Board::new();
    board.set_from_fen("rnbqkbnr/ppp2ppp/4p3/3pP3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3").unwrap();

    println!("1. WITH quiescence (depth=4):");
    let mut search_with_q = Search::new(board.clone(), 1, SearchParams::new().max_depth(4).qsearch_depth(6));
    let (best_move_with_q, score_with_q) = search_with_q.search(Some(4));

    search_with_q.print_stats();
    println!("Best move: {}, Score: {}\n", best_move_with_q, score_with_q);

    println!("2. WITHOUT quiescence (qsearch_depth=0):");
    let mut search_no_q = Search::new(board.clone(), 1, SearchParams::new().max_depth(4).qsearch_depth(0));
    let (best_move_no_q, score_no_q) = search_no_q.search(Some(4));

    search_no_q.print_stats();
    println!("Best move: {}, Score: {}\n", best_move_no_q, score_no_q);

    println!("Score difference: {} centipawns", score_with_q - score_no_q);

    if score_with_q != score_no_q {
        println!("✓ Quiescence found improved evaluation in tactical position!");
    } else {
        println!("! Quiescence didn't change evaluation (position may be quiet)");
    }
}