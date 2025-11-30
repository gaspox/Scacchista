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
