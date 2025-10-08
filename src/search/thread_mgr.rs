//! Threaded search manager (lazy-SMP) for Scacchista
//!
//! Responsibilities:
//! - Spawn worker threads honoring Threads option
//! - Provide start/stop/job submission API
//! - Share transposition table and stop flag via Arc<Mutex/Atomic>

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use crate::search::{Search, SearchParams, SearchResult};
use crate::search::tt::TranspositionTable;
use crate::board::Board;

pub struct SearchJob {
    pub board: Board,
    pub params: SearchParams,
}

pub struct ThreadManager {
    workers: Vec<thread::JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
    tt: Arc<Mutex<TranspositionTable>>,
}

impl ThreadManager {
    pub fn new(num_threads: usize, tt_mb: usize) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let tt = Arc::new(Mutex::new(TranspositionTable::new(tt_mb)));
        let mut workers = Vec::new();

        for _ in 0..num_threads {
            let stop_clone = stop_flag.clone();
            let tt_clone = tt.clone();

            let handle = thread::spawn(move || {
                // Worker loop — idle until a job is assigned (simple mock for now)
                while !stop_clone.load(Ordering::Relaxed) {
                    // Sleep briefly to avoid busy loop
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            });

            workers.push(handle);
        }

        ThreadManager { workers, stop_flag, tt }
    }

    /// Submit a job — for now we run search on current thread and signal workers
    pub fn submit_job(&self, job: SearchJob) -> (crate::board::Move, i16) {
        // For now, perform synchronous search on caller thread using provided params
        let mut search = Search::new(job.board, 16, job.params);
        let (mv, score) = search.search(Some(search.params.max_depth));
        (mv, score)
    }

    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        for w in self.workers {
            let _ = w.join();
        }
    }
}
