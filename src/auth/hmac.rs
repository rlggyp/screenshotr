use sha2::{Digest, Sha256};
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

    pub fn verify_payload(
        &self,
        body: &[u8],
        timestamp: &str,
        nonce: &str,
        signature: &str
    ) -> bool {
        let signature = match hex::decode(signature) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let body_hash = Sha256::digest(body);
        let body_hash_hex = hex::encode(body_hash);

        let canonical_message = format!(
            "{}\n{}\n{}",
            timestamp,
            nonce,
            body_hash_hex
        );

        let mut mac = HmacSha256::new_from_slice(&self.secret).unwrap();

        mac.update(canonical_message.as_bytes());
        mac.verify_slice(&signature).is_ok()
    }
}