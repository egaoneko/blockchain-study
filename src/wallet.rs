use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1};
use hex;
use crate::errors::AppError;

use crate::transaction::get_public_key;

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

#[cfg(test)]
mod test {
    use std::fs::{File, remove_file};
    use super::*;
    use crate::constants::PRIVATE_KEY_PATH;

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
}
