//! Time management helper for Scacchista

use crate::search::params::TimeManagement as TM;

pub struct TimeManager;

impl TimeManager {
    /// Compute milliseconds to allocate given TimeManagement and go parameters
    pub fn allocate_time(
        time_mgmt: &TM,
        wtime: Option<u64>,
        btime: Option<u64>,
        winc: Option<u64>,   // FIX Bug #4: Add increment parameters
        binc: Option<u64>,   // FIX Bug #4: Add increment parameters
        movetime: Option<u64>,
        side_is_white: bool,
    ) -> u64 {
        if let Some(mt) = movetime {
            return mt;
        }

        // FIX Bug #4: Use increment in time calculation
        // Strategy: allocate time_left / moves_to_go + partial increment
        // We use 80% of increment (conservatively, since we might not complete the move)
        if side_is_white {
            if let Some(w) = wtime {
                let base_time = (w / 40).max(10);
                let increment_bonus = winc.map(|inc| (inc * 8) / 10).unwrap_or(0);
                return base_time + increment_bonus;
            }
        } else if let Some(b) = btime {
            let base_time = (b / 40).max(10);
            let increment_bonus = binc.map(|inc| (inc * 8) / 10).unwrap_or(0);
            return base_time + increment_bonus;
        }

        // fall back to configured per-move millisecond default
        time_mgmt.msec_per_move
    }
}
