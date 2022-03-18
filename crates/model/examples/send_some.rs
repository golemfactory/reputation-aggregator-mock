use chrono::Utc;
use old_actix_rt::{Arbiter, System};
use reputation_aggregator_model::{RepuAggrClient, Status, StatusBuilder};
use std::error::Error;
use std::io;
use ya_client::web::WebInterface;

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let sys = System::run(|| {
        Arbiter::spawn(async move {
            if let Err(e) = async_main().await {
                log::error!("{:?}", e);
            }
            System::current().stop();
        });
    })?;

    Ok(())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    let yac = ya_client::web::WebClient::with_token("smok");
    let client = RepuAggrClient::with_url("http://reputation.dev.golem.network")?;

    let status = StatusBuilder::default()
        .requested(100i64)
        .accepted(0i64)
        .confirmed(0i64)
        .ts(Utc::now())
        .build()?;

    client
        .provider_report(
            "0xe0499005113c70C46608d06849dcCC3afdfe853E".parse()?,
            "91a7c4e0-a51f-11ec-a039-7f34cd22e448",
            "0x177bedD64Fe590066c4AdCf86782102071A520E9".parse()?,
            status,
        )
        .await?;

    let payment = ya_client::payment::PaymentApi::from_client(yac);
    for ev in payment
        .get_debit_note_events::<Utc>(None, None, Some(2), None)
        .await?
    {
        eprintln!("{:?}", ev);
    }
    Ok(())
}
