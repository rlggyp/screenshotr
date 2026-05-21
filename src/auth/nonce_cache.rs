use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct NonceCache {
    cache: Arc<RwLock<HashMap<String, i64>>>,
    max_size: usize,
    ttl_secs: i64,
}

impl NonceCache {
    pub fn new(max_size: usize, ttl_secs: i64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl_secs,
        }
    }

    pub async fn check_and_add_nonce(&self, nonce: &str, timestamp: i64) -> Result<(), String> {
        let mut cache = self.cache.write().await;
        let current_time = chrono::Utc::now().timestamp();

        self.cleanup_expired_nonces(&mut cache, current_time);

        if let Some(stored_timestamp) = cache.get(nonce) {
            let age = current_time.saturating_sub(*stored_timestamp);
            if age < self.ttl_secs {
                return Err(format!(
                    "Nonce already used (age: {} secs, TTL: {} secs)",
                    age, self.ttl_secs
                ));
            }
        }

        cache.insert(nonce.to_string(), timestamp);
        log::debug!(
            "[nonce_cache] Nonce added. Cache size: {}/{}",
            cache.len(),
            self.max_size
        );

        Ok(())
    }

    fn cleanup_expired_nonces(&self, cache: &mut HashMap<String, i64>, current_time: i64) {
        let before_count = cache.len();
        cache.retain(|_, timestamp| current_time.saturating_sub(*timestamp) < self.ttl_secs);
        let after_count = cache.len();

        if before_count != after_count {
            log::info!(
                "[nonce_cache] Cleaned {} expired nonces. Cache size: {}/{}",
                before_count - after_count,
                after_count,
                self.max_size
            );
        }

        if cache.len() > self.max_size {
            let overflow = cache.len() - self.max_size;
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|&(_, &ts)| ts);

            let nonces_to_remove: Vec<String> = entries
                .iter()
                .take(overflow)
                .map(|(nonce, _)| nonce.to_string())
                .collect();

            for nonce in nonces_to_remove {
                cache.remove(&nonce);
                log::warn!(
                    "[nonce_cache] Evicted oldest nonce due to cache overflow: {}",
                    nonce
                );
            }
        }
    }
}
