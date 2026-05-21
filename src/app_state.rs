use crate::Error;
use crate::auth::basic_auth::BasicAuth;
use crate::auth::hmac::Hmac;
use crate::auth::replay_protection::ReplayProtectionValidator;
use crate::config::Config;
use crate::screenshotr::screenshot::Screenshot;

pub struct AppState {
    pub hmac: Hmac,
    pub basic_auth: BasicAuth, 
    pub screenshot: Screenshot, 
    pub allowed_domains: Vec<String>,
    pub replay_protection_validator: ReplayProtectionValidator,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, Error> {
        let hmac = Hmac::new(&config.hmac_secret);
        let basic_auth = BasicAuth::new(config.basic_auth_users);
        let screenshot = Screenshot::new(&config.screenshot)?;
        let allowed_domains = config.allowed_domains;
        let replay_protection_validator = ReplayProtectionValidator::new(config.replay_protection);

        let app_state = Self {
            hmac,
            basic_auth,
            screenshot,
            allowed_domains,
            replay_protection_validator,
        };
        Ok(app_state)
    }
}