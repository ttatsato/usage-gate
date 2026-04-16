use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use usage_gate::adapters::quota_counter::valkey::ValkeyQuotaCounter;
use usage_gate::create_router;
use usage_gate::routes::system::quota_sync::do_sync_to_db;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let url = std::env::var("QUOTA_COUNTER_URL").expect("QUOTA_COUNTER_URL not set");
    let quota_counter = match std::env::var("QUOTA_COUNTER").as_deref() {
        Ok("valkey") => Arc::new(
            ValkeyQuotaCounter::new(&url)
                .await
                .expect("Failed to connect to Valkey"),
        ),
        _ => {
            panic!("QUOTA_COUNTER must be set (supported: valkey)")
        }
    };

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("sync-to-db") => {
            tracing::info!("Starting Valkey → DB sync");
            match do_sync_to_db(&pool, &*quota_counter).await {
                Ok(count) => tracing::info!("Synced {} consumers from Valkey to DB", count),
                Err(e) => tracing::error!("Sync failed: {}", e),
            }
            return;
        }
        Some(cmd) => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Usage: usage-gate [sync-to-db]");
            std::process::exit(1);
        }
        None => {}
    }

    // 定期バッチ: 1時間ごとに Valkey → DB 同期
    {
        let pool = pool.clone();
        let counter = quota_counter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                tracing::info!("Starting periodic quota sync to DB");
                match do_sync_to_db(&pool, &*counter).await {
                    Ok(count) => tracing::info!("Quota sync completed: {} consumers synced", count),
                    Err(e) => tracing::error!("Quota sync failed: {}", e),
                }
            }
        });
    }

    let port = std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());
    let app = create_router(pool, quota_counter);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &port)
        .await
        .expect("Failed to bind port");

    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await.expect("Failed to serve");
}
