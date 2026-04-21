# UsageGate

## Conncept / コンセプト
クライアントサーバーとAPIサーバーの前に挟むだけで公開APIのビジネスモデルを展開できる。  
*Just place it between your client and API server to instantly deploy a public API business model.*

### What's focus on ? / どんな悩みを解決するか
「従量課金性のAPIサービスを開発したい」「自社SaaSに公開API機能を搭載したい」という開発者体験を改善するためのツールです。  
*This is a tool to improve the developer experience for those who want to build pay-per-use API services or add a public API feature to their own SaaS product.*

APIGateWayとしてサーバーを立ててもらうことで、安全でハイパフォーマンスな公開APIビジネスを構築することができます。  
*By running this as an API Gateway, you can build a secure and high-performance public API business.*


### What kind of functions / どんな機能を提供するか？
- エンドユーザー向けにAPI Keyの発行・管理機能
  *API Key issuance and management for end users*
- マルチテナント管理
  *Multi-tenant management*
- API Keyの認証認可機能
  *API Key authentication and authorization*
- 公開APIの従量設定・制御機能(クォータ・レート制限)
  *Usage configuration and control for public APIs (quota and rate limiting)*
- 不正利用防止
  *Fraud and abuse prevention*
- ユーザーごとの毎月の従量確認ダッシュボード
  *Monthly usage dashboard per user*

# 🏗 Architecture / アーキテクチャ

Single-node architecture (monolith for simplicity and performance):

Axum（Rust）
├── 認証ミドルウェア *(Authentication middleware)*  
├── テナント解決 *(Tenant resolution)*  
├── Usage計測 *(Usage metering)*  
├── レート制限 *(Rate limiting)*  
├── APIハンドラ *(API handler)*  
├── PostgreSQL（永続データ） *(PostgreSQL - persistent data)*  
└── ValKey（カウンタ・レート制限） *(ValKey - counter & rate limiting)*  


⸻

# ⚙️ Tech Stack
	•	Language: Rust
	•	Framework: Axum
	•	Async runtime: Tokio
	•	Database: PostgreSQL
	•	Cache / Rate limit: Redis
	•	Containerization: Docker

⸻

# 📦 API Design (MVP)

Authentication

Header: x-api-key: <API_KEY>


# 🧪 Testing Strategy
	•	Integration tests (API behavior)
	•	Concurrency tests (rate limit & metering)
	•	Failure scenario tests

⸻

# 📊 Observability (Planned)
	•	Structured logging
	•	Request ID tracking
	•	Metrics (p95 / p99 latency)
	•	Usage dashboards

⸻

# 🔮 Roadmap

https://github.com/users/ttatsato/projects/3


# 🧭 Positioning

👉 A developer-first, usage-aware API gateway for modern SaaS and AI applications


# 🧾 License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).  
See the [LICENSE](./LICENSE) file for details.

Copyright (C) 2026 Tatsuya Satoh

⸻
