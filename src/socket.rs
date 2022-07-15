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

use crate::{Block, Config, Transaction, UnspentTxOut, Wallet};
use crate::block::{get_is_replace_chain, get_unspent_tx_outs};
use crate::connection::Connection;
use crate::events::BroadcastEvents;
use crate::payload::{Payload, PayloadType};
use crate::transaction_pool::add_to_transaction_pool;

const FIXED_SLEEP: u64 = 60;

pub fn launch_socket(
    config: &Config,
    blockchain: &Arc<RwLock<Vec<Block>>>,
    unspent_tx_outs: &Arc<RwLock<Vec<UnspentTxOut>>>,
    transaction_pool: &Arc<RwLock<Vec<Transaction>>>,
    wallet: &Arc<RwLock<Wallet>>,
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
            let u = Arc::clone(unspent_tx_outs);
            let t = Arc::clone(transaction_pool);
            let w = Arc::clone(wallet);
            broadcast(b, u, t, w, broadcast_sender.clone(), broadcast_receiver)
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
                    let u = Arc::clone(unspent_tx_outs);
                    let t = Arc::clone(transaction_pool);
                    let w = Arc::clone(wallet);
                    tokio::spawn(listen(b, u, t, w, broadcast_sender.clone(), ws_stream, peer.to_string()));
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

async fn broadcast(
    blockchain: Arc<RwLock<Vec<Block>>>,
    unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    transaction_pool: Arc<RwLock<Vec<Transaction>>>,
    wallet: Arc<RwLock<Wallet>>,
    tx: UnboundedSender<BroadcastEvents>,
    mut rx: UnboundedReceiver<BroadcastEvents>,
) {
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
                let u = Arc::clone(&unspent_tx_outs);
                let t = Arc::clone(&transaction_pool);
                let w = Arc::clone(&wallet);
                tokio::spawn(connect(b, u, t, w, tx.clone(), ws_stream, peer));
            }
            BroadcastEvents::Blockchain(blockchain, except) => {
                println!("NotifyBlockchain : \n{:#?}", blockchain);
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
            }
            BroadcastEvents::Transaction(transactions, except) => {
                println!("NotifyTransaction : \n{:#?}", transactions);
                let p = except.unwrap_or_default();
                for (peer, conn) in connections.iter_mut() {
                    if peer.eq(&p) {
                        continue;
                    }
                    if let Some(listener) = conn.listener.as_mut() {
                        listener.send(Payload::serialize(PayloadType::Transaction, &transactions)).await.expect("ResponseTransaction: listener send panic");
                    }
                    if let Some(connector) = conn.connector.as_mut() {
                        connector.send(Payload::serialize(PayloadType::Transaction, &transactions)).await.expect("ResponseTransaction: connector send panic");
                    }
                }
            }
        }
    }
}

async fn listen(
    blockchain: Arc<RwLock<Vec<Block>>>,
    unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    transaction_pool: Arc<RwLock<Vec<Transaction>>>,
    wallet: Arc<RwLock<Wallet>>,
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
            println!("Receive listen message : {:#?}", msg);
            if msg.is_text() {
                let b = Arc::clone(&blockchain);
                let u = Arc::clone(&unspent_tx_outs);
                let t = Arc::clone(&transaction_pool);
                let w = Arc::clone(&wallet);
                receive(b, u, t, w, &tx, peer.clone(), msg);
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
    unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    transaction_pool: Arc<RwLock<Vec<Transaction>>>,
    wallet: Arc<RwLock<Wallet>>,
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
            println!("Receive connect message : {:#?}", msg);
            if msg.is_text() {
                let b = Arc::clone(&blockchain);
                let u = Arc::clone(&unspent_tx_outs);
                let t = Arc::clone(&transaction_pool);
                let w = Arc::clone(&wallet);
                receive(b, u, t, w, &tx, peer.clone(), msg);
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
    unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    transaction_pool: Arc<RwLock<Vec<Transaction>>>,
    _wallet: Arc<RwLock<Wallet>>,
    tx: &UnboundedSender<BroadcastEvents>,
    peer: String,
    message: Message,
) {
    let payload = Payload::deserialize(message);
    match payload.r#type {
        PayloadType::Blockchain => {
            println!("Receive Blockchain");
            let b_guard = blockchain.read().unwrap().clone();
            let new_blockchain = serde_json::from_str::<Vec<Block>>(payload.data.as_str()).unwrap();
            println!("Receive Blockchain: \nnew_blockchain {:#?}", new_blockchain);

            if get_is_replace_chain(&b_guard, &new_blockchain) {
                let mut b_guard = blockchain.write().unwrap();
                let mut u_guard = unspent_tx_outs.write().unwrap();

                match get_unspent_tx_outs(&new_blockchain) {
                    Ok(new_unspent_tx_outs) => {
                        let _ = mem::replace(&mut *b_guard, new_blockchain);
                        let _ = mem::replace(&mut *u_guard, new_unspent_tx_outs);
                        println!("Receive Blockchain: \nadded_blockchain {:#?}, \nnew_unspent_tx_outs {:#?}", b_guard, u_guard);
                        tx.send(BroadcastEvents::Blockchain(b_guard.to_vec(), Some(peer.clone()))).unwrap();
                    }
                    Err(error) => {
                        println!("{:#?}", error);
                    }
                }
            }
        }
        PayloadType::Transaction => {
            println!("Receive Transaction");
            let u_guard = unspent_tx_outs.read().unwrap().clone();
            let mut t_guard = transaction_pool.write().unwrap();
            let received_transactions = serde_json::from_str::<Vec<Transaction>>(payload.data.as_str()).unwrap();
            println!("Receive Transaction: \nreceived_transactions {:#?}", received_transactions);

            for transaction in received_transactions {
                match add_to_transaction_pool(&transaction, &mut t_guard, &u_guard) {
                    Ok(_) => {
                        println!("Receive Transaction: \nadded_transactions {:#?}", t_guard);
                        tx.send(BroadcastEvents::Transaction(t_guard.to_vec(), Some(peer.clone()))).unwrap();
                    }
                    Err(error) => {
                        println!("{:#?}", error);
                    }
                }
            }
        }
    }
}
