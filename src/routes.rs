use std::sync::{Arc, RwLock};
use rocket::State;
use rocket_contrib::json::Json;

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{Block, BroadcastEvents, UnspentTxOut, Wallet};
use crate::block::{add_block};
use crate::errors::{ApiError, FieldValidator};
use crate::transaction::Transaction;
use crate::transaction_pool::add_to_transaction_pool;
use crate::wallet::{create_transaction, find_unspent_tx_outs, get_balance};

#[get("/ping")]
pub fn ping() -> &'static str {
    "ok"
}

#[get("/blocks")]
pub fn blocks(
    blockchain: State<Arc<RwLock<Vec<Block>>>>
) -> Json<Vec<Block>> {
    Json(blockchain.read().unwrap().to_vec())
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewBlock {
    pub data: Option<Vec<Transaction>>,
}

#[post("/mine-raw-block", format = "json", data = "<new_block>")]
pub fn mine_raw_block(
    new_block: Json<NewBlock>,
    blockchain: State<Arc<RwLock<Vec<Block>>>>,
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>,
    transaction_pool: State<Arc<RwLock<Vec<Transaction>>>>,
    broadcast_sender: State<UnboundedSender<BroadcastEvents>>,
) -> Result<Json<Block>, Json<ApiError>> {
    let new_block = new_block.0;
    let mut extractor = FieldValidator::validate(&new_block);
    let data = extractor.extract("data", new_block.data);
    extractor.check()?;

    let mut b_guard = blockchain.write().unwrap();
    let mut u_guard = unspent_tx_outs.write().unwrap();
    let mut t_guard = transaction_pool.write().unwrap();
    let new_block = Block::generate_raw(&b_guard, &data);
    if let Err(e) = add_block(&mut b_guard, &mut u_guard, &mut t_guard, &new_block) {
        return Err(Json(ApiError::new(500, format!("Add block fail: {}", e.code), None)));
    }

    let _ = broadcast_sender.send(BroadcastEvents::Blockchain(b_guard.to_vec(), None));
    Ok(Json(new_block))
}

#[post("/mine-block")]
pub fn mine_block(
    blockchain: State<Arc<RwLock<Vec<Block>>>>,
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>,
    transaction_pool: State<Arc<RwLock<Vec<Transaction>>>>,
    wallet: State<Arc<RwLock<Wallet>>>,
    broadcast_sender: State<UnboundedSender<BroadcastEvents>>,
) -> Result<Json<Block>, Json<ApiError>> {
    let mut b_guard = blockchain.write().unwrap();
    let mut u_guard = unspent_tx_outs.write().unwrap();
    let mut t_guard = transaction_pool.write().unwrap();
    let w_guard = wallet.read().unwrap();
    let new_block = Block::generate_with_coinbase_transaction(&b_guard, &t_guard, &w_guard);
    if let Err(e) = add_block(&mut b_guard, &mut u_guard, &mut t_guard, &new_block) {
        return Err(Json(ApiError::new(500, format!("Add block fail: {}", e.code), None)));
    }

    let _ = broadcast_sender.send(BroadcastEvents::Blockchain(b_guard.to_vec(), None));
    Ok(Json(new_block))
}

#[derive(Debug, Serialize)]
pub struct Address {
    pub public_key: String,
}

#[get("/address")]
pub fn address(wallet: State<Arc<RwLock<Wallet>>>) -> Json<Address> {
    let w_guard = wallet.read().unwrap();
    Json(Address {
        public_key: w_guard.public_key.clone(),
    })
}

#[derive(Debug, Serialize)]
pub struct Balance {
    pub balance: usize,
}

#[get("/balance")]
pub fn balance(
    wallet: State<Arc<RwLock<Wallet>>>,
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>,
) -> Json<Balance> {
    let w_guard = wallet.read().unwrap();
    let u_guard = unspent_tx_outs.read().unwrap();
    Json(Balance {
        balance: get_balance(w_guard.public_key.as_str(), &u_guard),
    })
}

#[get("/unspent-transaction-outputs")]
pub fn unspent_transaction_outputs(
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>
) -> Json<Vec<UnspentTxOut>> {
    let u_guard = unspent_tx_outs.read().unwrap();
    Json(u_guard.to_vec())
}

#[get("/my-unspent-transaction-outputs")]
pub fn my_unspent_transaction_outputs(
    wallet: State<Arc<RwLock<Wallet>>>,
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>,
) -> Json<Vec<UnspentTxOut>> {
    let w_guard = wallet.read().unwrap();
    let u_guard = unspent_tx_outs.read().unwrap();
    Json(find_unspent_tx_outs(w_guard.public_key.as_str(), &u_guard).to_vec())
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewTransaction {
    #[validate(length(min = 1))]
    pub address: Option<String>,

    #[validate(range(min = 0))]
    pub amount: Option<usize>,
}

#[post("/mine-transaction", format = "json", data = "<new_transaction>")]
pub fn mine_transaction(
    new_transaction: Json<NewTransaction>,
    blockchain: State<Arc<RwLock<Vec<Block>>>>,
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>,
    transaction_pool: State<Arc<RwLock<Vec<Transaction>>>>,
    wallet: State<Arc<RwLock<Wallet>>>,
    broadcast_sender: State<UnboundedSender<BroadcastEvents>>,
) -> Result<Json<Block>, Json<ApiError>> {
    let new_transaction = new_transaction.0;
    let mut extractor = FieldValidator::validate(&new_transaction);
    let address = extractor.extract("address", new_transaction.address);
    let amount = extractor.extract("amount", new_transaction.amount);
    extractor.check()?;

    let mut b_guard = blockchain.write().unwrap();
    let mut u_guard = unspent_tx_outs.write().unwrap();
    let mut t_guard = transaction_pool.write().unwrap();
    let w_guard = wallet.read().unwrap();

    return match Block::generate_with_transaction(&b_guard, &w_guard, &u_guard, &address, amount) {
        Ok(new_block) => {
            if let Err(e) = add_block(&mut b_guard, &mut u_guard, &mut t_guard, &new_block) {
                return Err(Json(ApiError::new(500, format!("Add block fail: {}", e.code), None)));
            }
            let _ = broadcast_sender.send(BroadcastEvents::Blockchain(b_guard.to_vec(), None));
            Ok(Json(new_block))
        }
        Err(e) => {
            Err(Json(ApiError::new(500, format!("Add block fail: {}", e.code), None)))
        }
    };
}

#[post("/send-transaction", format = "json", data = "<new_transaction>")]
pub fn send_transaction(
    new_transaction: Json<NewTransaction>,
    transaction_pool: State<Arc<RwLock<Vec<Transaction>>>>,
    unspent_tx_outs: State<Arc<RwLock<Vec<UnspentTxOut>>>>,
    wallet: State<Arc<RwLock<Wallet>>>,
    broadcast_sender: State<UnboundedSender<BroadcastEvents>>,
) -> Result<Json<Transaction>, Json<ApiError>> {
    let new_transaction = new_transaction.0;
    let mut extractor = FieldValidator::validate(&new_transaction);
    let address = extractor.extract("address", new_transaction.address);
    let amount = extractor.extract("amount", new_transaction.amount);
    extractor.check()?;

    let mut t_guard = transaction_pool.write().unwrap();
    let u_guard = unspent_tx_outs.write().unwrap();
    let w_guard = wallet.read().unwrap();

    return match create_transaction(&address, amount, &w_guard, &u_guard) {
        Ok(tx) => {
            match add_to_transaction_pool(&tx, &mut t_guard, &u_guard) {
                Ok(_) => {
                    let _ = broadcast_sender.send(BroadcastEvents::Transaction(t_guard.to_vec(), None));
                    Ok(Json(tx))
                }
                Err(e) => Err(Json(ApiError::new(500, format!("Add transaction pool fail: {}", e.code), None)))
            }
        }
        Err(e) => {
            Err(Json(ApiError::new(500, format!("Add transaction pool fail: {}", e.code), None)))
        }
    };
}

#[get("/transaction-pool")]
pub fn transaction_pool(
    transaction_pool: State<Arc<RwLock<Vec<Transaction>>>>,
) -> Json<Vec<Transaction>> {
    let t_guard = transaction_pool.read().unwrap();
    Json(t_guard.to_vec())
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewPeer {
    #[validate(length(min = 1))]
    pub peer: Option<String>,
}

#[post("/add-peer", format = "json", data = "<new_peer>")]
pub fn add_peer(
    new_peer: Json<NewPeer>,
    broadcast_sender: State<UnboundedSender<BroadcastEvents>>,
) -> Result<&'static str, Json<ApiError>> {
    let new_peer = new_peer.0;
    let mut extractor = FieldValidator::validate(&new_peer);
    let peer = extractor.extract("peer", new_peer.peer);
    extractor.check()?;

    let _ = broadcast_sender.send(BroadcastEvents::Peer(peer));
    Ok("ok")
}
