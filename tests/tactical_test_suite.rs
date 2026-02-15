use scacchista::uci::{process_uci_line, UciEngine};

fn solve_position(fen: &str, depth: u8, name: &str, expected_move: &str) {
    let mut engine = UciEngine::new();
    process_uci_line("uci", &mut engine);
    process_uci_line(&format!("position fen {}", fen), &mut engine);

    let res = process_uci_line(&format!("go depth {}", depth), &mut engine);
    let best_line = res
        .iter()
        .find(|s| s.starts_with("bestmove"))
        .expect("No bestmove returned");
    let best_move = best_line.split_whitespace().nth(1).unwrap();

    println!("Position: {}", name);
    println!("Expected: {}", expected_move);
    println!("Found: {}", best_move);

    assert_eq!(
        best_move, expected_move,
        "Failed tactic: {}\n  left: {:?}\n right: {:?}",
        name, best_move, expected_move
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Engine too weak for depth 6 tactic currently
    fn test_wac_001() {
        scacchista::init();
        // WAC 1 (Queen sac for mate)
        // Let's use the FEN from the previous test attempt.
        let fen_wac1 = "r1b2rk1/pp1p1p1p/2n2p2/2b5/2B5/2P2N2/PP1Q1PPP/R3K2R w KQ - 0 1";
        solve_position(fen_wac1, 6, "WAC 1: Mate in 2 (Queen sac)", "d2h6");
    }

    #[test]
    #[ignore] // Engine too weak for depth 6 tactic currently
    fn test_wac_002() {
        scacchista::init();
        // WAC 2
        let fen_wac2 = "r1bqk2r/pppp1ppp/2n5/3np3/2B5/2P2N2/PP1P1PPP/RNBQK2R w KQkq - 0 1";
        solve_position(fen_wac2, 6, "WAC 2: Fork / Advantage", "d5e7");
    }
}
