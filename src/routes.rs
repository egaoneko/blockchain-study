use std::sync::{Arc, RwLock};
use rocket::State;
use rocket_contrib::json::Json;

use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::{Block, BroadcastEvents};
use crate::block::get_latest_block;
use crate::errors::{ApiError, FieldValidator};

#[get("/ping")]
pub fn ping() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewBlock {
    #[validate(length(min = 1))]
    pub data: Option<String>,
}

#[post("/mine-block", format = "json", data = "<new_block>")]
pub fn mine_block(new_block: Json<NewBlock>, blockchain: State<Arc<RwLock<Vec<Block>>>>, broadcast_sender: State<UnboundedSender<BroadcastEvents>>) -> Result<&'static str, Json<ApiError>> {
    let new_block = new_block.0;
    let mut extractor = FieldValidator::validate(&new_block);
    let data = extractor.extract("data", new_block.data);
    extractor.check()?;

    let read = blockchain.read().unwrap().clone();
    let latest = get_latest_block(&read);
    let mut block = blockchain.write().unwrap();
    block.push(Block::generate(data.to_string(), latest));
    let _ = broadcast_sender.send(BroadcastEvents::ResponseBlockchain(block.to_vec()));
    Ok("ok")
}
