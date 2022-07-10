use std::sync::{Arc, RwLock};
use std::thread;
use rocket_contrib::json::Json;
use rocket_cors::{Cors, CorsOptions};
use tokio::sync::mpsc::UnboundedSender;

use crate::{Block, BroadcastEvents, Config, routes, UnspentTxOut};
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
    broadcast_sender: UnboundedSender<BroadcastEvents>,
) {
    let b = Arc::clone(blockchain);
    let u = Arc::clone(unspent_tx_outs);
    let config = rocket::config::Config::build(rocket::config::Environment::Development).port(config.http_port).finalize().unwrap();

    thread::spawn(move || {
        rocket::custom(config)
            .mount("/api", routes![
                routes::ping,
                routes::get_blocks,
                routes::mine_block,
                routes::post_peers
            ])
            .attach(cors_fairing())
            .manage(b)
            .manage(u)
            .manage(broadcast_sender)
            .launch();
    });
}
