//! Dashboard middleware for authentication, rate limiting, and security

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use bazbom_auth::{
    jwt::JwtAuthenticator, rbac::Permission, rbac::Role, AuditEvent, AuditEventType,
    AuditLogger, AuditResult,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Application state with auth and rate limiting
#[derive(Clone)]
pub struct AppState {
    /// Path to BazBOM cache directory
    pub cache_dir: std::path::PathBuf,
    /// Path to project root
    pub project_root: std::path::PathBuf,
    /// JWT authenticator (optional - if None, auth is disabled)
    pub jwt_auth: Option<Arc<JwtAuthenticator>>,
    /// Rate limiter for API endpoints
    pub rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Audit logger (optional)
    pub audit_logger: Option<Arc<AuditLogger>>,
}

impl AppState {
    /// Create new application state with defaults
    pub fn new(
        cache_dir: std::path::PathBuf,
        project_root: std::path::PathBuf,
    ) -> anyhow::Result<Self> {
        // Create rate limiter: 100 requests per minute per IP
        let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self {
            cache_dir,
            project_root,
            jwt_auth: None,
            rate_limiter,
            audit_logger: None,
        })
    }

    /// Enable JWT authentication
    pub fn with_jwt_auth(mut self, jwt_secret: &str) -> anyhow::Result<Self> {
        self.jwt_auth = Some(Arc::new(JwtAuthenticator::new(jwt_secret)?));
        Ok(self)
    }

    /// Enable audit logging
    pub fn with_audit_logging(mut self, logger: AuditLogger) -> Self {
        self.audit_logger = Some(Arc::new(logger));
        self
    }
}

/// JWT Authentication middleware
///
/// Validates Bearer tokens using JWT and extracts user claims
pub async fn jwt_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no JWT auth is configured, skip authentication (localhost-only mode)
    let Some(ref jwt_auth) = state.jwt_auth else {
        return Ok(next.run(req).await);
    };

    // Skip authentication for health check endpoint
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let Some(auth_value) = auth_header else {
        // Log failed auth attempt
        if let Some(ref audit) = state.audit_logger {
            let _ = audit.log(AuditEvent::new(
                AuditEventType::Authentication,
                "anonymous",
                "api_access",
                req.uri().path(),
                AuditResult::Failure("No authorization header".to_string()),
            ));
        }
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Extract token from "Bearer <token>"
    let Some(token) = auth_value.strip_prefix("Bearer ") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Validate JWT token
    let claims = jwt_auth.validate_token(token).map_err(|e| {
        // Log failed auth attempt
        if let Some(ref audit) = state.audit_logger {
            let _ = audit.log(AuditEvent::new(
                AuditEventType::Authentication,
                "unknown",
                "api_access",
                req.uri().path(),
                AuditResult::Failure(format!("Invalid token: {}", e)),
            ));
        }
        StatusCode::UNAUTHORIZED
    })?;

    // Log successful auth
    if let Some(ref audit) = state.audit_logger {
        let _ = audit.log(AuditEvent::new(
            AuditEventType::Authentication,
            &claims.sub,
            "api_access",
            req.uri().path(),
            AuditResult::Success,
        ));
    }

    // Store claims in request extensions for use in handlers
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Rate limiting middleware
///
/// Limits requests to prevent DoS attacks
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check rate limit
    match state.rate_limiter.check() {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => {
            // Log rate limit exceeded
            if let Some(ref audit) = state.audit_logger {
                let _ = audit.log(AuditEvent::new(
                    AuditEventType::SecurityEvent,
                    "system",
                    "rate_limit_exceeded",
                    req.uri().path(),
                    AuditResult::Failure("Rate limit exceeded".to_string()),
                ));
            }
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

/// Permission check middleware (requires JWT auth)
///
/// Checks if user has required permission for endpoint
pub fn require_permission(required: Permission) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req: Request, next: Next| {
        let permission = required.clone();
        Box::pin(async move {
            // Get claims from request extensions (set by jwt_auth_middleware)
            let claims = req
                .extensions()
                .get::<bazbom_auth::jwt::Claims>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // Check if user has required permission
            let has_permission = claims
                .roles
                .iter()
                .any(|role| role.has_permission(&permission));

            if !has_permission {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bazbom_auth::jwt::JwtAuthenticator;

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new(
            std::path::PathBuf::from(".bazbom/cache"),
            std::path::PathBuf::from("."),
        )
        .unwrap();

        assert!(state.jwt_auth.is_none());
        assert!(state.audit_logger.is_none());
    }

    #[tokio::test]
    async fn test_app_state_with_jwt() {
        let state = AppState::new(
            std::path::PathBuf::from(".bazbom/cache"),
            std::path::PathBuf::from("."),
        )
        .unwrap()
        .with_jwt_auth("test-secret-key")
        .unwrap();

        assert!(state.jwt_auth.is_some());
    }

    #[test]
    fn test_jwt_token_generation() {
        let auth = JwtAuthenticator::new("test-secret").unwrap();
        let token = auth
            .generate_token("alice@example.com", vec![Role::Admin])
            .unwrap();

        assert!(!token.is_empty());

        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "alice@example.com");
        assert!(claims.has_role(&Role::Admin));
    }
}
