use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

// テスト用のアプリを作成する
async fn setup() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect");
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
async fn create_and_list_tenants() {
    let (app, pool) = setup().await;

    // トランザクションでテストデータを隔離
    let mut tx = pool.begin().await.unwrap();

    // テナント作成
    let response = app
        .clone()
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

    // ロールバック（テストデータを残さない）
    tx.rollback().await.unwrap();
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

    // テスト用テナントと API キーを直接 DB に作成
    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name) VALUES ($1) RETURNING id"#,
        "test-tenant",
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

    // テスト用の平文キーとハッシュ
    let raw_key = "test-api-key-12345";
    let key_hash = usage_gate::utils::hash::hash_api_key(raw_key);
    let key_prefix = &raw_key[..8];

    sqlx::query!(
        r#"INSERT INTO api_keys (tenant_id, consumer_id, key_hash, key_prefix, name) VALUES ($1, $2, $3, $4, $5)"#,
        tenant.id,
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

    // テストデータを削除（外部キーの順番に注意: api_keys → consumers → tenants）
    sqlx::query!("DELETE FROM api_keys WHERE tenant_id = $1", tenant.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM consumers WHERE tenant_id = $1", tenant.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant.id)
        .execute(&pool)
        .await
        .unwrap();
}

// --- Consumer ---

#[tokio::test]
async fn create_consumer() {
    let (app, pool) = setup().await;

    // テスト用テナントを作成
    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name) VALUES ($1) RETURNING id"#,
        "test-tenant-for-consumer",
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
                    }))
                    .unwrap(),
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
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant.id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn create_consumer_without_external_id() {
    let (app, pool) = setup().await;

    let tenant = sqlx::query!(
        r#"INSERT INTO tenants (name) VALUES ($1) RETURNING id"#,
        "test-tenant-for-consumer-no-ext",
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

    sqlx::query!("DELETE FROM consumers WHERE tenant_id = $1", tenant.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant.id)
        .execute(&pool)
        .await
        .unwrap();
}
