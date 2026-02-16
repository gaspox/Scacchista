use scacchista::board::Board;
use scacchista::search::params::SearchParams;
use scacchista::search::search::Search;
use std::time::Instant;

fn main() {
    scacchista::init();
    let mut board = Board::new();
    board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let params = SearchParams::default();
    // params.debug = false; // Not available
    let mut search = Search::new(board, 64, params); // 64MB TT

    println!("Benchmarking depth 9...");
    let start = Instant::now();
    let (mv, score) = search.search(Some(9));
    let elapsed = start.elapsed();
    
    let nodes = search.stats().nodes;
    let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;
    
    println!("Time: {:.2?}", elapsed);
    println!("Nodes: {}", nodes);
    println!("NPS: {}", nps);
    println!("Best Move: {:?}", mv);
    println!("Score: {}", score);
    
    let stats = search.stats();
    println!("Total Cutoffs: {}", stats.cutoffs);
    println!("Countermove Cutoffs: {}", stats.countermove_cutoffs);
    if stats.cutoffs > 0 {
        println!("Countermove Effectiveness: {:.1}%", (stats.countermove_cutoffs as f64 / stats.cutoffs as f64) * 100.0);
    }
}
