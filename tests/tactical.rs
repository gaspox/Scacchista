// Tactical test suite - validates correct evaluation of material and tactical positions
// This suite was created to prevent regression of the double-negation bugs found in
// material_eval() and qsearch() that caused systematic sign inversion.

use scacchista::*;

/// Test that material evaluation is correct from White's perspective after captures
#[test]
fn test_material_after_knight_loss() {
    init();

    let mut board = Board::new();
    // Position after 1.b3 d5 2.c4 c5 3.f3 d4 4.Nc3 dxc3
    // Black pawn captured White knight - White should be down material
    board
        .set_from_fen("rnbqkbnr/pp2pppp/8/2p5/2P5/1Pp2P2/P2PP1PP/R1BQKBNR w KQkq - 0 5")
        .unwrap();

    let (_, score) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(5))
            .search(Some(5));

    // After losing a knight (320 points), White should have negative score
    // Tolerance: we expect roughly -320 +/- some positional compensation
    // With optimizations enabled, shallow depths may give less accurate scores
    assert!(
        score < -50,
        "After losing knight, White's score should be negative. Got: {}",
        score
    );
}

/// Test that material evaluation is correct after Queen loss
#[test]
fn test_material_after_queen_loss() {
    init();

    let mut board = Board::new();
    // Position after the game sequence where White lost the Queen on d4
    // 1.b3 d5 2.c4 c5 3.f3 d4 4.Nc3 dxc3 5.dxc3 Qa5 6.Qd4 Nc6 7.h3 Nxd4
    board
        .set_from_fen("r1b1kbnr/pp2pppp/8/q1p5/2Pn4/1PP2P1P/P3P1P1/R1B1KBNR w KQkq - 0 8")
        .unwrap();

    let (_, score) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(3))
            .search(Some(3));

    // After losing Queen (900 points), White should have very negative score
    // Expected: roughly -900 + knight captured (320) = -580 or so
    assert!(
        score < -400,
        "After losing Queen, White's score should be strongly negative. Got: {}",
        score
    );
}

/// Test starting position has roughly equal evaluation
#[test]
fn test_starting_position_balanced() {
    init();

    let mut board = Board::new();
    board
        .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    let (_, score) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(3))
            .search(Some(3));

    // Starting position should have score close to 0 (within +/- 200 for positional factors)
    assert!(
        score.abs() < 200,
        "Starting position should have near-zero score. Got: {}",
        score
    );
}

/// Test that White being up a pawn gives positive score
#[test]
fn test_white_up_pawn() {
    init();

    let mut board = Board::new();
    // White captured Black's d5 pawn with knight
    board
        .set_from_fen("rnbqkbnr/ppp1pppp/8/3N4/8/8/PPPPPPPP/R1BQKBNR b KQkq - 0 2")
        .unwrap();

    let (_, score) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(3))
            .search(Some(3));

    // White is up a pawn, score should be positive (roughly +100)
    assert!(
        score > 50,
        "White up a pawn should have positive score. Got: {}",
        score
    );
}

/// Test that Black being up a pawn gives negative score (from White's perspective)
#[test]
fn test_black_up_pawn() {
    init();

    let mut board = Board::new();
    // Black captured White's e4 pawn - White has 7 pawns, Black has 8
    board
        .set_from_fen("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
        .unwrap();

    let (_, score) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(3))
            .search(Some(3));

    // Black is up a pawn, score should be negative from White's perspective
    assert!(
        score < -50,
        "Black up a pawn should have negative score (from White's perspective). Got: {}",
        score
    );
}

/// Test that capturing a hanging piece is preferred
#[test]
fn test_captures_hanging_piece() {
    init();

    let mut board = Board::new();
    // Simplified position: Black knight on d4 is hanging, White can capture with c3 pawn
    board
        .set_from_fen("r1bqkbnr/pppppppp/8/8/3n4/2P5/PP1PPPPP/RNBQKBNR w KQkq - 0 3")
        .unwrap();

    let (best_move, score) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(5))
            .search(Some(5));

    // Should capture the hanging knight
    let from = move_from_sq(best_move);
    let to = move_to_sq(best_move);

    // c3 pawn (from square 18) captures d4 knight (to square 27)
    // Or any other piece captures on d4
    assert!(
        to == 27, // d4 square
        "Should capture hanging knight on d4. Best move: {} to {}, score: {}",
        from,
        to,
        score
    );
}

/// Regression test: Ensure negamax negation is consistent
/// This test validates the fix for the double-negation bug in qsearch calls
#[test]
fn test_negamax_sign_consistency() {
    init();

    let mut board = Board::new();
    // Position where material is clearly imbalanced
    board
        .set_from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    let (_, score_white) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(2))
            .search(Some(2));

    // White is up a knight, should have positive score
    assert!(
        score_white > 200,
        "White up a knight should be positive. Got: {}",
        score_white
    );

    // Now flip the position (Black up a knight)
    board
        .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR w KQkq - 0 1")
        .unwrap();

    let (_, score_black) =
        search::Search::new(board.clone(), 16, search::SearchParams::new().max_depth(2))
            .search(Some(2));

    // Black is up a knight, White's score should be negative
    assert!(
        score_black < -200,
        "Black up a knight should give negative score (from White's perspective). Got: {}",
        score_black
    );

    // The scores should be roughly opposite (within some tolerance for positional differences)
    // Note: PSQT introduce asymmetries, so we allow a larger tolerance (150 cp) compared to pure material eval
    let diff = (score_white + score_black).abs();
    assert!(
        diff < 150,
        "Scores for symmetric positions should be roughly opposite. Difference: {}, score_white={}, score_black={}",
        diff, score_white, score_black
    );
}
