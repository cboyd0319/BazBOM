//! BazBOM Authentication and Authorization
//!
//! This crate provides enterprise-grade authentication and authorization for BazBOM:
//!
//! - **JWT Authentication**: Secure, expiring tokens with role-based claims
//! - **API Key Management**: Long-lived keys with scopes for CI/CD
//! - **RBAC**: Role-based access control with fine-grained permissions
//! - **Audit Logging**: Comprehensive security event logging
//! - **Secret Management**: OS keychain integration for secure credential storage
//!
//! # Security Features
//!
//! - Constant-time comparisons to prevent timing attacks
//! - bcrypt password hashing for API keys
//! - Token expiration and rotation support
//! - Configurable token lifetime
//! - Secure secret storage in OS keychain
//!
//! # Example: JWT Authentication
//!
//! ```no_run
//! use bazbom_auth::jwt::{JwtAuthenticator, Claims};
//! use bazbom_auth::rbac::Role;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create authenticator with secret
//! let auth = JwtAuthenticator::new("super-secret-key")?;
//!
//! // Generate token for user
//! let token = auth.generate_token("alice@example.com", vec![Role::Admin])?;
//!
//! // Validate token
//! let claims = auth.validate_token(&token)?;
//! assert_eq!(claims.sub, "alice@example.com");
//! # Ok(())
//! # }
//! ```

pub mod api_key;
pub mod audit;
pub mod jwt;
pub mod rbac;
pub mod secrets;

pub use api_key::{ApiKey, ApiKeyManager};
pub use audit::{AuditEvent, AuditEventType, AuditLogger, AuditResult};
pub use jwt::{Claims, JwtAuthenticator, JwtConfig};
pub use rbac::{Permission, Role};
pub use secrets::SecretManager;

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Permission denied: required {0:?}")]
    PermissionDenied(Permission),

    #[error("API key not found: {0}")]
    ApiKeyNotFound(String),

    #[error("API key expired")]
    ApiKeyExpired,

    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type AuthResult<T> = Result<T, AuthError>;
