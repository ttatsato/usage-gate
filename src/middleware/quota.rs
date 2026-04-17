use crate::adapters::rate_limiter::{RateLimit, RateLimitPeriod, RateLimiter};
use crate::models::api_key::AuthedApiKey;
use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde_json::json;
use std::sync::Arc;

pub async fn quota(
    State(limiter): State<Arc<dyn RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let authed = request.extensions().get::<AuthedApiKey>().cloned();
    if let Some(authed) = authed {
        // プランに設定された制限を収集
        let mut limits = Vec::new();
        if let Some(v) = authed.monthly_request_quota {
            limits.push(RateLimit { period: RateLimitPeriod::Monthly, max_requests: v as i64 });
        }
        if let Some(v) = authed.daily_request_quota {
            limits.push(RateLimit { period: RateLimitPeriod::Daily, max_requests: v as i64 });
        }
        if let Some(v) = authed.hourly_request_quota {
            limits.push(RateLimit { period: RateLimitPeriod::Hourly, max_requests: v as i64 });
        }
        // 秒間制限: 未設定の場合はデフォルト 10 req/sec でバースト防止
        let per_second = authed.per_second_request_limit.unwrap_or(10);
        limits.push(RateLimit { period: RateLimitPeriod::PerSecond, max_requests: per_second as i64 });

        if !limits.is_empty() {
            let allowed = limiter
                .try_acquire(authed.consumer_id, &limits)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Internal server error"})),
                    )
                })?;

            if !allowed {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "Rate limit exceeded"})),
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
