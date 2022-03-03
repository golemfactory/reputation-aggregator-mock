use derive_builder::*;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
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
#[derive(Builder, Debug)]
#[builder(setter(into), pattern = "owned")]
#[non_exhaustive]
pub struct Status {
    #[builder(setter(into), default)]
    pub requested : BigDecimal,
    #[builder(setter(into), default)]
    pub accepted : BigDecimal,
    #[builder(setter(into), default)]
    pub confirmed : BigDecimal,
    pub ts : DateTime<Utc>,
    /// Reserved for furuee use
    #[builder(setter(skip))]
    pub payment : Option<JsonValue>
}
