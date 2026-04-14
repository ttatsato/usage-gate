use sqlx::PgPool;
use uuid::Uuid;
use crate::models::consumer::Consumer;

pub async fn find_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Consumer>, sqlx::Error> {
    sqlx::query_as!(
        Consumer,
        r#"SELECT id, tenant_id, external_id,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM consumers WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}
