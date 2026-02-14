use scacchista::board::{move_to_uci, Board};
use shakmaty::fen::Fen;
use shakmaty::{Chess, Position};
use std::collections::HashSet;

fn _scacchista_perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    for m in board.generate_moves() {
        let undo = board.make_move(m);
        nodes += _scacchista_perft(board, depth - 1);
        board.unmake_move(undo);
    }
    nodes
}

fn _shakmaty_perft(pos: &Chess, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    for m in pos.legal_moves() {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        nodes += _shakmaty_perft(&new_pos, depth - 1);
    }
    nodes
}

#[test]
fn debug_pos4_deep_dive() {
    scacchista::init();

    // Position 4
    let fen_str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    println!("Deep dive into Position 4 after c4c5");

    // Scacchista
    let mut board = Board::new();
    board.set_from_fen(fen_str).expect("Valid FEN");

    // Make c4c5
    // Need to find the move object first
    let moves = board.generate_moves();
    let m = moves
        .into_iter()
        .find(|m| move_to_uci(*m) == "c4c5")
        .expect("c4c5 not found");
    board.make_move(m);

    let s_moves = board.generate_moves();
    let s_set: HashSet<String> = s_moves.iter().map(|&m| move_to_uci(m)).collect();

    // Shakmaty
    let fen: Fen = fen_str.parse().unwrap();
    let pos: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

    // Make c4c5
    let m_sh = pos
        .legal_moves()
        .into_iter()
        .find(|m| m.to_uci(shakmaty::CastlingMode::Standard).to_string() == "c4c5")
        .expect("c4c5 not found in shakmaty");

    let mut new_pos = pos.clone();
    new_pos.play_unchecked(&m_sh);

    let sh_moves = new_pos.legal_moves();
    let sh_set: HashSet<String> = sh_moves
        .iter()
        .map(|m| m.to_uci(shakmaty::CastlingMode::Standard).to_string())
        .collect();

    // Diff
    println!("S count: {} SH count: {}", s_set.len(), sh_set.len());

    let extra_s: Vec<_> = s_set.difference(&sh_set).collect();
    for m in extra_s {
        println!("EXTRA S: {}", m);
    }

    let extra_sh: Vec<_> = sh_set.difference(&s_set).collect();
    for m in extra_sh {
        println!("MISSED S: {}", m);
    }
}
