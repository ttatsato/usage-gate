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

### 3. Consumer 作成

```sh
TENANT_ID=<手順2のid> ./sandbox/api-check/consumers.sh
```

レスポンスの `id` を控える。

### 4. API キー作成

```sh
CONSUMER_ID=<手順3のid> ./sandbox/api-check/api_keys.sh
```

レスポンスの `key` を控える。

### 5. Proxy エンドポイントの認証テスト

```sh
API_KEY=<手順4のkey> ./sandbox/api-check/proxy.sh
```

- 認証なし → 401
- 無効なキー → 401
- 有効なキー → 200

### 6. 使用量取得

手順5でリクエストを投げた後、メータリングにより使用量が記録されている。

```sh
TENANT_ID=<手順2のid> ./sandbox/api-check/usage.sh
```
