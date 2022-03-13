#![forbid(unsafe_code)]

use ::config::Config;
use actix_web::{get, web, App, HttpServer, Responder};
use serde::Deserialize;
use sqlx::types::Json;
use std::io;
use std::path::Path;
use std::sync::Arc;

mod config;
mod dao;

#[derive(Deserialize)]
struct ListQuery {
    start: Option<u64>,
    limit: Option<usize>,
}

#[get("/provider")]
async fn list_providers(
    _: web::Query<ListQuery>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<Vec<String>>> {
    let providers = data
        .list_providers()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(providers))
}

#[get("/provider/{node_id}")]
async fn list_provider_agreements(
    _: web::Query<ListQuery>,
    path: web::Path<(String,)>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<Vec<dao::Agreement>>> {
    let (node_id,) = path.into_inner();
    let agreements = data
        .list_agreements("P", &node_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(agreements))
}

#[get("/requestor")]
async fn list_requestors(
    _: web::Query<ListQuery>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<Vec<String>>> {
    let requestors = data
        .list_requestors()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(requestors))
}

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
            .service(list_providers)
            .service(list_provider_agreements)
            .service(list_requestors)
    })
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}
