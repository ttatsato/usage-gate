use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use sqlx::PgPool;
use serde_json::json;

pub async fn auth(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Unauthorized"})),
            ));
        }
    };

    let row = sqlx::query_as!(
        AuthedApiKey,
        r#"SELECT ak.id as api_key_id, ak.tenant_id, t.plan
        FROM api_keys ak
        JOIN tenants t ON t.id = ak.tenant_id
        WHERE ak.key = $1 AND ak.is_active = true"#,
        api_key,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Internal server error"})),
        )
    })?;

    match row {
        Some(_authed) => Ok(next.run(request).await),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )),
    }
}

#[derive(sqlx::FromRow)]
struct AuthedApiKey {
    pub api_key_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub plan: String,
}
