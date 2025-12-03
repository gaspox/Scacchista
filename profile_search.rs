// Profiling binary per identificare bottleneck reali
use scacchista::board::Board;
use std::time::Instant;

fn main() {
    println!("=== SCACCHISTA PROFILING ===\n");

    // Test 1: Profiling generate_moves
    {
        let mut board = Board::new();
        let _ = board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let iterations = 100_000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = board.generate_moves();
        }

        let elapsed = start.elapsed();
        let per_call = elapsed.as_nanos() / iterations;

        println!("generate_moves() performance:");
        println!("  Total time: {:?}", elapsed);
        println!("  Per call: {} ns", per_call);
        println!("  Calls/sec: {:.0}", 1_000_000_000.0 / per_call as f64);
    }

    println!();

    // Test 2: Profiling make_move + unmake_move
    {
        let mut board = Board::new();
        let _ = board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let moves = board.generate_moves();
        if !moves.is_empty() {
            let mv = moves[0];

            let iterations = 1_000_000;
            let start = Instant::now();

            for _ in 0..iterations {
                let undo = board.make_move(mv);
                board.unmake_move(undo);
            }

            let elapsed = start.elapsed();
            let per_call = elapsed.as_nanos() / iterations;

            println!("make_move() + unmake_move() performance:");
            println!("  Total time: {:?}", elapsed);
            println!("  Per call: {} ns", per_call);
            println!("  Calls/sec: {:.0}", 1_000_000_000.0 / per_call as f64);
        }
    }

    println!();

    // Test 3: Profiling is_square_attacked
    {
        let mut board = Board::new();
        let _ = board.set_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

        let iterations = 1_000_000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = board.is_square_attacked(4, scacchista::board::Color::Black); // e1 attacked by black
        }

        let elapsed = start.elapsed();
        let per_call = elapsed.as_nanos() / iterations;

        println!("is_square_attacked() performance:");
        println!("  Total time: {:?}", elapsed);
        println!("  Per call: {} ns", per_call);
        println!("  Calls/sec: {:.0}", 1_000_000_000.0 / per_call as f64);
    }

    println!();

    // Test 4: Perft con timing per funzione
    {
        let mut board = Board::new();
        let _ = board.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        println!("Running perft depth 5 with instrumentation...");
        let start = Instant::now();
        let nodes = perft_instrumented(&mut board, 5);
        let elapsed = start.elapsed();

        println!("  Nodes: {}", nodes);
        println!("  Time: {:?}", elapsed);
        println!("  Nodes/sec: {:.0}", nodes as f64 / elapsed.as_secs_f64());
    }
}

fn perft_instrumented(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = board.generate_moves();
    let mut nodes = 0u64;

    for mv in moves {
        let undo = board.make_move(mv);
        nodes += perft_instrumented(board, depth - 1);
        board.unmake_move(undo);
    }

    nodes
}
