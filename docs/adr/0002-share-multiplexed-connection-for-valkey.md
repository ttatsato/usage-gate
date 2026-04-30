# 0002. Share a MultiplexedConnection across Valkey adapter calls

# Status
accepted

# Context

`ValkeyAuthCache` and `ValkeyRateLimiter` previously held a `redis::Client` and
called `get_multiplexed_async_connection()` on every operation. That added
connection setup overhead to each cache lookup / rate-limit check, and made the
number of active Valkey connections at any given moment hard to reason about.

This is the same pattern we applied to the HTTP side in PR #70 (sharing one
`reqwest::Client` across proxy requests instead of constructing one per request).
Valkey adapters were the next obvious target.

# Decision

Both Valkey adapters now hold a `redis::aio::MultiplexedConnection` directly,
established once in `new()`:

- `ValkeyAuthCache` and `ValkeyRateLimiter` store `conn: MultiplexedConnection`
- The startup `PING` runs on the same connection that will be reused at runtime
- Each operation does `let mut conn = self.conn.clone();` — `MultiplexedConnection`
  is designed to be cheaply cloned and used concurrently from many tasks, so
  this is the intended usage pattern from the `redis` crate

# Alternatives Considered

- **Status quo: `get_multiplexed_async_connection()` per call.** Rejected:
  redundant setup work on every request; connection count is unpredictable.
- **Use a pool (bb8-redis / deadpool-redis).** Rejected for now: a single
  `MultiplexedConnection` is already safe to share across concurrent tasks, so
  a pool adds dependencies and complexity for little gain at the current scale.
  Worth revisiting if we ever need per-tenant isolation, bulkheads between
  high-volume callers, or tighter control over reconnection behavior.
- **Spawn / hold one connection per worker manually.** Rejected: defeats the
  purpose of a multiplexed connection, which already pipelines commands from
  many callers over a single underlying socket.

# Consequences

- Positive:
  - Eliminates per-operation connection setup on the hot path
  - Code is shorter and has fewer error branches (no per-call connection failure case)
  - Connection count per adapter instance is fixed and predictable
- Negative / trade-offs:
  - All callers share one underlying socket; if the connection is unhealthy,
    everyone is affected until the `redis` crate's reconnection logic recovers.
    No bulkhead between high-volume and low-volume callers.
  - If we later need pooling, we have to revisit both adapters.
- Affected modules / operations:
  - `src/adapters/auth_cache/valkey.rs`
  - `src/adapters/rate_limiter/valkey.rs`

# References

- [PR #71](https://github.com/ttatsato/usage-gate/pull/71) — this change
- [PR #70](https://github.com/ttatsato/usage-gate/pull/70) — same pattern applied to the shared `reqwest::Client`
