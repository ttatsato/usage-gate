use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

// エンドポイント別の集計結果
#[derive(sqlx::FromRow, Serialize)]
pub struct UsageSummary {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
}

// GET /admin/usage のレスポンス
#[derive(Serialize)]
pub struct UsageResponse {
    pub tenant_id: Uuid,
    pub total_requests: i64,
    pub records: Vec<UsageSummary>,
}

// GET /admin/usage のクエリパラメータ
#[derive(Deserialize)]
pub struct UsageQuery {
    pub tenant_id: Uuid,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}
