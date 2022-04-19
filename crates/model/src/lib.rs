#![deny(unsafe_code)]
//#![deny(missing_docs)]
//! # Golem Reputation Aggregator Client Library
//!
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use derive_builder::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
pub use ya_client_model::NodeId;

/// Agreement status report.
///
/// ```rust
/// use bigdecimal::*;
/// use chrono::Utc;
/// use reputation_aggregator_model::StatusBuilder;
///
/// let status = StatusBuilder::default()
///     .requested(100i64)
///     .accepted(0i64)
///     .confirmed(0i64)
///     .ts(Utc::now())
///     .build().unwrap();
///
/// assert_eq!(status.confirmed, BigDecimal::zero())
/// ```
///
#[derive(Builder, Debug, Serialize, Deserialize)]
#[builder(setter(into), pattern = "owned")]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The amount of money that provider has requested.
    #[builder(setter(into), default)]
    pub requested: BigDecimal,
    /// The amount of money that requestor has accepted.
    #[builder(setter(into), default)]
    pub accepted: BigDecimal,
    /// The amount of money the provider has confirmed that is paid.
    #[builder(setter(into), default)]
    pub confirmed: BigDecimal,
    /// Event timestamp.
    #[builder(default = "Utc::now()")]
    pub ts: DateTime<Utc>,
    /// Expected payment clearing time.
    #[builder(setter(into), default)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_due_ts: Option<DateTime<Utc>>,
    /// Reserved for future use
    #[builder(setter(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment: Option<JsonValue>,
}

/// Static part of the contract information, unlike the status, has to be reported only once.
#[derive(Builder, Debug, Serialize, Deserialize)]
#[builder(setter(into), pattern = "owned")]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct AgreementInfo {
    /// other party to the contract
    pub peer_id: NodeId,
    /// Contract creation timestamp. since it took effect for node.
    pub created_ts: DateTime<Utc>,
    ///
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<DateTime<Utc>>,
    /// offer `golem.runtime.name`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// Name of payment platform (eg. zksync-rinkeby-tglm, erc20-polygon-glm)
    pub payment_platform: String,
    /// payment receipt account
    pub payment_address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Offer `golem.node.debug.subnet`
    pub subnet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Demand `golem.srv.comp.task_package`
    pub task_package: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReportResult {
    #[serde(rename = "ok")]
    Ok {},
    /// Report is incomplete because agreement is missing.
    #[serde(rename = "unknownAgreement")]
    UnknownAgreement {},
}

impl ReportResult {
    pub fn is_unknown_agreement(&self) -> bool {
        matches!(self, Self::UnknownAgreement {})
    }
}

#[cfg(any(feature = "client", feature = "client-old"))]
mod client;

#[cfg(any(feature = "client", feature = "client-old"))]
pub use client::*;
