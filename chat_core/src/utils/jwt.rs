use crate::User;
use jwt_simple::prelude::*;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7; // 1 week
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: User) -> Result<String, jwt_simple::Error> {
        let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);
        self.0.sign(claims)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let opst = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };

        let claims = self.0.verify_token::<User>(token, None)?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_sign_verify_should_work() {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");

        let ek = EncodingKey::load(encoding_pem).unwrap();
        let dk = DecodingKey::load(decoding_pem).unwrap();

        let user = User::new(1, "test", "test@qq.com");

        let token = ek.sign(user.clone()).unwrap();

        let claims = dk.verify(&token).unwrap();

        assert_eq!(user, claims);
    }
}
