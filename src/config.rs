use crate::Error;

use std::collections::HashMap;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ScreenshotConfig {
    #[serde(default = "ScreenshotConfig::default_page_load_delay_secs")]
    pub page_load_delay_secs: u64,
    #[serde(default = "ScreenshotConfig::default_screenshots_dir")]
    pub screenshots_dir: String,
    #[serde(default = "ScreenshotConfig::default_public_base_url")]
    pub public_base_url: String,
    #[serde(default = "ScreenshotConfig::default_webdriver_url")]
    pub webdriver_url: String,
    #[serde(default = "ScreenshotConfig::default_webdriver_capabilities")]
    pub webdriver_capabilities: serde_json::Map<String, serde_json::Value>,
}

impl ScreenshotConfig {
    fn default_page_load_delay_secs() -> u64 {
        2
    }

    fn default_screenshots_dir() -> String {
        String::from("/assets/screenshots")
    }

    fn default_public_base_url() -> String {
        String::from("http://127.0.0.1:12009")
    }

    fn default_webdriver_url() -> String {
        String::from("http://127.0.0.1:4444")
    }

    fn default_webdriver_capabilities() -> serde_json::Map<String, serde_json::Value> {
        let capabilities = serde_json::json!({
            "browserName": "chrome",
            "goog:chromeOptions": {
              "args": [
                "--headless",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--disable-gpu",
                "--window-size=1920,1200"
              ]
            }
        });

        let mut map = serde_json::Map::new();
        map.insert("capabilities".into(), capabilities);
        map
    }

    fn normalize(&mut self) {
        self.screenshots_dir = self.screenshots_dir
            .trim_end_matches('/')
            .to_string();

        self.public_base_url = self.public_base_url
            .trim_end_matches('/')
            .to_string();

        self.webdriver_url = self.webdriver_url
            .trim_end_matches('/')
            .to_string();
    }
} 

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub hmac_secret: String,
    pub basic_auth_users: HashMap<String, String>,
    pub screenshot: ScreenshotConfig,
    pub allowed_domains: Vec<String>,
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

        let mut config: Config = serde_yaml::from_reader(file)?;

        config.validate()?;
        config.screenshot.normalize();

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