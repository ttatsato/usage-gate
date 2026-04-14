use crate::models::usage_record::UsageSummary;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// 使用量を1件記録する
pub async fn record_usage(
    pool: &PgPool,
    tenant_id: Uuid,
    api_key_id: Uuid,
    endpoint: &str,
    method: &str,
    status_code: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO usage_records (tenant_id, api_key_id, endpoint, method, status_code)
        VALUES ($1, $2, $3, $4, $5)"#,
        tenant_id,
        api_key_id,
        endpoint,
        method,
        status_code,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// テナントの使用量をエンドポイント別に集計する
pub async fn get_usage_summary(
    pool: &PgPool,
    tenant_id: Uuid,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<UsageSummary>, sqlx::Error> {
    sqlx::query_as!(
        UsageSummary,
        r#"SELECT endpoint, method, COUNT(*) as "request_count!"
        FROM usage_records
        WHERE tenant_id = $1
          AND ($2::timestamptz IS NULL OR created_at >= $2)
          AND ($3::timestamptz IS NULL OR created_at <= $3)
        GROUP BY endpoint, method
        ORDER BY COUNT(*) DESC"#,
        tenant_id,
        start_date,
        end_date,
    )
    .fetch_all(pool)
    .await
}

// テナントの合計リクエスト数を取得する
pub async fn get_total_requests(
    pool: &PgPool,
    tenant_id: Uuid,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT COUNT(*) as "count!"
        FROM usage_records
        WHERE tenant_id = $1
          AND ($2::timestamptz IS NULL OR created_at >= $2)
          AND ($3::timestamptz IS NULL OR created_at <= $3)"#,
        tenant_id,
        start_date,
        end_date,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.count)
}
