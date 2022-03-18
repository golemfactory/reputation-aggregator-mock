use crate::Status;

#[cfg(feature = "client-old")]
use awc_old as awc;

use awc::error::SendRequestError;
use thiserror::Error;
use ya_client_model::NodeId;

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
    #[error("{0}")]
    ProcessingError(String)
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

    async fn send_report(
        &self,
        role: &str,
        node_id: NodeId,
        agreement_id: &str,
        peer_id: NodeId,
        status: Status,
    ) -> Result<()> {
        // TODO add checks
        let url = format!(
            "{}/{}/{}/{}/{}",
            self.base_url, role, node_id, agreement_id, peer_id
        );
        let response = self.client.post(url).send_json(&status).await?;
        if !response.status().is_success() {
            return Err(RepuClientError::ProcessingError(format!("bad response: {}", response.status())))
        }
        Ok(())
    }

    pub async fn provider_report(
        &self,
        node_id: NodeId,
        agreement_id: &str,
        peer_id: NodeId,
        status: Status,
    ) -> Result<()> {
        self.send_report("provider", node_id, agreement_id, peer_id, status)
            .await
    }

    pub async fn requestor_report(
        &self,
        node_id: NodeId,
        agreement_id: &str,
        peer_id: NodeId,
        status: Status,
    ) -> Result<()> {
        self.send_report("requestor", node_id, agreement_id, peer_id, status)
            .await
    }
}
