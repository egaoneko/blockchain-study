use uuid::Uuid;
use rustop::opts;

const DEFAULT_WEBSOCKET_PORT: u16 = 2794;
const DEFAULT_HTTP_PORT: u16 = 8000;

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
            opt socket_port:u16 = DEFAULT_WEBSOCKET_PORT, desc:"The port of socket."; // an option -n or --socket-port
            opt http_port:u16 = DEFAULT_HTTP_PORT, desc:"The port of http."; // an option -n or --http-port
        }.parse_or_exit();

        Config { socket_port: args.socket_port, http_port: args.http_port, uuid }
    }
}
