//! Token-based authentication for agent connections.
//!
//! Uses constant-time comparison to prevent timing side-channel attacks.

/// Authenticator that validates agent tokens against a shared secret.
#[derive(Debug, Clone)]
pub struct Authenticator {
    /// The expected token. If `None`, authentication is disabled (all tokens accepted).
    expected_token: Option<String>,
}

impl Authenticator {
    /// Create an authenticator that requires the given token.
    pub fn new(token: String) -> Self {
        Self {
            expected_token: Some(token),
        }
    }

    /// Create an authenticator that accepts any token (no auth).
    pub fn allow_all() -> Self {
        Self {
            expected_token: None,
        }
    }

    /// Validate a token from an agent.
    ///
    /// Returns `true` if the token matches (or auth is disabled).
    /// Uses constant-time comparison to prevent timing attacks.
    pub fn validate(&self, token: &str) -> bool {
        match &self.expected_token {
            None => true,
            Some(expected) => constant_time_eq(expected.as_bytes(), token.as_bytes()),
        }
    }
}

/// Constant-time byte slice comparison.
///
/// Returns `true` iff both slices have the same length and identical contents.
/// The comparison time depends only on the length of `a`, not on where the first
/// difference occurs, preventing timing side-channel attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_token() {
        let auth = Authenticator::new("secret-token-123".into());
        assert!(auth.validate("secret-token-123"));
    }

    #[test]
    fn wrong_token() {
        let auth = Authenticator::new("secret-token-123".into());
        assert!(!auth.validate("wrong-token"));
    }

    #[test]
    fn wrong_length() {
        let auth = Authenticator::new("abc".into());
        assert!(!auth.validate("ab"));
        assert!(!auth.validate("abcd"));
    }

    #[test]
    fn empty_token() {
        let auth = Authenticator::new("secret".into());
        assert!(!auth.validate(""));
    }

    #[test]
    fn allow_all_accepts_anything() {
        let auth = Authenticator::allow_all();
        assert!(auth.validate("anything"));
        assert!(auth.validate(""));
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(constant_time_eq(b"", b""));
    }
}
