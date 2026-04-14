use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;
use serde_json::{json, Value};

// テスト用のアプリを作成する
async fn setup() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect");
    let app = usage_gate::create_router(pool.clone());
    (app, pool)
}

// レスポンスボディを JSON にパースする
async fn to_json(response: axum::response::Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

// --- Health ---

#[tokio::test]
async fn health_returns_ok() {
    let (app, _pool) = setup().await;

    let response = app
        .oneshot(Request::get("/health").body(axum::body::Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// --- Tenant ---

#[tokio::test]
async fn create_and_list_tenants() {
    let (app, pool) = setup().await;

    // トランザクションでテストデータを隔離
    let mut tx = pool.begin().await.unwrap();

    // テナント作成
    let response = app.clone()
        .oneshot(
            Request::post("/admin/tenants")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({"name": "test-tenant"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_json(response).await;
    assert_eq!(body["name"], "test-tenant");
    assert_eq!(body["plan"], "free");

    // ロールバック（テストデータを残さない）
    tx.rollback().await.unwrap();
}

// --- Auth Middleware ---

#[tokio::test]
async fn proxy_without_api_key_returns_401() {
    let (app, _pool) = setup().await;

    let response = app
        .oneshot(Request::get("/proxy/test").body(axum::body::Body::empty()).unwrap())
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

    // テスト用テナントと API キーを直接 DB に作成
    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name, plan) VALUES ($1, $2) RETURNING id"#,
        "test-tenant",
        "free",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // consumer を作成
    let consumer = sqlx::query!(
        r#"INSERT INTO consumers (tenant_id) VALUES ($1) RETURNING id"#,
        tenant.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let api_key = sqlx::query!(
        r#"INSERT INTO api_keys (tenant_id, consumer_id, key, name) VALUES ($1, $2, $3, $4) RETURNING key"#,
        tenant.id,
        consumer.id,
        "test-api-key-12345",
        "test-key",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::get("/proxy/test")
                .header("x-api-key", &api_key.key)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // テストデータを削除（外部キーの順番に注意: api_keys → consumers → tenants）
    sqlx::query!("DELETE FROM api_keys WHERE tenant_id = $1", tenant.id)
        .execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM consumers WHERE tenant_id = $1", tenant.id)
        .execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant.id)
        .execute(&pool).await.unwrap();
}

// --- Consumer ---

#[tokio::test]
async fn create_consumer() {
    let (app, pool) = setup().await;

    // テスト用テナントを作成
    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name, plan) VALUES ($1, $2) RETURNING id"#,
        "test-tenant-for-consumer",
        "free",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // consumer 作成リクエスト
    let response = app
        .oneshot(
            Request::post("/admin/consumers")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "tenant_id": tenant.id.to_string(),
                        "external_id": "user_12345"
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_json(response).await;
    assert_eq!(body["tenant_id"], tenant.id.to_string());
    assert_eq!(body["external_id"], "user_12345");

    // テストデータを削除
    sqlx::query!("DELETE FROM consumers WHERE tenant_id = $1", tenant.id)
        .execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant.id)
        .execute(&pool).await.unwrap();
}

#[tokio::test]
async fn create_consumer_without_external_id() {
    let (app, pool) = setup().await;

    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name, plan) VALUES ($1, $2) RETURNING id"#,
        "test-tenant-for-consumer-no-ext",
        "free",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // external_id を省略
    let response = app
        .oneshot(
            Request::post("/admin/consumers")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&json!({
                        "tenant_id": tenant.id.to_string()
                    })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_json(response).await;
    assert_eq!(body["external_id"], Value::Null);

    sqlx::query!("DELETE FROM consumers WHERE tenant_id = $1", tenant.id)
        .execute(&pool).await.unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant.id)
        .execute(&pool).await.unwrap();
}
