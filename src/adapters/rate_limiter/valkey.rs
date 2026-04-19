use super::{RateLimit, RateLimitPeriod, RateLimiter, RateLimiterError};
use async_trait::async_trait;
use chrono::{Datelike, Utc};
use redis::AsyncCommands;
use uuid::Uuid;

pub struct ValkeyRateLimiter {
    client: redis::Client,
}

impl ValkeyRateLimiter {
    pub async fn new(url: &str) -> Result<Self, RateLimiterError> {
        let client =
            redis::Client::open(url).map_err(|e| RateLimiterError::Internal(e.to_string()))?;

        // 起動時に接続確認
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;

        Ok(Self { client })
    }

    fn tokens_key(&self, consumer_id: Uuid, period: &RateLimitPeriod) -> String {
        let now = Utc::now();
        match period {
            RateLimitPeriod::Monthly => {
                format!("rl:{}:{}:monthly:tokens", consumer_id, now.format("%Y-%m"))
            }
            RateLimitPeriod::Daily => {
                format!("rl:{}:{}:daily:tokens", consumer_id, now.format("%Y-%m-%d"))
            }
            RateLimitPeriod::Hourly => {
                format!("rl:{}:{}:hourly:tokens", consumer_id, now.format("%Y-%m-%dT%H"))
            }
            RateLimitPeriod::PerSecond => {
                format!("rl:{}:persec:tokens", consumer_id)
            }
        }
    }

    fn last_key(&self, consumer_id: Uuid, period: &RateLimitPeriod) -> String {
        let now = Utc::now();
        match period {
            RateLimitPeriod::Monthly => {
                format!("rl:{}:{}:monthly:last", consumer_id, now.format("%Y-%m"))
            }
            RateLimitPeriod::Daily => {
                format!("rl:{}:{}:daily:last", consumer_id, now.format("%Y-%m-%d"))
            }
            RateLimitPeriod::Hourly => {
                format!("rl:{}:{}:hourly:last", consumer_id, now.format("%Y-%m-%dT%H"))
            }
            RateLimitPeriod::PerSecond => {
                format!("rl:{}:persec:last", consumer_id)
            }
        }
    }

    /// period に応じた補充レート（トークン/秒）を計算する
    fn refill_rate(period: &RateLimitPeriod, max_requests: i64) -> f64 {
        let seconds = match period {
            RateLimitPeriod::Monthly => {
                let now = Utc::now();
                let days = days_in_month(now.year(), now.month());
                (days * 86400) as f64
            }
            RateLimitPeriod::Daily => 86400.0,
            RateLimitPeriod::Hourly => 3600.0,
            RateLimitPeriod::PerSecond => 1.0,
        };
        max_requests as f64 / seconds
    }

    /// period に応じた TTL（秒）を返す
    fn ttl_seconds(period: &RateLimitPeriod) -> i64 {
        let now = Utc::now();
        match period {
            RateLimitPeriod::Monthly => {
                let days = days_remaining_in_month(now.year(), now.month());
                ((days + 1) * 86400) as i64
            }
            RateLimitPeriod::Daily => 2 * 86400,
            RateLimitPeriod::Hourly => 2 * 3600,
            RateLimitPeriod::PerSecond => 10,
        }
    }
}

/// Token Bucket 全バケット一括 check-and-consume（atomic）
/// KEYS: 各 limit ごとに (tokens_key, last_key) のペア
/// ARGV: now, num_limits, 各 limit ごとに (max_tokens, refill_rate, ttl)
/// 返り値: 1 = 許可, 0 = 拒否
const TOKEN_BUCKET_ACQUIRE_SCRIPT: &str = r#"
local now = tonumber(ARGV[1])
local n = tonumber(ARGV[2])

-- Phase 1: 全バケットを読み、補充後の残量を算出
local refilled = {}
for i = 1, n do
    local tokens_key = KEYS[(i - 1) * 2 + 1]
    local last_key = KEYS[(i - 1) * 2 + 2]
    local max_tokens = tonumber(ARGV[2 + (i - 1) * 3 + 1])
    local refill_rate = tonumber(ARGV[2 + (i - 1) * 3 + 2])

    local tokens = tonumber(redis.call('GET', tokens_key))
    local last = tonumber(redis.call('GET', last_key))

    if tokens == nil then
        tokens = max_tokens
        last = now
    end

    local elapsed = math.max(0, now - last)
    tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

    if tokens < 1 then
        return 0
    end

    refilled[i] = tokens
end

-- Phase 2: 全バケットが通ったので消費
for i = 1, n do
    local tokens_key = KEYS[(i - 1) * 2 + 1]
    local last_key = KEYS[(i - 1) * 2 + 2]
    local ttl = tonumber(ARGV[2 + (i - 1) * 3 + 3])

    local new_tokens = refilled[i] - 1
    redis.call('SET', tokens_key, tostring(new_tokens))
    redis.call('SET', last_key, tostring(now))
    redis.call('EXPIRE', tokens_key, ttl)
    redis.call('EXPIRE', last_key, ttl)
end

return 1
"#;

#[async_trait]
impl RateLimiter for ValkeyRateLimiter {
    async fn try_acquire(
        &self,
        consumer_id: Uuid,
        limits: &[RateLimit],
    ) -> Result<bool, RateLimiterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;

        let now = Utc::now().timestamp_millis() as f64 / 1000.0;

        // 全バケットを 1 本の Lua で atomic に check + consume する
        let script = redis::Script::new(TOKEN_BUCKET_ACQUIRE_SCRIPT);
        let mut invocation = script.prepare_invoke();

        for limit in limits {
            invocation
                .key(self.tokens_key(consumer_id, &limit.period))
                .key(self.last_key(consumer_id, &limit.period));
        }

        invocation.arg(now).arg(limits.len());

        for limit in limits {
            invocation
                .arg(limit.max_requests)
                .arg(Self::refill_rate(&limit.period, limit.max_requests))
                .arg(Self::ttl_seconds(&limit.period));
        }

        let allowed: i32 = invocation
            .invoke_async(&mut conn)
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;

        Ok(allowed == 1)
    }

    async fn get_usage(
        &self,
        consumer_id: Uuid,
        period: &RateLimitPeriod,
        max_requests: i64,
    ) -> Result<i64, RateLimiterError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;

        let tokens_key = self.tokens_key(consumer_id, period);
        let last_key = self.last_key(consumer_id, period);
        let refill_rate = Self::refill_rate(period, max_requests);
        let now = Utc::now().timestamp_millis() as f64 / 1000.0;

        let tokens: Option<f64> = conn
            .get(&tokens_key)
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;
        let last: Option<f64> = conn
            .get(&last_key)
            .await
            .map_err(|e| RateLimiterError::Internal(e.to_string()))?;

        let current_tokens = match (tokens, last) {
            (Some(t), Some(l)) => {
                let elapsed = (now - l).max(0.0);
                (t + elapsed * refill_rate).min(max_requests as f64)
            }
            _ => max_requests as f64,
        };

        // 使用量 = max - 残り
        Ok(max_requests - current_tokens as i64)
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month_start = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    let month_start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    next_month_start
        .unwrap()
        .signed_duration_since(month_start)
        .num_days() as u32
}

fn days_remaining_in_month(year: i32, month: u32) -> u32 {
    let today = Utc::now().date_naive();
    let next_month_start = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next_month_start
        .unwrap()
        .signed_duration_since(today)
        .num_days() as u32
}
