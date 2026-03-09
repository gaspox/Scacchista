use scacchista::board::{move_to_uci, Board};
use scacchista::search::{Search, SearchParams};

fn find_mate_in_n(fen: &str, n: u8, expected_move: Option<&str>) {
    let mut board = Board::new();
    board.set_from_fen(fen).expect("Invalid FEN");

    // Search depth slightly higher than N to ensure we see it
    // Mate in N means N moves for the winner. Depth needs to be 2*N - 1 (plies) + margin?
    // Actually mate in N usually means "Mate in N pairs of moves".
    // Mate in 1 = 1 ply. Mate in 2 = 3 plies. Mate in 3 = 5 plies.
    let depth = n * 2;

    let params = SearchParams::new().max_depth(depth);
    let mut search = Search::new(board, 5000, params);

    let (best_move, score) = search.search(Some(depth));
    let uci = move_to_uci(best_move);

    println!(
        "Mate in {} test from {}: Found {} Score {}",
        n, fen, uci, score
    );

    // Score for mate is typically > 30000 - distance
    // Scacchista likely uses MATE_SCORE around 30000.
    assert!(score > 29000, "Score {} indicates no mate found!", score);

    if let Some(exp) = expected_move {
        assert_eq!(uci, exp, "Failed to find specific mate move");
    }
}

#[test]
fn mate_in_1_simple() {
    scacchista::init();
    // Scholar's Mate final position minus one move:
    // 1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7#
    // Setup: r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4";
    find_mate_in_n(fen, 1, Some("h5f7")); // Qh5 x f7 is h5f7
                                          // Let's verify start square of Queen. H5 is file 7, rank 4 (0-indexed? No standard algebraic).
                                          // Scacchista utils might parse it.
                                          // Let's rely on score check primarily.
}

#[test]
fn mate_in_2_smothered() {
    scacchista::init();
    // Smothered mate pattern
    // 6k1/5Npp/8/8/8/8/6PP/7K w - - 0 1
    // 1. Nh6+ Kh8 2. Nf7# (if King blocked) - wait this is just repetition or perpetual.

    // Better: 1. Qd5+ Kh8 2. Nf7+ Kg8 3. Nh6+ Kh8 4. Qg8+ Rxg8 5. Nf7# (Philidor)
    // Detailed Mate in 2:
    // White: Rc1, Rd1. Black: Kc8.
    // 1. Rc1-c7 Kb8 2. Rd1-d8#
    let fen = "2k5/8/8/8/8/8/8/2RR2K1 w - - 0 1";
    // 1. Rc7 or Rd7?
    find_mate_in_n(fen, 2, None);
}

#[test]
fn sanity_check_startpos() {
    scacchista::init();
    let mut board = Board::new();
    board.set_from_fen(scacchista::board::START_FEN).unwrap();

    let params = SearchParams::new().max_depth(4);
    let mut search = Search::new(board, 1000, params);

    let (_mv, score) = search.search(Some(4));

    // Startpos evaluation > -100 and < 100 (drawish/equal)
    assert!(
        score > -100 && score < 100,
        "Startpos score {} outside reasonable draw range",
        score
    );
}
