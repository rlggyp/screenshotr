use std::collections::HashMap;
use std::sync::Arc;

use base64::{Engine, engine::general_purpose};

use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Credential {
    pub encoded: String,
    pub decoded: String,
}

#[derive(Clone)]
pub struct BasicAuth {
    pub credentials: HashMap<String, String>,
    pub auth_header_cache: Arc<RwLock<Vec<String>>>,
}

impl BasicAuth {
    pub fn new(credentials: &HashMap<String, String>) -> Self {
        let credentials = credentials.clone();
        let auth_header_cache: Arc<RwLock<Vec::<String>>> = Arc::new(RwLock::new(Vec::new()));

        Self { credentials, auth_header_cache }
    }

    pub fn is_valid_basic_auth_header(auth_header: &str) -> Option<Credential> {
        log::debug!(
            "Checking if Authorization header is valid Basic Auth: {}",
            auth_header
        );

        if !(auth_header.starts_with("Basic") && auth_header.len() > 6) {
            log::debug!("Auth header does not start with `Basic` or is too short");
            return None;
        }

        let encoded = auth_header[6..].to_string();

        let decoded = match general_purpose::STANDARD.decode(&encoded) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(_) => {
                    log::debug!("Failed to decode base64 to UTF-8");
                    return None;
                }
            },
            Err(_) => {
                log::debug!("Failed to decode base64 from auth header");
                return None;
            }
        };

        let credential = Credential { encoded, decoded };

        Some(credential)
    }

    fn parse_user_pass(&self, credential: &String) -> Option<(String, String)> {
        let parts: Vec<&str> = credential.splitn(2, ':').collect();

        if parts.len() != 2 {
            log::debug!("Credential does not contain a valid `user:pass` format");
            return None;
        }

        let username = parts[0].to_string();
        let password = parts[1].to_string();

        log::debug!("Parsed username: {}, password: [REDACTED]", username);
        Some((username, password))
    }

    pub async fn verify(&self, credential: Credential) -> bool {
        let Some((username, password)) = self.parse_user_pass(&credential.decoded) else {
            return false;
        };

        match self.credentials.get(&username) {
            Some(hash) => {
                let verified = bcrypt::verify(password, &hash).unwrap_or(false);

                let mut cache = self.auth_header_cache.write().await;
                if verified && !cache.contains(&credential.encoded) {
                    cache.push(credential.encoded.clone());
                }

                log::debug!("Password verification for user {}: {}", username, verified);
                verified
            }
            None => {
                log::debug!("User {} not found in credentials", username);
                false
            }
        }
    }
}
