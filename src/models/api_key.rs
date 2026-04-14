use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// DB レコード表現（key_hash は API レスポンスに含めない）
#[derive(FromRow, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
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
    pub consumer_id: Uuid,
    pub key: String,
    pub key_prefix: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Clone)]
pub struct AuthedApiKey {
    pub api_key_id: Uuid,
    pub tenant_id: Uuid,
    pub plan: String,
}
