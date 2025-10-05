pub mod board;
pub mod zobrist;
pub mod utils;

pub fn init() {
    utils::init_attack_tables();
    zobrist::init_zobrist();
}