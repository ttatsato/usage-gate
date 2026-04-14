use axum::{
    Extension, Json,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::Response,
};
use http_body_util::BodyExt;
use sqlx::PgPool;
use crate::models::api_key::AuthedApiKey;
use crate::repositories::upstream_service_repository;

// プロキシハンドラ
// /proxy/{name}/{*rest_path} のリクエストを upstream_services の base_url に転送する
pub async fn proxy(
    State(pool): State<PgPool>,
    Extension(authed): Extension<AuthedApiKey>,
    Path((name, rest_path)): Path<(String, String)>,
    request: Request,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // project 配下の upstream service を解決
    let upstream = upstream_service_repository::find_by_project_and_name(
        &pool, authed.project_id, &name,
    )
    .await
    .map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": "Internal server error"})),
    ))?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "Upstream service not found"})),
    ))?;

    // 転送先 URL を組み立て
    let base = upstream.base_url.trim_end_matches('/');
    let path = if rest_path.is_empty() {
        String::new()
    } else {
        format!("/{}", rest_path.trim_start_matches('/'))
    };
    let query = request.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();
    let target_url = format!("{}{}{}", base, path, query);

    // メソッド・ヘッダー・ボディを取り出す
    let method = request.method().clone();
    let headers = request.headers().clone();
    let body_bytes = request
        .into_body()
        .collect()
        .await
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Failed to read request body"})),
        ))?
        .to_bytes();

    // reqwest で転送
    let client = reqwest::Client::new();
    let mut req = client
        .request(to_reqwest_method(&method), &target_url)
        .body(body_bytes.to_vec());

    // クライアントから受け取ったヘッダーを下流に転送（一部除外）
    for (k, v) in headers.iter() {
        let name = k.as_str().to_lowercase();
        if matches!(name.as_str(), "host" | "x-api-key" | "content-length") {
            continue;
        }
        req = req.header(k.as_str(), v);
    }

    let upstream_resp = req.send().await.map_err(|_| (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({"error": "Upstream request failed"})),
    ))?;

    // レスポンスを Axum の Response に変換
    let status = StatusCode::from_u16(upstream_resp.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut resp_headers = HeaderMap::new();
    for (k, v) in upstream_resp.headers().iter() {
        if let (Ok(name), Ok(val)) = (
            axum::http::HeaderName::from_bytes(k.as_str().as_bytes()),
            axum::http::HeaderValue::from_bytes(v.as_bytes()),
        ) {
            resp_headers.insert(name, val);
        }
    }
    let body = upstream_resp.bytes().await.map_err(|_| (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({"error": "Failed to read upstream response"})),
    ))?;

    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = resp_headers;
    Ok(response)
}

fn to_reqwest_method(m: &Method) -> reqwest::Method {
    reqwest::Method::from_bytes(m.as_str().as_bytes()).unwrap_or(reqwest::Method::GET)
}
