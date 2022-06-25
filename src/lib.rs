pub mod block;
pub mod errors;
pub mod config;

use crate::block::Block;
use crate::config::Config;

/// # Rust Blockchain
///
/// A library for studying rust and blockchain.

pub fn run(config: Config) {
    let genesis_block: Block = Block::new(
        0,
        "816534932c2b7154836da6afc367695e6337db8a921823784c14378abed4f7d7".to_string(),
        "".to_string(),
        1465154705,
        "gene block".to_string(),
    );
    let blockchain: Vec<Block> = vec![genesis_block];

    println!("{:?}{:?}", blockchain, config);
}
