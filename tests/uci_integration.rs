//! Integration tests for UCI protocol implementation

use scacchista::uci::{UciEngine, UciState, process_uci_line};

#[test]
fn test_uci_engine_lifecycle() {
    let mut engine = UciEngine::new();

    // Test initial state
    assert_eq!(*engine.state(), UciState::Init);

    // Test UCI handshake
    let responses = process_uci_line("uci", &mut engine);
    assert_eq!(responses.len(), 3);
    assert!(responses[0].contains("id name"));
    assert!(responses[1].contains("id author"));
    assert_eq!(responses[2], "uciok");
    assert_eq!(*engine.state(), UciState::Ready);
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
    assert!(responses[0].starts_with("info string"));
    assert!(responses[1].starts_with("bestmove"));
}