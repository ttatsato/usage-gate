pub mod valkey;

use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub enum QuotaCounterError {
    Internal(String),
}

/// クォータの期間
#[derive(Debug, Clone)]
pub enum QuotaPeriod {
    Monthly,
    Daily,
    Hourly,
}

#[async_trait]
pub trait QuotaCounter: Send + Sync {
    /// 指定期間の現在のリクエスト数を取得
    async fn get_count(&self, consumer_id: Uuid, period: &QuotaPeriod) -> Result<i64, QuotaCounterError>;
    /// カウントを +1 する
    async fn increment(&self, consumer_id: Uuid, period: &QuotaPeriod) -> Result<(), QuotaCounterError>;
}
