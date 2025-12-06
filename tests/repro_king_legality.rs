use scacchista::board::{Board, PieceKind, move_to_uci};
use shakmaty::{Chess, Position, CastlingMode};
use shakmaty::fen::Fen;

#[test]
fn test_king_cannot_move_into_check() {
    let fen_str = "1r2k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R2BK2R w KQk - 2 2";
    
    // Scacchista
    let mut board = Board::new();
    board.set_from_fen(fen_str).expect("set_from_fen");
    let scacchista_moves = board.generate_moves();
    
    // Shakmaty (Oracle)
    let fen: Fen = fen_str.parse().unwrap();
    let pos: Chess = fen.into_position(CastlingMode::Standard).unwrap();
    let legal_moves: Vec<_> = pos.legal_moves().into_iter().map(|m| m.to_string()).collect();
    
    // Check if Scacchista generated any illegal moves
    let mut illegal_moves = Vec::new();
    println!("Scacchista moves:");
    for &mv in &scacchista_moves {
        let uci = move_to_uci(mv);
        // println!("  {}", uci);
        if scacchista::board::move_piece(mv) == PieceKind::King {
             println!("  King move: {}", uci);
             // Shakmaty uses slightly different UCI for castling (e1g1) vs checks? 
             // Scacchista move_to_uci returns standard UCI (e1g1).
             // Shakmaty to_string returns standard UCI.
             
             if !legal_moves.contains(&uci) {
                 println!("    -> Illegal!");
                 illegal_moves.push(uci);
             }
        }
    }
    println!("Shakmaty legal moves: {:?}", legal_moves);
    
    assert_eq!(illegal_moves.len(), 0, 
        "Scacchista generated illegal king moves not found in Shakmaty: {:?}\nShakmaty legal moves: {:?}", 
        illegal_moves, legal_moves);
}
