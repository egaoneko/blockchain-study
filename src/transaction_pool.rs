use crate::errors::AppError;
use crate::transaction::{get_is_valid_transaction, Transaction, TxIn};
use crate::UnspentTxOut;

pub fn get_tx_pool_ins(transaction_pool: &Vec<Transaction>) -> Vec<&TxIn> {
    transaction_pool
        .into_iter()
        .map(|tx| &tx.tx_ins)
        .flatten()
        .collect()
}

fn contains_tx_in(tx_pool_ins: &Vec<&TxIn>, tx_in: &TxIn) -> bool {
    tx_pool_ins
        .into_iter()
        .any(|&tx_pool_in| tx_pool_in.tx_out_index == tx_in.tx_out_index && tx_pool_in.tx_out_id.eq(&tx_in.tx_out_id))
}

fn get_is_valid_tx_for_pool(tx: &Transaction, transaction_pool: &Vec<Transaction>) -> bool {
    let tx_pool_ins = get_tx_pool_ins(transaction_pool);
    let ref_tx_ins = &tx.tx_ins;
    ref_tx_ins
        .into_iter()
        .all(|tx_in| !contains_tx_in(&tx_pool_ins, &tx_in))
}

fn has_tx_in(tx_in: &TxIn, unspent_tx_outs: &Vec<UnspentTxOut>) -> bool {
    unspent_tx_outs
        .into_iter()
        .any(|u_tx_o| u_tx_o.tx_out_id.eq(&tx_in.tx_out_id) && u_tx_o.tx_out_index == tx_in.tx_out_index)
}

pub fn add_to_transaction_pool(tx: &Transaction, transaction_pool: &mut Vec<Transaction>, unspent_tx_outs: &Vec<UnspentTxOut>) -> Result<(), AppError> {
    if !get_is_valid_transaction(tx, unspent_tx_outs) {
        return Err(AppError::new(4000));
    }

    if !get_is_valid_tx_for_pool(tx, transaction_pool) {
        return Err(AppError::new(4001));
    }

    transaction_pool.push(tx.clone());

    Ok(())
}

pub fn update_transaction_pool(transaction_pool: &Vec<Transaction>, unspent_tx_outs: &Vec<UnspentTxOut>) -> Vec<Transaction> {
    let invalid_txs = transaction_pool
        .into_iter()
        .filter(|&tx| tx.tx_ins.iter().any(|tx_in| !has_tx_in(tx_in, unspent_tx_outs)))
        .collect::<Vec<&Transaction>>();

    if invalid_txs.len() == 0 {
        return transaction_pool.clone();
    }

    let ref_invalid_txs = &invalid_txs;
    transaction_pool
        .into_iter()
        .filter(|&tx| ref_invalid_txs.into_iter().all(|&x| !x.eq(tx)))
        .map(|v| v.clone())
        .collect::<Vec<Transaction>>()
}

#[cfg(test)]
mod test {
    use crate::transaction::TxOut;
    use super::*;

    #[test]
    fn test_get_tx_pool_ins() {
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
        let tx_pool_ins = get_tx_pool_ins(&transaction_pool);
        assert_eq!(tx_pool_ins.len(), 1);

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                1,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let transaction_pool = vec![Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs)];
        let tx_pool_ins = get_tx_pool_ins(&transaction_pool);
        assert_eq!(tx_pool_ins.len(), 2);
    }

    #[test]
    fn test_contains_tx_in() {
        let tx_in = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        let tx_ins = vec![&tx_in];
        assert!(contains_tx_in(&tx_ins, &tx_in));

        let other = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        assert!(contains_tx_in(&tx_ins, &other));
    }

    #[test]
    fn test_get_is_valid_tx_for_pool() {
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
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        let transaction_pool = vec![transaction.clone()];
        assert!(!get_is_valid_tx_for_pool(&transaction, &transaction_pool));

        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                1,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
        ];
        let tx_outs = vec![
            TxOut::new("03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(), 50)
        ];
        let other_transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        assert!(get_is_valid_tx_for_pool(&other_transaction, &transaction_pool));
    }

    #[test]
    fn test_has_tx_in() {
        let tx_in = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            0,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        assert!(has_tx_in(&tx_in, &unspent_tx_outs));

        let tx_in = TxIn::new(
            "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
            1,
            "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
        );
        assert!(!has_tx_in(&tx_in, &unspent_tx_outs));
    }

    #[test]
    fn test_add_to_transaction_pool() {
        let tx_ins = vec![
            TxIn::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                1,
                "3045022100d73a8f9c7ce7fd44517ff0db38733af84a0ee1bc3ec89ed2c82dad412374057602203eac06b3c11dcb004991f39f9f23e46d3354ea6de8bfa73da8ca77adbb57988a".to_string(),
            ),
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
        let mut transaction_pool = vec![Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs)];

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
        let transaction = Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs);
        add_to_transaction_pool(&transaction, &mut transaction_pool, &unspent_tx_outs).unwrap();
        assert_eq!(transaction_pool.len(), 2);
    }

    #[test]
    fn test_update_transaction_pool() {
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
        let unspent_tx_outs = vec![
            UnspentTxOut::new(
                "f0ab1700e79b5f4c120062a791e7e69150577fea3ba9da15179025b3d2c061ea".to_string(),
                0,
                "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
                50,
            )
        ];
        let transaction_pool = vec![Transaction::new("2ffbf11ad81702d9a4b07b4a869b0ef304cdaebc7efcbb79e80942cdfef7cd0d".to_string(), &tx_ins, &tx_outs)];
        let new_transaction_pool = update_transaction_pool(&transaction_pool, &unspent_tx_outs);
        assert_eq!(new_transaction_pool.len(), 1);

        let new_transaction_pool = update_transaction_pool(&transaction_pool, &vec![]);
        assert_eq!(new_transaction_pool.len(), 0);
    }
}
