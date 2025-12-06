// Test case for Bug #1: Time Expiration Score Corruption
//
// This test reproduces the critical bug where engine returns suicide moves
// when using time control (go movetime X) instead of depth control.
//
// Bug mechanism:
// 1. Search runs out of time
// 2. negamax_pv() returns alpha (-30000)
// 3. Caller negates it: -(-30000) = +30000
// 4. Engine thinks it found mate and saves that move as best
// 5. Move is actually a blunder (e.g., losing queen for pawn)

use scacchista::board::Board;
use scacchista::search::{Search, SearchParams};

#[test]
fn test_time_expiration_no_fake_mate() {
    scacchista::init();

    // Position from actual game where bug occurred
    // After: 1.d4 Na6 2.c4 d6 3.e4 Qd7 4.Nf3 Qe6 5.Nc3 Nf6 6.d5
    // Black's queen on e6 can be captured by dxe6 (pawn takes queen!)
    let fen = "r1b1kb1r/pp2pppp/n2qnn2/2pP4/2P1P3/2N2N2/PP3PPP/R1BQKB1R b KQkq - 0 6";

    let mut board = Board::new();
    board.set_from_fen(fen).expect("Valid FEN");

    // Use very short time to force timeout during search
    // This should trigger the bug: score corruption when time expires
    let params = SearchParams::new()
        .max_depth(20)  // High depth to ensure it won't complete
        .time_limit(50); // 50ms - will timeout mid-search

    let mut search = Search::new(board, 16, params);
    let (_mv, score) = search.search(Some(20));

    // CRITICAL ASSERTION: Score must NOT be +30000 (fake mate from negated -INFINITE)
    // Valid scores are roughly -3000 to +3000 in this position
    // If we get 30000, it's the bug manifesting
    assert!(
        score < 29000 && score > -29000,
        "Score corruption detected! Score={}, expected < 29000. \
         This indicates timeout is returning -INFINITE which gets negated to +30000",
        score
    );

    // Additional check: move should not be 0 (null move)
    assert_ne!(_mv, 0, "Engine returned null move");
}

#[test]
fn test_movetime_vs_depth_consistency() {
    scacchista::init();

    // Same position, test multiple times with short movetime
    let fen = "r1b1kb1r/pp2pppp/n2qnn2/2pP4/2P1P3/2N2N2/PP3PPP/R1BQKB1R b KQkq - 0 6";

    for iteration in 1..=5 {
        let mut board = Board::new();
        board.set_from_fen(fen).expect("Valid FEN");

        let params = SearchParams::new()
            .max_depth(20)
            .time_limit(100); // 100ms timeout

        let mut search = Search::new(board, 16, params);
        let (_mv, score) = search.search(Some(20));

        // Every iteration should return reasonable score
        assert!(
            score.abs() < 10000,
            "Iteration {}: Score {} is unreasonable (possible corruption)",
            iteration,
            score
        );
    }
}

#[test]
fn test_depth_based_no_corruption() {
    scacchista::init();

    // Control test: depth-based search should work correctly
    let fen = "r1b1kb1r/pp2pppp/n2qnn2/2pP4/2P1P3/2N2N2/PP3PPP/R1BQKB1R b KQkq - 0 6";
    let mut board = Board::new();
    board.set_from_fen(fen).expect("Valid FEN");

    let params = SearchParams::new()
        .max_depth(5)
        .time_limit(0); // No time limit

    let mut search = Search::new(board, 16, params);
    let (mv, score) = search.search(Some(5));

    // With depth-based search, should get reasonable result
    assert!(score.abs() < 10000, "Depth-based search returned bad score: {}", score);
    assert_ne!(mv, 0, "Depth-based search returned null move");

    // Score should be negative (black is losing - queen hanging)
    // or at least not a huge positive (fake mate)
    assert!(score < 5000, "Score {} suggests black is winning, but queen is hanging!", score);
}

#[test]
#[ignore] // Ignore by default, run with --ignored to stress test
fn stress_test_time_control() {
    scacchista::init();

    // Stress test: many iterations with random short times
    // This test is ignored by default but can be run with: cargo test --ignored

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    let mut corrupt_count = 0;
    let iterations = 50;

    for i in 1..=iterations {
        let mut board = Board::new();
        board.set_from_fen(fen).expect("Valid FEN");

        // Random time between 10ms and 200ms
        let time_ms = 10 + (i * 7) % 190;

        let params = SearchParams::new()
            .max_depth(20)
            .time_limit(time_ms);

        let mut search = Search::new(board, 16, params);
        let (_mv, score) = search.search(Some(20));

        if score.abs() >= 29000 {
            corrupt_count += 1;
            eprintln!("Iteration {}: Corrupted score {} with time {}ms", i, score, time_ms);
        }
    }

    assert_eq!(
        corrupt_count, 0,
        "{}/{} iterations had score corruption",
        corrupt_count, iterations
    );
}
