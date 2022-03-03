use reputation_aggregator_model::Status;
use sqlx::{query, PgPool, Pool, Postgres};
use std::io;

pub struct StatusDao {
    pool: PgPool,
}

impl StatusDao {
    pub async fn connect() -> sqlx::Result<Self> {
        let pool =
            Pool::<Postgres>::connect(std::env::var("DATABASE_URL").unwrap().as_str()).await?;
        Ok(StatusDao { pool })
    }

    pub async fn list_providers(&self) -> sqlx::Result<Vec<String>> {
        struct Node {
            node_id : String
        }
        let nodes = sqlx::query_as!(Node, "SELECT distinct node_id FROM AGREEMENT_STATUS where ROLE_ID = 'P'",)
            .fetch_all(&self.pool).await?;

        Ok(nodes.into_iter().map(|node| node.node_id).collect())
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
