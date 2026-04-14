use sqlx::PgPool;
use uuid::Uuid;
use crate::models::api_key::{ApiKey, AuthedApiKey};

pub async fn find_active_api_key(pool: &PgPool, key: &str) -> Result<Option<AuthedApiKey>, sqlx::Error> {
    sqlx::query_as!(
        AuthedApiKey,
        r#"SELECT ak.id as api_key_id, ak.tenant_id, t.plan
        FROM api_keys ak
        JOIN tenants t ON t.id = ak.tenant_id
        WHERE ak.key = $1 AND ak.is_active = true"#,
        key,
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    tenant_id: Uuid,
    consumer_id: Uuid,
    key: &str,
    name: Option<&str>,
) -> Result<ApiKey, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"INSERT INTO api_keys (tenant_id, consumer_id, key, name) VALUES ($1, $2, $3, $4)
        RETURNING id, tenant_id, consumer_id, key, name, is_active,
                  created_at as "created_at!", updated_at as "updated_at!""#,
        tenant_id,
        consumer_id,
        key,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_all(pool: &PgPool) -> Result<Vec<ApiKey>, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"SELECT id, tenant_id, consumer_id, key, name, is_active,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM api_keys ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await
}
