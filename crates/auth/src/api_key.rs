use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::{ApiKeyRecord, GeneratedApiKey};

const KEY_PREFIX: &str = "stk_";
const RANDOM_LEN: usize = 32;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Generate a new API key for a user.
pub fn generate(user_id: &str, name: &str) -> GeneratedApiKey {
    let mut rng = rand::thread_rng();
    let random_part: String = (0..RANDOM_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    let raw_key = format!("{KEY_PREFIX}{random_part}");
    let key_hash = hash_key(&raw_key);
    let prefix = raw_key[..8.min(raw_key.len())].to_string();

    let record = ApiKeyRecord {
        id: format!("key_{}", Uuid::new_v4().simple()),
        user_id: user_id.to_string(),
        key_hash,
        prefix,
        name: name.to_string(),
        created_at: Utc::now(),
        last_used: None,
        revoked: false,
    };

    GeneratedApiKey { raw_key, record }
}

/// Hash a raw API key for storage/comparison.
pub fn hash_key(raw_key: &str) -> String {
    let digest = Sha256::digest(raw_key.as_bytes());
    hex::encode(digest)
}

/// Verify a raw key against a stored hash.
pub fn verify(raw_key: &str, stored_hash: &str) -> bool {
    hash_key(raw_key) == stored_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify() {
        let generated = generate("usr_123", "test key");
        assert!(generated.raw_key.starts_with("stk_"));
        assert_eq!(generated.raw_key.len(), 4 + RANDOM_LEN);
        assert!(verify(&generated.raw_key, &generated.record.key_hash));
        assert!(!verify("stk_wrong", &generated.record.key_hash));
    }

    #[test]
    fn test_prefix() {
        let generated = generate("usr_123", "test");
        assert_eq!(generated.record.prefix.len(), 8);
        assert!(generated.raw_key.starts_with(&generated.record.prefix));
    }
}
