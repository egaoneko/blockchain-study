use std::sync::{Arc, RwLock};
use std::thread;
use rocket_contrib::json::Json;
use rocket_cors::{Cors, CorsOptions};
use tokio::sync::mpsc::UnboundedSender;

use crate::{Block, BroadcastEvents, Config, routes, Transaction, UnspentTxOut, Wallet};
use crate::errors::ApiError;

#[catch(404)]
#[allow(dead_code)]
fn not_found() -> Json<ApiError> {
    Json(ApiError::new(404, "Resource was not found.".to_string(), None))
}

fn cors_fairing() -> Cors {
    CorsOptions::default()
        .to_cors()
        .expect("Cors fairing cannot be created")
}

pub fn launch_http(
    config: &Config,
    blockchain: &Arc<RwLock<Vec<Block>>>,
    unspent_tx_outs: &Arc<RwLock<Vec<UnspentTxOut>>>,
    transaction_pool: &Arc<RwLock<Vec<Transaction>>>,
    wallet: &Arc<RwLock<Wallet>>,
    broadcast_sender: UnboundedSender<BroadcastEvents>,
) {
    let b = Arc::clone(blockchain);
    let u = Arc::clone(unspent_tx_outs);
    let t = Arc::clone(transaction_pool);
    let w = Arc::clone(wallet);
    let config = rocket::config::Config::build(rocket::config::Environment::Development).port(config.http_port).finalize().unwrap();

    thread::spawn(move || {
        rocket::custom(config)
            .mount("/api", routes![
                routes::ping,
                routes::blocks,
                routes::mine_raw_block,
                routes::mine_block,
                routes::address,
                routes::balance,
                routes::unspent_transaction_outputs,
                routes::my_unspent_transaction_outputs,
                routes::mine_transaction,
                routes::send_transaction,
                routes::transaction_pool,
                routes::add_peer
            ])
            .attach(cors_fairing())
            .manage(b)
            .manage(u)
            .manage(t)
            .manage(w)
            .manage(broadcast_sender)
            .launch();
    });
}
