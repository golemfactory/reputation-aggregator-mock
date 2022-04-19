use actix_web::web::ServiceConfig;
use serde::Deserialize;

mod report;
mod score;

#[derive(Deserialize)]
struct ListQuery {
    start: Option<u64>,
    limit: Option<usize>,
}

pub fn configure(config: &mut ServiceConfig) {
    config
        .configure(report::configure)
        .configure(score::configure);
}
