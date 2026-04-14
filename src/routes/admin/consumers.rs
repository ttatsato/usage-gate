use crate::models::consumer::{Consumer, CreateConsumer};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use sqlx::PgPool;

pub async fn create_consumer(
    State(pool): State<PgPool>,
    Json(body): Json<CreateConsumer>,
) -> (StatusCode, Json<Consumer>) {
    let consumer = sqlx::query_as::<_, Consumer>(
        "INSERT INTO consumers (tenant_id, external_id) VALUES ($1, $2) RETURNING *",
    )
    .bind(body.tenant_id)
    .bind(body.external_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(consumer))
}
