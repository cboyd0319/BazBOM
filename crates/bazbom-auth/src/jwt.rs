//! JWT (JSON Web Token) Authentication
//!
//! Provides secure, stateless authentication using RFC 7519 JWTs.
//!
//! # Security Features
//!
//! - Token expiration (default: 24 hours)
//! - Token rotation support
//! - Role-based claims for RBAC
//! - Constant-time validation
//! - HS256 algorithm (HMAC-SHA256)

use crate::rbac::Role;
use crate::{AuthError, AuthResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

/// JWT Claims structure
/// Contains user identity and authorization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID or email)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
    /// Not before time (Unix timestamp)
    pub nbf: i64,
    /// User roles for RBAC
    pub roles: Vec<Role>,
    /// Token ID (for revocation support)
    pub jti: String,
}

impl Claims {
    /// Create new claims for a user with roles
    pub fn new(user_id: &str, roles: Vec<Role>, lifetime: Duration) -> Self {
        let now = OffsetDateTime::now_utc();
        let exp = now + lifetime;

        Self {
            sub: user_id.to_string(),
            exp: exp.unix_timestamp(),
            iat: now.unix_timestamp(),
            nbf: now.unix_timestamp(),
            roles,
            jti: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Check if token has expired
    pub fn is_expired(&self) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.exp < now
    }

    /// Check if token is valid yet (nbf check)
    pub fn is_valid_yet(&self) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.nbf <= now
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }
}

/// JWT Configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    pub secret: String,
    /// Token lifetime (default: 24 hours)
    pub lifetime: Duration,
    /// Issuer name
    pub issuer: Option<String>,
    /// Audience
    pub audience: Option<String>,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(), // Must be set explicitly
            lifetime: Duration::hours(24),
            issuer: Some("BazBOM".to_string()),
            audience: None,
        }
    }
}

/// JWT Authenticator
///
/// Handles token generation and validation
pub struct JwtAuthenticator {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtAuthenticator {
    /// Create new JWT authenticator with secret key
    pub fn new(secret: &str) -> AuthResult<Self> {
        if secret.is_empty() {
            return Err(AuthError::Internal(anyhow::anyhow!(
                "JWT secret cannot be empty"
            )));
        }

        let mut config = JwtConfig::default();
        config.secret = secret.to_string();

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            config,
        })
    }

    /// Create new authenticator with custom configuration
    pub fn with_config(config: JwtConfig) -> AuthResult<Self> {
        if config.secret.is_empty() {
            return Err(AuthError::Internal(anyhow::anyhow!(
                "JWT secret cannot be empty"
            )));
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            config,
        })
    }

    /// Generate JWT token for a user with roles
    pub fn generate_token(&self, user_id: &str, roles: Vec<Role>) -> AuthResult<String> {
        let claims = Claims::new(user_id, roles, self.config.lifetime);

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            AuthError::Internal(anyhow::anyhow!("Failed to encode JWT: {}", e))
        })
    }

    /// Validate JWT token and extract claims
    pub fn validate_token(&self, token: &str) -> AuthResult<Claims> {
        let mut validation = Validation::default();

        // Configure validation
        if let Some(ref issuer) = self.config.issuer {
            validation.set_issuer(&[issuer]);
        }
        if let Some(ref audience) = self.config.audience {
            validation.set_audience(&[audience]);
        }

        // Decode and validate token
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            })?;

        let claims = token_data.claims;

        // Additional validation checks
        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        if !claims.is_valid_yet() {
            return Err(AuthError::InvalidToken("Token not valid yet (nbf)".to_string()));
        }

        Ok(claims)
    }

    /// Refresh a token (generate new token with same user/roles but new expiry)
    pub fn refresh_token(&self, old_token: &str) -> AuthResult<String> {
        let claims = self.validate_token(old_token)?;
        self.generate_token(&claims.sub, claims.roles)
    }

    /// Get token lifetime configuration
    pub fn get_lifetime(&self) -> Duration {
        self.config.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_validation() {
        let auth = JwtAuthenticator::new("test-secret-key-123").unwrap();

        // Generate token
        let token = auth
            .generate_token("alice@example.com", vec![Role::Admin])
            .unwrap();

        // Validate token
        let claims = auth.validate_token(&token).unwrap();

        assert_eq!(claims.sub, "alice@example.com");
        assert!(claims.has_role(&Role::Admin));
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_jwt_invalid_token() {
        let auth = JwtAuthenticator::new("test-secret-key-123").unwrap();

        let result = auth.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_wrong_secret() {
        let auth1 = JwtAuthenticator::new("secret-1").unwrap();
        let auth2 = JwtAuthenticator::new("secret-2").unwrap();

        let token = auth1
            .generate_token("alice@example.com", vec![Role::Admin])
            .unwrap();

        // Should fail with wrong secret
        let result = auth2.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_refresh() {
        let auth = JwtAuthenticator::new("test-secret-key-123").unwrap();

        let token1 = auth
            .generate_token("alice@example.com", vec![Role::Admin])
            .unwrap();

        // Refresh token
        let token2 = auth.refresh_token(&token1).unwrap();

        // Both should be valid
        let claims1 = auth.validate_token(&token1).unwrap();
        let claims2 = auth.validate_token(&token2).unwrap();

        assert_eq!(claims1.sub, claims2.sub);
        assert_eq!(claims1.roles, claims2.roles);
        // JTI should be different (new token)
        assert_ne!(claims1.jti, claims2.jti);
    }

    #[test]
    fn test_empty_secret_rejected() {
        let result = JwtAuthenticator::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_expiration() {
        let claims = Claims::new("user@example.com", vec![Role::User], Duration::seconds(-10));

        assert!(claims.is_expired());
    }
}
