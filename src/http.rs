use std::sync::{Arc, RwLock};
use rocket_contrib::json::Json;
use rocket_cors::{Cors, CorsOptions};

use crate::{Block, Config, routes};
use crate::errors::ApiError;

#[catch(404)]
fn not_found() -> Json<ApiError> {
    Json(ApiError::new(404, "Resource was not found."))
}

fn cors_fairing() -> Cors {
    CorsOptions::default()
        .to_cors()
        .expect("Cors fairing cannot be created")
}

pub fn launch_http(config: &Config, blockchain: &Arc<RwLock<Vec<Block>>>) {
    rocket::ignite()
        .mount("/api", routes![
            routes::ping
        ])
        .attach(cors_fairing())
        .launch();
}
