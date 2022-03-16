use crate::Status;
use awc::error::SendRequestError;
use awc::http::Uri;
use thiserror::Error;

///
/// # Example
/// ```
/// use reputation_aggregator_model::{RepuAggrClient, Status};
/// let client = RepuAggrClient::with_url("http://reputation.dev.golem.network").unwrap();
///
/// client.provider_report(node_id, agreement_id, StatusBuilder::default().requested(10).build().unwrap()).await.unwrap();
///
#[derive(Clone)]
pub struct RepuAggrClient {
    client: awc::Client,
    base_url: String,
}

#[derive(Error, Debug)]
pub enum RepuClientError {
    #[error("send request error {0}")]
    SendRequestError(#[from] SendRequestError),
}

pub type Result<T> = std::result::Result<T, RepuClientError>;

impl RepuAggrClient {
    pub fn with_url(base_url: impl Into<String>) -> Result<Self> {
        let client = awc::Client::new();
        let base_url = base_url.into();
        Ok(RepuAggrClient { client, base_url })
    }

    async fn send_report(
        &self,
        role: &str,
        node_id: &str,
        agreement_id: &str,
        status: Status,
    ) -> Result<()> {
        // TODO add checks
        let url = format!("{}/{}/{}/{}", self.base_url, role, node_id, agreement_id);
        self.client.post(url).send_json(&status).await?;
        // TODO check if status 200 or 201
        Ok(())
    }

    pub async fn provider_report(
        &self,
        node_id: &str,
        agreement_id: &str,
        status: Status,
    ) -> Result<()> {
        self.send_report("provider", node_id, agreement_id, status)
            .await
    }

    pub async fn requestor_report(
        &self,
        node_id: &str,
        agreement_id: &str,
        status: Status,
    ) -> Result<()> {
        self.send_report("requestor", node_id, agreement_id, status)
            .await
    }
}
