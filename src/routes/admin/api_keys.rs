use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::api_key::{CreateApiKey, ApiKey};

pub async fn create_api_key(
    State(pool): State<PgPool>,
    Json(body): Json<CreateApiKey>,
) -> (StatusCode, Json<ApiKey>) {
    let raw_key = Uuid::new_v4().to_string().replace("-", "");
    // TODO: prefixはつけない？
    let api_key = sqlx::query_as!(
        ApiKey,
        r#"INSERT INTO api_keys (key, name, tenant_id) VALUES ($1, $2, $3)
        RETURNING id, tenant_id, key, name, is_active, created_at as "created_at!", updated_at as "updated_at!""#,
        raw_key,
        body.name,
        body.tenant_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create api key");

    (StatusCode::CREATED, Json(api_key))

}

pub async fn list_api_keys(
    State(pool): State<PgPool>,
) -> Json<Vec<ApiKey>> {
    let api_keys = sqlx::query_as!(
        ApiKey,
        r#"SELECT id, tenant_id, key, name, is_active, created_at as "created_at!", updated_at as "updated_at!" FROM api_keys ORDER BY created_at DESC"#,
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to list api keys");

    Json(api_keys)
}
