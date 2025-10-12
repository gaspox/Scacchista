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
    mpsc::{self, Receiver, RecvTimeoutError, Sender},
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
                    // Wait for a job with timeout to allow checking stop_flag
                    let recv_result = rx_clone
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_millis(200));
                    match recv_result {
                        Ok(job_wrapped) => {
                            // When a job starts, use its stop flag to cooperatively stop search
                            let SearchJob { board, params } = job_wrapped.job;
                            let _job_stop = job_wrapped.stop_flag.clone();
                            let max_depth = params.max_depth;
                            let mut search = Search::new(board, 16, params);
                            // Attach job_stop to search (search must check stop flag periodically)
                            // For now, call search.search which is blocking; future work: implement search that accepts stop flag
                            let (mv, score) = search.search(Some(max_depth));
                            // Send result back (ignore send errors)
                            let _ = job_wrapped.resp.send((mv, score));
                        }
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break,
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
            let mut guard = self.current_job_stop.lock().unwrap();
            *guard = Some(job_stop.clone());
        }
        // Send job (ignore send error)
        let _ = self.sender.send(wrapped);
        // Wait for result or timeout
        match resp_rx.recv_timeout(Duration::from_secs(300)) {
            Ok(res) => {
                // Clear current job stop
                let mut guard = self.current_job_stop.lock().unwrap();
                *guard = None;
                res
            }
            Err(_) => {
                // clear and return failure
                let mut guard = self.current_job_stop.lock().unwrap();
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
        if let Some(job_stop) = &*self.current_job_stop.lock().unwrap() {
            job_stop.store(true, Ordering::Relaxed);
        }
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
