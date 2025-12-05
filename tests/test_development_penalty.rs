//! Test per la penalità di sviluppo dei pezzi minori
//!
//! Verifica che:
//! 1. Prima di mossa 10, nessuna penalità viene applicata
//! 2. Dopo mossa 10, pezzi minori sulla prima traversa ricevono -10 cp
//! 3. Pezzi minori sviluppati non ricevono penalità

use scacchista::board::Board;
use scacchista::eval::evaluate;

#[test]
fn test_no_penalty_before_move_10() {
    // Posizione a mossa 5 con cavaliere su b1
    let mut board = Board::new();
    board
        .set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 5")
        .unwrap();

    let score = evaluate(&board);

    // A mossa 5, non dovrebbe esserci penalità, quindi score bilanciato
    assert!(
        score.abs() < 100,
        "Prima di mossa 10, non dovrebbe esserci penalità: score = {score}"
    );
}

#[test]
fn test_penalty_for_undeveloped_knight() {
    // Posizione a mossa 15 con cavaliere bianco su b1
    // (Tolgo gli altri pezzi per isolare il test)
    let mut board_undeveloped = Board::new();
    board_undeveloped
        .set_from_fen("4k3/8/8/8/8/8/8/1N2K3 w - - 0 15")
        .unwrap();

    // Posizione a mossa 15 con cavaliere bianco sviluppato (c3)
    let mut board_developed = Board::new();
    board_developed
        .set_from_fen("4k3/8/8/8/8/2N5/8/4K3 w - - 0 15")
        .unwrap();

    let score_undeveloped = evaluate(&board_undeveloped);
    let score_developed = evaluate(&board_developed);

    // Il cavaliere sviluppato dovrebbe avere score migliore
    // (penalità -10 cp + bonus PSQT per posizione centrale)
    assert!(
        score_developed > score_undeveloped,
        "Cavaliere sviluppato dovrebbe avere score migliore: developed={score_developed}, undeveloped={score_undeveloped}"
    );

    // Verifica che la differenza sia almeno 10 cp (la penalità)
    let difference = score_developed - score_undeveloped;
    assert!(
        difference >= 10,
        "Differenza dovrebbe essere almeno 10 cp (penalità): difference={difference}"
    );
}

#[test]
fn test_penalty_for_undeveloped_bishop() {
    // Posizione a mossa 15 con alfiere bianco su c1
    let mut board_undeveloped = Board::new();
    board_undeveloped
        .set_from_fen("4k3/8/8/8/8/8/8/2B1K3 w - - 0 15")
        .unwrap();

    // Posizione a mossa 15 con alfiere bianco sviluppato (c4)
    let mut board_developed = Board::new();
    board_developed
        .set_from_fen("4k3/8/8/8/2B5/8/8/4K3 w - - 0 15")
        .unwrap();

    let score_undeveloped = evaluate(&board_undeveloped);
    let score_developed = evaluate(&board_developed);

    // L'alfiere sviluppato dovrebbe avere score migliore
    assert!(
        score_developed > score_undeveloped,
        "Alfiere sviluppato dovrebbe avere score migliore: developed={score_developed}, undeveloped={score_undeveloped}"
    );

    let difference = score_developed - score_undeveloped;
    assert!(
        difference >= 10,
        "Differenza dovrebbe essere almeno 10 cp (penalità): difference={difference}"
    );
}

#[test]
fn test_penalty_for_black_undeveloped() {
    // Posizione a mossa 15 con cavaliere nero su b8 (non sviluppato)
    let mut board_undeveloped = Board::new();
    board_undeveloped
        .set_from_fen("1n2k3/8/8/8/8/8/8/4K3 b - - 0 15")
        .unwrap();

    // Posizione a mossa 15 con cavaliere nero sviluppato (c6)
    let mut board_developed = Board::new();
    board_developed
        .set_from_fen("4k3/8/2n5/8/8/8/8/4K3 b - - 0 15")
        .unwrap();

    let score_undeveloped = evaluate(&board_undeveloped);
    let score_developed = evaluate(&board_developed);

    // Il cavaliere sviluppato dovrebbe avere score migliore (dal PDV del Nero)
    assert!(
        score_developed > score_undeveloped,
        "Cavaliere nero sviluppato dovrebbe avere score migliore: developed={score_developed}, undeveloped={score_undeveloped}"
    );

    let difference = score_developed - score_undeveloped;
    assert!(
        difference >= 10,
        "Differenza dovrebbe essere almeno 10 cp (penalità): difference={difference}"
    );
}

#[test]
fn test_penalty_cumulative() {
    // Posizione a mossa 15 con 2 cavalieri e 2 alfieri bianchi sulla prima traversa
    let mut board_all_undeveloped = Board::new();
    board_all_undeveloped
        .set_from_fen("4k3/8/8/8/8/8/8/1NBBNK2 w - - 0 15")
        .unwrap();

    // Posizione a mossa 15 con tutti i pezzi sviluppati
    let mut board_all_developed = Board::new();
    board_all_developed
        .set_from_fen("4k3/8/8/2NBBN2/8/8/8/4K3 w - - 0 15")
        .unwrap();

    let score_undeveloped = evaluate(&board_all_undeveloped);
    let score_developed = evaluate(&board_all_developed);

    // I pezzi sviluppati dovrebbero avere score molto migliore
    assert!(
        score_developed > score_undeveloped,
        "Tutti i pezzi sviluppati dovrebbero avere score migliore: developed={score_developed}, undeveloped={score_undeveloped}"
    );

    // 4 pezzi × 10 cp = 40 cp di penalità (almeno)
    let difference = score_developed - score_undeveloped;
    assert!(
        difference >= 40,
        "Differenza dovrebbe essere almeno 40 cp (4 pezzi × 10 cp): difference={difference}"
    );
}
