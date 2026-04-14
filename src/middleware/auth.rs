use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use sqlx::PgPool;
use serde_json::json;
use crate::repositories::api_key_repository;
use crate::utils::hash::hash_api_key;

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

    // 受け取ったキーをハッシュ化して DB と照合
    let key_hash = hash_api_key(api_key);
    let row = api_key_repository::find_active_by_key_hash(&pool, &key_hash)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal server error"})),
            )
        })?;

    match row {
        Some(authed) => {
            // リクエストの extensions にテナント情報を添付
            // 後続のミドルウェアやハンドラから取り出せる
            let mut request = request;
            request.extensions_mut().insert(authed);
            Ok(next.run(request).await)
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )),
    }
}
