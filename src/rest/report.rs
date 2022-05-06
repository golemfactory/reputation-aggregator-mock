use crate::dao;
use actix_web::web;
use actix_web::Result;
use actix_web::{get, post};
use reputation_aggregator_model::{AgreementInfo, NodeId, ReportResult, Status};
use serde::Deserialize;

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .service(list_nodes)
        .service(list_agreements)
        .service(get_agreement_details)
        .service(save_agreement_details);
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Role {
    Provider,
    Requestor,
}

impl Role {
    fn as_db(&self) -> &'static str {
        match self {
            Self::Provider => "P",
            Self::Requestor => "R",
        }
    }
}

#[get("/{role_id}")]
async fn list_nodes(
    path: web::Path<(Role,)>,
    data: web::Data<dao::StatusDao>,
) -> Result<web::Json<Vec<String>>> {
    let (role,) = path.into_inner();

    Ok(web::Json(data.list(role.as_db()).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })?))
}

#[get("/{role_id}/{node_id}/agreement")]
async fn list_agreements(
    path: web::Path<(Role, NodeId)>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<Vec<dao::Agreement>>> {
    let (role, node_id) = path.into_inner();
    let agreements = data
        .list_agreements(role.as_db(), &node_id.to_string())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    Ok(web::Json(agreements))
}

#[get("/{role_id}/{node_id}/agreement/{agreement_id}")]
async fn get_agreement_details(
    path: web::Path<(Role, NodeId, String)>,
    data: web::Data<dao::StatusDao>,
) -> actix_web::Result<web::Json<AgreementInfo>> {
    let (role, node_id, agreement_id) = path.into_inner();
    if let Some(agr_info) = data
        .get_agreement_details(role.as_db(), &node_id.to_string(), &agreement_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        Ok(web::Json(agr_info))
    } else {
        Err(actix_web::error::ErrorNotFound(format!(
            "agreement {} not found",
            agreement_id
        )))
    }
}

#[post("/{role_id}/{node_id}/agreement/{agreement_id}")]
async fn save_agreement_details(
    path: web::Path<(Role, NodeId, String)>,
    data: web::Data<dao::StatusDao>,
    body: web::Json<AgreementInfo>
) -> actix_web::Result<web::Json<()>> {
    let (role, node_id, agreement_id) = path.into_inner();
    let _ = data.insert_agreement(role.as_db(), node_id, &agreement_id, body.into_inner()).await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(web::Json(()))
}

#[post("/{role_id}/{node_id}/agreement/{agreement_id}/status")]
async fn save_agreement_status(
    path: web::Path<(Role, NodeId, String)>,
    data: web::Data<dao::StatusDao>,
    body: web::Json<Status>,
) -> actix_web::Result<web::Json<ReportResult>> {
    let (role, node_id, agreement_id) = path.into_inner();
    let have_agreement = data
        .insert_status(role.as_db(), node_id, &agreement_id, &body)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(web::Json(if have_agreement {
        ReportResult::Ok {}
    } else {
        ReportResult::UnknownAgreement {}
    }))
}
