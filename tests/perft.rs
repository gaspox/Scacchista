use scacchista::board::{Board, START_FEN};
use shakmaty::{Chess, Position};

fn perft_shakmaty(pos: &Chess, depth: u8) -> u64 {
    if depth == 0 { return 1; }
    let mut nodes = 0u64;
    for m in pos.legal_moves() {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        nodes += perft_shakmaty(&new_pos, depth-1);
    }
    nodes
}

fn perft_scacchista(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 { return 1; }
    let mut nodes = 0u64;
    let moves = board.generate_moves();
    for mv in moves {
        let undo = board.make_move(mv);
        nodes += perft_scacchista(board, depth-1);
        board.unmake_move(undo);
    }
    nodes
}

#[test]
fn perft_regression_starting_pos() {
    scacchista::init();

    // prepare shakmaty pos
    let pos: Chess = Chess::default();

    let mut board = Board::new();
    board.set_from_fen(START_FEN).expect("set_from_fen");

    for depth in 1..=3u8 {
        let expected = perft_shakmaty(&pos, depth);
        let got = perft_scacchista(&mut board, depth);
        assert_eq!(got, expected, "perft mismatch at depth {}: got {} expected {}", depth, got, expected);
    }
}
