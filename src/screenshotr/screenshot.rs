use crate::config;
use crate::Error;

use fantoccini::ClientBuilder;
use uuid::Uuid;

pub struct Screenshot {
    webdriver_url: String,
    webdriver_capabilities: serde_json::Map<String, serde_json::Value>,
    page_load_delay_secs: u64,
}

impl Screenshot {
    pub fn new(config: &config::ScreenshotConfig) -> Result<Self, Error> {
        std::fs::create_dir_all("assets/screenshots")?;

        let screenshot = Self {
            webdriver_url: config.webdriver_url.clone(),
            webdriver_capabilities: config.webdriver_capabilities.clone(),
            page_load_delay_secs: config.page_load_delay_secs,
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

        client.goto(url).await?;
        self.wait_page_load_delay().await;
        self.undock_menu(&client).await;

        let png_data = client.screenshot().await?;

        let filename = format!("{}.png", Uuid::new_v4());
        let filepath = format!("assets/screenshots/{}", filename);
        std::fs::write(&filepath, &png_data)?;
        let image_url = format!("screenshotr/images/{}", filename);

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
                    log::error!("Failed to click dock menu button: {}", e);
                } else {
                    log::debug!("Successfully clicked dock menu button.");
                }
            },
            Err(e) => {
                log::error!("Dock menu button not found: {}", e);
            },
        };
    }
}