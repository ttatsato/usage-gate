use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct UpstreamService {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub base_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateUpstreamService {
    pub project_id: Uuid,
    pub name: String,
    pub base_url: String,
}
