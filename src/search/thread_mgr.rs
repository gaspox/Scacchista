//! Threaded search manager (lazy-SMP) for Scacchista
//!
//! True lazy-SMP implementation: all workers search the same position in parallel,
//! sharing a global transposition table. Workers naturally explore different parts
//! of the search tree due to timing differences in TT hits/misses.

use crate::board::Board;
use crate::search::tt::TranspositionTable;
use crate::search::{Search, SearchParams, SearchResult};
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
#[allow(clippy::type_complexity)]
pub struct ThreadManager {
    workers: Vec<thread::JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
    /// Current job broadcasted to all workers (None = idle)
    current_job: Arc<Mutex<Option<SearchJob>>>,
    /// Signal that a new job is available
    job_available: Arc<AtomicBool>,
    /// Stop flag for current search job
    job_stop_flag: Arc<AtomicBool>,
    /// Results from each worker [worker_id]
    results: Arc<Mutex<Vec<Option<SearchResult>>>>,
    /// Counter for workers that have completed current job
    workers_done: Arc<AtomicUsize>,
}

impl ThreadManager {
    pub fn new(num_threads: usize, tt_mb: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let tt = Arc::new(TranspositionTable::new(tt_mb));
        let current_job = Arc::new(Mutex::new(None));
        let job_available = Arc::new(AtomicBool::new(false));
        let job_stop_flag = Arc::new(AtomicBool::new(false));
        let results: Arc<Mutex<Vec<Option<SearchResult>>>> = Arc::new(Mutex::new(vec![None; num_threads]));
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
                        let guard = job_clone
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        guard.clone()
                    };

                    if let Some(SearchJob { board, params }) = job {
                        let max_depth = params.max_depth;

                        // Lazy-SMP diversity: helper threads search at slightly
                        // different depths and with wider aspiration windows.
                        let worker_depth = if worker_id == 0 {
                            max_depth
                        } else {
                            max_depth.saturating_sub((worker_id as u8) % 3)
                        };
                        let mut worker_params = params.clone();
                        worker_params.max_depth = worker_depth;
                        if worker_id > 0 {
                            worker_params.aspiration_window +=
                                (worker_id as i16) * 10;
                        }

                        // Create search with shared TT and job stop flag
                        let mut search = Search::new(board, 16, worker_params)
                            .with_shared_tt(tt_clone.clone())
                            .with_stop_flag(job_stop_clone.clone());

                        // Execute search
                        let (mv, score) = search.search(Some(worker_depth));

                        {
                            let mut results_guard = results_clone
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            let stats = search.stats();
                            let hashfull = (tt_clone.fill_percentage() * 1000.0) as u8;
                            results_guard[worker_id] = Some(SearchResult {
                                best_move: mv,
                                score,
                                completed_depth: stats.completed_depth,
                                pv: search.get_pv(),
                                nodes: stats.nodes,
                                nps: stats.nps,
                                seldepth: stats.seldepth,
                                hashfull,
                            });
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
            stop_flag,
            current_job,
            job_available,
            job_stop_flag,
            results,
            workers_done,
        }
    }

    /// Submit a job and wait for result (synchronous from caller perspective)
    pub fn submit_job(&self, job: SearchJob) -> SearchResult {
        // Reset state for new job
        self.workers_done.store(0, Ordering::Release);
        self.job_stop_flag.store(false, Ordering::Release);
        {
            let mut results_guard = self
                .results
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for r in results_guard.iter_mut() {
                *r = None;
            }
        }

        // Set current job (broadcast to all workers)
        {
            let mut job_guard = self
                .current_job
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
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
                // FIX Bug #3: Return depth 0 on timeout
                self.job_stop_flag.store(true, Ordering::Release);
                self.job_available.store(false, Ordering::Release);
                return SearchResult {
                    best_move: 0,
                    score: 0,
                    completed_depth: 0,
                    pv: Vec::new(),
                    nodes: 0,
                    nps: 0,
                    seldepth: 0,
                    hashfull: 0,
                };
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Collect results: use worker 0 result (main thread authority)
        let best_result = {
            let results_guard = self.results.lock().unwrap_or_else(|poisoned| {
                poisoned.into_inner()
            });
            results_guard
                .iter()
                .filter_map(|r| r.clone())
                .next()
                .unwrap_or_else(|| SearchResult {
                    best_move: 0,
                    score: 0,
                    completed_depth: 0,
                    pv: Vec::new(),
                    nodes: 0,
                    nps: 0,
                    seldepth: 0,
                    hashfull: 0,
                })
        };

        // Clear job (stop workers)
        self.job_available.store(false, Ordering::Release);
        {
            let mut job_guard = self
                .current_job
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
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

    /// Clone the internal job stop flag for external timer threads.
    pub fn get_stop_flag(&self) -> Arc<AtomicBool> {
        self.job_stop_flag.clone()
    }

    /// Start an async search (non-blocking). Used for "go infinite" mode.
    /// Starts all workers searching and returns immediately.
    pub fn start_async_search(&self, job: SearchJob) -> Arc<AtomicBool> {
        // Reset state for new job
        self.workers_done.store(0, Ordering::Release);
        self.job_stop_flag.store(false, Ordering::Release);
        {
            let mut results_guard = self
                .results
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for r in results_guard.iter_mut() {
                *r = None;
            }
        }

        // Set current job (broadcast to all workers)
        {
            let mut job_guard = self
                .current_job
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *job_guard = Some(job);
        }

        // Signal job available (all workers will start searching)
        self.job_available.store(true, Ordering::Release);

        // Return job stop flag for caller to stop search when needed
        self.job_stop_flag.clone()
    }

    /// Wait for async search result with timeout (blocking call).
    pub fn wait_async_result(&self, timeout_ms: u64) -> Option<SearchResult> {
        // Wait for at least one worker to complete
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        while self.workers_done.load(Ordering::Acquire) == 0 {
            if start.elapsed() > timeout {
                return None;
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Collect best result (use first available)
        let best_result = {
            let results_guard = self
                .results
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            results_guard
                .iter()
                .filter_map(|r| r.clone())
                .next()
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
        let res = tm.submit_job(job);
        assert!(res.score <= 30000);
        assert!(res.completed_depth >= 1);
        tm.stop();
    }
}
