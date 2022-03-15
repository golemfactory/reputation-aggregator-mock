use super::ListQuery;
use crate::dao;
use actix_web::web::ServiceConfig;
use actix_web::{get, post, web, HttpResponse, Responder};
use reputation_aggregator_model::Status;

#[get("/requestor")]
async fn list(
    _: web::Query<ListQuery>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<Vec<String>>> {
    let requestors = data
        .list_requestors()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(requestors))
}

#[get("/requestor/{node_id}")]
async fn list_agreements(
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

#[post("/requestor/{node_id}/{agreement_id}")]
async fn update_status(
    _: web::Query<ListQuery>,
    path: web::Path<(String, String)>,
    dao: web::Data<dao::StatusDao>,
    status: web::Json<Status>,
) -> impl Responder {
    let (node_id, agreement_id) = path.into_inner();
    if let Err(e) = dao
        .insert_status("R", &node_id, &agreement_id, &status)
        .await
    {
        log::error!(
            "Unable to update requestor status for {}/{}: {:?}",
            node_id,
            agreement_id,
            e
        );

        HttpResponse::InternalServerError().json(format!("{:?}", e))
    } else {
        HttpResponse::NoContent().finish()
    }
}

pub fn configure(config: &mut ServiceConfig) {
    config
        .service(list)
        .service(list_agreements)
        .service(update_status);
}
