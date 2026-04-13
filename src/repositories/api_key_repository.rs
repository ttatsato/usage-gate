use sqlx::PgPool;
use crate::models::api_key::AuthedApiKey;

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
