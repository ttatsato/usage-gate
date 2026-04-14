# API 動作確認手順

API Gateway の各エンドポイントを順番に叩いて動作確認するためのスクリプト集です。

## 前提

- サーバーが起動していること（`cargo run`）
- `jq` がインストールされていること（`brew install jq`）

## 一連の動作確認フロー

### 1. Health チェック

```sh
./sandbox/api-check/health.sh
```

### 2. テナント作成

```sh
./sandbox/api-check/tenants.sh
```

レスポンスの `id` を控える。

### 3. プロジェクト作成

```sh
TENANT_ID=<手順2のid> ./sandbox/api-check/projects.sh
```

レスポンスの `id` を控える。

### 4. プラン作成

```sh
PROJECT_ID=<手順3のid> ./sandbox/api-check/plans.sh
```

レスポンスの `id` を控える（プランに紐づけたい consumer で使う）。

### 5. 上流サービス登録

```sh
PROJECT_ID=<手順3のid> ./sandbox/api-check/upstream_services.sh
```

httpbin.org を登録する例。

### 6. Consumer 作成

```sh
PROJECT_ID=<手順3のid> PLAN_ID=<手順4のid> ./sandbox/api-check/consumers.sh
```

レスポンスの `id` を控える。

### 7. API キー作成

```sh
CONSUMER_ID=<手順6のid> ./sandbox/api-check/api_keys.sh
```

レスポンスの `key` を控える（発行時だけ平文が返る）。

### 8. Proxy の認証テスト

```sh
API_KEY=<手順7のkey> ./sandbox/api-check/proxy.sh
```

- 認証なし → 401
- 無効なキー → 401
- 有効なキー → 200

### 9. 実際の転送テスト（httpbin）

手順5で登録した `httpbin` に転送される:

```sh
curl -H "x-api-key: <key>" http://localhost:8080/proxy/httpbin/get
```

### 10. 使用量取得

```sh
TENANT_ID=<手順2のid> ./sandbox/api-check/usage.sh
```
