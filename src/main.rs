use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use usage_gate::adapters::auth_cache::valkey::ValkeyAuthCache;
use usage_gate::adapters::rate_limiter::valkey::ValkeyRateLimiter;
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

    let url = std::env::var("RATE_LIMITER_URL").expect("RATE_LIMITER_URL not set");
    let rate_limiter = match std::env::var("RATE_LIMITER").as_deref() {
        Ok("valkey") => Arc::new(
            ValkeyRateLimiter::new(&url)
                .await
                .expect("Failed to connect to Valkey"),
        ),
        _ => {
            panic!("RATE_LIMITER must be set (supported: valkey)")
        }
    };

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("sync-to-db") => {
            tracing::info!("Starting Valkey → DB sync");
            match do_sync_to_db(&pool, &*rate_limiter).await {
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

    let auth_cache_ttl_secs =
        std::env::var("AUTH_CACHE_TTL_SECS").expect("AUTH_CACHE_TTL_SECS not set");
    let ttl_seconds: u64 = auth_cache_ttl_secs
        .parse()
        .expect("AUTH_CACHE_TTL_SECS must be a valid number");
    let valkey_auth_cache = Arc::new(
        ValkeyAuthCache::new(&url)
            .await
            .expect("Failed to connect to Valkey"),
    );

    // 定期バッチ: 1時間ごとに Valkey → DB 同期
    {
        let pool = pool.clone();
        let limiter = rate_limiter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                tracing::info!("Starting periodic quota sync to DB");
                match do_sync_to_db(&pool, &*limiter).await {
                    Ok(count) => tracing::info!("Quota sync completed: {} consumers synced", count),
                    Err(e) => tracing::error!("Quota sync failed: {}", e),
                }
            }
        });
    }

    let port = std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string());
    let app = create_router(pool, rate_limiter, valkey_auth_cache, ttl_seconds);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &port)
        .await
        .expect("Failed to bind port");

    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await.expect("Failed to serve");
}
