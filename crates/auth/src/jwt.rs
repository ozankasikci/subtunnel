use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use subtunnel_shared::models::Plan;
use thiserror::Error;

const ISSUER: &str = "subtunnel";
const ACCESS_TOKEN_DURATION_SECS: i64 = 15 * 60; // 15 minutes
const REFRESH_TOKEN_DURATION_SECS: i64 = 7 * 24 * 3600; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user ID
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Expiration (Unix timestamp)
    pub exp: u64,
    /// Issued at
    pub iat: u64,
    /// Token type
    pub typ: TokenType,
    /// User's plan
    pub plan: Plan,
    /// Scopes
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT encoding error: {0}")]
    Encode(#[from] jsonwebtoken::errors::Error),
    #[error("token expired")]
    Expired,
    #[error("invalid token type: expected {expected:?}, got {got:?}")]
    WrongType { expected: TokenType, got: TokenType },
}

/// Keys used for JWT signing and verification.
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    /// Create from an HMAC secret.
    pub fn from_secret(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    /// Generate an access token.
    pub fn generate_access_token(
        &self,
        user_id: &str,
        plan: Plan,
        scopes: Vec<String>,
    ) -> Result<String, JwtError> {
        let now = Utc::now().timestamp() as u64;
        let claims = Claims {
            sub: user_id.to_string(),
            iss: ISSUER.to_string(),
            exp: now + ACCESS_TOKEN_DURATION_SECS as u64,
            iat: now,
            typ: TokenType::Access,
            plan,
            scopes,
        };
        Ok(encode(&Header::default(), &claims, &self.encoding)?)
    }

    /// Generate a refresh token.
    pub fn generate_refresh_token(
        &self,
        user_id: &str,
        plan: Plan,
    ) -> Result<String, JwtError> {
        let now = Utc::now().timestamp() as u64;
        let claims = Claims {
            sub: user_id.to_string(),
            iss: ISSUER.to_string(),
            exp: now + REFRESH_TOKEN_DURATION_SECS as u64,
            iat: now,
            typ: TokenType::Refresh,
            plan,
            scopes: vec![],
        };
        Ok(encode(&Header::default(), &claims, &self.encoding)?)
    }

    /// Validate a token and return claims.
    pub fn validate(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[ISSUER]);
        let data: TokenData<Claims> = decode(token, &self.decoding, &validation)?;
        Ok(data.claims)
    }

    /// Validate and ensure it's an access token.
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate(token)?;
        if claims.typ != TokenType::Access {
            return Err(JwtError::WrongType {
                expected: TokenType::Access,
                got: claims.typ,
            });
        }
        Ok(claims)
    }

    /// Validate and ensure it's a refresh token.
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate(token)?;
        if claims.typ != TokenType::Refresh {
            return Err(JwtError::WrongType {
                expected: TokenType::Refresh,
                got: claims.typ,
            });
        }
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_token_roundtrip() {
        let keys = JwtKeys::from_secret(b"test-secret-key-at-least-32-bytes!");
        let token = keys
            .generate_access_token("usr_123", Plan::Pro, vec!["tunnels:create".into()])
            .unwrap();
        let claims = keys.validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, "usr_123");
        assert_eq!(claims.plan, Plan::Pro);
        assert_eq!(claims.typ, TokenType::Access);
        assert_eq!(claims.scopes, vec!["tunnels:create"]);
    }

    #[test]
    fn test_refresh_token_roundtrip() {
        let keys = JwtKeys::from_secret(b"test-secret-key-at-least-32-bytes!");
        let token = keys.generate_refresh_token("usr_456", Plan::Free).unwrap();
        let claims = keys.validate_refresh_token(&token).unwrap();
        assert_eq!(claims.sub, "usr_456");
        assert_eq!(claims.typ, TokenType::Refresh);
    }

    #[test]
    fn test_wrong_token_type() {
        let keys = JwtKeys::from_secret(b"test-secret-key-at-least-32-bytes!");
        let token = keys.generate_refresh_token("usr_123", Plan::Free).unwrap();
        assert!(keys.validate_access_token(&token).is_err());
    }
}
