use crate::dao;
use crate::rest::ListQuery;
use actix_web::web::ServiceConfig;
use actix_web::{get, web};

#[get("/standard_score/{role_id}/{node_id}")]
async fn standard_score(
    _: web::Query<ListQuery>,
    path: web::Path<(String, String)>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<dao::StandardScore>> {
    let (role_id, node_id) = path.into_inner();
    let standard_score = data
        .standard_score(&role_id, &node_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(standard_score))
}

pub fn configure(config: &mut ServiceConfig) {
    config.service(standard_score);
}
