use crate::models::consumer::Consumer;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Consumer>, sqlx::Error> {
    sqlx::query_as!(
        Consumer,
        r#"SELECT id, tenant_id, project_id, plan_id, external_id,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM consumers WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    tenant_id: Uuid,
    project_id: Uuid,
    plan_id: Option<Uuid>,
    external_id: Option<&str>,
) -> Result<Consumer, sqlx::Error> {
    sqlx::query_as!(
        Consumer,
        r#"INSERT INTO consumers (tenant_id, project_id, plan_id, external_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, tenant_id, project_id, plan_id, external_id,
                  created_at as "created_at!", updated_at as "updated_at!""#,
        tenant_id,
        project_id,
        plan_id,
        external_id,
    )
    .fetch_one(pool)
    .await
}
