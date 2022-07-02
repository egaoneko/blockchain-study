extern crate blockchain;

use blockchain::config::Config;
use blockchain::run;

fn main() {
    let config = Config::new();
    run(config);
}
