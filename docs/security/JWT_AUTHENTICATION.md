# JWT Authentication Implementation Plan

This document outlines the plan to replace simple bearer tokens with JWT-based authentication for the BazBOM dashboard.

## Current Implementation

Currently, the dashboard uses simple bearer token authentication:
- Token stored in environment variable or OS keyring
- No expiration
- No rotation mechanism
- Non-standard format

## Proposed JWT Implementation

### Features

1. **Token Expiration**: Tokens expire after configured duration (default: 1 hour)
2. **Refresh Tokens**: Long-lived refresh tokens for obtaining new access tokens
3. **Token Rotation**: Automatic token refresh before expiration
4. **Claims**: Standard JWT claims (iss, sub, exp, iat, aud)
5. **Secure Storage**: Refresh tokens stored in OS keychain

### Implementation Steps

#### 1. Add JWT Dependencies

```toml
# crates/bazbom-dashboard/Cargo.toml
[dependencies]
jsonwebtoken = "9"
serde = { version = "1.0", features = ["derive"] }
chrono = "0.4"
```

#### 2. Create JWT Module

```rust
// crates/bazbom-dashboard/src/auth/mod.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
    pub aud: String,  // Audience (bazbom-dashboard)
    pub iss: String,  // Issuer (bazbom)
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_duration: Duration::hours(1),
            refresh_token_duration: Duration::days(30),
        }
    }

    pub fn generate_access_token(&self, user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            exp: (now + self.access_token_duration).timestamp() as usize,
            iat: now.timestamp() as usize,
            aud: "bazbom-dashboard".to_string(),
            iss: "bazbom".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}
```

#### 3. Update Authentication Middleware

```rust
// crates/bazbom-dashboard/src/lib.rs
async fn jwt_auth_middleware(
    State(jwt_manager): State<Arc<JwtManager>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_value) = auth_header {
        if auth_value.starts_with("Bearer ") {
            let token = &auth_value[7..];

            match jwt_manager.verify_token(token) {
                Ok(claims) => {
                    // Token is valid, proceed with request
                    // Could add claims to request extensions for use in handlers
                    return Ok(next.run(req).await);
                }
                Err(e) => {
                    eprintln!("JWT verification failed: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
```

#### 4. Add Token Refresh Endpoint

```rust
// POST /api/auth/refresh
pub async fn refresh_token(
    State(jwt_manager): State<Arc<JwtManager>>,
    Json(refresh_req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, StatusCode> {
    // Verify refresh token
    match jwt_manager.verify_token(&refresh_req.refresh_token) {
        Ok(claims) => {
            // Generate new access token
            let new_access_token = jwt_manager
                .generate_access_token(&claims.sub)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(TokenResponse {
                access_token: new_access_token,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
            }))
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
```

#### 5. Client-Side Token Management

For CLI clients:

```rust
// Auto-refresh before expiration
pub struct TokenManager {
    jwt_manager: Arc<JwtManager>,
    access_token: Mutex<Option<String>>,
    refresh_token: String,
}

impl TokenManager {
    pub async fn get_valid_token(&self) -> Result<String> {
        let mut token = self.access_token.lock().unwrap();

        // Check if token needs refresh (e.g., <5 minutes until expiration)
        if self.needs_refresh(&token) {
            let new_token = self.refresh().await?;
            *token = Some(new_token.clone());
            Ok(new_token)
        } else {
            Ok(token.clone().unwrap())
        }
    }

    async fn refresh(&self) -> Result<String> {
        // Call /api/auth/refresh with refresh_token
        // ...
    }
}
```

### Security Considerations

1. **Secret Management**: JWT secret stored in OS keychain or environment
2. **Token Rotation**: Refresh tokens rotated on each use
3. **Revocation**: Implement token blacklist for logout
4. **HTTPS**: Require HTTPS in production
5. **Claims Validation**: Validate aud, iss, exp claims
6. **Rate Limiting**: Limit refresh token usage (already implemented)

### Migration Path

1. **Phase 1**: Add JWT support alongside existing bearer tokens
2. **Phase 2**: Deprecate simple bearer tokens (warning messages)
3. **Phase 3**: Remove simple bearer token support

### Testing

```rust
#[tokio::test]
async fn test_jwt_authentication() {
    let jwt_manager = JwtManager::new("test-secret");
    let token = jwt_manager.generate_access_token("user123").unwrap();

    let claims = jwt_manager.verify_token(&token).unwrap();
    assert_eq!(claims.sub, "user123");
}

#[tokio::test]
async fn test_expired_token() {
    // Test with expired token
    let jwt_manager = JwtManager::new("test-secret");
    let mut claims = create_test_claims();
    claims.exp = (Utc::now() - Duration::hours(1)).timestamp() as usize;

    let token = encode(&Header::default(), &claims, &jwt_manager.encoding_key).unwrap();
    assert!(jwt_manager.verify_token(&token).is_err());
}
```

## References

- [JWT.io](https://jwt.io/)
- [jsonwebtoken crate](https://docs.rs/jsonwebtoken/)
- [RFC 7519](https://tools.ietf.org/html/rfc7519) - JWT specification
- [RFC 8725](https://tools.ietf.org/html/rfc8725) - JWT Best Practices
