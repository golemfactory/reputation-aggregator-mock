#![forbid(unsafe_code)]

use actix_web::{get, web, App, HttpServer, Responder};
use serde::Deserialize;
use sqlx::types::Json;
use std::io;

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
async fn main() -> io::Result<()> {
    let _ = dotenv::dotenv().unwrap_or_default();
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .data_factory(|| dao::StatusDao::connect())
            .service(list_providers)
            .service(list_provider_agreements)
            .service(list_requestors)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
