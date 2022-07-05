use sha2::{Sha256, Digest};
use chrono::{Utc};
use serde::{Serialize, Deserialize};

use crate::errors::AppError;
use crate::utils::get_is_hash_matches_difficulty;

const BLOCK_GENERATION_INTERVAL: usize = 10;
const DIFFICULTY_ADJUSTMENT_INTERVAL: usize = 10;
const TIMESTAMP_INTERVAL: usize = 60;

/// Block in blockchain has sequence, data, time, and so on.
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    /// Sequence in blockchain
    pub index: usize,

    /// Hash from other properties
    pub hash: String,

    /// Previous block hash
    pub previous_hash: String,

    /// Timestamp when created
    pub timestamp: usize,

    /// Data in block
    pub data: String,

    /// Difficulty to generate block
    pub difficulty: usize,

    /// Nonce to generate block
    pub nonce: usize,
}

impl Block {
    /// Returns a block with arguments
    pub fn new(
        index: usize,
        hash: String,
        previous_hash: String,
        timestamp: usize,
        data: String,
        difficulty: usize,
        nonce: usize,
    ) -> Block {
        Block {
            index,
            hash,
            previous_hash,
            timestamp,
            data,
            difficulty,
            nonce,
        }
    }

    /// Generate a block with data and previous block
    pub fn generate(data: String, previous: &Block, difficulty: usize) -> Block {
        let index = previous.index + 1;
        let timestamp = Utc::now().timestamp() as usize;
        let mut nonce = 0;

        return loop {
            let hash = calculate_hash(index, previous.hash.as_str(), timestamp, data.as_str(), difficulty, nonce);

            if !get_is_hash_matches_difficulty(hash.as_str(), difficulty) {
                nonce += 1;
                continue;
            }

            return Block::new(
                index,
                hash,
                previous.hash.to_string(),
                timestamp,
                data.to_string(),
                difficulty,
                nonce,
            );
        };
    }

    /// Recalculate and return hash
    pub fn get_calculated_hash(&self) -> String {
        calculate_hash(self.index, self.previous_hash.as_str(), self.timestamp, self.data.as_str(), self.difficulty, self.nonce)
    }

    /// Return structure is valid
    pub fn get_is_valid_structure(&self) -> bool {
        !self.hash.is_empty() && !self.previous_hash.is_empty() && !self.data.is_empty()
    }

    // Return hash is valid
    pub fn get_is_valid_hash(&self) -> bool {
        if !self.get_calculated_hash().eq(&self.hash) {
            return false;
        }

        if !get_is_hash_matches_difficulty(self.hash.as_str(), self.difficulty) {
            return false;
        }

        true
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index &&
            self.hash == other.hash &&
            self.previous_hash == other.previous_hash &&
            self.timestamp == other.timestamp &&
            self.data == other.data
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            hash: self.hash.clone(),
            previous_hash: self.previous_hash.clone(),
            timestamp: self.timestamp,
            data: self.data.clone(),
            difficulty: self.difficulty,
            nonce: self.nonce,
        }
    }
}

fn calculate_hash(index: usize, previous_hash: &str, timestamp: usize, data: &str, difficulty: usize, nonce: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}{}{}{}", index, previous_hash, timestamp, data, difficulty, nonce).as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_is_valid_timestamp(new_block: &Block, previous_block: &Block) -> bool {
    previous_block.timestamp - TIMESTAMP_INTERVAL < new_block.timestamp
        && new_block.timestamp - TIMESTAMP_INTERVAL < Utc::now().timestamp() as usize
}

fn get_is_valid_new_block(new_block: &Block, previous_block: &Block) -> bool {
    return if !new_block.get_is_valid_structure() {
        false
    } else if previous_block.index + 1 != new_block.index {
        false
    } else if previous_block.hash != new_block.previous_hash {
        false
    } else if !get_is_valid_timestamp(new_block, previous_block) {
        false
    } else if !new_block.get_is_valid_hash() {
        false
    } else {
        true
    };
}

fn get_is_valid_chain(genesis_block: &Block, blockchain: &Vec<Block>) -> bool {
    if genesis_block != blockchain.get(0).unwrap() {
        false
    } else if blockchain.len() == 1 {
        true
    } else {
        blockchain.windows(2).all(|window| get_is_valid_new_block(&window[1], &window[0]))
    }
}

fn get_accumulated_difficulty(blockchain: &Vec<Block>) -> i32 {
    blockchain.into_iter()
        .map(|block: &Block| block.difficulty)
        .fold(0, |total: i32, difficulty: usize| total + 2_i32.pow(difficulty as u32))
}

/// Get latest block from blockchain.
pub fn get_latest_block(blockchain: &Vec<Block>) -> &Block {
    blockchain.last().unwrap()
}

