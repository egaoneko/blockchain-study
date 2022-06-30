use std::sync::{Arc, RwLock};
use rocket::State;
use crate::Block;

#[get("/ping")]
pub fn ping(blockchain: State<Arc<RwLock<Vec<Block>>>>) -> &'static str {
    println!("{:?}", blockchain);
    "ok"
}
