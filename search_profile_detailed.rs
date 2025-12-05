// Profiling dettagliato del search per identificare bottleneck
use scacchista::board::Board;
use scacchista::search::{SearchParams, ThreadManager};
use std::time::Instant;

fn main() {
    println!("=== DETAILED SEARCH PROFILING ===\n");

    // Test posizione iniziale depth 6
    let mut board = Board::new();
    let _ = board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let params = SearchParams::new()
        .max_depth(6)
        .time_limit(30_000); // 30 secondi max

    let tm = ThreadManager::new(1, 16);
    let job = scacchista::search::thread_mgr::SearchJob {
        board: board.clone(),
        params,
    };

    println!("Running search depth 6...");
    let start = Instant::now();
    let (best_move, score) = tm.submit_job(job);
    let elapsed = start.elapsed();

    println!("\n=== RESULTS ===");
    println!("Best move: {}", scacchista::board::move_to_uci(best_move));
    println!("Score: {} cp", score);
    println!("Time: {:?}", elapsed);
    println!();

    // Le statistiche dettagliate dovrebbero essere disponibili nel ThreadManager
    // ma non sono esposte nell'API pubblica. Possiamo solo dedurre da nodes/sec

    // Calcolo approssimativo dei nodi
    let nodes_per_sec = 15_000; // Baseline misurato
    let estimated_nodes = (elapsed.as_millis() as u64 * nodes_per_sec) / 1000;

    println!("=== ESTIMATED BREAKDOWN ===");
    println!("Estimated nodes: ~{}", estimated_nodes);
    println!("Nodes/sec: ~{}", nodes_per_sec);
    println!();

    // Confronto con perft per calcolare overhead
    println!("=== COMPARISON WITH PERFT ===");
    let perft_nps = 4_300_000; // Misurato in precedenza
    let search_nps = nodes_per_sec;
    let overhead_factor = perft_nps / search_nps;

    println!("Perft NPS: {}", perft_nps);
    println!("Search NPS: {}", search_nps);
    println!("Overhead factor: {}x", overhead_factor);
    println!();

    // Stima tempo per nodo
    let ns_per_node = 1_000_000_000 / search_nps;
    let ns_per_node_perft = 1_000_000_000 / perft_nps;
    let overhead_ns = ns_per_node - ns_per_node_perft;

    println!("=== TIME PER NODE BREAKDOWN ===");
    println!("Total time/node (search): {} ns", ns_per_node);
    println!("Move gen time/node (perft): {} ns", ns_per_node_perft);
    println!("Search overhead: {} ns ({:.1}%)",
        overhead_ns,
        (overhead_ns as f64 / ns_per_node as f64) * 100.0);
    println!();

    println!("=== HYPOTHESIS ===");
    println!("The search overhead (~{} ns/node) is spent on:", overhead_ns);
    println!("  - Evaluation: ~{} ns (estimated 40%)", (overhead_ns as f64 * 0.40) as u64);
    println!("  - TT operations: ~{} ns (estimated 20%)", (overhead_ns as f64 * 0.20) as u64);
    println!("  - Move ordering: ~{} ns (estimated 15%)", (overhead_ns as f64 * 0.15) as u64);
    println!("  - Search logic: ~{} ns (estimated 15%)", (overhead_ns as f64 * 0.15) as u64);
    println!("  - Other: ~{} ns (estimated 10%)", (overhead_ns as f64 * 0.10) as u64);
    println!();
    println!("Next step: Instrument evaluation function to confirm.");
}
