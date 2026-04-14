use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

async fn setup() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect");
    let app = usage_gate::create_router(pool.clone());
    (app, pool)
}

async fn to_json(response: axum::response::Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

// テナント + プロジェクトを作る共通ヘルパー
async fn create_tenant_and_project(pool: &PgPool, tenant_name: &str) -> (uuid::Uuid, uuid::Uuid) {
    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name) VALUES ($1) RETURNING id"#,
        tenant_name,
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let project = sqlx::query!(
        r#"INSERT INTO projects (tenant_id, name) VALUES ($1, $2) RETURNING id"#,
        tenant.id,
        "default",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (tenant.id, project.id)
}

async fn cleanup(pool: &PgPool, tenant_id: uuid::Uuid) {
    sqlx::query!("DELETE FROM usage_records WHERE tenant_id = $1", tenant_id)
        .execute(pool).await.unwrap();
    sqlx::query!("DELETE FROM api_keys WHERE tenant_id = $1", tenant_id)
        .execute(pool).await.unwrap();
    sqlx::query!("DELETE FROM consumers WHERE tenant_id = $1", tenant_id)
        .execute(pool).await.unwrap();
    sqlx::query!(
        "DELETE FROM plans WHERE project_id IN (SELECT id FROM projects WHERE tenant_id = $1)",
        tenant_id,
    )
    .execute(pool).await.unwrap();
    sqlx::query!(
        "DELETE FROM upstream_services WHERE project_id IN (SELECT id FROM projects WHERE tenant_id = $1)",
        tenant_id,
    )
    .execute(pool).await.unwrap();
    sqlx::query!("DELETE FROM projects WHERE tenant_id = $1", tenant_id)
        .execute(pool).await.unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(pool).await.unwrap();
}

// --- Health ---

#[tokio::test]
async fn health_returns_ok() {
    let (app, _pool) = setup().await;

    let response = app
        .oneshot(
            Request::get("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// --- Tenant ---

#[tokio::test]
async fn create_tenant_returns_tenant() {
    let (app, _pool) = setup().await;

    let response = app
        .oneshot(
            Request::post("/admin/tenants")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({"name": "test-tenant-basic"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_json(response).await;
    assert_eq!(body["name"], "test-tenant-basic");
}

// --- Auth Middleware ---

#[tokio::test]
async fn proxy_without_api_key_returns_401() {
    let (app, _pool) = setup().await;

    let response = app
        .oneshot(
            Request::get("/proxy/test")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn proxy_with_invalid_api_key_returns_401() {
    let (app, _pool) = setup().await;

    let response = app
        .oneshot(
            Request::get("/proxy/test")
                .header("x-api-key", "invalid-key")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn proxy_with_valid_api_key_returns_200() {
    let (app, pool) = setup().await;

    let (tenant_id, project_id) = create_tenant_and_project(&pool, "test-tenant-auth").await;

    let consumer = sqlx::query!(
        r#"INSERT INTO consumers (tenant_id, project_id) VALUES ($1, $2) RETURNING id"#,
        tenant_id,
        project_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let raw_key = "test-api-key-12345";
    let key_hash = usage_gate::utils::hash::hash_api_key(raw_key);
    let key_prefix = &raw_key[..8];

    sqlx::query!(
        r#"INSERT INTO api_keys (tenant_id, project_id, consumer_id, key_hash, key_prefix, name) VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        project_id,
        consumer.id,
        key_hash,
        key_prefix,
        "test-key",
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::get("/proxy/test")
                .header("x-api-key", raw_key)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    cleanup(&pool, tenant_id).await;
}

// --- Consumer ---

#[tokio::test]
async fn create_consumer_basic() {
    let (app, pool) = setup().await;
    let (tenant_id, project_id) = create_tenant_and_project(&pool, "test-tenant-consumer").await;

    let response = app
        .oneshot(
            Request::post("/admin/consumers")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "project_id": project_id.to_string(),
                        "external_id": "user_12345"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_json(response).await;
    assert_eq!(body["project_id"], project_id.to_string());
    assert_eq!(body["tenant_id"], tenant_id.to_string());
    assert_eq!(body["external_id"], "user_12345");

    cleanup(&pool, tenant_id).await;
}

#[tokio::test]
async fn create_consumer_without_external_id() {
    let (app, pool) = setup().await;
    let (tenant_id, project_id) = create_tenant_and_project(&pool, "test-tenant-consumer-no-ext").await;

    let response = app
        .oneshot(
            Request::post("/admin/consumers")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "project_id": project_id.to_string()
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_json(response).await;
    assert_eq!(body["external_id"], Value::Null);

    cleanup(&pool, tenant_id).await;
}

// --- Quota Middleware ---

#[tokio::test]
async fn proxy_returns_403_when_monthly_quota_exceeded() {
    let (app, pool) = setup().await;
    let (tenant_id, project_id) = create_tenant_and_project(&pool, "test-tenant-quota").await;

    let plan = sqlx::query!(
        r#"INSERT INTO plans (project_id, name, monthly_request_quota) VALUES ($1, $2, $3) RETURNING id"#,
        project_id,
        "quota-test-plan",
        2i32,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let consumer = sqlx::query!(
        r#"INSERT INTO consumers (tenant_id, project_id, plan_id) VALUES ($1, $2, $3) RETURNING id"#,
        tenant_id,
        project_id,
        plan.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let raw_key = "quota-test-key";
    let key_hash = usage_gate::utils::hash::hash_api_key(raw_key);
    let key_prefix = &raw_key[..8];

    let api_key = sqlx::query!(
        r#"INSERT INTO api_keys (tenant_id, project_id, consumer_id, key_hash, key_prefix, name) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"#,
        tenant_id,
        project_id,
        consumer.id,
        key_hash,
        key_prefix,
        "quota-test",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 上限分の使用量を直接 INSERT
    for _ in 0..2 {
        sqlx::query!(
            r#"INSERT INTO usage_records (tenant_id, project_id, consumer_id, api_key_id, endpoint, method, status_code)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            tenant_id,
            project_id,
            consumer.id,
            api_key.id,
            "/proxy/test",
            "GET",
            200i16,
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    let response = app
        .oneshot(
            Request::get("/proxy/test")
                .header("x-api-key", raw_key)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup(&pool, tenant_id).await;
}

#[tokio::test]
async fn proxy_passes_when_under_quota() {
    let (app, pool) = setup().await;
    let (tenant_id, project_id) = create_tenant_and_project(&pool, "test-tenant-under-quota").await;

    let plan = sqlx::query!(
        r#"INSERT INTO plans (project_id, name, monthly_request_quota) VALUES ($1, $2, $3) RETURNING id"#,
        project_id,
        "under-quota-plan",
        100i32,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let consumer = sqlx::query!(
        r#"INSERT INTO consumers (tenant_id, project_id, plan_id) VALUES ($1, $2, $3) RETURNING id"#,
        tenant_id,
        project_id,
        plan.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let raw_key = "under-quota-key";
    let key_hash = usage_gate::utils::hash::hash_api_key(raw_key);
    let key_prefix = &raw_key[..8];

    sqlx::query!(
        r#"INSERT INTO api_keys (tenant_id, project_id, consumer_id, key_hash, key_prefix, name) VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        project_id,
        consumer.id,
        key_hash,
        key_prefix,
        "under-quota",
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::get("/proxy/test")
                .header("x-api-key", raw_key)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    cleanup(&pool, tenant_id).await;
}
