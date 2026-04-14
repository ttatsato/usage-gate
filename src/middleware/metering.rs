use crate::models::api_key::AuthedApiKey;
use crate::repositories::usage_repository;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

// メータリングミドルウェア
// Auth ミドルウェアの後に動き、リクエスト完了後に使用量を記録する
pub async fn metering(State(pool): State<PgPool>, request: Request, next: Next) -> Response {
    // リクエストからテナント情報を取得（Auth ミドルウェアが extensions に添付したもの）
    let authed = request.extensions().get::<AuthedApiKey>().cloned();
    // レスポンス後にはリクエスト情報にアクセスできないので、先に取得しておく
    let method = request.method().to_string();
    let endpoint = request.uri().path().to_string();

    // ハンドラを実行してレスポンスを得る
    let response = next.run(request).await;

    // 認証済みリクエストのみ記録する
    if let Some(authed) = authed {
        let status_code = response.status().as_u16() as i16;
        // tokio::spawn で非同期タスクとして投げる
        // DB 書き込みを待たずにレスポンスを返すので、レイテンシに影響しない
        tokio::spawn(async move {
            let _ = usage_repository::record_usage(
                &pool,
                authed.tenant_id,
                authed.api_key_id,
                &endpoint,
                &method,
                status_code,
            )
            .await;
        });
    }

    response
}
