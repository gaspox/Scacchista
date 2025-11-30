use clap::Parser;
use scacchista::board::Move as MoveType;
use scacchista::board::{move_from_sq, move_piece, move_to_sq, Board, Color, PieceKind, START_FEN}; // explicit alias for type in function signatures

use shakmaty::fen::Fen;
use shakmaty::{Chess, Position}; // used when parsing non-start FEN

use std::vec::Vec; // ensure Vec<Move> available in signature

// (move type is u32 alias in crate)

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(START_FEN))]
    fen: String,
    #[arg(short, long, default_value_t = 4)]
    depth: u8,
    #[arg(long, default_value_t = false)]
    divide: bool,
}
fn perft_scacchista(board: &mut Board, depth: u8, path: &mut Vec<MoveType>) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = board.generate_moves();
    let mut nodes = 0u64;
    for mv in moves {
        path.push(mv);
        let undo = board.make_move(mv);

        // After make: inspect pseudo and legal moves from this position
        let mut pseudo: Vec<scacchista::board::Move> = Vec::with_capacity(256);
        board.generate_pseudo_moves(&mut pseudo);
        let pseudo_len = pseudo.len();
        let legal_len = board.generate_moves().len();

        let child = perft_scacchista(board, depth - 1, path);
        nodes += child;
        if depth == 4 {
            let from = move_from_sq(mv);
            let to = move_to_sq(mv);
            let piece = move_piece(mv);
            if child == 0 {
                eprintln!("==== ROOT MOVE {}→{} ({:?}) -> {} nodes (pseudo={}, legal={}) - DETAILED DUMP ====", from, to, piece, child, pseudo_len, legal_len);
                // Print path
                eprintln!("Path (root->current):");
                for (i, pm) in path.iter().enumerate() {
                    eprintln!(
                        "  {}: {}->{} ({:?})",
                        i + 1,
                        move_from_sq(*pm),
                        move_to_sq(*pm),
                        move_piece(*pm)
                    );
                }
                // Print board position after make
                eprintln!("FEN-like board:\n{}", board);
                eprintln!("Zobrist: 0x{:x}", board.recalc_zobrist());
                eprintln!(
                    "white_occ={:x} black_occ={:x} occ={:x}",
                    board.white_occ, board.black_occ, board.occ
                );

                // Dump pseudo moves
                eprintln!("Pseudo moves (count={}):", pseudo_len);
                for (i, pmv) in pseudo.iter().enumerate() {
                    let pf = move_from_sq(*pmv);
                    let pt = move_to_sq(*pmv);
                    let pp = move_piece(*pmv);
                    eprintln!("  {}: {}->{} ({:?})", i + 1, pf, pt, pp);
                }

                // Dump legal moves from this position
                let legal_moves = board.generate_moves();
                eprintln!("Legal moves (count={}):", legal_moves.len());
                for (i, lm) in legal_moves.iter().enumerate() {
                    let lf = move_from_sq(*lm);
                    let lt = move_to_sq(*lm);
                    let lp = move_piece(*lm);
                    eprintln!("  {}: {}->{} ({:?})", i + 1, lf, lt, lp);
                }

                // Additional invariants: recompute occupancy from piece bitboards
                let mut recomputed_white = 0u64;
                let mut recomputed_black = 0u64;
                for kind in 0..6 {
                    recomputed_white |= board.piece_bb_raw(kind);
                    recomputed_black |= board.piece_bb_raw(kind + 6);
                }
                let recomputed_all = recomputed_white | recomputed_black;
                eprintln!(
                    "Recomputed occ white={:x} black={:x} all={:x}",
                    recomputed_white, recomputed_black, recomputed_all
                );
                if recomputed_white != board.white_occ
                    || recomputed_black != board.black_occ
                    || recomputed_all != board.occ
                {
                    eprintln!("OCCUPANCY MISMATCH detected!");
                }

                // Check king squares and presence
                let wking_sq = board.white_king_sq as usize;
                let bking_sq = board.black_king_sq as usize;
                eprintln!(
                    "White king sq: {} piece_on: {:?}",
                    wking_sq,
                    board.piece_on(wking_sq)
                );
                eprintln!(
                    "Black king sq: {} piece_on: {:?}",
                    bking_sq,
                    board.piece_on(bking_sq)
                );

                eprintln!("==== END DUMP ====");

                // Stop early so we can inspect the output
                std::process::exit(1);
            } else {
                eprintln!(
                    "ROOT MOVE {}->{} ({:?}) -> {} nodes (pseudo={}, legal={})",
                    from, to, piece, child, pseudo_len, legal_len
                );
            }
        }
        board.unmake_move(undo);
        path.pop();
    }
    if depth == 4 {
        eprintln!("TOTAL depth {}: {} nodes", depth, nodes);
    }
    nodes
}
fn perft_shakmaty(pos: &Chess, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    for m in pos.legal_moves() {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        nodes += perft_shakmaty(&new_pos, depth - 1);
    }
    nodes
}

