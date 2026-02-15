use scacchista::board::{Board, Color, PieceKind};
use scacchista::eval::evaluate_fast;

fn main() {
    scacchista::init();

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR w KQkq - 0 1";
    println!("Loading FEN: {}", fen);

    let mut board = Board::new();
    board.set_from_fen(fen).unwrap();

    println!("Board:\n{}", board);

    println!("Side to move: {:?}", board.side);

    let fast_eval = evaluate_fast(&board);
    println!("evaluate_fast: {}", fast_eval);

    // Dump material counts manually
    let mut w_mat = 0;
    let mut b_mat = 0;

    for kind in [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ] {
        let w_bb = board.piece_bb(kind, Color::White);
        let b_bb = board.piece_bb(kind, Color::Black);
        let w_count = w_bb.count_ones();
        let b_count = b_bb.count_ones();

        println!("{:?}: W={} B={}", kind, w_count, b_count);

        // Rough values
        let val = match kind {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 20000,
        };
        w_mat += w_count as i32 * val;
        b_mat += b_count as i32 * val;
    }

    println!("Manual White Mat: {}", w_mat);
    println!("Manual Black Mat: {}", b_mat);
    println!("Manual Diff: {}", w_mat - b_mat);
}
