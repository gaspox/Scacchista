use scacchista::board::{Board, Move};

// Very basic move parser for testing (UCI string to Move)
fn parse_move(board: &Board, uci: &str) -> Move {
    let mut b_clone = board.clone();
    let moves = b_clone.generate_moves();

    for m in moves {
        if scacchista::board::move_to_uci(m) == uci {
            return m;
        }
    }
    panic!("Move {} not found or illegal", uci);
}

#[test]
fn test_threefold_repetition_simple() {
    scacchista::init();
    let mut board = Board::new();
    board.set_from_fen(scacchista::board::START_FEN).unwrap();
    // Startpos

    // 1. Nf3 Nf6 2. Ng1 Ng8 3. Nf3 Nf6 4. Ng1 Ng8 -> Draw claimable
    let moves = [
        "g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1", "f6g8",
    ];
    // Apply moves and check is_draw()

    for (i, m_str) in moves.iter().enumerate() {
        let m = parse_move(&board, m_str);
        board.make_move(m);

        let is_draw = board.is_draw();

        if i >= 7 {
            // End of sequence
            assert!(is_draw, "Should be draw by repetition");
        }
    }
}

#[test]
fn test_50_move_rule() {
    scacchista::init();
    let mut board = Board::new();
    board
        .set_from_fen("8/8/8/8/8/8/1R6/k6K w - - 99 1")
        .unwrap();
    assert!(!board.is_draw(), "99 halfmoves is not yet draw");

    let m = parse_move(&board, "b2b3");
    board.make_move(m);

    assert!(
        board.is_draw(),
        "100 halfmoves should be draw (50 move rule)"
    );
}

#[test]
fn test_insufficient_material() {
    scacchista::init();
    let mut board = Board::new();

    // K vs K
    board.set_from_fen("8/8/8/8/8/8/8/k6K w - - 0 1").unwrap();
    assert!(board.is_insufficient_material(), "K vs K");
    assert!(board.is_draw(), "K vs K is draw");

    // K+N vs K
    board.set_from_fen("8/8/8/8/8/8/5N2/k6K w - - 0 1").unwrap();
    assert!(board.is_insufficient_material(), "K+N vs K");

    // K+B vs K
    board.set_from_fen("8/8/8/8/8/8/5B2/k6K w - - 0 1").unwrap();
    assert!(board.is_insufficient_material(), "K+B vs K");

    // K+B vs K+B (same color)
    board
        .set_from_fen("8/8/8/8/8/4b3/5B2/k6K w - - 0 1")
        .unwrap();
    assert!(
        board.is_insufficient_material(),
        "KB vs KB (same color: f2/e3)"
    );

    // K+B vs K+B (opposite color)
    board
        .set_from_fen("8/8/8/8/8/5b2/5B2/k6K w - - 0 1")
        .unwrap();
    assert!(
        !board.is_insufficient_material(),
        "KB vs KB (opposite color: f2/f3) is NOT insufficient material"
    );
}
