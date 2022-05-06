use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::types::BigDecimal;
use sqlx::{PgPool, Pool, Postgres};

use reputation_aggregator_model::{
    AgreementInfo, AgreementInfoBuilder, NodeId, Status, StatusBuilder,
};

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardScore {
    pub score: Option<BigDecimal>,
}

impl StatusDao {
    pub async fn connect(url: String) -> sqlx::Result<Self> {
        log::debug!("connect to {}", url);
        let pool = Pool::<Postgres>::connect(&url).await?;
        Ok(StatusDao { pool })
    }

    pub async fn list(&self, role_id: &str) -> sqlx::Result<Vec<String>> {
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

    pub async fn get_agreement_details(
        &self,
        role_id: &str,
        node_id: &str,
        agreement_id: &str,
    ) -> sqlx::Result<Option<AgreementInfo>> {
        struct DetailsRow {
            peer_id: String,
            created_ts: DateTime<Utc>,
            valid_to: Option<DateTime<Utc>>,
            runtime: Option<String>,
            payment_platform: Option<String>,
            payment_address: Option<String>,
            subnet: Option<String>,
            task_package: Option<String>,
        }
        let r: Option<DetailsRow> = sqlx::query_as!(
            DetailsRow,
            r#"
        SELECT
            peer_id, created_ts, valid_to, runtime,
            payment_platform, payment_address, subnet, task_package
        FROM agreement_details
        WHERE ROLE_ID = $1 and NODE_ID=$2 and agreement_id = $3
        "#,
            role_id,
            node_id,
            agreement_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.and_then(|row| {
            AgreementInfoBuilder::default()
                .peer_id(row.peer_id.parse::<NodeId>().ok()?)
                .created_ts(row.created_ts)
                .valid_to(row.valid_to)
                .runtime(row.runtime)
                .payment_platform(row.payment_platform?)
                .payment_address(row.payment_address?)
                .subnet(row.subnet)
                .task_package(row.task_package)
                .build()
                .ok()
        }))
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
            SELECT
                s.agreement_id as agreement_id,
                d.peer_id as "peer_id?",
                s.created_ts as created_ts,
                s.updated_ts as updated_ts,
                s.requested requested,
                s.accepted accepted,
                s.confirmed confirmed
             FROM AGREEMENT_STATUS s left join AGREEMENT_DETAILS d
               on (s.role_id = d.role_id and s.node_id = d.node_id and s.agreement_id = d.agreement_id)
             where s.ROLE_ID = $1 and s.NODE_ID=$2"#,
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

    pub async fn insert_agreement(
        &self,
        role: &str,
        node_id: NodeId,
        agreement_id: &str,
        agreement_info: AgreementInfo,
    ) -> sqlx::Result<bool> {
        let r = sqlx::query!(
            r#"
            INSERT INTO AGREEMENT_DETAILS(
                role_id, node_id, agreement_id,
                peer_id, created_ts, valid_to, runtime, payment_platform,
                payment_address, subnet, task_package)
                VALUES($1, $2, $3,
                $4, $5, $6, $7, $8,
                $9, $10, $11)
        "#,
            role,
            node_id.to_string(),
            agreement_id,
            agreement_info.peer_id.to_string(),
            agreement_info.created_ts,
            agreement_info.valid_to,
            agreement_info.runtime,
            agreement_info.payment_platform,
            agreement_info.payment_address,
            agreement_info.subnet,
            agreement_info.task_package
        )
        .execute(&self.pool)
        .await?;

        Ok(false)
    }

    pub async fn insert_status(
        &self,
        role: &str,
        node_id: NodeId,
        agreement_id: &str,
        status: &Status,
    ) -> sqlx::Result<bool> {
        let mut connection = self.pool.acquire().await?;
        let node_is_str = node_id.to_string();
        let _query = sqlx::query!(
            r#"
            INSERT INTO AGREEMENT_STATUS(role_id, node_id, agreement_id, requested,
            accepted, confirmed, reported_ts)
            VALUES($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT(role_id, node_id, agreement_id)
            DO
                UPDATE SET
                    requested = $4,
                    accepted = $5,
                    confirmed = $6,
                    updated_ts = CURRENT_TIMESTAMP,
                    reported_ts = $7
        "#,
            role,
            &node_is_str,
            agreement_id,
            status.requested,
            status.accepted,
            status.confirmed,
            status.ts
        )
        .execute(&mut connection)
        .await?;

        let have_details: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT *
                FROM AGREEMENT_DETAILS
                WHERE ROLE_ID = $1
                  AND NODE_ID = $2
                  AND AGREEMENT_ID = $3)
         "#,
            role,
            &node_is_str,
            agreement_id
        )
        .fetch_one(&mut connection)
        .await?
        .unwrap_or_default();

        Ok(have_details)
    }

    pub async fn standard_score(
        &self,
        role_id: &str,
        node_id: &str,
    ) -> sqlx::Result<StandardScore> {
        let standard_score = sqlx::query_as!(
            StandardScore,
            r#"SELECT CALC.STANDARD_SCORE($1, $2) as score"#,
            role_id,
            node_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(standard_score)
    }
}

pub async fn apply_migrations(database_url: &str) -> anyhow::Result<()> {
    let pool = Pool::<Postgres>::connect(&database_url).await?;
    MIGRATOR.run(&pool).await?;
    Ok(())
}
