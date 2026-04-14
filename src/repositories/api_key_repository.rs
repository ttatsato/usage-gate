use crate::models::api_key::{ApiKey, AuthedApiKey};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_active_by_key_hash(
    pool: &PgPool,
    key_hash: &str,
) -> Result<Option<AuthedApiKey>, sqlx::Error> {
    sqlx::query_as!(
        AuthedApiKey,
        r#"SELECT
            ak.id as api_key_id,
            ak.tenant_id,
            ak.project_id,
            ak.consumer_id,
            p.id as "plan_id?",
            p.name as "plan_name?",
            p.monthly_request_quota
        FROM api_keys ak
        JOIN consumers c ON c.id = ak.consumer_id
        LEFT JOIN plans p ON p.id = c.plan_id
        WHERE ak.key_hash = $1 AND ak.is_active = true"#,
        key_hash,
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    tenant_id: Uuid,
    project_id: Uuid,
    consumer_id: Uuid,
    key_hash: &str,
    key_prefix: &str,
    name: Option<&str>,
) -> Result<ApiKey, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"INSERT INTO api_keys (tenant_id, project_id, consumer_id, key_hash, key_prefix, name)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, tenant_id, project_id, consumer_id, key_prefix, name, is_active,
                  created_at as "created_at!", updated_at as "updated_at!""#,
        tenant_id,
        project_id,
        consumer_id,
        key_hash,
        key_prefix,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<ApiKey>, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"SELECT id, tenant_id, project_id, consumer_id, key_prefix, name, is_active,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM api_keys ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await
}
