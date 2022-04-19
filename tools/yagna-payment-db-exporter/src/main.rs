use chrono::{DateTime, NaiveDateTime};
use futures::prelude::*;
use reputation_aggregator_model::*;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::types::chrono::Utc;
use sqlx::types::BigDecimal;
use sqlx::{Connection, Sqlite, SqliteConnection, FromRow};
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long)]
    /// Database with payments
    payment_database: Option<PathBuf>,
    /// Market database
    market_database: Option<PathBuf>,
    #[structopt(long, conflicts_with_all=&["payment_database", "market_database"])]
    data_dir : Option<PathBuf>,
    #[structopt(long, default_value = "http://reputation.dev.golem.network")]
    url: String,
}

fn role_from_db(db_role_id:&str) -> Option<AgreementRole> {
    Some(match db_role_id {
        "R" => AgreementRole::Requestor,
        "P" => AgreementRole::Provider,
        _ => return None
    })
}

#[actix_rt::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    env_logger::init();
    log::info!("start");
        
    let database_path = args.payment_database.unwrap_or_else(|| {
        std::env::home_dir()
            .unwrap()
            .join(".local/share/yagna/payment.db")
    });
    let options = SqliteConnectOptions::default()
        .filename(&database_path)
        .immutable(true);

    let mut connection = SqliteConnection::connect_with(&options).await?;

    log::info!("connected");

    #[derive(Debug, FromRow)]
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

    let agreements: Vec<PayAgreement> = sqlx::query_as::<Sqlite, PayAgreement>(
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
                let role = role_from_db(&agreement.role).unwrap();
                let peer_id : NodeId = agreement.peer_id.parse()?;

                match client.report(role,  agreement.owner_id.parse()?,
                              &agreement.id,
                              status).await? {
                    ReportResult::UnknownAgreement {} => {
                        log::warn!("missing data for: {}", agreement.id)
                    }
                    _ => ()
                }
                Ok::<_, Box<dyn Error>>(())
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<std::result::Result<(), Box<dyn Error>>>>()
        .await;

    Ok(())
}
