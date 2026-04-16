pub mod adapters;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod utils;

use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};
use sqlx::PgPool;

use adapters::quota_counter::QuotaCounter;
use middleware::auth::auth;
use middleware::metering::metering;
use middleware::quota::quota;
use routes::admin::api_keys::{create_api_key, list_api_keys};
use routes::admin::consumers::create_consumer;
use routes::admin::plans::{create_plan, list_plans};
use routes::admin::projects::{create_project, list_projects};
use routes::admin::tenants::{create_tenant, list_tenants};
use routes::admin::upstream_services::{create_upstream_service, list_upstream_services};
use routes::admin::usage::get_usage;
use routes::health::health;
use routes::proxy::proxy;
use routes::system::quota_sync::{sync_to_db, sync_from_db};
use std::sync::Arc;

pub fn create_router(pool: PgPool, quota_counter: Arc<dyn QuotaCounter>) -> Router {
    // /proxy/{name}/{*rest_path} のルーティング
    // テスト用に /proxy/test も残しておく
    let protected_routes = Router::new()
        .route("/proxy/test", get(|| async { "ok" }))
        .route(
            "/proxy/{name}/{*rest_path}",
            get(proxy).post(proxy).put(proxy).delete(proxy).patch(proxy),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            (pool.clone(), quota_counter.clone()),
            metering,
        ))
        .route_layer(axum_middleware::from_fn_with_state(
            quota_counter.clone(),
            quota,
        ))
        .route_layer(axum_middleware::from_fn_with_state(pool.clone(), auth));

    let system_routes = Router::new()
        .route("/system/quota/sync-to-db", post(sync_to_db))
        .route("/system/quota/sync-from-db", post(sync_from_db))
        .with_state((pool.clone(), quota_counter.clone()));

    let public_routes = Router::new()
        .route("/health", get(health))
        .route("/admin/tenants", post(create_tenant).get(list_tenants))
        .route("/admin/projects", post(create_project).get(list_projects))
        .route("/admin/consumers", post(create_consumer))
        .route("/admin/plans", post(create_plan).get(list_plans))
        .route("/admin/api-keys", post(create_api_key).get(list_api_keys))
        .route(
            "/admin/upstream-services",
            post(create_upstream_service).get(list_upstream_services),
        )
        .route("/admin/usage", get(get_usage));

    public_routes
        .merge(protected_routes)
        .merge(system_routes)
        .with_state(pool)
}
