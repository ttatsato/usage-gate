use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub consumer_id: Uuid,
    pub key_prefix: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateApiKey {
    pub consumer_id: Uuid,
    pub name: Option<String>,
}

// 発行時のみ返すレスポンス（平文キー付き）
#[derive(Serialize)]
pub struct CreatedApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub consumer_id: Uuid,
    pub key: String,
    pub key_prefix: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct AuthedApiKey {
    pub api_key_id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Uuid,
    pub consumer_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub plan_name: Option<String>,
    pub monthly_request_quota: Option<i32>,
    pub daily_request_quota: Option<i32>,
    pub hourly_request_quota: Option<i32>,
    pub per_second_request_limit: Option<i32>,
}
