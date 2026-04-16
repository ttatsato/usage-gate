use super::{QuotaCounter, QuotaCounterError, QuotaPeriod};
use async_trait::async_trait;
use chrono::{Datelike, Utc};
use redis::AsyncCommands;
use uuid::Uuid;

pub struct ValkeyQuotaCounter {
    client: redis::Client,
}

impl ValkeyQuotaCounter {
    pub async fn new(url: &str) -> Result<Self, QuotaCounterError> {
        let client =
            redis::Client::open(url).map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        // 起動時に接続確認
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        Ok(Self { client })
    }

    /// consumer_id + period でキーを作る
    /// 例: quota:550e8400-...:2026-04-monthly
    ///     quota:550e8400-...:2026-04-16-daily
    ///     quota:550e8400-...:2026-04-16T14-hourly
    fn key(&self, consumer_id: Uuid, period: &QuotaPeriod) -> String {
        let now = Utc::now();
        match period {
            QuotaPeriod::Monthly => {
                format!("quota:{}:{}-monthly", consumer_id, now.format("%Y-%m"))
            }
            QuotaPeriod::Daily => {
                format!("quota:{}:{}-daily", consumer_id, now.format("%Y-%m-%d"))
            }
            QuotaPeriod::Hourly => {
                format!("quota:{}:{}-hourly", consumer_id, now.format("%Y-%m-%dT%H"))
            }
        }
    }

    /// period に応じた TTL（秒）を返す。余裕を持たせて +1 単位分
    fn ttl_seconds(&self, period: &QuotaPeriod) -> i64 {
        let now = Utc::now();
        match period {
            QuotaPeriod::Monthly => {
                let days = days_remaining_in_month(now.year(), now.month());
                ((days + 1) * 86400) as i64
            }
            QuotaPeriod::Daily => 2 * 86400, // 2日
            QuotaPeriod::Hourly => 2 * 3600, // 2時間
        }
    }
}

#[async_trait]
impl QuotaCounter for ValkeyQuotaCounter {
    async fn get_count(
        &self,
        consumer_id: Uuid,
        period: &QuotaPeriod,
    ) -> Result<i64, QuotaCounterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        let count: Option<i64> = conn
            .get(self.key(consumer_id, period))
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        Ok(count.unwrap_or(0))
    }

    async fn increment(
        &self,
        consumer_id: Uuid,
        period: &QuotaPeriod,
    ) -> Result<(), QuotaCounterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        let key = self.key(consumer_id, period);

        // INCR + 初回のみ TTL 設定
        let count: i64 = conn
            .incr(&key, 1)
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        if count == 1 {
            let ttl = self.ttl_seconds(period);
            let _: () = conn
                .expire(&key, ttl)
                .await
                .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    async fn restore(
        &self,
        consumer_id: Uuid,
        period: &QuotaPeriod,
        count: i64,
    ) -> Result<(), QuotaCounterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

        let key = self.key(consumer_id, period);
        let ttl = self.ttl_seconds(period);

        let _: () = conn
            .set(&key, count)
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;
        let _: () = conn
            .expire(&key, ttl)
            .await
            .map_err(|e| QuotaCounterError::Internal(e.to_string()))?;

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
