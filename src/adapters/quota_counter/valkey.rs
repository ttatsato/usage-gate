use super::{QuotaCounter, QuotaCounterError};
use async_trait::async_trait;
use chrono::{Datelike, Utc};
use redis::AsyncCommands;
use uuid::Uuid;

pub struct ValkeyQuotaCounter {
    client: redis::Client,
}

impl ValkeyQuotaCounter {
    pub fn new(url: &str) -> Result<Self, QuotaCounterError> {
        let client =
            redis::Client::open(url).map_err(|e| QuotaCounterError::Internal(e.to_string()))?;
        Ok(Self { client })
    }

    /// consumer_id + 年月 でキーを作る（月が変わると自動リセット）
    fn key(&self, consumer_id: Uuid) -> String {
        let now = Utc::now();
        format!("quota:{}:{}", consumer_id, now.format("%Y-%m"))
    }
}

#[async_trait]
impl QuotaCounter for ValkeyQuotaCounter {
    async fn get_count(&self, consumer_id: Uuid) -> Result<i64, QuotaCounterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        let count: Option<i64> = conn
            .get(self.key(consumer_id))
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        Ok(count.unwrap_or(0))
    }

    async fn increment(&self, consumer_id: Uuid) -> Result<(), QuotaCounterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        let key = self.key(consumer_id);

        // INCR + 初回のみ TTL 設定
        let count: i64 = conn
            .incr(&key, 1)
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        if count == 1 {
            // 新しいキー → 月末まで + 1日の TTL を設定
            let now = Utc::now();
            let days = days_remaining_in_month(now.year(), now.month());
            let ttl_seconds = (days + 1) * 86400;
            let _: () = conn
                .expire(&key, ttl_seconds as i64)
                .await
                .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;
        }

        Ok(())
    }
}

fn days_remaining_in_month(year: i32, month: u32) -> u32 {
    let today = Utc::now().date_naive();
    let next_month_start = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next_month_start
        .unwrap()
        .signed_duration_since(today)
        .num_days() as u32
}
