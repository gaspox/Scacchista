//! Time management helper for Scacchista

use crate::search::params::TimeManagement as TM;

pub struct TimeManager;

impl TimeManager {
    /// Compute milliseconds to allocate given TimeManagement and go parameters
    pub fn allocate_time(
        time_mgmt: &TM,
        wtime: Option<u64>,
        btime: Option<u64>,
        winc: Option<u64>,
        binc: Option<u64>,
        movetime: Option<u64>,
        movestogo: Option<u64>, // Added movestogo
        side_is_white: bool,
    ) -> u64 {
        if let Some(mt) = movetime {
            return mt;
        }

        let moves_to_go = movestogo.unwrap_or(40).max(2); // Default 40, min 2 to avoid huge time alloc

        if side_is_white {
            if let Some(w) = wtime {
                let base_time = (w / moves_to_go).max(10);
                let increment_bonus = winc.map(|inc| (inc * 8) / 10).unwrap_or(0);
                return base_time + increment_bonus;
            }
        } else if let Some(b) = btime {
            let base_time = (b / moves_to_go).max(10);
            let increment_bonus = binc.map(|inc| (inc * 8) / 10).unwrap_or(0);
            return base_time + increment_bonus;
        }

        // fall back to configured per-move millisecond default
        time_mgmt.msec_per_move
    }
}
