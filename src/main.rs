#![forbid(unsafe_code)]

use std::sync::Arc;

use actix_web::{App, HttpServer};
use actix_web_static_files::ResourceFiles;
use tracing::Level;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::prelude::*;
use tracing_subscriber::FmtSubscriber;

mod config;
mod dao;
mod rest;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv().unwrap_or_default();
    env_logger::init();
    let config = Arc::new(config::ReputationServerConfig::load()?);
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    let bind_addr = config.listen_on;

    if config.apply_migrations {
        dao::apply_migrations(&config.database_url).await?;
        log::info!("migrations applied");
    } else {
        log::info!("skip db migrations");
    }

    HttpServer::new(move || {
        let config = config.clone();
        let generated = generate();

        App::new()
            .wrap(TracingLogger::default())
            .data_factory(move || dao::StatusDao::connect(config.database_url.clone()))
            .configure(rest::configure)
            .service(ResourceFiles::new("/", generated))
    })
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}
