
use axum::{middleware as axum_middleware, routing::{get, post}, Router};
use sqlx::postgres::PgPoolOptions;

mod middleware;
mod models;
mod repositories;
mod routes;

use middleware::auth::auth;
use middleware::metering::metering;
use routes::health::health;
use routes::admin::tenants::{list_tenants, create_tenant};
use routes::admin::api_keys::{create_api_key, list_api_keys};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        // NOTE: プール数目安 = CPU コア数 × 2 + 1
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // with_env_filterは出力するログステータスを制御
    // NOTE: RUST_LOGという環境変数で設定可能
    tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
    )
    .init();

    let port = std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());

    // 認証が必要なルート（プロキシなど、今後ここに追加）
    // ミドルウェアの実行順: auth → metering → ハンドラ → metering(記録) → レスポンス
    // route_layer は下から順に適用されるので、auth を後に書く
    let protected_routes = Router::new()
        .route("/proxy/test", get(|| async { "ok" }))
        .route_layer(axum_middleware::from_fn_with_state(pool.clone(), metering))
        .route_layer(axum_middleware::from_fn_with_state(pool.clone(), auth));

    // 認証不要なルート
    let public_routes = Router::new()
        .route("/health", get(health))
        .route("/admin/tenants", post(create_tenant).get(list_tenants))
        .route("/admin/api-keys", post(create_api_key).get(list_api_keys));

    let app = public_routes
        .merge(protected_routes)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &port).await.expect("Failed to bind port");

    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await.expect("Failed to serve");
}
