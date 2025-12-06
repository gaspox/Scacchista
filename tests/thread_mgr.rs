use scacchista::board::Board;
use scacchista::search::SearchParams;
use scacchista::search::ThreadManager;

#[test]
fn test_thread_manager_basic() {
    let tm = ThreadManager::new(1, 16);
    let board = Board::new();
    let params = SearchParams::new().max_depth(2);
    let job = scacchista::search::thread_mgr::SearchJob { board, params };
    let result = tm.submit_job(job);
    // FIX Bug #3: submit_job now returns (Move, score, completed_depth)
    let (_mv, score, completed_depth) = result;
    assert!(score <= 30000);
    assert!(completed_depth >= 1 && completed_depth <= 2, "Depth should be 1 or 2");
    tm.stop();
}
