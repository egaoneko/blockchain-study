use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{thread, time};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, connect_async, MaybeTlsStream, WebSocketStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{SinkExt, StreamExt};
use url::Url;

use crate::{Block, Config};
use crate::block::get_latest_block;
use crate::connection::Connection;
use crate::events::BroadcastEvents;

const FIXED_SLEEP: u64 = 60;

pub fn launch_socket(
    config: &Config,
    blockchain: &Arc<RwLock<Vec<Block>>>,
    broadcast_channel: (UnboundedSender<BroadcastEvents>, UnboundedReceiver<BroadcastEvents>),
) {
    let mut runtime = tokio::runtime::Builder::new_multi_thread().enable_io().build().unwrap();

    runtime.block_on(async {
        let addr = format!("127.0.0.1:{}", config.socket_port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Listening to TCP failed.");

        let (broadcast_sender, broadcast_receiver) = broadcast_channel;

        tokio::spawn(broadcast(broadcast_sender.clone(), broadcast_receiver));
        tokio::spawn({
            let b = Arc::clone(blockchain);
            run(b, broadcast_sender.clone())
        });

        println!("Listening on: {}", addr);

        // A counter to use as client ids.

        // Accept new clients.
        while let Ok((stream, peer)) = listener.accept().await {
            match accept_async(stream).await {
                Err(e) => println!("Websocket connection error : {:?}", e),
                Ok(ws_stream) => {
                    println!("New Connection : {:?}", peer);
                    tokio::spawn(listen(broadcast_sender.clone(), ws_stream, peer.to_string()));
                }
            }
        }
    });
}

async fn run(blockchain: Arc<RwLock<Vec<Block>>>, tx: UnboundedSender<BroadcastEvents>) {
    loop {
        thread::sleep(time::Duration::from_secs(FIXED_SLEEP));
        println!("run {:?}", blockchain);
        // let _ = tx.send(BroadcastEvents::ResponseBlockchain(blockchain.read().unwrap().to_vec()));
        //
        // let read = blockchain.read().unwrap().clone();
        // let latest = get_latest_block(&read);
        // let mut block = blockchain.write().unwrap();
        // block.push(Block::generate("test".to_string(), latest));
    }
}

async fn broadcast(tx: UnboundedSender<BroadcastEvents>, mut rx: UnboundedReceiver<BroadcastEvents>) {
    let mut connections: HashMap<String, Connection> = HashMap::new();

    while let Some(event) = rx.recv().await {
        match event {
            BroadcastEvents::Join(conn) => {
                println!("Connection join : {:?}", conn);
                connections.insert(conn.peer.clone(), conn);
            }
            BroadcastEvents::Quit(peer) => {
                println!("Connection quit : {}", peer);
                connections.remove(peer.as_str());
            }
            BroadcastEvents::Peer(peer) => {
                println!("Connection peer : {:?}", peer);
                let (ws_stream, _) = connect_async(Url::parse(peer.as_str()).unwrap()).await.expect("Failed to connect");
                tokio::spawn(connect(tx.clone(), ws_stream, peer));
            }
            BroadcastEvents::Blockchain(blockchain) => {
                println!("ResponseBlockchain : {:?}, {:?}", blockchain, connections);
                for (_, conn) in connections.iter_mut() {
                    if let Some(listener) = conn.listener.as_mut() {
                        listener.send(Message::Text(serde_json::to_string(&blockchain).unwrap())).await.expect("ResponseBlockchain: listener send panic");
                    }
                    if let Some(connector) = conn.connector.as_mut() {
                        connector.send(Message::Text(serde_json::to_string(&blockchain).unwrap())).await.expect("ResponseBlockchain: connector send panic");
                    }
                }
            }
        }
    }
}

async fn listen(
    tx: UnboundedSender<BroadcastEvents>,
    ws_stream: WebSocketStream<TcpStream>,
    peer: String,
) {
    let (sender, mut receiver) = ws_stream.split();
    let conn = Connection::new(peer.clone(), Some(sender), None);
    let _ = tx.send(BroadcastEvents::Join(conn));

    while let Some(msg) = receiver.next().await {
        println!("Receive listen message");
        if let Ok(msg) = msg {
            println!("Receive listen message : {:?}", msg);
            if msg.is_text() {
                println!("Receive listen message : {:?}", serde_json::from_str::<Vec<Block>>(msg.into_text().unwrap().as_str()).unwrap())
            } else if msg.is_close() {
                break; // When we break, we disconnect.
            }
        } else {
            break; // When we break, we disconnect.
        }
    }
    // If we reach here, it means the client got disconnected.
    tx.send(BroadcastEvents::Quit(peer.clone())).unwrap();
}

async fn connect(
    blockchain_sender: UnboundedSender<BroadcastEvents>,
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    peer: String,
) {
    let (sender, mut receiver) = ws_stream.split();
    let conn = Connection::new(peer.clone(), None, Some(sender));
    let _ = blockchain_sender.send(BroadcastEvents::Join(conn));

    while let Some(msg) = receiver.next().await {
        println!("Receive connect message");
        if let Ok(msg) = msg {
            println!("Receive connect message : {:?}", msg);
            if msg.is_text() {
                println!("Receive connect message : {:?}", msg)
            } else if msg.is_close() {
                break; // When we break, we disconnect.
            }
        } else {
            break; // When we break, we disconnect.
        }
    }
    // If we reach here, it means the client got disconnected.
    blockchain_sender.send(BroadcastEvents::Quit(peer.clone())).unwrap();
}
