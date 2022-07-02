use sha2::{Sha256, Digest};
use chrono::{Utc};
use serde::{Serialize, Deserialize};

use crate::errors::AppError;

/// Block in blockchain has sequence, data, time, and so on.
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    /// Sequence in blockchain
    pub index: u64,

    /// Hash from other properties
    pub hash: String,

    /// Previous block hash
    pub previous_hash: String,

    /// Timestamp when created
    pub timestamp: i64,

    /// Data in block
    pub data: String,
}

impl Block {
    /// Returns a block with arguments
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::block::{Block};
    /// let block = Block::new(
    ///     0,
    ///     "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
    ///     "".to_string(),
    ///     1465154705,
    ///     "block".to_string(),
    /// );
    /// ```
    pub fn new(index: u64, hash: String, previous_hash: String, timestamp: i64, data: String) -> Block {
        Block {
            index,
            hash,
            previous_hash,
            timestamp,
            data,
        }
    }

    /// Generate a block with data and previous block
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::block::{Block};
    /// let previous = Block::new(
    ///     0,
    ///     "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
    ///     "".to_string(),
    ///     1465154705,
    ///     "previous block".to_string(),
    /// );
    /// let next = Block::generate("next block".to_string(), &previous);
    /// ```
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

    /// Recalculate and return hash
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::block::{Block};
    /// let block = Block::new(
    ///     0,
    ///     "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
    ///     "".to_string(),
    ///     1465154705,
    ///     "block".to_string(),
    /// );
    /// assert_eq!(block.get_calculated_hash(), block.hash);
    /// ```
    pub fn get_calculated_hash(&self) -> String {
        calculate_hash(self.index, &self.previous_hash, self.timestamp, &self.data)
    }

    /// Return structure is valid
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::block::{Block};
    /// let block = Block::new(
    ///     0,
    ///     "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
    ///     "".to_string(),
    ///     1465154705,
    ///     "block".to_string(),
    /// );
    /// assert!(block.get_is_valid_structure());
    /// ```
    pub fn get_is_valid_structure(&self) -> bool {
        !self.hash.is_empty() && !self.previous_hash.is_empty() && !self.data.is_empty()
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
        }
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

    if new_block.get_calculated_hash() != new_block.hash {
        return false;
    }

    true
}

fn get_is_valid_chain(genesis_block: &Block, blockchain: &[Block]) -> bool {
    if genesis_block != blockchain.get(0).unwrap() {
        false
    } else if blockchain.len() == 1 {
        true
    } else {
        blockchain.windows(2).all(|window| get_is_valid_new_block(&window[1], &window[0]))
    }
}

/// Get latest block from blockchain.
///
/// # Examples
///
/// ```
/// use blockchain::block::{Block, get_latest_block};
/// let blockchain = [Block::new(
///     0,
///     "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
///     "".to_string(),
///     1465154705,
///     "genesis block".to_string(),
/// )];
/// assert_eq!(get_latest_block(&blockchain) as *const Block, blockchain.last().unwrap() as *const Block);
/// ```
pub fn get_latest_block(blockchain: &[Block]) -> &Block {
    blockchain.last().unwrap()
}

/// Add block to blockchain.
///
/// # Examples
///
/// ```
/// use blockchain::block::{Block, get_latest_block, add_block};
/// let blockchain = [Block::new(
///     0,
///     "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
///     "".to_string(),
///     1465154705,
///     "genesis block".to_string(),
/// )];
/// let previous = get_latest_block(&blockchain);
/// let next = Block::generate("next block".to_string(), previous);
/// assert!(add_block(&blockchain, &next).is_ok());
/// ```
///
/// # Errors
///
/// If it is not valid compared to the previous block, it returns error 1000.
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
    fn test_block_calculated_hash() {
        let block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        assert_eq!(block.get_calculated_hash(), calculate_hash(0, "", 1465154705, "block"));
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
    fn test_block_equal() {
        let a = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        let b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        assert_eq!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        b.index = 1;
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        b.hash = "invalid".to_string();
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        b.previous_hash = "invalid".to_string();
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
        );
        b.timestamp = 0;
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
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
        );
        let b = a.clone();
        assert_eq!(a, b);
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
    fn test_chain_validate() {
        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
        );
        let blockchain = [genesis_block.clone()];
        assert!(get_is_valid_chain(&genesis_block, &blockchain));

        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
        );
        let next_block = Block::generate("next block".to_string(), &genesis_block);
        let blockchain = [
            genesis_block.clone(),
            next_block.clone()
        ];
        assert!(get_is_valid_chain(&genesis_block, &blockchain));

        let other_genesis_block = Block::new(
            1,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "other genesis block".to_string(),
        );
        let blockchain = [genesis_block.clone()];
        assert!(!get_is_valid_chain(&other_genesis_block, &blockchain));

        let genesis_block = Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
        );
        let mut  next_block = Block::generate("next block".to_string(), &genesis_block);
        next_block.index = 2;
        let blockchain = [
            genesis_block.clone(),
            next_block.clone()
        ];
        assert!(!get_is_valid_chain(&genesis_block, &blockchain));
    }

    #[test]
    fn test_get_last_block() {
        let blockchain = [Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "genesis block".to_string(),
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
            "genesis block".to_string(),
        )];
        let previous = get_latest_block(&blockchain);
        let next = Block::generate("next block".to_string(), previous);
        assert!(add_block(&blockchain, &next).is_ok());

        let mut next = Block::generate("next block".to_string(), previous);
        next.data = "invalid".to_string();
        assert!(add_block(&blockchain, &next).is_err());
    }
}
