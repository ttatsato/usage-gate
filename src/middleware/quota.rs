use crate::adapters::quota_counter::{QuotaCounter, QuotaPeriod};
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
    State(counter): State<Arc<dyn QuotaCounter>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let authed = request.extensions().get::<AuthedApiKey>().cloned();
    if let Some(authed) = authed {
        if let Some(quota) = authed.monthly_request_quota {
            let current = counter
                .get_count(authed.consumer_id, &QuotaPeriod::Monthly)
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
