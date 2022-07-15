use crate::{Block, Transaction};
use crate::connection::Connection;

#[derive(Debug)]
pub enum BroadcastEvents {
    Join(Connection),
    Quit(String),
    Peer(String),
    Blockchain(Vec<Block>, Option<String>),
    Transaction(Vec<Transaction>, Option<String>),
}
