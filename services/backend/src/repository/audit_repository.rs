use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;

use crate::domain::AuditLogEntry;
use crate::errors::Result;

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn log_event(
        &self,
        event_type: &str,
        aggregate_id: &str,
        user_id: Option<&str>,
        before_state: Option<serde_json::Value>,
        after_state: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
        actor: &str,
    ) -> Result<()>;
}

pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn log_event(
        &self,
        event_type: &str,
        aggregate_id: &str,
        user_id: Option<&str>,
        before_state: Option<serde_json::Value>,
        after_state: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
        actor: &str,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO audit_log (
                event_time, event_type, aggregate_id, user_id,
                before_state, after_state, metadata, actor
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            now,
            event_type,
            aggregate_id,
            user_id,
            before_state,
            after_state,
            metadata,
            actor
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
