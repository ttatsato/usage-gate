use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// 使用量レコード: リクエストごとの記録
#[derive(sqlx::FromRow, Serialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub api_key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i16,
    pub created_at: DateTime<Utc>,
}
