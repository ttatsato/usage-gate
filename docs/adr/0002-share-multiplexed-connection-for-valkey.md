# 0002. Share a ConnectionManager across Valkey adapter calls

# Status
accepted

# Context

`ValkeyAuthCache` and `ValkeyRateLimiter` originally held a `redis::Client` and called
`get_multiplexed_async_connection()` on every operation. That created a fresh connection
per cache lookup / rate-limit check, added setup overhead to the hot path, and made the
number of active Valkey connections at any given moment hard to reason about.

This is the same shape of problem we addressed for HTTP in PR #70 (sharing one
`reqwest::Client` across proxy requests instead of constructing one per request). The
Valkey adapters were the next obvious target.

# Decision

Both Valkey adapters now hold a `redis::aio::ConnectionManager`, established once in `new()`:

- `ValkeyAuthCache` and `ValkeyRateLimiter` store `conn: ConnectionManager`
- `ConnectionManager::new(client)` is called at startup; the same handle is reused for
  the lifetime of the process
- Each operation does `let mut conn = self.conn.clone();` — `ConnectionManager` is
  cheaply cloneable (Arc-backed) and is designed to be shared across many concurrent tasks
- The startup `PING` runs on the same connection that will be reused at runtime

`ConnectionManager` is a `MultiplexedConnection`-based handle from the `redis` crate
that adds **automatic reconnection** on transient failures. Memory profile and
per-request behavior are the same as a bare `MultiplexedConnection` (one underlying
socket, pipelined commands), but if the connection drops, the manager re-establishes
it in the background instead of leaving callers with a permanently dead handle.

Adds the `connection-manager` feature to the `redis` crate in `Cargo.toml`.

# Alternatives Considered

- **Status quo: `get_multiplexed_async_connection()` per call.** Rejected: redundant
  setup work on every request; connection count is unpredictable; nothing about it
  was an intentional design choice.
- **Bare `MultiplexedConnection` (no auto-reconnect).** Initially proposed in PR #71.
  Rejected after review (Codex flagged this as P1/P2): the `redis` crate's
  `MultiplexedConnection` does **not** reconnect on failure, so a Valkey restart or
  TCP drop leaves all cloned handles permanently failing. For an authenticated request
  path, that turns a transient Valkey blip into a continuous HTTP 500 stream until the
  API Gateway process is restarted. `ConnectionManager` solves this without changing
  the per-request cost or memory profile.
- **Traditional connection pool (`bb8-redis` / `deadpool-redis`).** Rejected: these
  pools maintain N independent connections with lease/return semantics, which adds
  bookkeeping `ConnectionManager` doesn't need (each `ConnectionManager` is already
  safe to share across many concurrent tasks). The N-socket cost negates the memory
  advantage that makes a shared connection attractive for an API Gateway, with no
  matching benefit at our current scale.
- **Sharded `ConnectionManager` (`Vec<ConnectionManager>` with round-robin /
  random selection).** Not yet — kept as a future option. This is closer to
  "sharding" than "pooling": no lease/return needed, just pick one of N handles per
  request. Trade-offs:
  - Pros: relieves multiplexer mutex contention at very high concurrency, exploits
    Valkey 8's IO threads on the server side, mitigates head-of-line blocking from
    heavy Lua scripts (e.g. the rate-limiter token bucket), enables bulkheading
    between workload classes (auth_cache vs rate_limiter, or per-tenant)
  - Cons: linear FD / memory growth in N (small), reconnect thundering-herd risk on
    Valkey restart, more complex to monitor per-connection health
  - Trigger to revisit: profiling shows the single multiplexer / single socket is
    the bottleneck, OR a concrete bulkheading requirement emerges (e.g. one tenant's
    rate-limit traffic starving the auth cache). Adopting it speculatively in the
    MVP phase is over-engineering.
- **Handle reconnection at the infrastructure layer (k8s liveness probe, HA Valkey).**
  Rejected as a substitute, kept as defense-in-depth. Liveness-probe-based restart
  converts every Valkey hiccup into a full pod restart (during which all requests
  fail); HA Valkey reduces failure frequency but failover still produces TCP drops
  the client must handle. Application-level reconnection is the right primary
  mechanism; infra HA is complementary, not a replacement.
- **Hold one connection per worker manually.** Rejected: defeats the purpose of a
  multiplexed connection, which already pipelines commands from many callers over a
  single underlying socket.

# Consequences

- Positive:
  - Eliminates per-operation connection setup on the hot path
  - Auto-reconnects on transient Valkey failures — transient outages no longer require
    a process restart
  - Code is shorter and has fewer error branches than the original per-call setup
  - Connection count per adapter instance is fixed and predictable (one socket each)
- Negative / trade-offs:
  - All callers share one underlying socket per adapter; no bulkhead between
    high-volume and low-volume callers
  - Reconnection is implicit — failures are retried behind the application's back,
    which can hide a degraded Valkey from monitoring unless we instrument it explicitly
  - Adds the `connection-manager` feature to the `redis` crate (negligible build cost)
- Affected modules / operations:
  - `src/adapters/auth_cache/valkey.rs`
  - `src/adapters/rate_limiter/valkey.rs`
  - `Cargo.toml` (added `connection-manager` feature)

# References

- [PR #71](https://github.com/ttatsato/usage-gate/pull/71) — this change, and the
  Codex review thread that prompted switching from bare `MultiplexedConnection` to
  `ConnectionManager`
- [PR #70](https://github.com/ttatsato/usage-gate/pull/70) — same shared-handle
  pattern applied to `reqwest::Client`
