// Direct test of material_eval() to verify correct sign

use scacchista::*;

#[test]
fn test_material_eval_sign() {
    init();

    // Test 1: Starting position - material_eval is now private, accessed via search
    // We test via actual search to verify the full pipeline

    // Better approach: test via qsearch with depth 0 on quiet positions
    let mut board_equal = Board::new();
    board_equal
        .set_from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1")
        .unwrap();
    let mut search_equal = search::Search::new(board_equal, 16, search::SearchParams::new());
    let score_equal = search_equal.search(Some(1)).1;
    println!("Equal material (just kings): {}", score_equal);

    // Test 2: White up a queen
    let mut board_white_queen = Board::new();
    board_white_queen
        .set_from_fen("4k3/8/8/8/8/8/8/4K2Q w - - 0 1")
        .unwrap();
    let mut search_white_queen =
        search::Search::new(board_white_queen, 16, search::SearchParams::new());
    let score_white_queen = search_white_queen.search(Some(1)).1;
    println!("White up a queen: {}", score_white_queen);
    assert!(
        score_white_queen > 800,
        "White up a queen should be positive and large. Got: {}",
        score_white_queen
    );

    // Test 3: Black up a queen
    let mut board_black_queen = Board::new();
    board_black_queen
        .set_from_fen("4k2q/8/8/8/8/8/8/4K3 w - - 0 1")
        .unwrap();
    let mut search_black_queen =
        search::Search::new(board_black_queen, 16, search::SearchParams::new());
    let score_black_queen = search_black_queen.search(Some(1)).1;
    println!("Black up a queen: {}", score_black_queen);
    assert!(
        score_black_queen < -800,
        "Black up a queen should be negative and large. Got: {}",
        score_black_queen
    );
}
