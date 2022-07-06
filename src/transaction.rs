use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use secp256k1::{Secp256k1, Message, ecdsa, PublicKey};

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
    pub fn new(tx_ins: &Vec<TxIn>, tx_outs: &Vec<TxOut>) -> Transaction {
        Transaction {
            id: get_transaction_id(tx_ins, tx_outs),
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

fn validate_tx_in(tx_in: &TxIn, transaction: &Transaction, unspent_tx_outs: &Vec<UnspentTxOut>) -> bool {
    let referenced_utx_out =
        unspent_tx_outs.into_iter().find(|utx_o| utx_o.tx_out_id.eq(&tx_in.tx_out_id));
    return if let Some(referenced_utx_out) = referenced_utx_out {
        let secp = Secp256k1::verification_only();
        let public_key = PublicKey::from_slice(referenced_utx_out.address.as_bytes()).unwrap();
        let message = Message::from_slice(transaction.id.as_bytes()).unwrap();
        let sig = ecdsa::Signature::from_compact(tx_in.signature.as_bytes()).unwrap();
        secp.verify_ecdsa(&message, &sig, &public_key).is_ok()
    } else {
        false
    };
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
            TxOut::new("04cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b8a022c6fa9ca22c67213bdd372e074b97b31c064fc247944147e73a71dea0b17".to_string(), 50)
        ];

        assert_eq!(get_transaction_id(&tx_ins, &tx_outs), "65c57232f637ce937ea04864f125a4647817d84daf098920644c697519a4c4d8");
    }

    #[test]
    fn test_transaction_get_transaction_id() {
        let tx_ins = vec![
            TxIn::new("".to_string(), 1, "".to_string()),
        ];
        let tx_outs = vec![
            TxOut::new("04cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b8a022c6fa9ca22c67213bdd372e074b97b31c064fc247944147e73a71dea0b17".to_string(), 50)
        ];
        let transaction = Transaction::new(&tx_ins, &tx_outs);

        assert_eq!(transaction.id, get_transaction_id(&tx_ins, &tx_outs), );
    }

    #[test]
    fn test_validate_tx_in() {
        let tx_in = TxIn::new("".to_string(), 1, "".to_string());
        let tx_ins = vec![tx_in.clone()];
        let tx_outs = vec![
            TxOut::new("04cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b8a022c6fa9ca22c67213bdd372e074b97b31c064fc247944147e73a71dea0b17".to_string(), 50)
        ];
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "65c57232f637ce937ea04864f125a4647817d84daf098920644c697519a4c4d8".to_string(),
                0,
                "04cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b8a022c6fa9ca22c67213bdd372e074b97b31c064fc247944147e73a71dea0b17".to_string(),
                50
            )
        ];
        let transaction = Transaction::new(&tx_ins, &tx_outs);

        assert!(validate_tx_in(&tx_in, &transaction, &unspent_tx_outs));
    }
}
