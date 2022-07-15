#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_cors;

#[macro_use]
extern crate validator_derive;

use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

pub mod block;
pub mod errors;
pub mod config;
mod socket;
mod events;
mod connection;
mod http;
mod routes;
mod payload;
mod utils;
mod transaction;
mod secp256k1;
mod wallet;
mod constants;
mod transaction_pool;

use crate::block::{Block, get_unspent_tx_outs};
use crate::config::Config;
use crate::events::BroadcastEvents;
use crate::socket::launch_socket;
use crate::http::launch_http;
use crate::transaction::{Transaction, TxIn, TxOut, UnspentTxOut};
use crate::wallet::Wallet;

/// # Rust Blockchain
///
/// A library for studying rust and blockchain.

pub fn run(config: Config) {
    let genesis_transaction = Transaction::new(
        "b5516eb9915e9be6868575e87bb450d8285505f004f944bf0d99c6131995bf41".to_string(),
        &vec![TxIn::new("".to_string(), 0, "".to_string())],
        &vec![TxOut::new(
            "03cbad07a30fa3c44cf3709e005149c5b41464070c15e783589d937a071f62930b".to_string(),
            50,
        )]
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
    let blockchain: Arc<RwLock<Vec<Block>>> = Arc::new(RwLock::new(vec![genesis_block]));
    let transaction_pool: Arc<RwLock<Vec<Transaction>>> = Arc::new(RwLock::new(vec![]));
    let wallet: Arc<RwLock<Wallet>> = Arc::new(RwLock::new(Wallet::new(config.private_key_path.to_string())));
    let broadcast_channel = mpsc::unbounded_channel::<BroadcastEvents>();

    let b = blockchain.read().unwrap();
    let unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>> = Arc::new(RwLock::new(get_unspent_tx_outs(&b).unwrap()));
    drop(b);

    println!("{:?}{:?}", blockchain, config);

    launch_http(&config, &blockchain, &unspent_tx_outs, &transaction_pool, &wallet, broadcast_channel.0.clone());
    launch_socket(&config, &blockchain, &unspent_tx_outs, &transaction_pool, &wallet, broadcast_channel);
}
