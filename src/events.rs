use crate::Block;
use crate::connection::Connection;

#[derive(Debug)]
pub enum BroadcastEvents {
    Join(Connection),
    Quit(String),
    Peer(String),
    Blockchain(Vec<Block>),
}
