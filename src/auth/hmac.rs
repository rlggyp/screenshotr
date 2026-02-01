use sha2::Sha256;
use hmac::Mac;

pub type HmacSha256 = hmac::Hmac<Sha256>;

pub struct Hmac {
    secret: Vec<u8>
}

impl Hmac {
    pub fn new(secret: &str) -> Self {
        let secret = secret.as_bytes().to_vec();
        Self { secret }
    }

    pub fn verify_payload(&self, payload: &[u8], signature: &str) -> bool {
        let signature = match hex::decode(signature) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let mut mac = HmacSha256::new_from_slice(&self.secret).unwrap();

        mac.update(payload);
        mac.verify_slice(&signature).is_ok()
    }
}