use crate::models::consumer::{Consumer, CreateConsumer};
use crate::repositories::{consumer_repository, project_repository};
use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;

pub async fn create_consumer(
    State(pool): State<PgPool>,
    Json(body): Json<CreateConsumer>,
) -> Result<(StatusCode, Json<Consumer>), (StatusCode, Json<serde_json::Value>)> {
    // project_id から tenant_id を逆引き
    let project = project_repository::find_by_id(&pool, body.project_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        ))?;

    let consumer = consumer_repository::create(
        &pool,
        project.tenant_id,
        project.id,
        body.plan_id,
        body.external_id.as_deref(),
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create consumer"})),
        )
    })?;

    Ok((StatusCode::CREATED, Json(consumer)))
}
