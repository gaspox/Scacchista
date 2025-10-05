use clap::Parser;
use scacchista::board::{Board, START_FEN, PieceKind, Color};
use shakmaty::{Chess, Position};
use shakmaty::fen::Fen;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(START_FEN))]
    fen: String,
    #[arg(short, long, default_value_t = 4)]
    depth: u8,
}
fn perft_scacchista(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 { return 1; }
    let moves = board.generate_moves();
    let mut nodes = 0u64;
    for mv in moves {
        let undo = board.make_move(mv);
        nodes += perft_scacchista(board, depth-1);
        board.unmake_move(undo);
    }
    nodes
}
fn perft_shakmaty(pos: &Chess, depth: u8) -> u64 {
    if depth == 0 { return 1; }
    let mut nodes = 0;
    for m in pos.legal_moves() {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        nodes += perft_shakmaty(&new_pos, depth-1);
    }
    nodes
}

fn main() {
    scacchista::init();
    // Force initialization of attack tables
    scacchista::utils::init_attack_tables();
    scacchista::zobrist::init_zobrist();

    let args = Args::parse();

    println!("Running perft on FEN: '{}' at depth {}", args.fen, args.depth);

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
    let nodes_sc = perft_scacchista(&mut board, args.depth);
    let dur_sc = start.elapsed();

    println!("Shakmaty perft({}) = {} nodes ({} ms, {:.2} Mnps)", args.depth, nodes_sh, dur_sh.as_millis(), nodes_sh as f64 / (dur_sh.as_micros() as f64));
    println!("Scacchista perft({}) = {} nodes ({} ms, {:.2} Mnps)", args.depth, nodes_sc, dur_sc.as_millis(), nodes_sc as f64 / (dur_sc.as_micros() as f64));
    if nodes_sh == nodes_sc {
        println!("✅ Counts match!");
    } else {
        println!("❌ Mismatch! difference = {}", (nodes_sc as i64) - (nodes_sh as i64));
    }
}