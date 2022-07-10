use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{thread, time};
use std::mem;
use tokio_tungstenite::{accept_async, connect_async, MaybeTlsStream, WebSocketStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::{Block, Config};
use crate::block::{get_is_replace_chain};
use crate::connection::Connection;
use crate::events::BroadcastEvents;
use crate::payload::{Payload, PayloadType};

const FIXED_SLEEP: u64 = 60;

pub fn launch_socket(
    config: &Config,
    blockchain: &Arc<RwLock<Vec<Block>>>,
    broadcast_channel: (UnboundedSender<BroadcastEvents>, UnboundedReceiver<BroadcastEvents>),
) {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_io().build().unwrap();

    runtime.block_on(async {
        let addr = format!("127.0.0.1:{}", config.socket_port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Listening to TCP failed.");

        let (broadcast_sender, broadcast_receiver) = broadcast_channel;

        tokio::spawn({
            let b = Arc::clone(blockchain);
            broadcast(b, broadcast_sender.clone(), broadcast_receiver)
        });
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
                    let b = Arc::clone(blockchain);
                    tokio::spawn(listen(b, broadcast_sender.clone(), ws_stream, peer.to_string()));
                }
            }
        }
    });
}

async fn run(blockchain: Arc<RwLock<Vec<Block>>>, _tx: UnboundedSender<BroadcastEvents>) {
    loop {
        thread::sleep(time::Duration::from_secs(FIXED_SLEEP));
        println!("run {:?}", blockchain);
    }
}

async fn broadcast(blockchain: Arc<RwLock<Vec<Block>>>, tx: UnboundedSender<BroadcastEvents>, mut rx: UnboundedReceiver<BroadcastEvents>) {
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
                let b = Arc::clone(&blockchain);
                tokio::spawn(connect(b, tx.clone(), ws_stream, peer));
            }
            BroadcastEvents::Blockchain(blockchain, except) => {
                println!("NotifyBlockchain : {:?}, {:?}", blockchain, connections);
                let p = except.unwrap_or_default();
                for (peer, conn) in connections.iter_mut() {
                    if peer.eq(&p) {
                        continue;
                    }
                    if let Some(listener) = conn.listener.as_mut() {
                        listener.send(Payload::serialize(PayloadType::Blockchain, &blockchain)).await.expect("ResponseBlockchain: listener send panic");
                    }
                    if let Some(connector) = conn.connector.as_mut() {
                        connector.send(Payload::serialize(PayloadType::Blockchain, &blockchain)).await.expect("ResponseBlockchain: connector send panic");
                    }
                }
            },
        }
    }
}

async fn listen(
    blockchain: Arc<RwLock<Vec<Block>>>,
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
                let b = Arc::clone(&blockchain);
                receive(b, &tx, peer.clone(), msg);
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
    blockchain: Arc<RwLock<Vec<Block>>>,
    tx: UnboundedSender<BroadcastEvents>,
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    peer: String,
) {
    let (sender, mut receiver) = ws_stream.split();
    let conn = Connection::new(peer.clone(), None, Some(sender));
    let _ = tx.send(BroadcastEvents::Join(conn));

    while let Some(msg) = receiver.next().await {
        println!("Receive connect message");
        if let Ok(msg) = msg {
            println!("Receive connect message : {:?}", msg);
            if msg.is_text() {
                let b = Arc::clone(&blockchain);
                receive(b, &tx, peer.clone(), msg);
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

fn receive(
    blockchain: Arc<RwLock<Vec<Block>>>,
    tx: &UnboundedSender<BroadcastEvents>,
    peer: String,
    message: Message
) {
    let payload = Payload::deserialize(message);
    match payload.r#type {
        PayloadType::Blockchain => {
            let guard = blockchain.read().unwrap().clone();
            let new_blockchain = serde_json::from_str::<Vec<Block>>(payload.data.as_str()).unwrap();
            if get_is_replace_chain(&guard, &new_blockchain) {
                let mut guard = blockchain.write().unwrap();
                let _ = mem::replace(&mut *guard, new_blockchain);
                tx.send(BroadcastEvents::Blockchain(guard.to_vec(), Some(peer))).unwrap();
            }
        }
    }
}
