use reputation_aggregator_model::{Status, StatusBuilder};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::types::BigDecimal;
use sqlx::{query, PgPool, Pool, Postgres};
use std::io;

pub struct StatusDao {
    pool: PgPool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agreement {
    agreement_id: String,
    created_ts: DateTime<Utc>,
    status: Status,
}

impl StatusDao {
    pub async fn connect() -> sqlx::Result<Self> {
        let pool =
            Pool::<Postgres>::connect(std::env::var("DATABASE_URL").unwrap().as_str()).await?;
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
            created_ts: NaiveDateTime,
            updated_ts: NaiveDateTime,
            requested: BigDecimal,
            accepted: BigDecimal,
            confirmed: BigDecimal,
        }

        let agreement_rows = sqlx::query_as!(
            AgreementRow,
            r#"
            SELECT agreement_id, created_ts, updated_ts,
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
        &mut self,
        role: &str,
        node_id: &str,
        agreement_id: &str,
        status: &Status,
    ) -> sqlx::Result<()> {
        sqlx::query!(r#"
            INSERT INTO AGREEMENT_STATUS(role_id, node_id, agreement_id, requested, accepted, confirmed)
            VALUES($1, $2, $3, $4, $5, $6)
            ON CONFLICT(role_id, node_id, agreement_id)
            DO
                UPDATE SET
                    requested = $4,
                    accepted = $5,
                    confirmed = $6,
                    updated_ts = CURRENT_TIMESTAMP
        "#,
            role, node_id, agreement_id,
            status.requested, status.accepted, status.confirmed
        ).execute(&self.pool).await?;
        Ok(())
    }
}
