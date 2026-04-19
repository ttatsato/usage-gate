# CLAUDE.md

Claude Code 向けのプロジェクト契約。README.md と重複する製品説明は避け、**コードから読み取れない前提 / 手順 / 罠**に絞る。

## プロジェクト概要

Rust + Axum 製の multi-tenant API Gateway。主要機能はキー認証・テナント解決・使用量計測・レート制限・クォータ。

```
src/
├── routes/       HTTP ハンドラ (admin/*, proxy, health, system/quota_sync)
├── middleware/   auth → quota → metering の順で適用 (lib.rs 参照)
├── models/       ドメインモデル
├── repositories/ SQLx 経由の DB アクセス
├── adapters/     外部依存の抽象化 (現状 rate_limiter/valkey のみ)
└── utils/        hash 等
```

## 開発環境

### 依存

- Docker（Postgres 17 / Valkey 8 をコンテナで起動）
- `rustup component add rustfmt clippy`
- sqlx-cli（migration 操作に必要）

### ポート / 認証情報

`.env`（`cp .env.example .env`）で一元管理。docker-compose.yml は `${POSTGRES_PORT:-5433}` 等の形で同じ変数を読むので、**ホスト側ポートは .env を正とする**。

| サービス | デフォルト（ホスト側）| コンテナ側 | 制御変数 |
|---|---|---|---|
| Postgres | **5433** | 5432 | `POSTGRES_PORT` |
| Valkey   | **6380** | 6379 | `VALKEY_PORT` |

ポート衝突時は `.env` の `POSTGRES_PORT` / `VALKEY_PORT` と `DATABASE_URL` / `RATE_LIMITER_URL` を**両方**書き換える（URL 内補間はしていないため二重定義）。

### 環境変数

| 変数 | 用途 |
|---|---|
| `DATABASE_URL` | Postgres 接続（URL 形式。ポートは `POSTGRES_PORT` と一致させる） |
| `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` / `POSTGRES_PORT` | docker-compose の Postgres 設定 |
| `VALKEY_PORT` | docker-compose の Valkey ホスト側ポート |
| `RATE_LIMITER` | 現状 `valkey` のみ対応（それ以外は main.rs で panic） |
| `RATE_LIMITER_URL` | Valkey 接続（例：`redis://localhost:6380`） |
| `QUOTA_COUNTER` / `QUOTA_COUNTER_URL` | クォータカウンタ（テストで併用される） |
| `API_PORT` | デフォルト 8080 |
| `SQLX_OFFLINE=true` | sqlx の offline check を有効化してビルド |

## 基本コマンド

### 起動
```bash
docker compose up -d                 # Postgres + Valkey
sqlx migrate run                     # マイグレーション適用
cargo run                            # API サーバー
cargo run -- sync-to-db              # Valkey → DB の一括同期バッチ
```

### テスト

統合テスト（`tests/api_test.rs`）は Valkey を前提とする。Valkey 未起動だと落ちる。

```bash
# 必ず Valkey を起動した状態で
RATE_LIMITER_URL=redis://localhost:6380 cargo test
RATE_LIMITER=valkey RATE_LIMITER_URL=redis://localhost:6380 cargo test
QUOTA_COUNTER=valkey QUOTA_COUNTER_URL=redis://localhost:6380 cargo test
```

### フォーマット / リント

```bash
cargo fmt
cargo clippy
```

### DB 操作

```bash
sqlx migrate add <NAME>              # 新規 migration ファイル
sqlx migrate run
docker exec -it usage-gate-db psql -U usage_gate -d usage_gate -c '\dt'
```

## 設計上のルール / ハマりどころ

- **`auth → quota → metering`** の順に middleware が積まれている（`src/lib.rs`）。順序変更はテナント解決や計測の前提を壊すので慎重に。
- **token bucket のバースト上限は DB 管理**（`plans` テーブルの rate_limit カラム群）。コード側で定数化しないこと。直近コミット `e76793f` 参照。
- **Valkey → DB 同期** は `src/main.rs` で `tokio::spawn` 起動の 1 時間間隔バッチ。`sync-to-db` サブコマンドでも手動実行可能。
- **`SQLX_OFFLINE=true`** でのビルドは `.sqlx/` の query キャッシュに依存。スキーマを変えたら `cargo sqlx prepare` を忘れない。
- **RATE_LIMITER が未設定 / 非対応値の場合 main.rs で panic**。production 起動スクリプトでは必ず set。

## 変更時のチェックリスト

コードを編集したら最低限これを通す：

```bash
cargo fmt
cargo clippy
RATE_LIMITER_URL=redis://localhost:6380 cargo test
```

migration を追加した場合：

```bash
sqlx migrate run
cargo sqlx prepare     # SQLX_OFFLINE=true でビルド通すため
```

## ドキュメント

- `README.md` — 製品コンセプト / ロードマップ
- `README.dev.md` — セットアップ手順（本ファイルと一部重複）
- `docs/README.ja.md` — 日本語詳細仕様
