use actix_web::web::ServiceConfig;
use serde::Deserialize;

mod provider;
mod requestor;
mod score;

#[derive(Deserialize)]
struct ListQuery {
    start: Option<u64>,
    limit: Option<usize>,
}

pub fn configure(config: &mut ServiceConfig) {
    config
        .configure(provider::configure)
        .configure(requestor::configure)
        .configure(score::configure);
}
