use crate::models::plan::Plan;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    project_id: Uuid,
    name: &str,
    monthly_request_quota: Option<i32>,
) -> Result<Plan, sqlx::Error> {
    sqlx::query_as!(
        Plan,
        r#"INSERT INTO plans (project_id, name, monthly_request_quota) VALUES ($1, $2, $3)
        RETURNING id, project_id, name, monthly_request_quota,
                  created_at as "created_at!", updated_at as "updated_at!""#,
        project_id,
        name,
        monthly_request_quota,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_by_project(pool: &PgPool, project_id: Uuid) -> Result<Vec<Plan>, sqlx::Error> {
    sqlx::query_as!(
        Plan,
        r#"SELECT id, project_id, name, monthly_request_quota,
           created_at as "created_at!", updated_at as "updated_at!"
           FROM plans WHERE project_id = $1 ORDER BY created_at DESC"#,
        project_id,
    )
    .fetch_all(pool)
    .await
}
