use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Datelike, TimeZone, Utc};
use super::{QuotaCounter, QuotaCounterError};
use async_trait::async_trait;

pub struct DatabaseQuotaCounter {
    pool: PgPool,
}

impl DatabaseQuotaCounter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuotaCounter for DatabaseQuotaCounter {
    async fn get_count(&self, consumer_id: Uuid) -> Result<i64, QuotaCounterError> {
             let now = Utc::now();
             let month_start = Utc
                 .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
                 .unwrap();

             let row = sqlx::query!(
                 r#"SELECT COUNT(*) as "count!" FROM usage_records WHERE consumer_id = $1 AND created_at >= $2"#,
                 consumer_id,
                 month_start,
             )
             .fetch_one(&self.pool)
             .await
             .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

             Ok(row.count)
         }

    async fn increment(&self, _consumer_id: Uuid) -> Result<(), QuotaCounterError> {
        Ok(())
    }
}
