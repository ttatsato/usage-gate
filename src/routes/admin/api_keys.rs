use crate::models::api_key::{ApiKey, CreateApiKey, CreatedApiKey};
use crate::repositories::{api_key_repository, consumer_repository};
use crate::utils::hash::hash_api_key;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_api_key(
    State(pool): State<PgPool>,
    Json(body): Json<CreateApiKey>,
) -> Result<(StatusCode, Json<CreatedApiKey>), (StatusCode, Json<serde_json::Value>)> {
    let consumer = consumer_repository::find_by_id(&pool, body.consumer_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Consumer not found"})),
        ))?;

    // 平文キーを生成し、ハッシュと prefix を計算
    let raw_key = Uuid::new_v4().to_string().replace("-", "");
    let key_hash = hash_api_key(&raw_key);
    let key_prefix = raw_key.chars().take(8).collect::<String>();

    let api_key = api_key_repository::create(
        &pool,
        consumer.tenant_id,
        consumer.id,
        &key_hash,
        &key_prefix,
        body.name.as_deref(),
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create api key"})),
        )
    })?;

    // 発行時のみ平文キーを含めてレスポンス
    let created = CreatedApiKey {
        id: api_key.id,
        tenant_id: api_key.tenant_id,
        consumer_id: api_key.consumer_id,
        key: raw_key,
        key_prefix: api_key.key_prefix,
        name: api_key.name,
        is_active: api_key.is_active,
        created_at: api_key.created_at,
        updated_at: api_key.updated_at,
    };

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list_api_keys(State(pool): State<PgPool>) -> Json<Vec<ApiKey>> {
    let api_keys = api_key_repository::list_all(&pool)
        .await
        .expect("Failed to list api keys");

    Json(api_keys)
}
