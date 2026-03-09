use scacchista::search::params::TimeManagement;
use scacchista::time::TimeManager;

#[test]
fn test_allocate_normal() {
    let tm = TimeManagement::new();
    // 60s left, no inc, white to move
    let time = TimeManager::allocate_time(&tm, Some(60000), None, None, None, None, None, true);
    // Heuristic: usually ~1/20 to 1/30 of remaining time.
    // 60000 / 30 = 2000. 60000 / 20 = 3000.
    assert!(
        time > 1000 && time < 5000,
        "Allocated {} ms for 60s remaining",
        time
    );
}

#[test]
fn test_allocate_increment() {
    let tm = TimeManagement::new();
    // 10s left, 1s increment
    let time =
        TimeManager::allocate_time(&tm, Some(10000), None, Some(1000), None, None, None, true);
    // Should use some of the time + increment.
    // Base: 10000/20 = 500. + Inc/2 = 500? Total ~1000.
    assert!(time > 500, "Should use feasible time with increment");
    // Should not exceed time left significantly (checking logic buffer)
    assert!(time < 10000, "Should not use all time");
}

#[test]
fn test_movetime_exact() {
    let tm = TimeManagement::new();
    // Fixed movetime 5000ms
    let time = TimeManager::allocate_time(&tm, None, None, None, None, Some(5000), None, true);
    // Should be exactly 5000 minus overhead? Or roughly 5000.
    // Implementation: `if let Some(mt) = movetime { return mt - overhead; }`
    assert!(
        (4900..=5000).contains(&time),
        "Movetime {} should be close to 5000",
        time
    );
}

#[test]
fn test_moves_to_go() {
    let mut tm = TimeManagement::new();
    tm.moves_to_go_left = 5; // 5 moves to go!

    // 60s left, 5 moves to go.
    // Should use ~ 60s / 5 = 12s.
    let time = TimeManager::allocate_time(&tm, Some(60000), None, None, None, None, Some(5), true);

    assert!(
        time > 8000 && time < 14000,
        "Allocated {} with 5 moves to go",
        time
    );
}
