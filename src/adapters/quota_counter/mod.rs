pub mod database;
pub mod valkey;

use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub enum QuotaCounterError {
    Internal(String),
}

#[async_trait]
pub trait QuotaCounter: Send + Sync {
 /// 現在の月間リクエスト数を取得
 async fn get_count(&self, consumer_id: Uuid) -> Result<i64, QuotaCounterError>;
 /// カウントを +1 する
 async fn increment(&self, consumer_id: Uuid) -> Result<(), QuotaCounterError>;
}
