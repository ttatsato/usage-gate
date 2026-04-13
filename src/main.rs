
use axum::{Json, routing::get, Router};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;


#[derive(Serialize)]
struct HealthResponse {
 status: String,
}

async fn health () -> Json<HealthResponse> {
    Json(HealthResponse{
        status: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
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
    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &port).await.expect("Failed to bind port");

    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await.expect("Failed to serve");
}
