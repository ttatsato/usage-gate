use crate::adapters::auth_cache::AuthCache;
use async_trait::async_trait;

pub struct ValkeyAuthCache {
    client: redis::Client,
}

impl ValkeyAuthCache {
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;

        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
                .map_err(|e| eprintln!("{}", e))
                .ok();
        }

        Ok(Self { client })
    }
}

#[async_trait]
impl AuthCache for ValkeyAuthCache {
    async fn get(&self, key_hash: &str) -> Option<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
        redis::cmd("GET")
            .arg(key_hash)
            .query_async::<String>(&mut conn)
            .await
            .ok()
    }

    async fn set(&self, key_hash: &str, value: &str, ttl_secs: u64) {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await {
            redis::cmd("SET")
                .arg(key_hash)
                .arg(value)
                .arg("EX")
                .arg(ttl_secs)
                .query_async::<String>(&mut conn)
                .await
                .ok();
        }
    }

    async fn delete(&self, key_hash: &str) {
        if let Ok(mut conn) = self.client.get_multiplexed_async_connection().await
            && let Err(e) = redis::cmd("DEL")
                .arg(key_hash)
                .query_async::<i32>(&mut conn)
                .await
        {
            eprintln!("failed to delete auth cache key {}: {}", key_hash, e);
        }
    }
}
