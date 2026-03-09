
use scacchista::board::Board;
use scacchista::search::search::Search;
use scacchista::search::params::SearchParams;
use std::time::Instant;

#[test]
fn benchmark_optimization_impact() {
    let mut board = Board::new();
    board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    let depth = 9;
    
    // 1. Run Baseline (No Optimizations)
    let mut params_base = SearchParams::new();
    params_base.max_depth = depth;
    params_base.time_limit_ms = 30000;
    params_base.enable_qsearch_optimizations = false; // DISABLE

    println!("Benchmarking BASELINE (Depth {})... ", depth);
    let start_base = Instant::now();
    let mut search_base = Search::new(board.clone(), 64, params_base);
    let (_, _score_base) = search_base.search(None);
    let duration_base = start_base.elapsed();
    let nodes_base = search_base.stats().nodes;
    let nps_base = nodes_base as f64 / duration_base.as_secs_f64();
    
    println!("BASELINE:");
    println!("  Time: {:.4}s", duration_base.as_secs_f64());
    println!("  Nodes: {}", nodes_base);
    println!("  NPS: {:.0}", nps_base);

    // 2. Run Optimized (Phase 1.1 + 1.2)
    let mut params_opt = SearchParams::new();
    params_opt.max_depth = depth;
    params_opt.time_limit_ms = 30000;
    params_opt.enable_qsearch_optimizations = true; // ENABLE (Default)

    println!("\nBenchmarking OPTIMIZED (Depth {})... ", depth);
    let start_opt = Instant::now();
    let mut search_opt = Search::new(board.clone(), 64, params_opt);
    let (_, _score_opt) = search_opt.search(None);
    let duration_opt = start_opt.elapsed();
    let nodes_opt = search_opt.stats().nodes;
    let nps_opt = nodes_opt as f64 / duration_opt.as_secs_f64();

    println!("OPTIMIZED:");
    println!("  Time: {:.4}s", duration_opt.as_secs_f64());
    println!("  Nodes: {}", nodes_opt);
    println!("  NPS: {:.0}", nps_opt);

    // Report Summary
    let speedup = (nps_opt - nps_base) / nps_base * 100.0;
    println!("\nRESULTS:");
    println!("  Speedup: {:.2}%", speedup);
    println!("  Time Reduction: {:.2}%", (1.0 - duration_opt.as_secs_f64() / duration_base.as_secs_f64()) * 100.0);
    
    assert!(nps_opt > nps_base, "Optimized version should be faster");
}
