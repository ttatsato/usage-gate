use crate::models::project::Project;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(pool: &PgPool, tenant_id: Uuid, name: &str) -> Result<Project, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"INSERT INTO projects (tenant_id, name) VALUES ($1, $2)
        RETURNING id, tenant_id, name,
                  created_at as "created_at!", updated_at as "updated_at!""#,
        tenant_id,
        name,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"SELECT id, tenant_id, name,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM projects WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_by_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as!(
        Project,
        r#"SELECT id, tenant_id, name,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM projects WHERE tenant_id = $1 ORDER BY created_at DESC"#,
        tenant_id,
    )
    .fetch_all(pool)
    .await
}
