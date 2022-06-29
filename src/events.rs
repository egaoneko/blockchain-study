use crate::Block;
use crate::connection::Connection;

#[derive(Debug)]
pub enum BroadcastEvents {
    Join(Connection),
    Quit(u32),
    QueryLatest(u32, Block),
    QueryAll(u32, Vec<Block>),
    ResponseBlockchain(Vec<Block>),
}
