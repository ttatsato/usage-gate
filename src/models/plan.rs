use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct Plan {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub monthly_request_quota: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreatePlan {
    pub project_id: Uuid,
    pub name: String,
    pub monthly_request_quota: Option<i32>,
}
