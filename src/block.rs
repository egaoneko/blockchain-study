use std::fmt;
use sha2::{Sha256, Digest};
use chrono::{Utc};
use crate::errors::AppError;

pub struct Block {
    pub index: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
}

impl Block {
    pub fn new(index: u64, hash: String, previous_hash: String, timestamp: i64, data: String) -> Block {
        Block {
            index,
            hash,
            previous_hash,
            timestamp,
            data,
        }
    }

    pub fn generate(data: String, previous: &Block) -> Block {
        let index = previous.index + 1;
        let timestamp = Utc::now().timestamp();
        let hash = calculate_hash(index, &previous.hash, timestamp, &data);
        Block::new(
            index,
            hash,
            previous.hash.to_string(),
            timestamp,
            data,
        )
    }

    pub fn calculate_hash(&self) -> String {
        calculate_hash(self.index, &self.previous_hash, self.timestamp, &self.data)
    }

    pub fn get_is_valid_structure(&self) -> bool {
        !self.hash.is_empty() && !self.previous_hash.is_empty() && !self.data.is_empty()
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block {{ index: {}, hash: {}, previous_hash: {}, timestamp: {}, data: {} }}", self.index, self.hash, self.previous_hash, self.timestamp, self.data)
    }
}

fn calculate_hash(index: u64, previous_hash: &str, timestamp: i64, data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}{}", index, previous_hash, timestamp, data).as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_is_valid_new_block(new_block: &Block, previous_block: &Block) -> bool {
    if !new_block.get_is_valid_structure() {
        return false;
    }

    if previous_block.index + 1 != new_block.index {
        return false;
    }

    if previous_block.hash != new_block.previous_hash {
        return false;
    }

    if new_block.calculate_hash() != new_block.hash {
        return false;
    }

    true
}

pub fn get_latest_block(blockchain: &[Block]) -> &Block {
    blockchain.last().unwrap()
}

pub fn add_block(blockchain: &[Block], new_block: &Block) -> Result<(), AppError> {
    if !get_is_valid_new_block(new_block, get_latest_block(blockchain)) {
        Err(AppError::new(1000))
    } else {
        Ok(())
    }
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
        );

        assert_eq!(hash, "60bdc20dfa04847d6ebd5fc49a7b84c97eb7e577b9c2cd1af4d4e233e259d9c9");
    }

    #[test]
    fn test_block_generate() {
        let previous = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
        );
        let data = "next block";
        let next = Block::generate(data.to_string(), &previous);
        let timestamp = Utc::now().timestamp();
        assert_eq!(next.index, 1);
        assert_eq!(next.timestamp, timestamp);
        assert_eq!(next.hash, calculate_hash(1, &previous.hash, timestamp, &data));
        assert_eq!(next.data, data);
    }

    #[test]
    fn test_block_calculate_hash() {
        let block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
        );
        assert_eq!(block.calculate_hash(), calculate_hash(0, "", 1465154705, "prev block"));
    }

    #[test]
    fn test_block_validate() {
        let invalid = Block::new(
            0,
            "".to_string(),
            "valid".to_string(),
            1465154705,
            "valid".to_string(),
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "".to_string(),
            1465154705,
            "valid".to_string(),
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "valid".to_string(),
            1465154705,
            "".to_string(),
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "valid".to_string(),
            1465154705,
            "valid".to_string(),
        );
        assert!(invalid.get_is_valid_structure());
    }

    #[test]
    fn test_new_block_validate() {
        let previous = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
        );
        let next = Block::generate("next block".to_string(), &previous);
        assert!(get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous);
        next.index = 2;
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous);
        next.hash = "invalid".to_string();
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous);
        next.previous_hash = "invalid".to_string();
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate("next block".to_string(), &previous);
        next.data = "invalid".to_string();
        assert!(!get_is_valid_new_block(&next, &previous));
    }

    #[test]
    fn test_get_last_block() {
        let blockchain = [Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
        )];
        assert_eq!(get_latest_block(&blockchain) as *const Block, blockchain.last().unwrap() as *const Block);
    }

    #[test]
    fn test_add_block() {
        let blockchain = [Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "prev block".to_string(),
        )];
        let previous = get_latest_block(&blockchain);
        let next = Block::generate("next block".to_string(), previous);
        assert!(add_block(&blockchain, &next).is_ok());

        let mut next = Block::generate("next block".to_string(), previous);
        next.data = "invalid".to_string();
        assert!(add_block(&blockchain, &next).is_err());
    }
}
