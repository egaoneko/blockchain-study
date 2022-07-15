use std::mem;
use sha2::{Sha256, Digest};
use chrono::{Utc};
use serde::{Serialize, Deserialize};

use crate::errors::AppError;
use crate::transaction::{get_coinbase_transaction, process_transactions, Transaction};
use crate::transaction_pool::update_transaction_pool;
use crate::UnspentTxOut;
use crate::utils::get_is_hash_matches_difficulty;
use crate::wallet::{create_transaction, Wallet};

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
    pub data: Vec<Transaction>,

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
        data: Vec<Transaction>,
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
    pub fn generate(data: &Vec<Transaction>, previous: &Block, difficulty: usize) -> Block {
        let index = previous.index + 1;
        let timestamp = Utc::now().timestamp() as usize;
        let mut nonce = 0;

        loop {
            let hash = calculate_hash(index, previous.hash.as_str(), timestamp, data, difficulty, nonce);

            if !get_is_hash_matches_difficulty(hash.as_str(), difficulty) {
                nonce += 1;
                continue;
            }

            return Block::new(
                index,
                hash,
                previous.hash.to_string(),
                timestamp,
                data.to_vec(),
                difficulty,
                nonce,
            );
        }
    }

    /// Generate a raw block with data
    pub fn generate_raw(blockchain: &Vec<Block>, data: &Vec<Transaction>) -> Block {
        let latest = get_latest_block(blockchain);
        let difficulty = get_difficulty(blockchain);
        Block::generate(data, latest, difficulty)
    }

    /// Generate a block with coinbase transaction and previous block
    pub fn generate_with_coinbase_transaction(blockchain: &Vec<Block>, transaction_pool: &Vec<Transaction>, wallet: &Wallet) -> Block {
        let latest = get_latest_block(blockchain);
        Block::generate_raw(
            blockchain,
            &vec![
                get_coinbase_transaction(wallet.public_key.as_str(), latest.index + 1),
            ]
                .into_iter()
                .chain(transaction_pool.clone())
                .collect(),
        )
    }

    /// Generate a block with transaction
    pub fn generate_with_transaction(
        blockchain: &Vec<Block>,
        wallet: &Wallet,
        unspent_tx_outs: &Vec<UnspentTxOut>,
        receiver_address: &str,
        amount: usize,
    ) -> Result<Block, AppError> {
        let latest = get_latest_block(blockchain);
        let coinbase_tx = get_coinbase_transaction(wallet.public_key.as_str(), latest.index + 1);
        let tx = create_transaction(receiver_address, amount, wallet, unspent_tx_outs)?;
        Ok(Block::generate_raw(blockchain, &vec![coinbase_tx, tx]))
    }

    /// Recalculate and return hash
    pub fn get_calculated_hash(&self) -> String {
        calculate_hash(self.index, self.previous_hash.as_str(), self.timestamp, &self.data, self.difficulty, self.nonce)
    }

    /// Return structure is valid
    pub fn get_is_valid_structure(&self) -> bool {
        !self.hash.is_empty() && !self.previous_hash.is_empty()
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

fn calculate_hash(index: usize, previous_hash: &str, timestamp: usize, data: &Vec<Transaction>, difficulty: usize, nonce: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}{}{}{}", index, previous_hash, timestamp, serde_json::to_string(&data).unwrap(), difficulty, nonce).as_bytes());
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
    if let Some(last) = blockchain.get(0) {
        if genesis_block != last {
            false
        } else if blockchain.len() == 1 {
            true
        } else {
            blockchain.windows(2).all(|window| get_is_valid_new_block(&window[1], &window[0]))
        }
    } else {
        false
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
pub fn add_block(blockchain: &mut Vec<Block>, unspent_tx_outs: &mut Vec<UnspentTxOut>, transaction_pool: &mut Vec<Transaction>, new_block: &Block) -> Result<(), AppError> {
    if !get_is_valid_new_block(&new_block, get_latest_block(blockchain)) {
        Err(AppError::new(1000))
    } else {
        let processed_unspent_tx_outs = process_transactions(&new_block.data, unspent_tx_outs, new_block.index)?;
        blockchain.push(new_block.clone());
        let _ = mem::replace(&mut *unspent_tx_outs, processed_unspent_tx_outs);
        let updated_transaction_pool = update_transaction_pool(transaction_pool, unspent_tx_outs);
        let _ = mem::replace(&mut *transaction_pool, updated_transaction_pool);
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

/// Get UnspentTxOut from blockchain.
pub fn get_unspent_tx_outs(blockchain: &Vec<Block>) -> Result<Vec<UnspentTxOut>, AppError> {
    let mut unspent_tx_outs = vec![];
    blockchain.into_iter().for_each(|block| {
        unspent_tx_outs = process_transactions(&block.data, &unspent_tx_outs, block.index).unwrap();
    });
    Ok(unspent_tx_outs)
}

#[cfg(test)]
mod test {
    use crate::transaction::{TxIn, TxOut};
    use crate::constants::COINBASE_AMOUNT;
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let hash = calculate_hash(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d",
            1465154705,
            &vec![],
            0,
            0,
        );

        assert_eq!(hash, "12c7538225556354e750653f746fea1414b43fb09062f279162725d7748df7c9");

        let hash = calculate_hash(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d",
            1465154705,
            &vec![Transaction::generate(&vec![], &vec![])],
            0,
            0,
        );
        assert_eq!(hash, "e57a5313832eb6755a61a9ea87308ebfe04cb5aea378b3a0c0e2fba1051ceb1e");
    }

    #[test]
    fn test_block_generate() {
        let previous = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let data = vec![];
        let next = Block::generate(&data, &previous, 0);
        let timestamp = Utc::now().timestamp() as usize;
        assert_eq!(next.index, 1);
        assert_eq!(next.timestamp, timestamp);
        assert_eq!(next.hash, calculate_hash(1, previous.hash.as_str(), timestamp, &data, 0, 0));
        assert_eq!(next.data, data);
    }

    #[test]
    fn test_block_generate_raw() {
        let previous = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let data = vec![];
        let blockchain = vec![previous.clone()];
        let next = Block::generate_raw(&blockchain, &data);
        let timestamp = Utc::now().timestamp() as usize;
        assert_eq!(next.index, 1);
        assert_eq!(next.timestamp, timestamp);
        assert_eq!(next.hash, calculate_hash(1, previous.hash.as_str(), timestamp, &data, 0, 0));
        assert_eq!(next.data, data);
    }

    #[test]
    fn test_block_generate_with_coinbase_transaction() {
        let wallet = Wallet {
            private_key: "eb35a95c6c1bcd1164e5f23629797131bd24aae3995b831be94c8e8fa37ee2d8".to_string(),
            public_key: "03196c144d93ba0ca200221b507312a41c67eafb9b0d9b9348b286a693969b8192".to_string(),
        };
        let previous = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let blockchain = vec![previous];
        let transaction_pool = vec![];
        let block = Block::generate_with_coinbase_transaction(&blockchain, &transaction_pool, &wallet);
        let timestamp = Utc::now().timestamp() as usize;
        assert_eq!(block.index, 1);
        assert_eq!(block.timestamp, timestamp);
        assert_eq!(block.data.len(), 1);

        let tx = block.data.get(0).unwrap();
        let tx_out = tx.tx_outs.get(0).unwrap();
        assert_eq!(tx_out.address, "03196c144d93ba0ca200221b507312a41c67eafb9b0d9b9348b286a693969b8192");
        assert_eq!(tx_out.amount, COINBASE_AMOUNT);

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transaction_pool = vec![Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs)];
        let block = Block::generate_with_coinbase_transaction(&blockchain, &transaction_pool, &wallet);
        assert_eq!(block.data.len(), 2);
    }

    #[test]
    fn test_block_generate_with_transaction() {
        let wallet = Wallet {
            private_key: "eb35a95c6c1bcd1164e5f23629797131bd24aae3995b831be94c8e8fa37ee2d8".to_string(),
            public_key: "03196c144d93ba0ca200221b507312a41c67eafb9b0d9b9348b286a693969b8192".to_string(),
        };
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                wallet.public_key.to_string(),
                50,
            ),
            UnspentTxOut::new(
                "05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e".to_string(),
                0,
                wallet.public_key.to_string(),
                50,
            ),
            UnspentTxOut::new(
                "69202784cf6c645b87027eb1ccc0500609182f9f76f5be6e2fbe60bb1037b6ed".to_string(),
                0,
                wallet.public_key.to_string(),
                50,
            ),
            UnspentTxOut::new(
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                0,
                "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40".to_string(),
                50,
            ),
        ];
        let previous = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let blockchain = vec![previous];
        let block = Block::generate_with_transaction(
            &blockchain,
            &wallet,
            &unspent_tx_outs,
            "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40",
            150,
        ).unwrap();
        let timestamp = Utc::now().timestamp() as usize;
        assert_eq!(block.index, 1);
        assert_eq!(block.timestamp, timestamp);

        let tx = block.data.get(0).unwrap();
        let tx_out = tx.tx_outs.get(0).unwrap();
        assert_eq!(tx_out.address, "03196c144d93ba0ca200221b507312a41c67eafb9b0d9b9348b286a693969b8192");
        assert_eq!(tx_out.amount, COINBASE_AMOUNT);

        let tx = block.data.get(1).unwrap();
        let tx_out = tx.tx_outs.get(0).unwrap();
        assert_eq!(tx_out.address, "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40");
        assert_eq!(tx_out.amount, 150);
    }

    #[test]
    fn test_block_calculated_hash() {
        let block = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        assert_eq!(block.get_calculated_hash(), calculate_hash(0, "", 1465154705, &vec![], 0, 0));
    }

    #[test]
    fn test_block_get_is_valid_structure() {
        let invalid = Block::new(
            0,
            "".to_string(),
            "valid".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        assert!(!invalid.get_is_valid_structure());

        let invalid = Block::new(
            0,
            "valid".to_string(),
            "valid".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        assert!(invalid.get_is_valid_structure());
    }

    #[test]
    fn test_block_get_is_valid_hash() {
        let block = Block::new(
            0,
            "12c7538225556354e750653f746fea1414b43fb09062f279162725d7748df7c9".to_string(),
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        assert!(block.get_is_valid_hash());

        let mut block = Block::new(
            0,
            "12c7538225556354e750653f746fea1414b43fb09062f279162725d7748df7c9".to_string(),
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        block.hash = "invalid".to_string();
        assert!(!block.get_is_valid_hash());

        let mut block = Block::new(
            0,
            "12c7538225556354e750653f746fea1414b43fb09062f279162725d7748df7c9".to_string(),
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            1465154705,
            vec![],
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
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let b = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        assert_eq!(a, b);

        let mut b = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        b.index = 1;
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        b.hash = "invalid".to_string();
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        b.previous_hash = "invalid".to_string();
        assert_ne!(a, b);

        let mut b = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        b.timestamp = 0;
        assert_ne!(a, b);

        let b = Block::new(
            0,
            "e57a5313832eb6755a61a9ea87308ebfe04cb5aea378b3a0c0e2fba1051ceb1e".to_string(),
            "".to_string(),
            1465154705,
            vec![Transaction::generate(&vec![], &vec![])],
            0,
            0,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn test_block_clone() {
        let a = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
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
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            Utc::now().timestamp() as usize,
            vec![],
            0,
            0,
        );
        let next = Block::generate(&vec![], &previous, 0);
        assert!(get_is_valid_timestamp(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.timestamp = previous.timestamp + TIMESTAMP_INTERVAL + 1;
        assert!(!get_is_valid_timestamp(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.timestamp = Utc::now().timestamp() as usize - TIMESTAMP_INTERVAL - 1;
        assert!(!get_is_valid_timestamp(&next, &previous));
    }

    #[test]
    fn test_get_is_valid_new_block() {
        let previous = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let next = Block::generate(&vec![], &previous, 0);
        assert!(get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.index = 2;
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.previous_hash = "invalid".to_string();
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.data = vec![Transaction::generate(&vec![], &vec![])];
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.timestamp = previous.timestamp + TIMESTAMP_INTERVAL + 1;
        assert!(!get_is_valid_new_block(&next, &previous));

        let mut next = Block::generate(&vec![], &previous, 0);
        next.timestamp = previous.timestamp + TIMESTAMP_INTERVAL + 1;
        assert!(!get_is_valid_new_block(&next, &previous));
    }

    #[test]
    fn test_get_is_valid_chain() {
        let genesis_block = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let blockchain = vec![genesis_block.clone()];
        assert!(get_is_valid_chain(&genesis_block, &blockchain));

        let genesis_block = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let next_block = Block::generate(&vec![], &genesis_block, 0);
        let blockchain = vec![
            genesis_block.clone(),
            next_block.clone(),
        ];
        assert!(get_is_valid_chain(&genesis_block, &blockchain));

        let other_genesis_block = Block::new(
            1,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let blockchain = vec![genesis_block.clone()];
        assert!(!get_is_valid_chain(&other_genesis_block, &blockchain));

        let genesis_block = Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let mut next_block = Block::generate(&vec![], &genesis_block, 0);
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
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        );
        let blockchain = vec![genesis_block.clone()];
        assert_eq!(get_accumulated_difficulty(&blockchain), 1);

        let blockchain = vec![
            genesis_block.clone(),
            Block::generate(&vec![], &genesis_block, 2),
        ];
        assert_eq!(get_accumulated_difficulty(&blockchain), 5);

        let blockchain = vec![
            genesis_block.clone(),
            Block::generate(&vec![], &genesis_block, 2),
            Block::generate(&vec![], &genesis_block, 2),
        ];
        assert_eq!(get_accumulated_difficulty(&blockchain), 9);
    }

    #[test]
    fn test_get_last_block() {
        let blockchain = vec![Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        )];
        assert_eq!(get_latest_block(&blockchain) as *const Block, blockchain.last().unwrap() as *const Block);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = vec![Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        )];
        let tx_ins = vec![
            TxIn::new(
                "".to_string(),
                1,
                "".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transactions = vec![
            Transaction::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), &tx_ins, &tx_outs)
        ];
        let mut unspent_tx_outs = vec![];
        let mut transaction_pool = vec![];
        let block = Block::generate_raw(&blockchain, &transactions);
        assert!(add_block(&mut blockchain, &mut unspent_tx_outs, &mut transaction_pool, &block).is_ok());
        assert_eq!(blockchain.len(), 2);
        assert_eq!(unspent_tx_outs.len(), 1);
        assert_eq!(transaction_pool.len(), 0);
    }

    #[test]
    fn test_get_is_replace_chain() {
        let blockchain = vec![Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        )];
        let previous = get_latest_block(&blockchain);

        let mut new_blockchain = blockchain.clone();
        new_blockchain.push(Block::generate(&vec![], previous, 0));
        assert!(get_is_replace_chain(&blockchain, &new_blockchain));

        let mut next = Block::generate(&vec![], previous, 0);
        next.hash = "invalid".to_string();
        let mut new_blockchain = blockchain.clone();
        new_blockchain.push(next);
        assert!(!get_is_replace_chain(&blockchain, &new_blockchain));

        let mut new_blockchain = blockchain.clone();
        new_blockchain.push(Block::generate(&vec![], previous, 1));
        assert!(get_is_replace_chain(&blockchain, &new_blockchain));

        let mut a_blockchain = blockchain.clone();
        a_blockchain.push(Block::generate(&vec![], previous, 1));
        let mut b_blockchain = blockchain.clone();
        b_blockchain.push(Block::generate(&vec![], previous, 0));
        assert!(!get_is_replace_chain(&a_blockchain, &b_blockchain));
    }

    #[test]
    fn test_get_difficulty() {
        let mut blockchain = vec![Block::new(
            0,
            "41cdda1f3f0f6bd2497997a6bbab3188090b0404c1da5fc854c174dd42cefd2d".to_string(),
            "".to_string(),
            1465154705,
            vec![],
            0,
            0,
        )];
        let mut unspent_tx_outs = vec![];
        let mut transaction_pool = vec![];
        let difficulty = get_difficulty(&blockchain);
        assert_eq!(difficulty, 0);

        for i in 1..11 {
            let tx_ins = vec![
                TxIn::new(
                    "".to_string(),
                    i,
                    "".to_string(),
                )
            ];
            let tx_outs = vec![
                TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
            ];
            let transactions = vec![Transaction::generate(&tx_ins, &tx_outs)];
            let block = Block::generate_raw(&blockchain, &transactions);
            add_block(&mut blockchain, &mut unspent_tx_outs, &mut transaction_pool, &block).expect("error");
        }
        let difficulty = get_difficulty(&blockchain);
        assert_eq!(difficulty, 1);
    }

    #[test]
    fn test_get_unspent_tx_outs() {
        let tx_ins = vec![
            TxIn::new(
                "".to_string(),
                1,
                "".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transactions = vec![
            Transaction::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), &tx_ins, &tx_outs)
        ];
        let genesis_transaction = Transaction::new(
            "b5516eb9915e9be6868575e87bb450d8285505f004f944bf0d99c6131995bf41".to_string(),
            &vec![TxIn::new("".to_string(), 0, "".to_string())],
            &vec![TxOut::new(
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )],
        );
        let genesis_block = Block::new(
            0,
            "c1fcd470499b2871ed8276cfcd3abbdca6ac1432515f30d59835c9d7e35e2756".to_string(),
            "".to_string(),
            1655831820,
            vec![genesis_transaction],
            0,
            0,
        );
        let mut blockchain = vec![
            genesis_block.clone(),
            Block::generate(&transactions, &genesis_block, 0),
        ];
        let unspent_tx_outs = get_unspent_tx_outs(&blockchain).unwrap();
        assert_eq!(unspent_tx_outs.len(), 2);
    }
}
