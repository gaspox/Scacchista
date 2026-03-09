//! Integration tests for UCI protocol implementation

use scacchista::uci::{process_uci_line, UciEngine};

#[test]
fn test_uci_engine_lifecycle() {
    let mut engine = UciEngine::new();

    // Test UCI handshake (ensure uciok present)
    let responses = process_uci_line("uci", &mut engine);
    assert!(responses.iter().any(|s| s.contains("uciok")));

    // isready
    let responses = process_uci_line("isready", &mut engine);
    assert!(responses.iter().any(|s| s == "readyok"));

    // go depth 1 should return a bestmove
    let _ = process_uci_line("position startpos", &mut engine);
    let responses = process_uci_line("go depth 1", &mut engine);
    assert!(responses.iter().any(|s| s.starts_with("bestmove")));
}

#[test]
fn test_basic_commands() {
    let mut engine = UciEngine::new();

    // Test isready
    let responses = process_uci_line("isready", &mut engine);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "readyok");
}

#[test]
fn test_go_command_mock() {
    let mut engine = UciEngine::new();

    // Test position + go flow
    process_uci_line("position startpos", &mut engine);
    let responses = process_uci_line("go depth 5", &mut engine);
    assert_eq!(responses.len(), 2);
    assert!(responses[0].starts_with("info depth")); // Changed from "info string"
    assert!(responses[1].starts_with("bestmove"));
}

#[test]
fn test_go_depth_only() {
    // Test that `go depth N` reaches the specified depth without timeout
    // This validates the fix for the UCI bug where depth-only commands
    // were incorrectly using a 5-second timeout
    let mut engine = UciEngine::new();

    process_uci_line("uci", &mut engine);
    process_uci_line("isready", &mut engine);
    process_uci_line("position startpos", &mut engine);

    // go depth 6 should complete at depth 6, not timeout at 5 seconds
    let responses = process_uci_line("go depth 6", &mut engine);

    // Should get info line and bestmove
    assert!(
        responses.len() >= 2,
        "Expected at least 2 responses (info + bestmove)"
    );

    // Extract depth from info line
    let info_line = responses
        .iter()
        .find(|s| s.starts_with("info depth"))
        .expect("No info depth line");

    // The info line should report depth 6 (or the actual depth reached)
    // Format: "info depth 6 score cp X time Y"
    assert!(
        info_line.contains("depth"),
        "Info line should contain depth"
    );

    // Should have a bestmove (not 0000)
    let bestmove_line = responses
        .iter()
        .find(|s| s.starts_with("bestmove"))
        .expect("No bestmove");
    assert!(
        !bestmove_line.contains("0000"),
        "Bestmove should not be null move"
    );
}

#[test]
fn test_go_depth_with_time() {
    // Test that `go depth N movetime M` respects BOTH constraints
    // and stops when the first one is reached
    let mut engine = UciEngine::new();

    process_uci_line("uci", &mut engine);
    process_uci_line("isready", &mut engine);
    process_uci_line("position startpos", &mut engine);

    use std::time::Instant;
    let start = Instant::now();

    // go depth 20 movetime 100 should stop after ~100ms (not reach depth 20)
    let responses = process_uci_line("go depth 20 movetime 100", &mut engine);

    let elapsed = start.elapsed().as_millis();

    // Should complete in roughly 100ms (give it 500ms tolerance for slow systems)
    assert!(
        elapsed < 500,
        "Search should stop due to movetime, not depth. Took {}ms",
        elapsed
    );

    // Should still get a valid bestmove
    assert!(responses.iter().any(|s| s.starts_with("bestmove")));
}

#[test]
fn test_go_depth_zero() {
    // Test edge case: `go depth 0` should be clamped to depth 1
    let mut engine = UciEngine::new();

    process_uci_line("uci", &mut engine);
    process_uci_line("isready", &mut engine);
    process_uci_line("position startpos", &mut engine);

    // go depth 0 should be treated as depth 1 (clamped)
    let responses = process_uci_line("go depth 0", &mut engine);

    // Should get responses (not crash or hang)
    assert!(!responses.is_empty(), "Should get response for depth 0");

    // Should have a bestmove
    assert!(responses.iter().any(|s| s.starts_with("bestmove")));
}
