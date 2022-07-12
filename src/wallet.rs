use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1};
use hex;
use crate::errors::AppError;

use crate::transaction::{get_public_key, sign_tx_in, Transaction, TxIn, TxOut};
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

fn find_tx_outs_for_amount(my_unspent_tx_outs: &Vec<UnspentTxOut>, amount: usize) -> Result<(Vec<UnspentTxOut>, usize), AppError> {
    let mut current_amount = 0;
    let mut included_unspent_tx_outs = vec![];
    for my_unspent_tx_out in my_unspent_tx_outs {
        included_unspent_tx_outs.push(my_unspent_tx_out.clone());
        current_amount = current_amount + my_unspent_tx_out.amount;

        if current_amount >= amount {
            return Ok((included_unspent_tx_outs, current_amount - amount));
        }
    }
    Err(AppError::new(2002))
}

fn create_tx_outs(receiver_address: &str, my_address: &str, amount: usize, left_over_amount: usize) -> Vec<TxOut> {
    let tx_out: TxOut = TxOut::new(receiver_address.to_string(), amount);
    return if left_over_amount == 0 {
        vec![tx_out]
    } else {
        vec![tx_out, TxOut::new(my_address.to_string(), left_over_amount)]
    };
}

pub fn get_balance(address: &str, unspent_tx_outs: &Vec<UnspentTxOut>) -> usize {
    unspent_tx_outs
        .into_iter()
        .filter(|u_tx_o| u_tx_o.address.eq(address))
        .map(|u_tx_o| u_tx_o.amount)
        .sum()
}

pub fn create_transaction(
    receiver_address: &str,
    amount: usize,
    wallet: &Wallet,
    unspent_tx_outs: &Vec<UnspentTxOut>,
) -> Result<Transaction, AppError> {
    let my_address = wallet.public_key.as_str();
    let my_unspent_tx_outs = unspent_tx_outs
        .into_iter()
        .filter(|&u_tx_o| u_tx_o.address.eq(my_address))
        .map(|v| v.clone())
        .collect::<Vec<UnspentTxOut>>();
    let (included_unspent_tx_outs, left_over_amount) = find_tx_outs_for_amount(&my_unspent_tx_outs, amount)?;

    let tx_ins = included_unspent_tx_outs
        .into_iter()
        .map(|unspent_tx_out| TxIn::new(unspent_tx_out.tx_out_id.clone(), unspent_tx_out.tx_out_index, "".to_string()))
        .collect();
    let tx_outs = create_tx_outs(receiver_address, my_address, amount, left_over_amount);

    let mut tx = Transaction::generate(&tx_ins, &tx_outs);

    tx.tx_ins = tx_ins
        .into_iter()
        .map(|tx_in| TxIn::new(
            tx_in.tx_out_id.clone(),
            tx_in.tx_out_index,
            sign_tx_in(&tx.id, &tx_in, &wallet.private_key, unspent_tx_outs).unwrap(),
        ))
        .collect();

    Ok(tx)
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
    fn test_find_tx_outs_for_amount() {
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
        ];

        let (included_unspent_tx_outs, left_over_amount) = find_tx_outs_for_amount(&unspent_tx_outs, 100).unwrap();
        assert_eq!(included_unspent_tx_outs.len(), 2);
        assert_eq!(included_unspent_tx_outs.get(0).unwrap().tx_out_id, "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea");
        assert_eq!(included_unspent_tx_outs.get(1).unwrap().tx_out_id, "05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e");
        assert_eq!(left_over_amount, 0);

        let (included_unspent_tx_outs, left_over_amount) = find_tx_outs_for_amount(&unspent_tx_outs, 70).unwrap();
        assert_eq!(included_unspent_tx_outs.len(), 2);
        assert_eq!(included_unspent_tx_outs.get(0).unwrap().tx_out_id, "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea");
        assert_eq!(included_unspent_tx_outs.get(1).unwrap().tx_out_id, "05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e");
        assert_eq!(left_over_amount, 30);

        assert!(find_tx_outs_for_amount(&unspent_tx_outs, 200).is_err());
    }

    #[test]
    fn test_create_tx_outs() {
        let tx_outs = create_tx_outs(
            "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40",
            "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b",
            50,
            0,
        );
        assert_eq!(tx_outs.len(), 1);

        let actual = tx_outs.get(0).unwrap();
        assert_eq!(actual.address, "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40");
        assert_eq!(actual.amount, 50);

        let tx_outs = create_tx_outs(
            "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40",
            "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b",
            50,
            20,
        );
        assert_eq!(tx_outs.len(), 2);

        let actual = tx_outs.get(0).unwrap();
        assert_eq!(actual.address, "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40");
        assert_eq!(actual.amount, 50);

        let actual = tx_outs.get(1).unwrap();
        assert_eq!(actual.address, "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b");
        assert_eq!(actual.amount, 20);
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

    #[test]
    fn test_create_transaction() {
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

        let tx = create_transaction(
            "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40",
            50,
            &wallet,
            &unspent_tx_outs,
        ).unwrap();
        assert_eq!(tx.tx_ins.len(), 1);
        assert_eq!(tx.tx_outs.get(0).unwrap().amount, 50);

        let tx = create_transaction(
            "03b375875391f1dcd5af49e64a477d1be23ccbd0c7765bdde1b46072fb3703ec40",
            150,
            &wallet,
            &unspent_tx_outs,
        ).unwrap();
        assert_eq!(tx.tx_ins.len(), 3);
        assert_eq!(tx.tx_outs.get(0).unwrap().amount, 150);
    }
}
