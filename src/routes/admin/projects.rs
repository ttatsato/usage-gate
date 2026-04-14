use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::project::{CreateProject, Project};
use crate::repositories::project_repository;

pub async fn create_project(
    State(pool): State<PgPool>,
    Json(body): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, Json<serde_json::Value>)> {
    let project = project_repository::create(&pool, body.tenant_id, &body.name)
        .await
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create project"})),
        ))?;
    Ok((StatusCode::CREATED, Json(project)))
}

#[derive(Deserialize)]
pub struct ListProjectsQuery {
    pub tenant_id: Uuid,
}

pub async fn list_projects(
    State(pool): State<PgPool>,
    Query(q): Query<ListProjectsQuery>,
) -> Json<Vec<Project>> {
    let projects = project_repository::list_by_tenant(&pool, q.tenant_id)
        .await
        .expect("Failed to list projects");
    Json(projects)
}
