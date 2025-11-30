//! Threaded search manager (lazy-SMP) for Scacchista
//!
//! Responsibilities:
//! - Spawn worker threads honoring Threads option
//! - Provide start/stop/job submission API
//! - Share transposition table and stop flag via Arc<Mutex/Atomic>

use crate::board::Board;
use crate::search::tt::TranspositionTable;
use crate::search::{Search, SearchParams};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

/// Job submitted to worker, includes a response channel to send back result
pub struct SearchJob {
    pub board: Board,
    pub params: SearchParams,
}

struct JobWithResp {
    job: SearchJob,
    resp: Sender<(crate::board::Move, i16)>,
    stop_flag: Arc<AtomicBool>,
}

/// Thread manager implementing a simple job queue consumed by workers
pub struct ThreadManager {
    workers: Vec<thread::JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
    sender: Sender<JobWithResp>,
    tt: Arc<Mutex<TranspositionTable>>,
    /// Current job stop flag (set when a job is active). Guarded by mutex.
    current_job_stop: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    /// Async result receiver for go infinite mode (stored for retrieval after stop)
    async_result_rx: Arc<Mutex<Option<Receiver<(crate::board::Move, i16)>>>>,
}

impl ThreadManager {
    pub fn new(num_threads: usize, tt_mb: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let tt = Arc::new(Mutex::new(TranspositionTable::new(tt_mb)));
        let (tx, rx): (Sender<JobWithResp>, Receiver<JobWithResp>) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::new();
        for _ in 0..num_threads {
            let stop_clone = stop_flag.clone();
            let _tt_clone = tt.clone();
            let rx_clone = rx.clone();
            let handle = thread::spawn(move || {
                loop {
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    // Non-blocking receive with brief sleep for low latency stop response
                    let rx_guard = match rx_clone.lock() {
                        Ok(g) => g,
                        Err(poisoned) => {
                            eprintln!("WARN: Worker receiver mutex poisoned, recovering");
                            poisoned.into_inner()
                        }
                    };
                    match rx_guard.try_recv() {
                        Ok(job_wrapped) => {
                            // When a job starts, use its stop flag to cooperatively stop search
                            let SearchJob { board, params } = job_wrapped.job;
                            let job_stop = job_wrapped.stop_flag.clone();
                            let max_depth = params.max_depth;

                            // Create search with stop flag attached
                            let mut search =
                                Search::new(board, 16, params).with_stop_flag(job_stop);

                            let (mv, score) = search.search(Some(max_depth));
                            // Send result back (ignore send errors)
                            let _ = job_wrapped.resp.send((mv, score));
                        }
                        Err(TryRecvError::Empty) => {
                            // No job available, sleep briefly and recheck (10ms for ~20-50ms latency)
                            std::thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            });
            workers.push(handle);
        }

        ThreadManager {
            workers,
            stop_flag,
            sender: tx,
            tt,
            current_job_stop: Arc::new(Mutex::new(None)),
            async_result_rx: Arc::new(Mutex::new(None)),
        }
    }

    /// Submit a job and wait for result (synchronous from caller perspective)
    pub fn submit_job(&self, job: SearchJob) -> (crate::board::Move, i16) {
        let (resp_tx, resp_rx) = mpsc::channel();
        let job_stop = Arc::new(AtomicBool::new(false));
        let wrapped = JobWithResp {
            job,
            resp: resp_tx,
            stop_flag: job_stop.clone(),
        };
        // Record current job stop flag
        {
            let mut guard = match self.current_job_stop.lock() {
                Ok(g) => g,
                Err(poisoned) => {
                    eprintln!("WARN: current_job_stop mutex poisoned (submit_job), recovering");
                    poisoned.into_inner()
                }
            };
            *guard = Some(job_stop.clone());
        }
        // Send job (ignore send error)
        let _ = self.sender.send(wrapped);

        // Wait for result with generous timeout (10 minutes for deep searches)
        // The stop mechanism should handle cancellation before timeout
        match resp_rx.recv_timeout(Duration::from_secs(600)) {
            Ok(res) => {
                // Clear current job stop
                let mut guard = match self.current_job_stop.lock() {
                    Ok(g) => g,
                    Err(poisoned) => {
                        eprintln!("WARN: current_job_stop mutex poisoned (clear), recovering");
                        poisoned.into_inner()
                    }
                };
                *guard = None;
                res
            }
            Err(_) => {
                // Timeout or disconnected after 10 minutes - something went wrong
                // Return 0000 (null move) to signal error
                let mut guard = match self.current_job_stop.lock() {
                    Ok(g) => g,
                    Err(poisoned) => {
                        eprintln!("WARN: current_job_stop mutex poisoned (timeout), recovering");
                        poisoned.into_inner()
                    }
                };
                *guard = None;
                (0, -30000)
            }
        }
    }

    /// Signal workers to stop and join
    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        // Dropping sender will cause workers recv to return Disconnected
        drop(self.sender);
        for w in self.workers {
            let _ = w.join();
        }
    }

    /// Signal the currently running job (if any) to stop
    pub fn stop_current_job(&self) {
        let guard = match self.current_job_stop.lock() {
            Ok(g) => g,
            Err(poisoned) => {
                eprintln!("WARN: current_job_stop mutex poisoned (stop_current_job), recovering");
                poisoned.into_inner()
            }
        };
        if let Some(job_stop) = &*guard {
            job_stop.store(true, Ordering::Relaxed);
        }
    }

    /// Start an async search (non-blocking). Returns the job stop flag for manual control.
    /// Used for "go infinite" mode. Caller must later call wait_async_result() to retrieve result.
    pub fn start_async_search(&self, job: SearchJob) -> Arc<AtomicBool> {
        let (resp_tx, resp_rx) = mpsc::channel();
        let job_stop = Arc::new(AtomicBool::new(false));

        let wrapped = JobWithResp {
            job,
            resp: resp_tx,
            stop_flag: job_stop.clone(),
        };

        // Store the result receiver for later retrieval
        {
            let mut guard = match self.async_result_rx.lock() {
                Ok(g) => g,
                Err(poisoned) => {
                    eprintln!(
                        "WARN: async_result_rx mutex poisoned (start_async_search), recovering"
                    );
                    poisoned.into_inner()
                }
            };
            *guard = Some(resp_rx);
        }

        // Record current job stop flag
        {
            let mut guard = match self.current_job_stop.lock() {
                Ok(g) => g,
                Err(poisoned) => {
                    eprintln!(
                        "WARN: current_job_stop mutex poisoned (start_async_search), recovering"
                    );
                    poisoned.into_inner()
                }
            };
            *guard = Some(job_stop.clone());
        }

        // Send job to worker (non-blocking from caller perspective)
        let _ = self.sender.send(wrapped);

        job_stop
    }

    /// Wait for async search result with timeout (blocking call).
    /// Returns None if timeout expires or no async search is active.
    pub fn wait_async_result(&self, timeout_ms: u64) -> Option<(crate::board::Move, i16)> {
        // Take the receiver (only one retrieval allowed)
        let rx = {
            let mut guard = match self.async_result_rx.lock() {
                Ok(g) => g,
                Err(poisoned) => {
                    eprintln!(
                        "WARN: async_result_rx mutex poisoned (wait_async_result), recovering"
                    );
                    poisoned.into_inner()
                }
            };
            guard.take()
        }?;

        // Wait for result with timeout
        let result = rx.recv_timeout(Duration::from_millis(timeout_ms)).ok();

        // Clear current job stop flag after result is received
        if result.is_some() {
            let mut guard = match self.current_job_stop.lock() {
                Ok(g) => g,
                Err(poisoned) => {
                    eprintln!("WARN: current_job_stop mutex poisoned (wait_async_result clear), recovering");
                    poisoned.into_inner()
                }
            };
            *guard = None;
        }

        result
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
