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
    // submit_job returns (Move, score)
    assert!(result.1 <= 30000);
    tm.stop();
}
