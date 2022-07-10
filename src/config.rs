use uuid::Uuid;
use rustop::opts;

use crate::constants::{DEFAULT_WEBSOCKET_PORT, DEFAULT_HTTP_PORT, PRIVATE_KEY_PATH};

/// Current app config for blockchain
#[derive(Debug)]
pub struct Config {
    /// port of websocket
    pub socket_port: u16,

    /// port of websocket
    pub http_port: u16,

    /// port of websocket
    pub uuid: String,
}

impl Config {
    /// Returns a config with args
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::config::{Config};
    /// let config = Config::new();
    /// ```
    pub fn new() -> Config {
        let uuid = format!("{}", Uuid::new_v4());
        let (args, _) = opts! {
            synopsis "This is a blockchain program."; // short info message for the help page
            opt socket_port:u16 = DEFAULT_WEBSOCKET_PORT, desc:"The port of socket."; // an option -s or --socket-port
            opt http_port:u16 = DEFAULT_HTTP_PORT, desc:"The port of http."; // an option -t or --http-port
            opt private_key_path:String = PRIVATE_KEY_PATH.to_string(), desc:"The path of private key."; // an option -u or --private-key-path
        }.parse_or_exit();

        Config { socket_port: args.socket_port, http_port: args.http_port, uuid }
    }
}
