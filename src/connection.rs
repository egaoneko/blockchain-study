use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use futures_util::stream::SplitSink;

#[derive(Debug)]
pub struct Connection {
    pub peer: String,
    pub listener: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
    pub connector: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

impl Connection {
    pub fn new(
        peer: String,
        listener: Option<SplitSink<WebSocketStream<TcpStream>, Message>>,
        connector: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>
    ) -> Self {
        Self { peer, listener, connector }
    }
}
