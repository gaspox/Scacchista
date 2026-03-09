use scacchista::board::Board;
use scacchista::eval::evaluate;

#[test]
fn test_material_difference() {
    scacchista::init();
    let mut board = Board::new();

    // Equal position
    board.set_from_fen(scacchista::board::START_FEN).unwrap();
    let score_initial = evaluate(&board);
    assert_eq!(score_initial, 0, "Initial position should be 0");

    // White pawn up
    board
        .set_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
        .unwrap();
    // Material is same, but e4 pawn might give positional bonus.
    // Let's use a capture to be sure.
    board
        .set_from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
        .unwrap();
    // White captures d5? No, simpler:
    board
        .set_from_fen("rnbqkbnr/pppp1ppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap(); // Black missing a pawn
    let score_pawn_up = evaluate(&board);
    assert!(score_pawn_up > 50, "White should be up material");
}

#[test]
fn test_knight_psqt() {
    scacchista::init();
    let mut board = Board::new();

    // Knight in corner
    board
        .set_from_fen("N7/8/8/4k3/4K3/8/8/8 w - - 0 1")
        .unwrap();
    let score_corner = evaluate(&board);

    // Knight in center
    board
        .set_from_fen("8/8/8/3Nk3/4K3/8/8/8 w - - 0 1")
        .unwrap();
    let score_center = evaluate(&board);

    assert!(
        score_center > score_corner,
        "Knight in center ({}) should be better than corner ({})",
        score_center,
        score_corner
    );
}

#[test]
fn test_passed_pawn_bonus() {
    scacchista::init();
    let mut board = Board::new();

    // Passed pawn on 2nd rank
    board.set_from_fen("8/8/8/k7/K7/8/1P6/8 w - - 0 1").unwrap();
    let score_low = evaluate(&board);

    // Passed pawn on 7th rank
    board.set_from_fen("8/1P6/8/k7/K7/8/8/8 w - - 0 1").unwrap();
    let score_high = evaluate(&board);

    assert!(
        score_high > score_low + 100,
        "Pawn on 7th rank ({}) should have substantial bonus over 2nd rank ({})",
        score_high,
        score_low
    );
}

#[test]
fn test_king_safety_castled() {
    scacchista::init();
    let mut board = Board::new();

    // Castled king (safe behind pawns)
    board
        .set_from_fen("r1bq1rk1/pppp1ppp/2n2n2/4p3/2B1P3/2N2N2/PPPP1PPP/R1BQ1RK1 w - - 0 1")
        .unwrap();
    let score_castled = evaluate(&board);

    // Unsafe king (open files, pawns gone)
    // Board roughly similar material but white king exposed
    // We modify just the white king surroundings
    board
        .set_from_fen("r1bq1rk1/pppp1ppp/2n2n2/4p3/2B1P3/2N2N2/PPPP4/R1BQ1RK1 w - - 0 1")
        .unwrap();

    let score_uncastled = evaluate(&board);

    assert!(
        score_castled > score_uncastled,
        "Castled king should be evaluated higher than uncastled in opening. Castled={}, Uncastled={}",
        score_castled,
        score_uncastled
    );
}
