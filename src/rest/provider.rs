use crate::dao;
use crate::rest::ListQuery;
use actix_web::web::ServiceConfig;
use actix_web::{get, post, web, HttpResponse, Responder};
use reputation_aggregator_model::{NodeId, Status};

#[get("/provider")]
async fn list(
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

#[post("/provider/{node_id}/{agreement_id}/{peer_id}")]
async fn update_status(
    _: web::Query<ListQuery>,
    path: web::Path<(NodeId, String, NodeId)>,
    dao: web::Data<dao::StatusDao>,
    status: web::Json<Status>,
) -> impl Responder {
    let (node_id, agreement_id, peer_id) = path.into_inner();
    if let Err(e) = dao
        .insert_status("P", node_id, &agreement_id, peer_id, &status)
        .await
    {
        log::error!(
            "Unable to update provider status for {}/{}: {:?}",
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
