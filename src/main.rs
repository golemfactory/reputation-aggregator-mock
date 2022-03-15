#![forbid(unsafe_code)]

use std::sync::Arc;

use actix_web::{App, HttpServer};

mod config;
mod dao;
mod rest;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv().unwrap_or_default();
    env_logger::init();
    let config = Arc::new(config::ReputationServerConfig::load()?);

    let bind_addr = config.listen_on;

    if config.apply_migrations {
        dao::apply_migrations(&config.database_url).await?;
        log::info!("migations applied");
    } else {
        log::info!("skip db migrations");
    }

    HttpServer::new(move || {
        let config = config.clone();
        App::new()
            .data_factory(move || dao::StatusDao::connect(config.database_url.clone()))
            .configure(rest::configure)
    })
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}
