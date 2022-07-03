use std::sync::{Arc, RwLock};
use rocket::State;
use rocket_contrib::json::Json;

use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::{Block, BroadcastEvents};
use crate::block::{add_block};
use crate::errors::{ApiError, FieldValidator};

#[get("/ping")]
pub fn ping() -> &'static str {
    "ok"
}

#[get("/blocks")]
pub fn get_blocks(blockchain: State<Arc<RwLock<Vec<Block>>>>) -> Json<Vec<Block>> {
    Json(blockchain.read().unwrap().to_vec())
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewBlock {
    #[validate(length(min = 1))]
    pub data: Option<String>,
}

#[post("/blocks", format = "json", data = "<new_block>")]
pub fn post_blocks(new_block: Json<NewBlock>, blockchain: State<Arc<RwLock<Vec<Block>>>>, broadcast_sender: State<UnboundedSender<BroadcastEvents>>) -> Result<&'static str, Json<ApiError>> {
    let new_block = new_block.0;
    let mut extractor = FieldValidator::validate(&new_block);
    let data = extractor.extract("data", new_block.data);
    extractor.check()?;

    let mut guard = blockchain.write().unwrap();
    if let Err(e) = add_block(&mut guard, data) {
        return Err(Json(ApiError::new(500, format!("Add block fail: {}", e.code), None)))
    }

    let _ = broadcast_sender.send(BroadcastEvents::Blockchain(guard.to_vec(), None));
    Ok("ok")
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewPeer {
    #[validate(length(min = 1))]
    pub peer: Option<String>,
}

#[post("/peers", format = "json", data = "<new_peer>")]
pub fn post_peers(new_peer: Json<NewPeer>, broadcast_sender: State<UnboundedSender<BroadcastEvents>>) -> Result<&'static str, Json<ApiError>> {
    let new_peer = new_peer.0;
    let mut extractor = FieldValidator::validate(&new_peer);
    let peer = extractor.extract("peer", new_peer.peer);
    extractor.check()?;

    let _ = broadcast_sender.send(BroadcastEvents::Peer(peer));
    Ok("ok")
}
