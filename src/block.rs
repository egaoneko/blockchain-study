use std::fmt;
use sha2::{Sha256, Digest};
use chrono::{Utc};

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_hash() {
        let hash = calculate_hash(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D",
            1465154705,
            "get hash",
        );

        assert_eq!(hash, "60bdc20dfa04847d6ebd5fc49a7b84c97eb7e577b9c2cd1af4d4e233e259d9c9");
    }

    #[test]
    fn gen_one() {
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
}
