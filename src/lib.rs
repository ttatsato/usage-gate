pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;

use axum::{middleware as axum_middleware, routing::{get, post}, Router};
use sqlx::PgPool;

use middleware::auth::auth;
use middleware::metering::metering;
use routes::health::health;
use routes::admin::tenants::{list_tenants, create_tenant};
use routes::admin::api_keys::{create_api_key, list_api_keys};
use routes::admin::usage::get_usage;
use routes::admin::consumers::{create_consumer};

pub fn create_router(pool: PgPool) -> Router {
    let protected_routes = Router::new()
        .route("/proxy/test", get(|| async { "ok" }))
        .route_layer(axum_middleware::from_fn_with_state(pool.clone(), metering))
        .route_layer(axum_middleware::from_fn_with_state(pool.clone(), auth));

    let public_routes = Router::new()
        .route("/health", get(health))
        .route("/admin/tenants", post(create_tenant).get(list_tenants))
        .route("/admin/api-keys", post(create_api_key).get(list_api_keys))
        .route("/admin/usage", get(get_usage))
        .route("/admin/consumers", post(create_consumer));

    public_routes
        .merge(protected_routes)
        .with_state(pool)
}
