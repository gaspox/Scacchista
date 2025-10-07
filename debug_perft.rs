use clap::Parser;
use scacchista::board::{Board, START_FEN};
use shakmaty::fen::Fen;
use shakmaty::{Chess, Position};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(START_FEN))]
    fen: String,
    #[arg(short, long, default_value_t = 1)]
    depth: u8,
}

fn perft_debug(board: &mut Board, depth: u8) -> (u64, Vec<(String, u64)>) {
    if depth == 0 {
        return (1, Vec::new());
    }

    let moves = board.generate_moves();
    let mut total_nodes = 0u64;
    let mut move_counts = Vec::new();

    for mv in moves {
        // Save board state and convert to notation after undo
        let from = scacchista::board::move_from_sq(mv);
        let to = scacchista::board::move_to_sq(mv);
        let piece = scacchista::board::move_piece(mv);

        let files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        let ranks = ['1', '2', '3', '4', '5', '6', '7', '8'];

        let piece_char = match piece {
            scacchista::board::PieceKind::Pawn => "",
            scacchista::board::PieceKind::Knight => "N",
            scacchista::board::PieceKind::Bishop => "B",
            scacchista::board::PieceKind::Rook => "R",
            scacchista::board::PieceKind::Queen => "Q",
            scacchista::board::PieceKind::King => "K",
        };

        let mv_str = format!(
            "{}{}{}{}",
            piece_char,
            files[from % 8],
            ranks[7 - (from / 8)],
            files[to % 8]
        );
        let undo = board.make_move(mv);
        let (nodes, _) = perft_debug(board, depth - 1);
        board.unmake_move(undo);

        total_nodes += nodes;
        move_counts.push((mv_str, nodes));
    }

    (total_nodes, move_counts)
}

fn perft_debug_shakmaty(pos: &Chess, depth: u8) -> (u64, Vec<(String, u64)>) {
    if depth == 0 {
        return (1, Vec::new());
    }

    let mut total_nodes = 0u64;
    let mut move_counts = Vec::new();

    for m in pos.legal_moves() {
        let mv_str = format!("{}", m);
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        let (nodes, _) = perft_debug_shakmaty(&new_pos, depth - 1);

        total_nodes += nodes;
        move_counts.push((mv_str, nodes));
    }

    (total_nodes, move_counts)
}

fn main() {
    scacchista::init();
    scacchista::utils::init_attack_tables();

    let args = Args::parse();

    println!(
        "Running debug perft on FEN: '{}' at depth {}",
        args.fen, args.depth
    );

    // Shakmaty
    let pos: Chess = if args.fen != START_FEN {
        let fen: Fen = args.fen.parse().unwrap();
        fen.into_position(shakmaty::CastlingMode::Standard).unwrap()
    } else {
        Chess::default()
    };

    let (nodes_sh, moves_sh) = perft_debug_shakmaty(&pos, args.depth);

    // Scacchista
    let mut board = Board::new();
    board.set_from_fen(&args.fen).unwrap();
    let (nodes_sc, moves_sc) = perft_debug(&mut board, args.depth);

    println!("\nShakmaty results:");
    for (mv, count) in &moves_sh {
        println!("  {}: {}", mv, count);
    }
    println!("Total: {}", nodes_sh);

    println!("\nScacchista results:");
    for (mv, count) in &moves_sc {
        println!("  {}: {}", mv, count);
    }
    println!("Total: {}", nodes_sc);

    if nodes_sh == nodes_sc {
        println!("\n✅ Counts match!");
    } else {
        println!(
            "\n❌ Mismatch! difference = {}",
            (nodes_sc as i64) - (nodes_sh as i64)
        );

        // Find mismatches
        println!("\nMove comparison:");
        let mut sh_counts = std::collections::HashMap::new();
        for (mv, count) in &moves_sh {
            sh_counts.insert(mv.clone(), *count);
        }

        for (mv, sc_count) in &moves_sc {
            if let Some(&sh_count) = sh_counts.get(mv) {
                if sh_count != *sc_count {
                    println!("  {}: Shakmaty={}, Scacchista={}", mv, sh_count, sc_count);
                }
            } else {
                println!("  {}: Shakmaty=missing, Scacchista={}", mv, sc_count);
            }
        }

        for (mv, sh_count) in &moves_sh {
            if !moves_sc.iter().any(|(m, _)| m == mv) {
                println!("  {}: Shakmaty={}, Scacchista=missing", mv, sh_count);
            }
        }
    }
}
