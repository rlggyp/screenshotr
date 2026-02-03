use crate::Error;
use crate::auth::basic_auth::BasicAuth;
use crate::auth::hmac::Hmac;
use crate::config::Config;
use crate::screenshotr::screenshot::Screenshot;

pub struct AppState {
    pub hmac: Hmac,
    pub basic_auth: BasicAuth, 
    pub screenshot: Screenshot, 
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, Error> {
        let hmac = Hmac::new(&config.hmac_secret);
        let basic_auth = BasicAuth::new(&config.basic_auth_users);
        let screenshot = Screenshot::new(&config.screenshot)?;

        let app_state = Self { hmac, basic_auth, screenshot };
        Ok(app_state)
    }
}