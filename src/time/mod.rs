//! Time management helper for Scacchista

use crate::search::params::TimeManagement as TM;

pub struct TimeManager;

impl TimeManager {
    /// Compute milliseconds to allocate given TimeManagement and go parameters.
    ///
    /// Applies emergency logic for very low time reserves and subtracts
    /// `move_overhead_ms` to compensate for network/GUI lag.
    #[allow(clippy::too_many_arguments)]
    pub fn allocate_time(
        time_mgmt: &TM,
        wtime: Option<u64>,
        btime: Option<u64>,
        winc: Option<u64>,
        binc: Option<u64>,
        movetime: Option<u64>,
        movestogo: Option<u64>,
        side_is_white: bool,
        move_overhead_ms: u64,
    ) -> u64 {
        if let Some(mt) = movetime {
            return mt.saturating_sub(move_overhead_ms).max(1);
        }

        let time_left = if side_is_white { wtime } else { btime };
        let inc = if side_is_white { winc } else { binc };

        if let Some(t) = time_left {
            // Emergency: less than 1 second on the clock
            if t < 1000 {
                return t.saturating_sub(move_overhead_ms).max(1);
            }

            // Conservative: less than 5 seconds
            if t < 5000 {
                let alloc = (t / 10).max(10);
                return alloc.saturating_sub(move_overhead_ms).max(1);
            }

            // Normal allocation
            let moves_to_go = movestogo.unwrap_or(40).max(2);
            let base_time = (t / moves_to_go).max(10);
            let increment_bonus = inc.map(|i| (i * 8) / 10).unwrap_or(0);
            let alloc = base_time + increment_bonus;
            return alloc.saturating_sub(move_overhead_ms).max(1);
        }

        // Fallback when no clock info is provided
        time_mgmt.msec_per_move
            .saturating_sub(move_overhead_ms)
            .max(1)
    }
}
