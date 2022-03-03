#![forbid(unsafe_code)]

use actix_web::{get, web, App, HttpServer, Responder};
use serde::Deserialize;
use std::io;
use sqlx::types::Json;

mod dao;

#[derive(Deserialize)]
struct ListQuery {
    start: Option<u64>,
    limit: Option<usize>,
}

#[get("/provider")]
async fn list_providers(_: web::Query<ListQuery>, data : web::Data<dao::StatusDao>) -> actix_web::Result<web::Json<Vec<String>>> {
    let providers = data.list_providers().await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(providers))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let _ = dotenv::dotenv().unwrap_or_default();
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .data_factory(|| dao::StatusDao::connect())
            .service(list_providers)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
