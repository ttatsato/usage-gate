pub mod valkey;
use async_trait::async_trait;

#[async_trait]
pub trait AuthCache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;

    async fn set(&self, key: &str, value: &str, ttl_secs: u64);

    async fn delete(&self, key: &str);
}
