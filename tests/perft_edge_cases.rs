use scacchista::board::Board;
use shakmaty::fen::Fen;
use shakmaty::{Chess, Position};

fn run_perft_check(fen_str: &str, depth: u8, name: &str) {
    let mut board = Board::new();
    board.set_from_fen(fen_str).expect("Valid FEN");

    // Scacchista moves count
    let scacchista_cnt = scacchista_perft(&mut board, depth);

    // Shakmaty moves count
    let fen: Fen = fen_str.parse().unwrap();
    let pos: Chess = fen
        .into_position(shakmaty::CastlingMode::Standard)
        .expect("Shakmaty should accept FEN");
    let shakmaty_cnt = shakmaty_perft(&pos, depth);

    assert_eq!(
        scacchista_cnt, shakmaty_cnt,
        "Mismatch in {name} at depth {depth}"
    );
}

// Recursive perft
fn scacchista_perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    for m in board.generate_moves() {
        let undo = board.make_move(m);
        nodes += scacchista_perft(board, depth - 1);
        board.unmake_move(undo);
    }
    nodes
}

fn shakmaty_perft(pos: &Chess, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    for m in pos.legal_moves() {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        nodes += shakmaty_perft(&new_pos, depth - 1);
    }
    nodes
}

#[test]
fn perft_en_passant_discovered_check() {
    scacchista::init();
    // Valid EP case: White pawn on d5, black just moved c7->c5.
    // The King setup must be legal (not in check from move that didn't happen).
    let fen = "8/8/8/k1pP4/8/8/8/4K3 w - c6 0 1";
    run_perft_check(fen, 3, "En Passant Discovered Check");
}

#[test]
fn perft_castling_prevented_by_attack() {
    scacchista::init();
    // 4k3/8/8/8/8/8/8/R3K2r w Q - 0 1 -> King is in check, cannot castle
    // (Ensure black king is not in check by white in starting position)
    let fen = "4k3/8/8/8/8/8/8/R3K2r w Q - 0 1";
    run_perft_check(fen, 2, "Castling in Check (Illegal)");
}

#[test]
fn perft_castling_through_check() {
    scacchista::init();
    // f1 is attacked by black rook
    let fen = "4k3/8/8/8/8/5r2/8/R3K2R w KQ - 0 1";
    run_perft_check(fen, 2, "Castling Through Check");
}

#[test]
fn perft_promotion_capture() {
    scacchista::init();
    // Promote and capture at the same time. Move kings apart.
    let fen = "n1n5/P5P1/8/2k5/8/8/8/4K3 w - - 0 1";
    run_perft_check(fen, 2, "Promotion Capture");
}

#[test]
fn perft_double_check_response() {
    scacchista::init();
    // Complex position
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    run_perft_check(fen, 2, "Complex Check Response");
}
