use sha2::{Sha256, Digest};

// API キーを SHA-256 でハッシュ化して16進文字列で返す
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}
