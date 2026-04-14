use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::api_key::{CreateApiKey, ApiKey};
use crate::repositories::{api_key_repository, consumer_repository};

pub async fn create_api_key(
    State(pool): State<PgPool>,
    Json(body): Json<CreateApiKey>,
) -> Result<(StatusCode, Json<ApiKey>), (StatusCode, Json<serde_json::Value>)> {
    // consumer_id から tenant_id を逆引き
    let consumer = consumer_repository::find_by_id(&pool, body.consumer_id)
        .await
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal server error"})),
        ))?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Consumer not found"})),
        ))?;

    let raw_key = Uuid::new_v4().to_string().replace("-", "");

    let api_key = api_key_repository::create(
        &pool,
        consumer.tenant_id,
        consumer.id,
        &raw_key,
        body.name.as_deref(),
    )
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": "Failed to create api key"})),
    ))?;

    Ok((StatusCode::CREATED, Json(api_key)))
}

pub async fn list_api_keys(
    State(pool): State<PgPool>,
) -> Json<Vec<ApiKey>> {
    let api_keys = api_key_repository::list_all(&pool)
        .await
        .expect("Failed to list api keys");

    Json(api_keys)
}
