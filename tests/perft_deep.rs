use scacchista::board::{Board, START_FEN};
use shakmaty::fen::Fen;
use shakmaty::{Chess, Position};

// Helper function to run perft comparison
fn run_perft_comparison(fen_str: &str, depth: u8, position_name: &str) {
    let mut board = Board::new();
    board.set_from_fen(fen_str).expect("Valid FEN");

    // Scacchista perft
    let scacchista_nodes = scacchista_perft(&mut board, depth);

    // Shakmaty perft (Oracle)
    let fen: Fen = fen_str.parse().unwrap();
    let pos: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();
    let shakmaty_nodes = shakmaty_perft(&pos, depth);

    assert_eq!(
        scacchista_nodes, shakmaty_nodes,
        "Perft mismatch for {} at depth {}: Scacchista={} Shakmaty={}",
        position_name, depth, scacchista_nodes, shakmaty_nodes
    );
}

// Scacchista implementation
fn scacchista_perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    let moves = board.generate_moves();
    for m in moves {
        let undo = board.make_move(m);
        nodes += scacchista_perft(board, depth - 1);
        board.unmake_move(undo);
    }
    nodes
}

// Shakmaty implementation
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
fn perft_startpos_depth_4() {
    scacchista::init();
    run_perft_comparison(START_FEN, 4, "Startpos");
}

#[test]
#[ignore] // expensive
fn perft_startpos_depth_5() {
    scacchista::init();
    run_perft_comparison(START_FEN, 5, "Startpos");
}

#[test]
fn perft_kiwipete_depth_3() {
    scacchista::init();
    let kiwipete = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    run_perft_comparison(kiwipete, 3, "Kiwipete");
}

#[test]
#[ignore] // expensive
fn perft_kiwipete_depth_4() {
    scacchista::init();
    let kiwipete = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    run_perft_comparison(kiwipete, 4, "Kiwipete");
}

#[test]
fn perft_position_3_depth_4() {
    scacchista::init();
    let pos3 = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    run_perft_comparison(pos3, 4, "Position 3");
}

#[test]
fn perft_position_4_depth_3() {
    scacchista::init();
    let pos4 = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    run_perft_comparison(pos4, 3, "Position 4");
}

#[test]
#[ignore] // expensive
fn perft_position_4_depth_4() {
    scacchista::init();
    let pos4 = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    run_perft_comparison(pos4, 4, "Position 4");
}

#[test]
fn perft_position_5_depth_3() {
    scacchista::init();
    let pos5 = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    run_perft_comparison(pos5, 3, "Position 5");
}

#[test]
fn perft_position_6_depth_3() {
    scacchista::init();
    let pos6 = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    run_perft_comparison(pos6, 3, "Position 6");
}
