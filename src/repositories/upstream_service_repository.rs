use crate::models::upstream_service::UpstreamService;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    project_id: Uuid,
    name: &str,
    base_url: &str,
) -> Result<UpstreamService, sqlx::Error> {
    sqlx::query_as!(
        UpstreamService,
        r#"INSERT INTO upstream_services (project_id, name, base_url) VALUES ($1, $2, $3)
        RETURNING id, project_id, name, base_url,
                  created_at as "created_at!", updated_at as "updated_at!""#,
        project_id,
        name,
        base_url,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_project_and_name(
    pool: &PgPool,
    project_id: Uuid,
    name: &str,
) -> Result<Option<UpstreamService>, sqlx::Error> {
    sqlx::query_as!(
        UpstreamService,
        r#"SELECT id, project_id, name, base_url,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM upstream_services
           WHERE project_id = $1 AND name = $2"#,
        project_id,
        name,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_by_project(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<UpstreamService>, sqlx::Error> {
    sqlx::query_as!(
        UpstreamService,
        r#"SELECT id, project_id, name, base_url,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM upstream_services WHERE project_id = $1 ORDER BY created_at DESC"#,
        project_id,
    )
    .fetch_all(pool)
    .await
}
