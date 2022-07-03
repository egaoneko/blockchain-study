#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
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

use crate::block::Block;
use crate::config::Config;
use crate::events::BroadcastEvents;
use crate::socket::launch_socket;
use crate::http::launch_http;

/// # Rust Blockchain
///
/// A library for studying rust and blockchain.

pub fn run(config: Config) {
    let genesis_block: Block = Block::new(
        0,
        "816534932c2b7154836da6afc367695e6337db8a921823784c14378abed4f7d7".to_string(),
        "".to_string(),
        1465154705,
        "gene block".to_string(),
        0,
        0,
    );
    let blockchain: Arc<RwLock<Vec<Block>>> = Arc::new(RwLock::new(vec![genesis_block]));
    let broadcast_channel = mpsc::unbounded_channel::<BroadcastEvents>();

    println!("{:?}{:?}", blockchain, config);

    launch_http(&config, &blockchain, broadcast_channel.0.clone());
    launch_socket(&config, &blockchain, broadcast_channel);
}
