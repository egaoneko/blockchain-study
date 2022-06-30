const DEFAULT_ROLE: &str = "server";
const DEFAULT_WEBSOCKET_PORT: &str = "2794";

/// Current app config for blockchain
#[derive(Debug)]
pub struct Config {
    /// role of app
    pub role: String,

    /// port of websocket
    pub port: String,

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
    /// let config = Config::new(&vec!["server".to_string(), "2794".to_string()], "67e55044-10b1-426f-9247-bb680e5fe0c8".to_string());
    /// ```
    pub fn new(args: &Vec<String>, uuid: String) -> Config {
        match args.len() {
            1 => Config { role: DEFAULT_ROLE.to_string(), port:DEFAULT_WEBSOCKET_PORT.to_string(), uuid },
            2 => {
                let role = args[1].clone();
                let port = DEFAULT_WEBSOCKET_PORT.to_string();

                Config { role, port, uuid }
            },
            _ => {
                let role = args[1].clone();
                let port = args[2].clone();

                Config { role, port, uuid }
            }
        }
    }
}
