#![deny(unsafe_code)]
//#![deny(missing_docs)]
//! # Golem Reputation Aggregator Client Library
//!
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use derive_builder::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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
    /// Reserved for future use
    #[builder(setter(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment: Option<JsonValue>,
}

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
pub use client::*;
