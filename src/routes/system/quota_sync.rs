use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use crate::adapters::quota_counter::QuotaCounter;

/// Valkey → DB: カウンターの値を DB に永続化（請求用）
pub async fn sync_to_db(
    State((pool, counter)): State<(PgPool, Arc<dyn QuotaCounter>)>,
) -> impl IntoResponse {
    match do_sync_to_db(&pool, &*counter).await {
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

/// DB → Valkey: DB のカウンター値を Valkey に復元（手動復旧用）
pub async fn sync_from_db(
    State((pool, counter)): State<(PgPool, Arc<dyn QuotaCounter>)>,
) -> impl IntoResponse {
    match do_sync_from_db(&pool, &*counter).await {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({"status": "ok", "restored": count})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "message": e})),
        ),
    }
}

/// 定期バッチから呼ばれる sync-to-db の本体
pub async fn do_sync_to_db(pool: &PgPool, counter: &dyn QuotaCounter) -> Result<i64, String> {
    use crate::adapters::quota_counter::QuotaPeriod;

    // 全 consumer を取得
    let consumers = sqlx::query!(r#"SELECT id FROM consumers"#)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut synced = 0i64;
    let period = QuotaPeriod::Monthly;

    for consumer in &consumers {
        let count = counter
            .get_count(consumer.id, &period)
            .await
            .map_err(|e| format!("{:?}", e))?;

        if count > 0 {
            let period_str = period.to_key_suffix();
            sqlx::query!(
                r#"INSERT INTO quota_counters (consumer_id, period, count, synced_at)
                VALUES ($1, $2, $3, now())
                ON CONFLICT (consumer_id, period)
                DO UPDATE SET count = $3, synced_at = now()"#,
                consumer.id,
                period_str,
                count,
            )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            synced += 1;
        }
    }

    Ok(synced)
}

/// 手動復旧用の sync-from-db の本体
pub async fn do_sync_from_db(pool: &PgPool, counter: &dyn QuotaCounter) -> Result<i64, String> {
    use crate::adapters::quota_counter::QuotaPeriod;

    let rows = sqlx::query!(
        r#"SELECT consumer_id, period, count FROM quota_counters"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut restored = 0i64;

    for row in &rows {
        let period = QuotaPeriod::from_key_suffix(&row.period)
            .ok_or_else(|| format!("Unknown period: {}", row.period))?;

        counter
            .restore(row.consumer_id, &period, row.count)
            .await
            .map_err(|e| format!("{:?}", e))?;

        restored += 1;
    }

    Ok(restored)
}
