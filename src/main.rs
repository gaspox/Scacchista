//! Scacchista UCI chess engine main entry point.


fn main() {
    scacchista::init();

    // Run UCI main loop
    if let Err(e) = scacchista::uci::run_uci_loop() {
        eprintln!("UCI loop failed: {:?}", e);
    }
}
