use std::collections::HashMap;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::sync::{Arc, RwLock};
use std::{thread, time};
use futures_util::{SinkExt, StreamExt};

use crate::{Block, Config};
use crate::block::get_latest_block;
use crate::connection::Connection;
use crate::events::BroadcastEvents;

const FIXED_SLEEP: u64 = 60;

pub fn launch_server(config: &Config, blockchain: &Arc<RwLock<Vec<Block>>>) {
    let mut runtime = tokio::runtime::Builder::new_multi_thread().enable_io().build().unwrap();

    runtime.block_on(async {
        let addr = format!("127.0.0.1:{}", config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Listening to TCP failed.");

        let (broadcast_sender, broadcast_receiver) = mpsc::unbounded_channel::<BroadcastEvents>();
        tokio::spawn(broadcast(broadcast_receiver));

        let (blockchain_sender, blockchain_receiver) = mpsc::unbounded_channel::<BroadcastEvents>();
        thread::spawn({
            let b = Arc::clone(blockchain);
            move || run(b, broadcast_sender, blockchain_receiver)
        });

        println!("Listening on: {}", addr);

        // A counter to use as client ids.
        let mut id = 0;

        // Accept new clients.
        while let Ok((stream, peer)) = listener.accept().await {
            let b = Arc::clone(blockchain);
            match tokio_tungstenite::accept_async(stream).await {
                Err(e) => println!("Websocket connection error : {:?}", e),
                Ok(ws_stream) => {
                    println!("New Connection : {:?}", peer);
                    id += 1;
                    tokio::spawn(listen(b, blockchain_sender.clone(), ws_stream, id));
                }
            }
        }
    });
}

fn run(mut blockchain: Arc<RwLock<Vec<Block>>>, tx: UnboundedSender<BroadcastEvents>, mut receiver: UnboundedReceiver<BroadcastEvents>) {
    loop {
        thread::sleep( time::Duration::from_secs(FIXED_SLEEP));
        println!("run {:?}", blockchain);
        let _ = tx.send(BroadcastEvents::ResponseBlockchain(blockchain.read().unwrap().to_vec()));

        let read = blockchain.read().unwrap().clone();
        let latest = get_latest_block(&read);
        let mut block = blockchain.write().unwrap();
        block.push(Block::generate("test".to_string(), latest));
    }
}

async fn broadcast(mut rx: UnboundedReceiver<BroadcastEvents>) {
    let mut connections: HashMap<u32, Connection> = HashMap::new();

    while let Some(event) = rx.recv().await {
        match event {
            BroadcastEvents::Join(conn) => {
                connections.insert(conn.id, conn);
            }
            BroadcastEvents::Quit(id) => {
                connections.remove(&id);
                println!("Connection lost : {}", id);
            }
            BroadcastEvents::QueryLatest(id, block) => {
                println!("QueryLatest {:?}", block);
                if let Some(conn) = connections.get_mut(&id) {
                    let _ = conn.sender.send(Message::Text(serde_json::to_string(&block).unwrap())).await;
                }
            }
            BroadcastEvents::QueryAll(id, blockchain) => {
                println!("QueryAll {:?}", blockchain);
                if let Some(conn) = connections.get_mut(&id) {
                    let _ = conn.sender.send(Message::Text(serde_json::to_string(&blockchain).unwrap())).await;
                }
            }
            BroadcastEvents::ResponseBlockchain(blockchain) => {
                println!("ResponseBlockchain {:?}", blockchain);
                for (_, conn) in connections.iter_mut() {
                    let _ = conn.sender.send(Message::Text(serde_json::to_string(&blockchain).unwrap())).await;
                }
            }
        }
    }
}

async fn listen(
    blockchain: Arc<RwLock<Vec<Block>>>,
    blockchain_sender: UnboundedSender<BroadcastEvents>,
    ws_stream: WebSocketStream<TcpStream>,
    id: u32,
) {
    let (sender, mut receiver) = ws_stream.split();
    let conn = Connection::new(id, sender);
    let _ = blockchain_sender.send(BroadcastEvents::Join(conn));

    while let Some(msg) = receiver.next().await {
        print!("listen");
        if let Ok(msg) = msg {
            if msg.is_binary() {
                print!("listen {:?}", msg)
            } else if msg.is_close() {
                break; // When we break, we disconnect.
            }
        } else {
            break; // When we break, we disconnect.
        }
    }
    // If we reach here, it means the client got disconnected.
    blockchain_sender.send(BroadcastEvents::Quit(id)).unwrap();
}
