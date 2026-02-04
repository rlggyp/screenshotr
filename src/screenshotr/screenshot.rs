use crate::config;
use crate::Error;

use fantoccini::ClientBuilder;
use uuid::Uuid;

pub struct Screenshot {
    page_load_delay_secs: u64,
    screenshots_dir: String,
    public_base_url: String,
    webdriver_url: String,
    webdriver_capabilities: serde_json::Map<String, serde_json::Value>,
}

impl Screenshot {
    pub fn new(config: &config::ScreenshotConfig) -> Result<Self, Error> {
        std::fs::create_dir_all(&config.screenshots_dir)?;
        let public_base_url = config.public_base_url.clone()
            .trim_end_matches('/')
            .to_string();

        let screenshot = Self {
            page_load_delay_secs: config.page_load_delay_secs,
            screenshots_dir: config.screenshots_dir.clone(),
            public_base_url: public_base_url,
            webdriver_url: config.webdriver_url.clone(),
            webdriver_capabilities: config.webdriver_capabilities.clone(),
        };

        Ok(screenshot)
    }

    pub async fn take_screenshot(
        &self,
        url: &str,
    ) -> Result<String, Error> {
        let client = ClientBuilder::native()
            .capabilities(self.webdriver_capabilities.clone())
            .connect(&self.webdriver_url)
            .await?;

        if let Err(e) = client.goto(url).await {
            log::error!("Failed go to url: {url} {e}");
            client.close().await.ok();
            return Err(e.into());
        }

        self.wait_page_load_delay().await;
        self.undock_menu(&client).await;

        let png_data = match client.screenshot().await {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed screenshot web page: {url} {e}");
                client.close().await.ok();
                return Err(e.into());
            }
        };

        let filename = format!("{}.png", Uuid::new_v4());
        let filepath = format!("{}/{}", self.screenshots_dir, filename);

        if let Err(e) = tokio::fs::write(&filepath, &png_data).await {
            log::error!("Failed to write screenshot file: {e}");
            client.close().await.ok();
            return Err(e.into());
        }

        log::info!("Screenshot saved: {}", filepath);

        let image_path = format!("/screenshotr/images/{}", filename);
        let image_url = format!("{}{}", self.public_base_url, image_path);

        client.close().await.ok();

        Ok(image_url)
    }

    async fn wait_page_load_delay(&self) {
        tokio::time::sleep(std::time::Duration::from_secs(self.page_load_delay_secs)).await
    }

    async fn undock_menu(&self, client: &fantoccini::Client) {
        match client.find(fantoccini::Locator::Css("#dock-menu-button")).await {
            Ok(element) => {
                log::debug!("Dock menu button found, attempting to click.");
                if let Err(e) = element.click().await {
                    log::debug!("Failed to click dock menu button: {}", e);
                } else {
                    log::debug!("Successfully clicked dock menu button.");
                }
            },
            Err(e) => {
                log::debug!("Dock menu button not found: {}", e);
            },
        };
    }
}