use actix_rt::main;
use chrono::Utc;
use reputation_aggregator_model::{RepuAggrClient, Status, StatusBuilder};
use std::error::Error;
use std::io;

#[main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let client = RepuAggrClient::with_url("http://reputation.dev.golem.network")?;

    let status = StatusBuilder::default()
        .requested(100i64)
        .accepted(0i64)
        .confirmed(0i64)
        .ts(Utc::now())
        .build()?;

    client
        .provider_report(
            "0xe0499005113c70C46608d06849dcCC3afdfe853E",
            "91a7c4e0-a51f-11ec-a039-7f34cd22e448",
            status,
        )
        .await?;

    Ok(())
}
