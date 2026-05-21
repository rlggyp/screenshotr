use crate::config::ReplayProtectionConfig;
use crate::auth::nonce_cache::NonceCache;

const MAX_NONCE_LENGTH_SIZE: usize = 256;

pub struct ReplayProtectionValidator {
    nonce_ttl: i64,
    nonce_cache: NonceCache,
}

impl ReplayProtectionValidator {
    pub fn new(config: ReplayProtectionConfig) -> Self {
        let nonce_ttl = config.nonce_ttl;
        let nonce_cache = NonceCache::new(config.max_nonce_cache_size, nonce_ttl);
        
        Self {
            nonce_ttl,
            nonce_cache,
        }
    }

    pub async fn validate(
        &self,
        timestamp_header: Option<&str>,
        nonce_header: Option<&str>,
    ) -> Result<(String, i64), String> {
        let timestamp = self.validate_timestamp(timestamp_header)?;
        log::debug!("[replay_protection] Timestamp validated: {}", timestamp);

        let nonce = self.validate_nonce(nonce_header)?;

        self.nonce_cache.check_and_add_nonce(&nonce, timestamp).await?;
        log::debug!("[replay_protection] Nonce validated and cached");

        Ok((nonce, timestamp))
    }

    fn validate_timestamp(&self, timestamp_header: Option<&str>) -> Result<i64, String> {
        let timestamp_str = timestamp_header
            .ok_or_else(|| "Missing Timestamp header".to_string())?;

        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| "Invalid Timestamp format, must be Unix timestamp in seconds".to_string())?;

        let current_time = chrono::Utc::now().timestamp();

        let time_diff = if current_time > timestamp {
            current_time - timestamp
        } else {
            timestamp - current_time
        };

        if time_diff > self.nonce_ttl {
            return Err(format!(
                "Request timestamp too old or too far in the future. Difference: {} secs (ttl: {} secs)",
                time_diff, self.nonce_ttl
            ));
        }

        Ok(timestamp)
    }

    fn validate_nonce(&self, nonce_header: Option<&str>) -> Result<String, String> {
        let nonce = nonce_header
            .ok_or_else(|| "Missing Nonce header".to_string())?
            .to_string();

        if nonce.is_empty() {
            return Err("Nonce header is empty".to_string());
        }

        if nonce.len() > MAX_NONCE_LENGTH_SIZE {
            return Err("Nonce header too long (max 256 characters)".to_string());
        }

        Ok(nonce)
    }
}
