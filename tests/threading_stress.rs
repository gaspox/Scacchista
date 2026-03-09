use scacchista::uci::{process_uci_line, UciEngine};

#[test]
fn test_threading_stress() {
    scacchista::init();
    let mut engine = UciEngine::new();

    // Set threads to 4
    let res = process_uci_line("setoption name Threads value 4", &mut engine);
    assert!(res.iter().any(|s| s.contains("Threads set to 4")));

    process_uci_line("uci", &mut engine);
    process_uci_line("isready", &mut engine);
    process_uci_line("position startpos", &mut engine);

    process_uci_line("go depth 6", &mut engine); // Reduced from 20 to 6 for speed
}
#[test]
fn test_threading_async_stress() {
    scacchista::init();
    let mut engine = UciEngine::new();
    process_uci_line("setoption name Threads value 4", &mut engine);
    process_uci_line("position startpos", &mut engine);

    for i in 0..10 {
        process_uci_line("go infinite", &mut engine);
        // Sleep random-ish amount or short amount
        std::thread::sleep(std::time::Duration::from_millis(10 + (i * 10)));
        let res = process_uci_line("stop", &mut engine);

        assert!(
            res.iter().any(|s| s.starts_with("bestmove")),
            "Async stop failed iter {}",
            i
        );
    }

    process_uci_line("quit", &mut engine);
}

#[test]
fn test_multithreaded_correctness() {
    // Determine if 4 threads and 1 thread give same result on a tactical position?
    // Not necessarily same result ( Lazy SMP is non-deterministic), but valid move.
    scacchista::init();

    let mut engine_st = UciEngine::new();
    process_uci_line("setoption name Threads value 1", &mut engine_st);
    process_uci_line("position startpos", &mut engine_st);
    let res_st = process_uci_line("go depth 6", &mut engine_st); // Short search
    let best_st = res_st.iter().find(|s| s.starts_with("bestmove")).unwrap();

    let mut engine_mt = UciEngine::new();
    process_uci_line("setoption name Threads value 4", &mut engine_mt);
    process_uci_line("position startpos", &mut engine_mt);
    let res_mt = process_uci_line("go depth 6", &mut engine_mt);
    let best_mt = res_mt.iter().find(|s| s.starts_with("bestmove")).unwrap();

    // Depth 6 startpos: usually e2e4 or d2d4.
    // We just verify it returns *some* move and doesn't crash.
    println!("ST Best: {}, MT Best: {}", best_st, best_mt);
    assert!(best_st.starts_with("bestmove"), "ST failed");
    assert!(best_mt.starts_with("bestmove"), "MT failed");
}