fn perft_simple(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = board.generate_moves();
    let mut nodes = 0u64;
    for mv in moves {
        let undo = board.make_move(mv);
        nodes += perft_simple(board, depth - 1);
        board.unmake_move(undo);
    }
    nodes
}

fn perft_divide(board: &mut Board, depth: u8) {
    use scacchista::board::move_to_uci;

    let moves = board.generate_moves();
    println!("Total legal moves from position: {}", moves.len());
    println!("\nPerft divide at depth {}:", depth);
    println!("{:<10} | Nodes", "Move");
    println!("{:-<10}-+-------", "");

    let mut total = 0u64;
    for mv in moves {
        let undo = board.make_move(mv);
        let count = if depth > 1 {
            perft_simple(board, depth - 1)
        } else {
            1
        };
        board.unmake_move(undo);

        println!("{:<10} : {}", move_to_uci(mv), count);
        total += count;
    }

    println!("\nTotal nodes: {}", total);
}

fn main() {
    scacchista::init();
    // Force initialization of attack tables
    scacchista::utils::init_attack_tables();
    scacchista::zobrist::init_zobrist();

    let args = Args::parse();

    if args.divide {
        // Divide mode: just show per-move breakdown
        let mut board = Board::new();
        board.set_from_fen(&args.fen).unwrap();
        println!("Running perft divide on FEN: {}", args.fen);
        println!("\nBoard:\n{}", board);
        perft_divide(&mut board, args.depth);
        return;
    }

    println!("Running perft on FEN: {} at depth {}", args.fen, args.depth);

    // Shakmaty
    let pos: Chess = if args.fen != START_FEN {
        let fen: Fen = args.fen.parse().unwrap();
        fen.into_position(shakmaty::CastlingMode::Standard).unwrap()
    } else {
        Chess::default()
    };
    let start = std::time::Instant::now();
    let nodes_sh = perft_shakmaty(&pos, args.depth);
    let dur_sh = start.elapsed();

    // Scacchista
    let mut board = Board::new();
    board.set_from_fen(&args.fen).unwrap();
    let start = std::time::Instant::now();
    // Debug: dump piece bitboards at root
    for kind in &[
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ] {
        let wbb = board.piece_bb(*kind, Color::White);
        let bbb = board.piece_bb(*kind, Color::Black);
        eprintln!("piece {:?} white bb: {:x} black bb: {:x}", kind, wbb, bbb);
    }
    let mut path: Vec<MoveType> = Vec::new();
    let nodes_sc = perft_scacchista(&mut board, args.depth, &mut path);

    let dur_sc = start.elapsed();

    println!(
        "Shakmaty perft({}) = {} nodes ({} ms, {:.2} Mnps)",
        args.depth,
        nodes_sh,
        dur_sh.as_millis(),
        nodes_sh as f64 / (dur_sh.as_micros() as f64)
    );
    println!(
        "Scacchista perft({}) = {} nodes ({} ms, {:.2} Mnps)",
        args.depth,
        nodes_sc,
        dur_sc.as_millis(),
        nodes_sc as f64 / (dur_sc.as_micros() as f64)
    );
    if nodes_sh == nodes_sc {
        println!("✅ Counts match!");
    } else {
        println!(
            "Mismatch difference = {}",
            (nodes_sc as i64) - (nodes_sh as i64)
        );
    }
}
