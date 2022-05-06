use crate::{AgreementInfo, ReportResult, Status};

#[cfg(feature = "client-old")]
use awc_old as awc;

use awc::error::SendRequestError;
use thiserror::Error;
use ya_client_model::NodeId;

#[derive(Clone)]
pub enum AgreementRole {
    Provider,
    Requestor,
}

impl AgreementRole {
    fn as_path(&self) -> &str {
        match self {
            Self::Provider => "/provider",
            Self::Requestor => "/requestor",
        }
    }
}

///
/// # Example
/// ```rust
/// use reputation_aggregator_model::{RepuAggrClient, Status};
/// let client = RepuAggrClient::with_url("http://reputation.dev.golem.network").unwrap();
///
/// client.provider_report(node_id, agreement_id, peer_id, StatusBuilder::default().requested(10).build().unwrap()).await.unwrap();
///```
///
#[derive(Clone)]
pub struct RepuAggrClient {
    client: awc::Client,
    base_url: String,
}

/// Error type.
#[derive(Error, Debug)]
pub enum RepuClientError {
    /// Http transport error.
    #[error("send request error {0}")]
    SendRequestError(#[from] SendRequestError),
    /// Server responds with communication error.
    #[error("{0}")]
    ProcessingError(String),
}

/// A specialized Result type for client operations.
pub type Result<T> = std::result::Result<T, RepuClientError>;

#[allow(missing_docs)]
impl RepuAggrClient {
    pub fn with_url(base_url: impl Into<String>) -> Result<Self> {
        let client = awc::Client::new();
        let base_url = base_url.into();
        Ok(RepuAggrClient { client, base_url })
    }

    pub async fn agreement(
        &self,
        role: AgreementRole,
        node_id: NodeId,
        agreement_id: &str,
        agreement: AgreementInfo,
    ) -> Result<()> {
        let url = format!(
            "{}{}/{node_id}/agreement/{agreement_id}",
            self.base_url,
            role.as_path()
        );
        let response = self.client.post(url).send_json(&agreement).await?;
        if !response.status().is_success() {
            return Err(RepuClientError::ProcessingError(format!(
                "bad response: {}",
                response.status()
            )));
        }
        Ok(())
    }

    pub async fn report(
        &self,
        role: AgreementRole,
        node_id: NodeId,
        agreement_id: &str,
        status: Status,
    ) -> Result<ReportResult> {
        // TODO add checks
        let base_url = &self.base_url;
        let role_path = role.as_path();
        let url = format!("{base_url}{role_path}/{node_id}/agreement/{agreement_id}/status");
        let mut response = self.client.post(url).send_json(&status).await?;
        if !response.status().is_success() {
            return Err(RepuClientError::ProcessingError(format!(
                "bad response: {}",
                response.status()
            )));
        }

        Ok(response.json().await.map_err(|e| RepuClientError::ProcessingError(e.to_string()))?)
    }
}
