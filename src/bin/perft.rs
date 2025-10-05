use clap::Parser;
use scacchista::board::START_FEN;
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

fn main() {
    let args = Args::parse();

    println!("Running perft on FEN: '{}' at depth {}", args.fen, args.depth);

    let pos: Chess = if args.fen != START_FEN {
        let fen: Fen = args.fen.parse().unwrap();
        fen.into_position(shakmaty::CastlingMode::Standard).unwrap()
    } else {
        Chess::default()
    };

    let start = std::time::Instant::now();
    let nodes = perft_shakmaty(&pos, args.depth);
    let duration = start.elapsed();

    println!("Shakmaty perft({}) = {} nodes ({} ms, {:.2} Mnps)", args.depth, nodes, duration.as_millis(), nodes as f64 / (duration.as_micros() as f64));
}

fn perft_shakmaty(pos: &Chess, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let moves = pos.legal_moves();
    for m in moves {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&m);
        nodes += perft_shakmaty(&new_pos, depth - 1);
    }
    nodes
}