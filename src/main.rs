//! Scacchista UCI chess engine main entry point.

use scacchista;

fn main() {
    scacchista::init();

    // Run UCI main loop
    if let Err(e) = uci::run_uci_loop() {
    }
}