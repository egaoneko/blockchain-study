use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1};
use hex;
use crate::errors::AppError;

use crate::transaction::get_public_key;
use crate::UnspentTxOut;

#[derive(Debug)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
}

impl Wallet {
    pub fn new(private_key_path: String) -> Wallet {
        let (private_key, public_key) = get_keypair(private_key_path).unwrap();

        Wallet {
            private_key,
            public_key,
        }
    }
}

fn get_keypair_from_file(file: File) -> Result<(String, String), AppError> {
    let mut private_key = String::from("");
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(key) = line {
            private_key = key;
        } else {
            return Err(AppError::new(3000));
        }
    }
    let public_key = get_public_key(&private_key);

    Ok((private_key, public_key))
}

fn create_keypair(private_key_path: &str) -> Result<(String, String), AppError> {
    let secp = Secp256k1::new();
    let keypair = secp.generate_keypair(&mut OsRng);
    let private_key = hex::encode(keypair.0.secret_bytes());
    let public_key = keypair.1.to_string();

    let path = Path::new(private_key_path);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    if let Ok(mut buffer) = File::create(private_key_path) {
        if buffer.write(private_key.as_bytes()).is_err() {
            return Err(AppError::new(3002));
        }
    } else {
        return Err(AppError::new(3001));
    }


    Ok((private_key, public_key))
}

fn get_keypair(private_key_path: String) -> Result<(String, String), AppError> {
    return if let Ok(file) = File::open(&private_key_path) {
        get_keypair_from_file(file)
    } else {
        create_keypair(&private_key_path)
    };
}

pub fn get_balance(address: &str, unspent_tx_outs: &Vec<UnspentTxOut>) -> usize {
    unspent_tx_outs
        .into_iter()
        .filter(|u_tx_o| u_tx_o.address.eq(address))
        .map(|u_tx_o| u_tx_o.amount)
        .sum()
}

#[cfg(test)]
mod test {
    use std::fs::{File, remove_file};
    use super::*;

    #[test]
    fn test_new() {
        let path = "sample/private_key";
        let wallet = Wallet::new(path.to_string());

        let file = File::open(&path).unwrap();
        let (private_key, public_key) = get_keypair_from_file(file).unwrap();
        assert_eq!(wallet.private_key, private_key);
        assert_eq!(wallet.public_key, public_key);

        let wallet = Wallet::new(path.to_string());
        assert_eq!(wallet.private_key, private_key);
        assert_eq!(wallet.public_key, public_key);

        remove_file(&path).unwrap();
    }

    #[test]
    fn test_get_balance() {
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            ),
            UnspentTxOut::new(
                "05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            ),
            UnspentTxOut::new(
                "69202784cf6c645b87027eb1ccc0500609182f9f76f5be6e2fbe60bb1037b6ed".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            ),
            UnspentTxOut::new(
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                0,
                "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40".to_string(),
                50,
            ),
        ];

        assert_eq!(get_balance("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b", &unspent_tx_outs), 150);
        assert_eq!(get_balance("03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40", &unspent_tx_outs), 50);
    }
}
