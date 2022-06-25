extern crate blockchain;

use std::env;
use blockchain::config::Config;
use blockchain::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    run(config);
}
