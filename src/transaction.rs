use std::collections::HashMap;
use std::str::FromStr;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use secp256k1::{Secp256k1, ecdsa, PublicKey, SecretKey};
use crate::errors::AppError;
use crate::secp256k1::{message_from_str};

const COINBASE_AMOUNT: usize = 50;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnspentTxOut {
    pub tx_out_id: String,
    pub tx_out_index: usize,
    pub address: String,
    pub amount: usize,
}

impl UnspentTxOut {
    pub fn new(tx_out_id: String, tx_out_index: usize, address: String, amount: usize) -> UnspentTxOut {
        UnspentTxOut {
            tx_out_id,
            tx_out_index,
            address,
            amount,
        }
    }
}

impl Clone for UnspentTxOut {
    fn clone(&self) -> Self {
        Self {
            tx_out_id: self.tx_out_id.clone(),
            tx_out_index: self.tx_out_index.clone(),
            address: self.address.clone(),
            amount: self.amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxIn {
    pub tx_out_id: String,
    pub tx_out_index: usize,
    pub signature: String,
}

impl TxIn {
    pub fn new(tx_out_id: String, tx_out_index: usize, signature: String) -> TxIn {
        TxIn {
            tx_out_id,
            tx_out_index,
            signature,
        }
    }
}

impl Clone for TxIn {
    fn clone(&self) -> Self {
        Self {
            tx_out_id: self.tx_out_id.clone(),
            tx_out_index: self.tx_out_index.clone(),
            signature: self.signature.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxOut {
    pub address: String,
    pub amount: usize,
}

impl TxOut {
    pub fn new(address: String, amount: usize) -> TxOut {
        TxOut {
            address,
            amount,
        }
    }
}

impl Clone for TxOut {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            amount: self.amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,
}

impl Transaction {
    pub fn generate(tx_ins: &Vec<TxIn>, tx_outs: &Vec<TxOut>) -> Transaction {
        Transaction {
            id: get_transaction_id(tx_ins, tx_outs),
            tx_ins: tx_ins.to_vec(),
            tx_outs: tx_outs.to_vec(),
        }
    }

    pub fn new(id: String, tx_ins: &Vec<TxIn>, tx_outs: &Vec<TxOut>) -> Transaction {
        Transaction {
            id,
            tx_ins: tx_ins.to_vec(),
            tx_outs: tx_outs.to_vec(),
        }
    }

    pub fn get_transaction_id(&self) -> String {
        get_transaction_id(&self.tx_ins, &self.tx_outs)
    }
}

fn get_transaction_id(tx_ins: &Vec<TxIn>, tx_outs: &Vec<TxOut>) -> String {
    let tx_in_content = tx_ins.into_iter()
        .map(|tx_in: &TxIn| format!("{}{}", tx_in.tx_out_id.to_string(), tx_in.tx_out_index))
        .fold("".to_string(), |total: String, content: String| format!("{}{}", total, content));

    let tx_out_content = tx_outs.into_iter()
        .map(|tx_out: &TxOut| format!("{}{}", tx_out.address.to_string(), tx_out.amount))
        .fold("".to_string(), |total: String, content: String| format!("{}{}", total, content));

    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", tx_in_content, tx_out_content).as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_is_valid_tx_in(tx_in: &TxIn, transaction: &Transaction, unspent_tx_outs: &Vec<UnspentTxOut>) -> bool {
    let u_tx_out =
        unspent_tx_outs.into_iter().find(|u_tx_o| u_tx_o.tx_out_id.eq(&tx_in.tx_out_id));
    return if let Some(referenced_utx_out) = u_tx_out {
        let secp = Secp256k1::verification_only();
        let public_key = PublicKey::from_str(&referenced_utx_out.address).unwrap();
        let message = message_from_str(&transaction.id).unwrap();
        let sig = ecdsa::Signature::from_str(&tx_in.signature).unwrap();
        secp.verify_ecdsa(&message, &sig, &public_key).is_ok()
    } else {
        false
    };
}

fn find_unspent_tx_out<'a>(transaction_id: &'a str, index: usize, unspent_tx_outs: &'a Vec<UnspentTxOut>) -> Option<&'a UnspentTxOut> {
    unspent_tx_outs.into_iter().find(|u_tx_o| u_tx_o.tx_out_id.eq(transaction_id) && u_tx_o.tx_out_index == index)
}

fn get_tx_in_amount(tx_in: &TxIn, unspent_tx_outs: &Vec<UnspentTxOut>) -> usize {
    return if let Some(u_tx_o) = find_unspent_tx_out(tx_in.tx_out_id.as_str(), tx_in.tx_out_index, unspent_tx_outs) {
        u_tx_o.amount
    } else {
        0
    };
}

fn get_is_valid_transaction(transaction: &Transaction, unspent_tx_outs: &Vec<UnspentTxOut>) -> bool {
    if !transaction.get_transaction_id().eq(&transaction.id) {
        return false;
    }

    let ref_tx_ins = &transaction.tx_ins;

    let has_invalid_tx_ins = ref_tx_ins
        .into_iter()
        .any(|tx_in| !get_is_valid_tx_in(&tx_in, transaction, unspent_tx_outs));

    if has_invalid_tx_ins {
        return false;
    }

    let total_tx_in_values = ref_tx_ins
        .into_iter()
        .map(|tx_in| get_tx_in_amount(&tx_in, unspent_tx_outs))
        .fold(0, |sum, amount| sum + amount);

    let ref_tx_outs = &transaction.tx_outs;
    let total_tx_out_values = ref_tx_outs
        .into_iter()
        .map(|tx_out| tx_out.amount)
        .fold(0, |sum, amount| sum + amount);

    if total_tx_out_values != total_tx_in_values {
        return false;
    }

    true
}

fn get_is_valid_coinbase_tx(transaction: Option<&Transaction>, block_index: usize) -> bool {
    if transaction.is_none() {
        return false;
    }

    let transaction = transaction.unwrap();

    if !transaction.get_transaction_id().eq(&transaction.id) {
        return false;
    }

    if transaction.tx_ins.len() != 1 {
        return false;
    }

    let tx_in = transaction.tx_ins.get(0).unwrap();

    if tx_in.tx_out_index != block_index {
        return false;
    }

    if transaction.tx_outs.len() != 1 {
        return false;
    }

    let tx_out = transaction.tx_outs.get(0).unwrap();

    if tx_out.amount != COINBASE_AMOUNT {
        return false;
    }

    true
}

fn has_duplicates(tx_ins: &Vec<&TxIn>) -> bool {
    tx_ins
        .into_iter()
        .fold(HashMap::new(), |mut acc, tx_in| {
            let counter = acc.entry(format!("{}{}", tx_in.tx_out_id, tx_in.tx_out_index).to_string()).or_insert(0);
            *counter += 1;
            acc
        }).values().any(|count| *count > 1)
}

fn get_is_valid_block_transactions(transactions: &Vec<Transaction>, unspent_tx_outs: &Vec<UnspentTxOut>, block_index: usize) -> bool {
    let coinbase_tx = transactions.get(0);
    if !get_is_valid_coinbase_tx(coinbase_tx, block_index) {
        return false;
    }

    let tx_ins = transactions.into_iter()
        .map(|tx| &tx.tx_ins)
        .flatten()
        .collect();

    if has_duplicates(&tx_ins) {
        return false;
    }

    transactions.into_iter()
        .skip(1)
        .map(|tx| get_is_valid_transaction(tx, unspent_tx_outs))
        .all(|valid| valid)
}

fn update_unspent_tx_outs(new_transactions: &Vec<Transaction>, unspent_tx_outs: &Vec<UnspentTxOut>) -> Vec<UnspentTxOut> {
    let new_unspent_tx_outs: Vec<UnspentTxOut> = new_transactions
        .into_iter()
        .map(|t| {
            let ref_tx_outs = &t.tx_outs;
            ref_tx_outs
                .into_iter()
                .enumerate()
                .map(|(index, tx_out)| UnspentTxOut::new(t.id.clone(), index, tx_out.address.clone(), tx_out.amount))
        })
        .flatten()
        .collect();

    let consumed_tx_outs: Vec<UnspentTxOut> = new_transactions
        .into_iter()
        .map(|t| &t.tx_ins)
        .flatten()
        .map(|tx_in| UnspentTxOut::new(tx_in.tx_out_id.clone(), tx_in.tx_out_index, "".to_string(), 0))
        .collect();

    unspent_tx_outs
        .into_iter()
        .filter(|u_tx_o| find_unspent_tx_out(&u_tx_o.tx_out_id, u_tx_o.tx_out_index, &consumed_tx_outs).is_none())
        .map(|u_tx_o| u_tx_o.clone())
        .chain(new_unspent_tx_outs)
        .collect()
}

pub fn get_coinbase_transaction(address: String, block_index: usize) -> Transaction {
    return Transaction::generate(
        &vec![TxIn::new("".to_string(), block_index, "".to_string())],
        &vec![TxOut::new(address, COINBASE_AMOUNT)],
    );
}

pub fn get_public_key(private_key: &str) -> String {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_str(private_key).unwrap();
    PublicKey::from_secret_key(&secp, &secret_key).to_string()
}

pub fn sign_tx_in(
    transaction: &Transaction,
    tx_in_index: usize,
    private_key: &str,
    unspent_tx_outs: &Vec<UnspentTxOut>,
) -> Result<String, AppError> {
    let tx_in = transaction.tx_ins.get(tx_in_index).unwrap();
    let referenced_unspent_tx_out = find_unspent_tx_out(&tx_in.tx_out_id, tx_in.tx_out_index, &unspent_tx_outs);
    if referenced_unspent_tx_out.is_none() {
        return Err(AppError::new(2000));
    }

    if !get_public_key(private_key).eq(&referenced_unspent_tx_out.unwrap().address) {
        return Err(AppError::new(2000));
    }

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_str(private_key).unwrap();
    let message = message_from_str(&transaction.id).unwrap();
    Ok(secp.sign_ecdsa(&message, &secret_key).to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_transaction_id() {
        let tx_ins = vec![
            TxIn::new("".to_string(), 1, "".to_string()),
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];

        assert_eq!(get_transaction_id(&tx_ins, &tx_outs), "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea");
    }

    #[test]
    fn test_transaction_get_transaction_id() {
        let tx_ins = vec![
            TxIn::new("".to_string(), 1, "".to_string()),
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transaction = Transaction::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), &tx_ins, &tx_outs);

        assert_eq!(transaction.id, get_transaction_id(&tx_ins, &tx_outs), );
    }

    #[test]
    fn test_get_is_valid_tx_in() {
        let tx_in = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        let tx_ins = vec![tx_in.clone()];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);

        assert!(get_is_valid_tx_in(&tx_in, &transaction, &unspent_tx_outs));
    }

    #[test]
    fn test_find_unspent_tx_out() {
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        assert!(find_unspent_tx_out("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea", 0, &unspent_tx_outs).is_some());
        assert!(find_unspent_tx_out("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea", 1, &unspent_tx_outs).is_none());
    }

    #[test]
    fn test_get_tx_in_amount() {
        let tx_in = TxIn::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), 0, "".to_string());
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        assert_eq!(get_tx_in_amount(&tx_in, &unspent_tx_outs), 50);

        let tx_in = TxIn::new("".to_string(), 0, "".to_string());
        assert_eq!(get_tx_in_amount(&tx_in, &unspent_tx_outs), 0);

        let tx_in = TxIn::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), 1, "".to_string());
        assert_eq!(get_tx_in_amount(&tx_in, &unspent_tx_outs), 0);
    }

    #[test]
    fn test_get_is_valid_transaction() {
        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(get_is_valid_transaction(&transaction, &unspent_tx_outs));

        let tx_ins = vec![
            TxIn::new(
                "invalid".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            )
        ];
        let transaction = Transaction::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), &tx_ins, &tx_outs);
        assert!(!get_is_valid_transaction(&transaction, &unspent_tx_outs));

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 0)
        ];
        let transaction = Transaction::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), &tx_ins, &tx_outs);
        assert!(!get_is_valid_transaction(&transaction, &unspent_tx_outs));
    }

    #[test]
    fn test_get_is_valid_coinbase_tx() {
        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(get_is_valid_coinbase_tx(Some(&transaction), 0));

        assert!(!get_is_valid_coinbase_tx(None, 0));

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(!get_is_valid_coinbase_tx(Some(&transaction), 0));

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(!get_is_valid_coinbase_tx(Some(&transaction), 1));

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50),
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50),
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(!get_is_valid_coinbase_tx(Some(&transaction), 0));

        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 0)
        ];
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(!get_is_valid_coinbase_tx(Some(&transaction), 0));
    }

    #[test]
    fn test_has_duplicates() {
        let a = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        let b = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        let tx_ins = vec![
            &a,
            &b,
        ];
        assert!(has_duplicates(&tx_ins));

        let a = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        let tx_ins = vec![
            &a,
        ];
        assert!(!has_duplicates(&tx_ins));
    }

    #[test]
    fn test_get_is_valid_block_transactions() {
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
        let unspent_tx_outs = vec![];
        assert!(get_is_valid_block_transactions(&transactions, &unspent_tx_outs, 1));

        let tx_ins = vec![
            TxIn::new(
                "".to_string(),
                2,
                "".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transactions = vec![
            Transaction::new("05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e".to_string(), &tx_ins, &tx_outs)
        ];
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        assert!(get_is_valid_block_transactions(&transactions, &unspent_tx_outs, 2));
    }

    #[test]
    fn test_update_unspent_tx_outs() {
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
        let unspent_tx_outs = vec![];
        let updated_unspent_tx_outs = update_unspent_tx_outs(&transactions, &unspent_tx_outs);
        let expect = updated_unspent_tx_outs.get(0).unwrap();
        assert_eq!(expect.tx_out_id, "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea");
        assert_eq!(expect.tx_out_index, 0);
        assert_eq!(expect.address, "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b");
        assert_eq!(expect.amount, 50);

        let tx_ins = vec![
            TxIn::new(
                "".to_string(),
                2,
                "".to_string(),
            )
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transactions = vec![
            Transaction::new("05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e".to_string(), &tx_ins, &tx_outs)
        ];
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        let updated_unspent_tx_outs = update_unspent_tx_outs(&transactions, &unspent_tx_outs);
        let expect = updated_unspent_tx_outs.get(0).unwrap();

        let expect = updated_unspent_tx_outs.get(1).unwrap();
        println!("{:?}", updated_unspent_tx_outs);
        assert_eq!(expect.tx_out_id, "05f756fca4edb257e7ba26a4377246fcbef6de9e948886dad91355cdbfc32d9e");
        assert_eq!(expect.tx_out_index, 0);
        assert_eq!(expect.address, "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b");
        assert_eq!(expect.amount, 50);
    }

    #[test]
    fn test_get_coinbase_transaction() {
        let block_index: usize = 1;
        let address = "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b";
        let transaction = get_coinbase_transaction(address.to_string(), block_index);
        assert_eq!(transaction.id, get_transaction_id(&transaction.tx_ins, &transaction.tx_outs));

        let tx_in = transaction.tx_ins.get(0).unwrap();
        assert_eq!(tx_in.tx_out_id, "");
        assert_eq!(tx_in.tx_out_index, block_index);
        assert_eq!(tx_in.signature, "");

        let tx_out = transaction.tx_outs.get(0).unwrap();
        assert_eq!(tx_out.address, address);
        assert_eq!(tx_out.amount, COINBASE_AMOUNT);
    }

    #[test]
    fn test_get_public_key() {
        assert_eq!(get_public_key("27f5005f5f58f8711e99577e8b87e28ab4c2151f9289ac1203ccecdb94602a5b"), "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b");
    }

    #[test]
    fn test_sign_tx_in() {
        let tx_ins = vec![TxIn::new("f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(), 0, "".to_string())];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transaction = Transaction::generate(&tx_ins, &tx_outs);
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        assert_eq!(
            sign_tx_in(&transaction, 0, "27f5005f5f58f8711e99577e8b87e28ab4c2151f9289ac1203ccecdb94602a5b", &unspent_tx_outs).unwrap(),
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a"
        );
    }
}
