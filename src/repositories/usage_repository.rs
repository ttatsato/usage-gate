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
