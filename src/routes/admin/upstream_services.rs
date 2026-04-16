use crate::models::upstream_service::{CreateUpstreamService, UpstreamService};
use crate::repositories::upstream_service_repository;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_upstream_service(
    State(pool): State<PgPool>,
    Json(body): Json<CreateUpstreamService>,
) -> Result<(StatusCode, Json<UpstreamService>), (StatusCode, Json<serde_json::Value>)> {
    // 簡易 SSRF 対策: http/https のみ許可
    if !body.base_url.starts_with("http://") && !body.base_url.starts_with("https://") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "base_url must start with http:// or https://"})),
        ));
    }

    let service =
        upstream_service_repository::create(&pool, body.project_id, &body.name, &body.base_url)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create upstream service"})),
                )
            })?;
    Ok((StatusCode::CREATED, Json(service)))
}

#[derive(Deserialize)]
pub struct ListUpstreamServicesQuery {
    pub project_id: Uuid,
}

pub async fn list_upstream_services(
    State(pool): State<PgPool>,
    Query(q): Query<ListUpstreamServicesQuery>,
) -> Json<Vec<UpstreamService>> {
    let services = upstream_service_repository::list_by_project(&pool, q.project_id)
        .await
        .expect("Failed to list upstream services");
    Json(services)
}
