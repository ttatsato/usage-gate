use crate::adapters::rate_limiter::RateLimiter;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

/// Valkey → DB: 使用量を DB に永続化（請求用）
pub async fn sync_to_db(
    State((pool, limiter)): State<(PgPool, Arc<dyn RateLimiter>)>,
) -> impl IntoResponse {
    match do_sync_to_db(&pool, &*limiter).await {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({"status": "ok", "synced": count})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": e})),
        ),
    }
}

/// 定期バッチから呼ばれる sync-to-db の本体
pub async fn do_sync_to_db(pool: &PgPool, limiter: &dyn RateLimiter) -> Result<i64, String> {
    use crate::adapters::rate_limiter::RateLimitPeriod;

    // consumer + plan 情報を取得
    let rows = sqlx::query!(
        r#"SELECT c.id as "consumer_id!", p.monthly_request_quota
        FROM consumers c
        LEFT JOIN plans p ON p.id = c.plan_id"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut synced = 0i64;

    for row in &rows {
        if let Some(max_requests) = row.monthly_request_quota {
            let usage = limiter
                .get_usage(
                    row.consumer_id,
                    &RateLimitPeriod::Monthly,
                    max_requests as i64,
                )
                .await
                .map_err(|e| format!("{:?}", e))?;

            if usage > 0 {
                let period_str = RateLimitPeriod::Monthly.to_key_suffix();
                sqlx::query!(
                    r#"INSERT INTO quota_counters (consumer_id, period, count, synced_at)
                    VALUES ($1, $2, $3, now())
                    ON CONFLICT (consumer_id, period)
                    DO UPDATE SET count = $3, synced_at = now()"#,
                    row.consumer_id,
                    period_str,
                    usage,
                )
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

                synced += 1;
            }
        }
    }

    Ok(synced)
}
