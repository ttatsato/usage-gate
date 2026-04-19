use axum::{Router, response::Json, routing::any};
use serde_json::json;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("MOCK_UPSTREAM_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    let app = Router::new().fallback(any(|| async { Json(json!({"ok": true})) }));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("mock upstream listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
