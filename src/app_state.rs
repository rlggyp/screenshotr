use crate::auth::basic_auth::BasicAuth;
use crate::auth::hmac::Hmac;
use crate::config::Config;

pub struct AppState {
    pub hmac: Hmac,
    pub basic_auth: BasicAuth, 
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let hmac = Hmac::new(&config.hmac_secret);
        let basic_auth = BasicAuth::new(config.basic_auth_users);

        Self { hmac, basic_auth }
    }
}