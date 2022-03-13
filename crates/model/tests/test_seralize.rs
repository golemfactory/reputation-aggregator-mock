use chrono::Utc;
use reputation_aggregator_model::*;
use serde_json::json;

#[test]
fn test_build() {
    let now = Utc::now();
    let status0 = StatusBuilder::default()
        .requested(10)
        .accepted(10)
        .ts(now)
        .build()
        .unwrap();

    assert_eq!(
        serde_json::to_value(&status0).unwrap(),
        json!({
            "requested":"10",
            "accepted":"10",
            "confirmed":"0",
            "ts": now
        })
    );
}
