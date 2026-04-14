use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub consumer_id: Uuid,
    pub key: Option<String>,
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

#[derive(sqlx::FromRow, Clone)]
pub struct AuthedApiKey {
    pub api_key_id: Uuid,
    pub tenant_id: Uuid,
    pub plan: String,
}
