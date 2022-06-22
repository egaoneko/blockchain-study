pub mod block;

use crate::block::Block;

pub fn run() {
    let genesis_block: Block = Block::new(
        0,
        "816534932c2b7154836da6afc367695e6337db8a921823784c14378abed4f7d7".to_string(),
        "".to_string(),
        1465154705,
        "gene block".to_string(),
    );
    let blockchain: Vec<Block> = vec![genesis_block];

    println!("{:?}", blockchain);
}
