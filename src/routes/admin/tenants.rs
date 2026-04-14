use crate::models::tenant::{CreateTenant, Tenant};
use axum::Json;
use axum::extract::State;
use sqlx::PgPool;

pub async fn create_tenant(
    State(pool): State<PgPool>,
    Json(body): Json<CreateTenant>,
) -> Json<Tenant> {
    let plan = body.plan.unwrap_or_else(|| "free".to_string());
    let tenant = sqlx::query_as!(
        Tenant,
        "INSERT INTO tenants (name, plan) VALUES ($1, $2) RETURNING *",
        &body.name,
        &plan,
    )
    .fetch_one(&pool)
    .await
    .expect("Faild to create tenant");

    Json(tenant)
}

pub async fn list_tenants(State(pool): State<PgPool>) -> Json<Vec<Tenant>> {
    let tenants = sqlx::query_as::<_, Tenant>(r#"SELECT * FROM tenants ORDER BY created_at DESC"#)
        .fetch_all(&pool)
        .await
        .expect("Failed to list tenants");
    Json(tenants)
}
