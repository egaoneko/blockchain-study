use std::fmt;

const DEFAULT_ROLE: &str = "server";
const DEFAULT_WEBSOCKET_PORT: &str = "2794";

/// Current app config for blockchain
pub struct Config {
    /// role of app
    role: String,

    /// port of websocket
    port: String,
}

impl Config {
    /// Returns a config with args
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::config::{Config};
    /// let config = Config::new(&vec!["server".to_string(), "2794".to_string()]);
    /// ```
    pub fn new(args: &[String]) -> Config {
        match args.len() {
            1 => Config { role: DEFAULT_ROLE.to_string(), port:DEFAULT_WEBSOCKET_PORT.to_string() },
            2 => {
                let role = args[1].clone();
                let port = DEFAULT_WEBSOCKET_PORT.to_string();

                Config { role, port }
            },
            _ => {
                let role = args[1].clone();
                let port = args[2].clone();

                Config { role, port }
            }
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config {{ role: {}, port: {} }}", self.role, self.port)
    }
}
