use crate::Error;

use std::collections::HashMap;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ScreenshotConfig {
    pub webdriver_url: String,
    pub webdriver_capabilities: serde_json::Map<String, serde_json::Value>,
    pub page_load_delay_secs: u64,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub hmac_secret: String,
    pub basic_auth_users: HashMap<String, String>,
    pub screenshot: ScreenshotConfig,
}

impl Config {
    pub fn get_config() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config_file = std::env::var("CONFIG_FILE")
            .map_err(|e| {
                let error = format!("Environment variable `CONFIG_FILE` not found {}", e);
                log::error!("{}", error);
                error
            })?;

        let file = std::fs::File::open(config_file)
            .map_err(|e| {
                log::error!("Failed to open config file: {}", e);
                e
            })?;

        let config: Config = serde_yaml::from_reader(file)?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.hmac_secret.trim().is_empty() {
            return Err("hmac_secret cannot be empty".into());
        }

        if self.hmac_secret.len() < 16 {
            return Err("hmac_secret must be at least 16 characters".into());
        }

        if self.basic_auth_users.is_empty() {
            return Err("basic_auth_users cannot be empty".into());
        }

        for (user, pass) in &self.basic_auth_users {
            if user.trim().is_empty() || pass.trim().is_empty() {
                return Err("basic_auth username/password cannot be empty".into())
            }
        }

        Ok(())
    }
}