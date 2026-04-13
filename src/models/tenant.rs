use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;


#[derive(sqlx::FromRow, Serialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub plan: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Deserialize)]
pub struct CreateTenant {
    pub name: String,
    pub plan: Option<String>, // default to "free"
}
