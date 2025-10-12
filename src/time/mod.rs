//! Time management helper for Scacchista

use crate::search::params::TimeManagement as TM;

pub struct TimeManager;

impl TimeManager {
    /// Compute milliseconds to allocate given TimeManagement and go parameters
    pub fn allocate_time(
        time_mgmt: &TM,
        wtime: Option<u64>,
        btime: Option<u64>,
        movetime: Option<u64>,
        side_is_white: bool,
    ) -> u64 {
        if let Some(mt) = movetime {
            return mt;
        }

        // Simplified: use per-move msec or time_left / moves_to_go
        if side_is_white {
            if let Some(w) = wtime {
                return (w / 40).max(10);
            }
        } else if let Some(b) = btime {
            return (b / 40).max(10);
        }

        // fall back to configured per-move millisecond default


        time_mgmt.msec_per_move
    }
}
