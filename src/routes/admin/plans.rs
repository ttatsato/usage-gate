use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::plan::{CreatePlan, Plan};
use crate::repositories::plan_repository;

pub async fn create_plan(
    State(pool): State<PgPool>,
    Json(body): Json<CreatePlan>,
) -> Result<(StatusCode, Json<Plan>), (StatusCode, Json<serde_json::Value>)> {
    let plan = plan_repository::create(&pool, body.project_id, &body.name, body.monthly_request_quota)
        .await
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create plan"})),
        ))?;
    Ok((StatusCode::CREATED, Json(plan)))
}

#[derive(Deserialize)]
pub struct ListPlansQuery {
    pub project_id: Uuid,
}

pub async fn list_plans(
    State(pool): State<PgPool>,
    Query(q): Query<ListPlansQuery>,
) -> Json<Vec<Plan>> {
    let plans = plan_repository::list_by_project(&pool, q.project_id)
        .await
        .expect("Failed to list plans");
    Json(plans)
}