/// Add block to blockchain.
///
/// # Errors
/// If it is not valid compared to the previous block, it returns error 1000.
pub fn add_block(blockchain: &mut Vec<Block>, data: String) -> Result<(), AppError> {
    let latest = get_latest_block(blockchain);
    let difficulty = get_difficulty(blockchain);
    let new_block = Block::generate(data, latest, difficulty);

    if !get_is_valid_new_block(&new_block, get_latest_block(blockchain)) {
        Err(AppError::new(1000))
    } else {
        blockchain.push(new_block);
        Ok(())
    }
}

/// Get flag to replace blockchain.
pub fn get_is_replace_chain(blockchain: &Vec<Block>, new_blockchain: &Vec<Block>) -> bool {
    get_is_valid_chain(&blockchain[0], new_blockchain) && get_accumulated_difficulty(blockchain) < get_accumulated_difficulty(new_blockchain)
}

/// Get difficulty from blockchain.
pub fn get_difficulty(blockchain: &Vec<Block>) -> usize {
    let latest_block = get_latest_block(blockchain);
    if (latest_block.index % DIFFICULTY_ADJUSTMENT_INTERVAL) != 0 || latest_block.index == 0 {
        return latest_block.difficulty;
    }

    let prev_adjustment_block: &Block = blockchain.get(blockchain.len() - DIFFICULTY_ADJUSTMENT_INTERVAL).unwrap();
    let time_expected = BLOCK_GENERATION_INTERVAL * DIFFICULTY_ADJUSTMENT_INTERVAL;
    let time_taken = latest_block.timestamp - prev_adjustment_block.timestamp;

    return if time_taken < time_expected / 2 {
        prev_adjustment_block.difficulty + 1
    } else if time_taken > time_expected * 2 {
        prev_adjustment_block.difficulty - 1
    } else {
        prev_adjustment_block.difficulty
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let hash = calculate_hash(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D",
            1465154705,
            "get hash",
            0,
            0,
        );

        assert_eq!(hash, "278d7ac5b56a22896069f3064ab82ca610068c5c6494a2fa1658f02741349444");
    }

    #[test]
    fn test_block_generate() {
        let previous = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
            0,
            0,
        );
        let data = "next block";
        let next = Block::generate(data.to_string(), &previous, 0);
        let timestamp = Utc::now().timestamp() as usize;
        assert_eq!(next.index, 1);
        assert_eq!(next.timestamp, timestamp);
        assert_eq!(next.hash, calculate_hash(1, previous.hash.as_str(), timestamp, &data, 0, 0));
        assert_eq!(next.data, data);
    }

    #[test]
    fn test_block_calculated_hash() {
        let block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        assert_eq!(block.get_calculated_hash(), calculate_hash(0, "", 1465154705, "block", 0, 0));
    }

    #[test]
    fn test_block_get_is_valid_structure() {
        let invalid = Block::new(
            0,
            "".to_string(),
            "valid".to_string(),
            1465154705,
            "valid".to_string(),
            0,
            0,
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "".to_string(),
            1465154705,
            "valid".to_string(),
            0,
            0,
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "valid".to_string(),
            1465154705,
            "".to_string(),
            0,
            0,
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "valid".to_string(),
            1465154705,
            "valid".to_string(),
            0,
            0,
        );
        assert!(invalid.get_is_valid_structure());
    }

    #[test]
    fn test_block_get_is_valid_hash() {
        let block = Block::new(
            0,
            "278d7ac5b56a22896069f3064ab82ca610068c5c6494a2fa1658f02741349444".to_string(),
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            1465154705,
            "get hash".to_string(),
            0,
            0,
        );
        assert!(block.get_is_valid_hash());

        let mut block = Block::new(
            0,
            "278d7ac5b56a22896069f3064ab82ca610068c5c6494a2fa1658f02741349444".to_string(),
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            1465154705,
            "get hash".to_string(),
            0,
            0,
        );
        block.hash = "invalid".to_string();
        assert!(!block.get_is_valid_hash());

        let mut block = Block::new(
            0,
            "278d7ac5b56a22896069f3064ab82ca610068c5c6494a2fa1658f02741349444".to_string(),
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            1465154705,
            "get hash".to_string(),
            0,
            0,
        );
        block.difficulty = 2;
        assert!(!block.get_is_valid_hash());
    }

    #[test]
    fn test_block_equal() {
        let a = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        let b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        assert_eq!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        b.index = 1;
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        b.hash = "invalid".to_string();
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        b.previous_hash = "invalid".to_string();
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        b.timestamp = 0;
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        b.data = "invalid".to_string();
        assert_ne!(a, b);
    }

    #[test]
    fn test_block_clone() {
        let a = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        );
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_get_is_valid_timestamp() {
        let previous = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            Utc::now().timestamp() as usize,
            "prev block".to_string(),
            0,
            0,
        );
        let next = Block::generate("next block".to_string(), &previous, 0);
        assert!(get_is_valid_timestamp(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.timestamp = previous.timestamp + TIMESTAMP_INTERVAL + 1;
        assert!(!get_is_valid_timestamp(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.timestamp = Utc::now().timestamp() as usize - TIMESTAMP_INTERVAL - 1;
        assert!(!get_is_valid_timestamp(&next, &previous));
    }

    #[test]
    fn test_get_is_valid_new_block() {
        let previous = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
            0,
            0,
        );
        let next = Block::generate("next block".to_string(), &previous, 0);
        assert!(get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.index = 2;
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.previous_hash = "invalid".to_string();
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.data = "invalid".to_string();
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.timestamp = previous.timestamp + TIMESTAMP_INTERVAL + 1;
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous, 0);
        next.timestamp = previous.timestamp + TIMESTAMP_INTERVAL + 1;
        assert!(!get_is_valid_new_block(&next, &previous));
    }

    #[test]
    fn test_get_is_valid_chain() {
        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        );
        let blockchain = vec![genesis_block.clone()];
        assert!(get_is_valid_chain(&genesis_block, &blockchain));

        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        );
        let next_block = Block::generate("next block".to_string(), &genesis_block, 0);
        let blockchain = vec![
            genesis_block.clone(),
            next_block.clone(),
        ];
        assert!(get_is_valid_chain(&genesis_block, &blockchain));

        let other_genesis_block = Block::new(
            1,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "other genesis block".to_string(),
            0,
            0,
        );
        let blockchain = vec![genesis_block.clone()];
        assert!(!get_is_valid_chain(&other_genesis_block, &blockchain));

        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        );
        let mut next_block = Block::generate("next block".to_string(), &genesis_block, 0);
        next_block.index = 2;
        let blockchain = vec![
            genesis_block.clone(),
            next_block.clone(),
        ];
        assert!(!get_is_valid_chain(&genesis_block, &blockchain));
    }

    #[test]
    fn test_get_accumulated_difficulty() {
        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        );
        let mut blockchain = vec![genesis_block.clone()];
        assert_eq!(get_accumulated_difficulty(&blockchain), 1);

        let mut blockchain = vec![
            genesis_block.clone(),
            Block::generate("next block".to_string(), &genesis_block, 2),
        ];
        assert_eq!(get_accumulated_difficulty(&blockchain), 5);

        let mut blockchain = vec![
            genesis_block.clone(),
            Block::generate("next block".to_string(), &genesis_block, 2),
            Block::generate("next block".to_string(), &genesis_block, 2),
        ];
        assert_eq!(get_accumulated_difficulty(&blockchain), 9);
    }

    #[test]
    fn test_get_last_block() {
        let blockchain = vec![Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        )];
        assert_eq!(get_latest_block(&blockchain) as *const Block, blockchain.last().unwrap() as *const Block);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = vec![Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        )];
        assert!(add_block(&mut blockchain, "next block".to_string()).is_ok());
        assert_eq!(blockchain.len(), 2);
    }

    #[test]
    fn test_get_is_replace_chain() {
        let blockchain = vec![Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        )];
        let previous = get_latest_block(&blockchain);

        let mut new_blockchain = blockchain.clone();
        new_blockchain.push(Block::generate("next block".to_string(), previous, 0));
        assert!(get_is_replace_chain(&blockchain, &new_blockchain));

        let mut next = Block::generate("next block".to_string(), previous, 0);
        next.hash = "invalid".to_string();
        let mut new_blockchain = blockchain.clone();
        new_blockchain.push(next);
        assert!(!get_is_replace_chain(&blockchain, &new_blockchain));

        let mut new_blockchain = blockchain.clone();
        new_blockchain.push(Block::generate("next block".to_string(), previous, 1));
        assert!(get_is_replace_chain(&blockchain, &new_blockchain));

        let mut a_blockchain = blockchain.clone();
        a_blockchain.push(Block::generate("next block".to_string(), previous, 1));
        let mut b_blockchain = blockchain.clone();
        b_blockchain.push(Block::generate("next block".to_string(), previous, 0));
        assert!(!get_is_replace_chain(&a_blockchain, &b_blockchain));
    }

    #[test]
    fn test_get_difficulty() {
        let mut blockchain = vec![Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
            0,
            0,
        )];
        let difficulty = get_difficulty(&blockchain);
        assert_eq!(difficulty, 0);

        for i in 1..11 {
            add_block(&mut blockchain, format!("next block {i}")).expect("error");
        }
        let difficulty = get_difficulty(&blockchain);
        assert_eq!(difficulty, 1);
    }
}
