//! Threaded search manager (lazy-SMP) for Scacchista
//!
//! True lazy-SMP implementation: all workers search the same position in parallel,
//! sharing a global transposition table. Workers naturally explore different parts
//! of the search tree due to timing differences in TT hits/misses.

use crate::board::Board;
use crate::search::tt::TranspositionTable;
use crate::search::{Search, SearchParams};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

/// Job to broadcast to all workers
#[derive(Clone)]
pub struct SearchJob {
    pub board: Board,
    pub params: SearchParams,
}

/// Thread manager implementing true lazy-SMP parallel search
pub struct ThreadManager {
    workers: Vec<thread::JoinHandle<()>>,
    num_threads: usize,
    stop_flag: Arc<AtomicBool>,
    tt: Arc<Mutex<TranspositionTable>>,
    /// Current job broadcasted to all workers (None = idle)
    current_job: Arc<Mutex<Option<SearchJob>>>,
    /// Signal that a new job is available
    job_available: Arc<AtomicBool>,
    /// Stop flag for current search job
    job_stop_flag: Arc<AtomicBool>,
    /// Results from each worker [worker_id] -> (move, score)
    results: Arc<Mutex<Vec<Option<(crate::board::Move, i16)>>>>,
    /// Counter for workers that have completed current job
    workers_done: Arc<AtomicUsize>,
}

impl ThreadManager {
    pub fn new(num_threads: usize, tt_mb: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let tt = Arc::new(Mutex::new(TranspositionTable::new(tt_mb)));
        let current_job = Arc::new(Mutex::new(None));
        let job_available = Arc::new(AtomicBool::new(false));
        let job_stop_flag = Arc::new(AtomicBool::new(false));
        let results = Arc::new(Mutex::new(vec![None; num_threads]));
        let workers_done = Arc::new(AtomicUsize::new(0));

        let mut workers = Vec::new();
        for worker_id in 0..num_threads {
            let stop_clone = stop_flag.clone();
            let tt_clone = tt.clone();
            let job_clone = current_job.clone();
            let job_avail_clone = job_available.clone();
            let job_stop_clone = job_stop_flag.clone();
            let results_clone = results.clone();
            let workers_done_clone = workers_done.clone();

            let handle = thread::spawn(move || {
                loop {
                    // Check global stop flag
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    // Wait for job to become available
                    if !job_avail_clone.load(Ordering::Acquire) {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }

                    // Get current job (if any)
                    let job = {
                        let guard = job_clone.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                        guard.clone()
                    };

                    if let Some(SearchJob { board, params }) = job {
                        let max_depth = params.max_depth;

                        // Create search with shared TT and job stop flag
                        let mut search = Search::new(board, 16, params)
                            .with_shared_tt(tt_clone.clone())
                            .with_stop_flag(job_stop_clone.clone());

                        // Execute search
                        let (mv, score) = search.search(Some(max_depth));

                        // Store result
                        {
                            let mut results_guard = results_clone.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                            results_guard[worker_id] = Some((mv, score));
                        }

                        // Signal completion
                        workers_done_clone.fetch_add(1, Ordering::Release);

                        // Wait for job to be cleared
                        while job_avail_clone.load(Ordering::Acquire)
                            && !stop_clone.load(Ordering::Relaxed)
                        {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                }
            });
            workers.push(handle);
        }

        ThreadManager {
            workers,
            num_threads,
            stop_flag,
            tt,
            current_job,
            job_available,
            job_stop_flag,
            results,
            workers_done,
        }
    }

    /// Submit a job and wait for result (synchronous from caller perspective)
    /// Broadcasts job to all workers and waits for first completion (or best result)
    pub fn submit_job(&self, job: SearchJob) -> (crate::board::Move, i16) {
        // Reset state for new job
        self.workers_done.store(0, Ordering::Release);
        self.job_stop_flag.store(false, Ordering::Release);
        {
            let mut results_guard = self.results.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            for r in results_guard.iter_mut() {
                *r = None;
            }
        }

        // Set current job (broadcast to all workers)
        {
            let mut job_guard = self.current_job.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            *job_guard = Some(job);
        }

        // Signal job available (all workers will start searching)
        self.job_available.store(true, Ordering::Release);

        // Wait for at least one worker to complete (or timeout after 10 minutes)
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(600);

        while self.workers_done.load(Ordering::Acquire) == 0 {
            if start.elapsed() > timeout {
                // Timeout - stop all workers and return draw score
                // FIX Bug #2A: Return 0 (draw) instead of -30000 to avoid score corruption
                self.job_stop_flag.store(true, Ordering::Release);
                self.job_available.store(false, Ordering::Release);
                return (0, 0);
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Collect results from workers (take best score)
        let best_result = {
            // FIX Bug #2B: Handle mutex poisoning gracefully
            let results_guard = self.results.lock().unwrap_or_else(|poisoned| {
                // Mutex was poisoned - recover the guard and continue
                poisoned.into_inner()
            });
            results_guard
                .iter()
                .filter_map(|r| *r)
                .max_by_key(|(_mv, score)| *score)
                // FIX Bug #2B: Return 0 (draw) instead of -30000 when no results
                .unwrap_or((0, 0))
        };

        // Clear job (stop workers)
        self.job_available.store(false, Ordering::Release);
        {
            let mut job_guard = self.current_job.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            *job_guard = None;
        }

        best_result
    }

    /// Signal workers to stop and join
    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.job_available.store(false, Ordering::Relaxed);
        for w in self.workers {
            let _ = w.join();
        }
    }

    /// Signal the currently running job (if any) to stop
    pub fn stop_current_job(&self) {
        self.job_stop_flag.store(true, Ordering::Relaxed);
    }

    /// Start an async search (non-blocking). Used for "go infinite" mode.
    /// Starts all workers searching and returns immediately.
    pub fn start_async_search(&self, job: SearchJob) -> Arc<AtomicBool> {
        // Reset state for new job
        self.workers_done.store(0, Ordering::Release);
        self.job_stop_flag.store(false, Ordering::Release);
        {
            let mut results_guard = self.results.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            for r in results_guard.iter_mut() {
                *r = None;
            }
        }

        // Set current job (broadcast to all workers)
        {
            let mut job_guard = self.current_job.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            *job_guard = Some(job);
        }

        // Signal job available (all workers will start searching)
        self.job_available.store(true, Ordering::Release);

        // Return job stop flag for caller to stop search when needed
        self.job_stop_flag.clone()
    }

    /// Wait for async search result with timeout (blocking call).
    /// Returns None if timeout expires or no async search is active.
    pub fn wait_async_result(&self, timeout_ms: u64) -> Option<(crate::board::Move, i16)> {
        // Wait for at least one worker to complete
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        while self.workers_done.load(Ordering::Acquire) == 0 {
            if start.elapsed() > timeout {
                return None;
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Collect best result
        let best_result = {
            let results_guard = self.results.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            results_guard
                .iter()
                .filter_map(|r| *r)
                .max_by_key(|(_mv, score)| *score)
        };

        // Clear job
        self.job_available.store(false, Ordering::Release);
        {
            let mut job_guard = self.current_job.lock().unwrap();
            *job_guard = None;
        }

        best_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchParams;

    #[test]
    fn basic_worker_smoke() {
        let tm = ThreadManager::new(2, 16);
        let job = SearchJob {
            board: Board::new(),
            params: SearchParams::new().max_depth(1),
        };
        let (mv, score) = tm.submit_job(job);
        assert!(score <= 30000);
        tm.stop();
    }
}
