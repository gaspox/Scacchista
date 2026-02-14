use scacchista::uci::{process_uci_line, UciEngine};

#[test]
fn test_uci_handshake() {
    scacchista::init();
    let mut engine = UciEngine::new();

    let res = process_uci_line("uci", &mut engine);
    assert!(res.contains(&"uciok".to_string()));
    assert!(res.iter().any(|s| s.starts_with("id name")));

    let res = process_uci_line("isready", &mut engine);
    assert!(res.contains(&"readyok".to_string()));
}

#[test]
fn test_go_infinite_stop() {
    scacchista::init();
    let mut engine = UciEngine::new();

    // Setup position
    process_uci_line("uci", &mut engine);
    process_uci_line("position startpos", &mut engine);

    // Go infinite
    let res_start = process_uci_line("go infinite", &mut engine);
    // Should not return bestmove yet
    assert!(!res_start.iter().any(|s| s.starts_with("bestmove")));

    // Wait a bit to let it think (in real life GUI would wait)
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Stop
    let res_stop = process_uci_line("stop", &mut engine);

    // Should return bestmove now
    assert!(
        res_stop.iter().any(|s| s.starts_with("bestmove")),
        "Stop command should trigger bestmove response"
    );
    // Should verify it's not 0000 (null move) unless completely broken
    let best = res_stop.iter().find(|s| s.starts_with("bestmove")).unwrap();
    assert_ne!(best, "bestmove 0000", "Should return a valid move");
}
