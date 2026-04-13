use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;
use crate::models::usage_record::{UsageQuery, UsageResponse};
use crate::repositories::usage_repository;

// GET /admin/usage?tenant_id=xxx&start_date=...&end_date=...
pub async fn get_usage(
    State(pool): State<PgPool>,
    Query(query): Query<UsageQuery>,
) -> Json<UsageResponse> {
    let records = usage_repository::get_usage_summary(
        &pool, query.tenant_id, query.start_date, query.end_date,
    )
    .await
    .expect("Failed to get usage summary");

    let total_requests = usage_repository::get_total_requests(
        &pool, query.tenant_id, query.start_date, query.end_date,
    )
    .await
    .expect("Failed to get total requests");

    Json(UsageResponse {
        tenant_id: query.tenant_id,
        total_requests,
        records,
    })
}
