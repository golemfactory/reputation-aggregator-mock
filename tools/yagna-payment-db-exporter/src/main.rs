use chrono::{DateTime, NaiveDateTime};
use futures::prelude::*;
use reputation_aggregator_model::*;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::types::chrono::Utc;
use sqlx::types::BigDecimal;
use sqlx::{Connection, SqliteConnection};
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long)]
    database: Option<PathBuf>,
    #[structopt(long, default_value = "http://reputation.dev.golem.network")]
    url: String,
}

#[actix_rt::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    env_logger::init();
    log::info!("start");
    let database_path = args.database.unwrap_or_else(|| {
        std::env::home_dir()
            .unwrap()
            .join(".local/share/yagna/payment.db")
    });
    let options = SqliteConnectOptions::default()
        .filename(&database_path)
        .immutable(true);

    let mut connection = SqliteConnection::connect_with(&options).await?;

    log::info!("connected");

    #[derive(Debug)]
    struct PayAgreement {
        id: String,
        role: String,
        owner_id: String,
        peer_id: String,
        total_amount_due: String,
        total_amount_accepted: String,
        total_amount_paid: String,
        updated_ts: Option<NaiveDateTime>,
    }

    let agreements: Vec<PayAgreement> = sqlx::query_as!(
        PayAgreement,
        r#"
        SELECT id, role, owner_id, peer_id,
            total_amount_due, total_amount_accepted, total_amount_paid,
            updated_ts
        FROM pay_agreement"#
    )
    .fetch_all(&mut connection)
    .await?;

    let client = RepuAggrClient::with_url(&args.url)?;

    let _ = stream::iter(agreements)
        .map(|agreement| {
            let client = &client;
            async move {
                log::info!("sending: {}", agreement.id);
                let requested: BigDecimal = agreement.total_amount_due.parse()?;
                let accepted: BigDecimal = agreement.total_amount_accepted.parse()?;
                let paid: BigDecimal = agreement.total_amount_paid.parse()?;
                let ts = agreement
                    .updated_ts
                    .map(|ts| DateTime::from_utc(ts, Utc))
                    .unwrap_or_else(|| Utc::now());

                let status = StatusBuilder::default()
                    .requested(requested)
                    .accepted(accepted)
                    .confirmed(paid)
                    .ts(ts)
                    .build()?;
                match agreement.role.as_str() {
                    "R" => {
                        client
                            .requestor_report(
                                agreement.owner_id.parse()?,
                                &agreement.id,
                                agreement.peer_id.parse()?,
                                status,
                            )
                            .await?
                    }
                    "P" => {
                        client
                            .provider_report(
                                agreement.owner_id.parse()?,
                                &agreement.id,
                                agreement.peer_id.parse()?,
                                status,
                            )
                            .await?
                    }
                    _ => panic!("Invalid agreement role: {:?}", agreement),
                }
                Ok::<_, Box<dyn Error>>(())
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<std::result::Result<(), Box<dyn Error>>>>()
        .await;

    Ok(())
}
