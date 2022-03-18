use reputation_aggregator_model::{NodeId, Status, StatusBuilder};
use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::types::BigDecimal;
use sqlx::{PgPool, Pool, Postgres};

static MIGRATOR: Migrator = sqlx::migrate!();

pub struct StatusDao {
    pool: PgPool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agreement {
    agreement_id: String,
    peer_id: String,
    created_ts: DateTime<Utc>,
    status: Status,
}

impl StatusDao {
    pub async fn connect(url: String) -> sqlx::Result<Self> {
        log::debug!("connect to {}", url);
        let pool = Pool::<Postgres>::connect(&url).await?;
        Ok(StatusDao { pool })
    }

    pub async fn list_providers(&self) -> sqlx::Result<Vec<String>> {
        self.list("P").await
    }

    pub async fn list_requestors(&self) -> sqlx::Result<Vec<String>> {
        self.list("R").await
    }

    async fn list(&self, role_id: &str) -> sqlx::Result<Vec<String>> {
        struct Node {
            node_id: String,
        }
        let nodes = sqlx::query_as!(
            Node,
            "SELECT distinct node_id FROM AGREEMENT_STATUS where ROLE_ID = $1",
            role_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes.into_iter().map(|node| node.node_id).collect())
    }

    pub async fn list_agreements(
        &self,
        role_id: &str,
        node_id: &str,
    ) -> sqlx::Result<Vec<Agreement>> {
        struct AgreementRow {
            agreement_id: String,
            peer_id: Option<String>,
            created_ts: NaiveDateTime,
            updated_ts: NaiveDateTime,
            requested: BigDecimal,
            accepted: BigDecimal,
            confirmed: BigDecimal,
        }

        let agreement_rows = sqlx::query_as!(
            AgreementRow,
            r#"
            SELECT agreement_id, peer_id, created_ts, updated_ts,
                requested, accepted, confirmed
             FROM AGREEMENT_STATUS where ROLE_ID = $1 and NODE_ID=$2"#,
            role_id,
            node_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(agreement_rows
            .into_iter()
            .map(|agreement_row: AgreementRow| {
                Ok(Agreement {
                    agreement_id: agreement_row.agreement_id,
                    peer_id: agreement_row.peer_id.unwrap_or_default(),
                    created_ts: DateTime::from_utc(agreement_row.created_ts, Utc),
                    status: StatusBuilder::default()
                        .requested(agreement_row.requested)
                        .accepted(agreement_row.accepted)
                        .confirmed(agreement_row.confirmed)
                        .ts(DateTime::<Utc>::from_utc(agreement_row.updated_ts, Utc))
                        .build()
                        .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                })
            })
            .collect::<sqlx::Result<_>>()?)
    }

    pub async fn insert_status(
        &self,
        role: &str,
        node_id: NodeId,
        agreement_id: &str,
        peer_id: NodeId,
        status: &Status,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO AGREEMENT_STATUS(role_id, node_id, agreement_id, requested,
            accepted, confirmed, peer_id, reported_ts)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(role_id, node_id, agreement_id)
            DO
                UPDATE SET
                    requested = $4,
                    accepted = $5,
                    confirmed = $6,
                    updated_ts = CURRENT_TIMESTAMP,
                    reported_ts = $8
        "#,
            role,
            node_id.to_string(),
            agreement_id,
            status.requested,
            status.accepted,
            status.confirmed,
            peer_id.to_string(),
            status.ts
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub async fn apply_migrations(database_url: &str) -> anyhow::Result<()> {
    let pool = Pool::<Postgres>::connect(&database_url).await?;
    MIGRATOR.run(&pool).await?;
    Ok(())
}
