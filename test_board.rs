use scacchista::board::{Board, START_FEN};

fn main() {
    scacchista::init();
    scacchista::utils::init_attack_tables();

    let mut board = Board::new();
    board.set_from_fen(START_FEN).unwrap();

    println!("Board FEN: {}", START_FEN);
    println!("Side to move: {:?}", board.side);

    // Check white pawn positions
    let white_pawns = board.piece_bb(
        scacchista::board::PieceKind::Pawn,
        scacchista::board::Color::White,
    );
    println!("White pawn bitboard: {:064b}", white_pawns);

    // Print white pawn squares
    let mut wp = white_pawns;
    while let Some(sq) = scacchista::utils::pop_lsb(&mut wp) {
        let file = (sq % 8) as u8;
        let rank = (7 - (sq / 8)) as u8; // Convert to chess notation
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        println!(
            "White pawn at {}{} (internal index: {})",
            file_char, rank_char, sq
        );
    }

    // Check black pawn positions
    let black_pawns = board.piece_bb(
        scacchista::board::PieceKind::Pawn,
        scacchista::board::Color::Black,
    );
    println!("Black pawn bitboard: {:064b}", black_pawns);

    // Print black pawn squares
    let mut bp = black_pawns;
    while let Some(sq) = scacchista::utils::pop_lsb(&mut bp) {
        let file = (sq % 8) as u8;
        let rank = (7 - (sq / 8)) as u8; // Convert to chess notation
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        println!(
            "Black pawn at {}{} (internal index: {})",
            file_char, rank_char, sq
        );
    }

    // Test move generation
    let moves = board.generate_moves();
    println!("\nGenerated {} moves", moves.len());

    for (i, &mv) in moves.iter().take(5).enumerate() {
        let from = scacchista::board::move_from_sq(mv);
        let to = scacchista::board::move_to_sq(mv);
        let piece = scacchista::board::move_piece(mv);

        let from_file = (from % 8) as u8;
        let from_rank = (7 - (from / 8)) as u8;
        let to_file = (to % 8) as u8;
        let to_rank = (7 - (to / 8)) as u8;

        let from_file_char = (b'a' + from_file) as char;
        let from_rank_char = (b'1' + from_rank) as char;
        let to_file_char = (b'a' + to_file) as char;
        let to_rank_char = (b'1' + to_rank) as char;

        println!(
            "Move {}: {:?} {}{} -> {}{}",
            i, piece, from_file_char, from_rank_char, to_file_char, to_rank_char
        );
    }

    if moves.len() > 5 {
        println!("... and {} more", moves.len() - 5);
    }
}
