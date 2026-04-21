# UsageGate

Usage-aware, multi-tenant API gateway built in Rust

UsageGate is a lightweight developer-focused API gateway that provides:
	•	API key authentication
	•	Multi-tenant isolation
	•	Usage metering
	•	Rate limiting
	•	Quota enforcement

Designed as a foundation for usage-based billing systems and AI-ready APIs.

# 🚀 Overview

Modern SaaS and AI APIs require:
	•	Secure API exposure
	•	Fine-grained usage tracking
	•	Rate limiting and abuse protection
	•	Flexible billing integration

However, existing solutions are often:
	•	Overly complex (DevOps-heavy)
	•	Fragmented across multiple tools
	•	Not optimized for developer-first usage

UsageGate aims to solve this by providing a simple, extensible, and high-performance gateway layer.

⸻

# 🧩 Core Features

1. API Key Authentication
	•	Header-based API key validation
	•	Key rotation & revocation (planned)
	•	Tenant-aware authentication

⸻

2. Multi-Tenant Architecture
	•	Tenant isolation at request level
	•	Per-tenant configuration
	•	Plan-based access control

⸻

3. Usage Metering
	•	Request-level usage tracking
	•	Endpoint-based aggregation
	•	Tenant/API key granularity

⸻

4. Rate Limiting
	•	Per-tenant / per-key rate limiting
	•	Sliding window (initially fixed window)
	•	Redis-backed counters

⸻

5. Quota Enforcement
	•	Monthly usage limits
	•	Plan-based quota control
	•	Over-quota request rejection

⸻

6. Middleware-based Architecture
	•	Auth middleware
	•	Tenant resolution middleware
	•	Metering middleware
	•	Rate-limiting middleware

⸻

# 🏗 Architecture (Phase 1)

Single-node architecture (monolith for simplicity and performance):

Axum (Rust)
├── Auth Middleware
├── Tenant Middleware
├── Metering Middleware
├── Rate Limit Middleware
├── API Handlers
│
├── PostgreSQL (persistent data)
└── Redis (rate limit & counters)


⸻

# ⚙️ Tech Stack
	•	Language: Rust
	•	Framework: Axum
	•	Async runtime: Tokio
	•	Database: PostgreSQL
	•	Cache / Rate limit: Redis
	•	Containerization: Docker (optional)

⸻

# 📦 API Design (MVP)

Authentication

Header: x-api-key: <API_KEY>


⸻

Core Endpoints

Health

GET /health


⸻

Proxy (example)

POST /proxy/{service}


⸻

Admin APIs (simplified)

Create Tenant

POST /admin/tenants

Create API Key

POST /admin/api-keys

Get Usage

GET /admin/usage


⸻

# 🧪 Testing Strategy
	•	Integration tests (API behavior)
	•	Concurrency tests (rate limit & metering)
	•	Tenant isolation tests
	•	Failure scenario tests

⸻

# 📊 Observability (Planned)
	•	Structured logging
	•	Request ID tracking
	•	Metrics (p95 / p99 latency)
	•	Usage dashboards

⸻

# 🧠 Why Rust?

UsageGate is not just an API service.
It is a high-throughput, correctness-critical gateway layer.

Rust is chosen for the following reasons:

⸻

## 1. Safe Concurrency

UsageGate handles:
	•	Concurrent requests
	•	Shared state (rate limits, usage counters)
	•	Multi-tenant isolation

Rust’s ownership model prevents:
	•	Data races
	•	Invalid memory access
	•	Unsafe shared state mutations

⸻

## 2. High Performance

Gateway layers require:
	•	Low latency
	•	High throughput
	•	Efficient async I/O

Rust provides:
	•	Zero-cost abstractions
	•	No garbage collection pauses
	•	Efficient async execution via Tokio

⸻

## 3. Strong Type Safety

Critical domain concepts:
	•	TenantId
	•	ApiKeyId
	•	Plan
	•	UsageUnit
	•	Quota

are enforced at compile time, reducing runtime errors.

⸻

## 4. Explicit Error Handling

UsageGate must handle failures such as:
	•	Invalid API keys
	•	Rate limit exceeded
	•	Quota exceeded
	•	Downstream failure

Rust’s Result type ensures all failure paths are explicitly handled.

⸻

## 5. Reliable Middleware Layer

All requests pass through:
	•	Authentication
	•	Metering
	•	Rate limiting

Rust enables writing robust and predictable middleware pipelines.

⸻

# 🔮 Roadmap

## Phase 1 (MVP)
	•	API key auth
	•	Usage tracking
	•	Rate limiting
	•	Basic tenant support

⸻

## Phase 2
	•	Plan management
	•	Webhook events
	•	Observability improvements

⸻

## Phase 3
	•	Stripe integration
	•	Usage-based billing
	•	Subscription management

⸻

## Phase 4
	•	Model Context Protocol support
	•	AI agent integration
	•	Tool-level metering

⸻

# 💡 Use Cases
	•	SaaS APIs with usage-based billing
	•	AI APIs (LLM / tool APIs)
	•	Internal API governance
	•	Developer platforms

⸻

# 🧭 Positioning

UsageGate is:
	•	Not a full infrastructure gateway like enterprise solutions
	•	Not just an analytics tool

It is:

👉 A developer-first, usage-aware API gateway for modern SaaS and AI applications

⸻

# 📌 Status

🚧 Work in progress (Phase 1)

⸻

# 🧾 License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0).  
See the [LICENSE](./LICENSE) file for details.

Copyright (C) 2026 Tatsuya Satoh

⸻
