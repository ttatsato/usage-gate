use crate::models::api_key::AuthedApiKey;
use crate::repositories::usage_repository;
use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub async fn quota(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let authed = request.extensions().get::<AuthedApiKey>().cloned();
    if let Some(authed) = authed {
        // NOTE: 課金の期間やトークンなのか回数なのかで分岐が増えていく。
        // 全てのカラムがnullの時はチェックは通らない
        if let Some(quota) = authed.monthly_request_quota {
            // NOTE: 月間使用量
            let current = usage_repository::count_current_month_requests(&pool, authed.consumer_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Internal server error"})),
                    )
                })?;

            if current >= quota as i64 {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Monthly quota exceeded"})),
                ));
            }
        }
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized" })),
        ));
    }

    Ok(next.run(request).await)
}
