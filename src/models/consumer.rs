use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize)]
pub struct Consumer {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateConsumer {
    pub tenant_id: uuid::Uuid,
    pub external_id: Option<String>,
}
