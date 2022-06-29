extern crate blockchain;

use std::env;
use blockchain::config::Config;
use blockchain::run;
use uuid::Uuid;

fn main() {
    let uuid: Uuid = Uuid::new_v4();
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args, format!("{}", uuid));

    run(config);
}
