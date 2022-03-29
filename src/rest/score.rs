use crate::dao;
use crate::rest::ListQuery;
use actix_web::web::ServiceConfig;
use actix_web::{get, web};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Role {
    Provider,
    Requestor,
}

impl Role {
    #[inline]
    fn as_db_role(&self) -> &str {
        match self {
            Role::Provider => "P",
            Role::Requestor => "R",
        }
    }
}

#[get("/standard_score/{role_id}/{node_id}")]
async fn standard_score(
    data: web::Data<dao::StatusDao>,
    path: web::Path<(Role, String)>,
) -> actix_web::Result<web::Json<dao::StandardScore>> {
    let (role_id, node_id) = path.into_inner();
    let standard_score = data
        .standard_score(role_id.as_db_role(), &node_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(standard_score))
}

pub fn configure(config: &mut ServiceConfig) {
    config.service(standard_score);
}
