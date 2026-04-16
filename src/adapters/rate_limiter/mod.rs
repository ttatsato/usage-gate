pub mod valkey;

use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub enum RateLimiterError {
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub period: RateLimitPeriod,
    pub max_requests: i64,
}

#[derive(Debug, Clone)]
pub enum RateLimitPeriod {
    Monthly,
    Daily,
    Hourly,
    PerSecond,
}

impl RateLimitPeriod {
    /// DB 保存用のサフィックス文字列を返す
    pub fn to_key_suffix(&self) -> String {
        let now = chrono::Utc::now();
        match self {
            RateLimitPeriod::Monthly => format!("{}-monthly", now.format("%Y-%m")),
            RateLimitPeriod::Daily => format!("{}-daily", now.format("%Y-%m-%d")),
            RateLimitPeriod::Hourly => format!("{}-hourly", now.format("%Y-%m-%dT%H")),
            RateLimitPeriod::PerSecond => "per-second".to_string(),
        }
    }
}

#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// 全バケットをチェックし、許可されたらトークンを1つ消費する
    /// Ok(true) = 許可、Ok(false) = 拒否
    async fn try_acquire(
        &self,
        consumer_id: Uuid,
        limits: &[RateLimit],
    ) -> Result<bool, RateLimiterError>;

    /// 指定期間の消費済みトークン数を取得（sync-to-db 用）
    async fn get_usage(
        &self,
        consumer_id: Uuid,
        period: &RateLimitPeriod,
        max_requests: i64,
    ) -> Result<i64, RateLimiterError>;
}
