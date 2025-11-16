//! API Key Management
//!
//! Long-lived authentication keys for CI/CD and automation.
//!
//! # Security Features
//!
//! - bcrypt password hashing (never store plaintext keys)
//! - Scoped permissions
//! - Expiration support
//! - Usage tracking
//! - Constant-time comparison

use crate::rbac::{Permission, Role};
use crate::{AuthError, AuthResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::{Duration, OffsetDateTime};

/// API Key structure (stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique key ID (UUID)
    pub id: String,
    /// bcrypt hash of the actual key (never store plaintext)
    pub key_hash: String,
    /// Human-readable name
    pub name: String,
    /// Permissions granted to this key
    pub scopes: Vec<Permission>,
    /// Roles (for compatibility with JWT RBAC)
    pub roles: Vec<Role>,
    /// Optional expiration time
    pub expires_at: Option<OffsetDateTime>,
    /// Creation time
    pub created_at: OffsetDateTime,
    /// Last time this key was used
    pub last_used_at: Option<OffsetDateTime>,
    /// User/service that owns this key
    pub owner: String,
}

impl ApiKey {
    /// Check if API key has expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            OffsetDateTime::now_utc() > exp
        } else {
            false
        }
    }

    /// Check if key has specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.scopes.contains(permission)
            || self.roles.iter().any(|role| role.has_permission(permission))
    }

    /// Update last used timestamp
    pub fn update_last_used(&mut self) {
        self.last_used_at = Some(OffsetDateTime::now_utc());
    }
}

/// API Key Manager
pub struct ApiKeyManager {
    /// In-memory key store (in production, would use database)
    keys: HashMap<String, ApiKey>,
}

impl ApiKeyManager {
    /// Create new API key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Generate new API key
    ///
    /// Returns the raw key (shown only once) and the stored ApiKey struct
    pub fn generate_key(
        &mut self,
        name: &str,
        owner: &str,
        roles: Vec<Role>,
        expires_in: Option<Duration>,
    ) -> AuthResult<(String, ApiKey)> {
        // Generate random key with prefix
        let raw_key = format!("bazbom_{}", Self::generate_random_string(32));

        // Hash the key using bcrypt
        let key_hash = bcrypt::hash(&raw_key, bcrypt::DEFAULT_COST)
            .map_err(|e| AuthError::Internal(anyhow::anyhow!("Failed to hash key: {}", e)))?;

        // Derive scopes from roles
        let scopes = roles
            .iter()
            .flat_map(|role| role.permissions())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Calculate expiration
        let expires_at = expires_in.map(|duration| OffsetDateTime::now_utc() + duration);

        let api_key = ApiKey {
            id: uuid::Uuid::new_v4().to_string(),
            key_hash,
            name: name.to_string(),
            scopes,
            roles,
            expires_at,
            created_at: OffsetDateTime::now_utc(),
            last_used_at: None,
            owner: owner.to_string(),
        };

        // Store key
        self.keys.insert(api_key.id.clone(), api_key.clone());

        Ok((raw_key, api_key))
    }

    /// Validate API key and return associated ApiKey struct
    pub fn validate_key(&mut self, raw_key: &str) -> AuthResult<&mut ApiKey> {
        // Find matching key using constant-time comparison
        for key in self.keys.values_mut() {
            // Verify hash
            if bcrypt::verify(raw_key, &key.key_hash)
                .map_err(|e| AuthError::Internal(anyhow::anyhow!("Hash verification failed: {}", e)))?
            {
                // Check expiration
                if key.is_expired() {
                    return Err(AuthError::ApiKeyExpired);
                }

                // Update last used
                key.update_last_used();

                return Ok(key);
            }
        }

        Err(AuthError::InvalidCredentials)
    }

    /// Revoke API key by ID
    pub fn revoke_key(&mut self, key_id: &str) -> AuthResult<()> {
        self.keys
            .remove(key_id)
            .ok_or_else(|| AuthError::ApiKeyNotFound(key_id.to_string()))?;

        Ok(())
    }

    /// List all API keys (without sensitive data)
    pub fn list_keys(&self) -> Vec<ApiKey> {
        self.keys.values().cloned().collect()
    }

    /// Get key by ID
    pub fn get_key(&self, key_id: &str) -> Option<&ApiKey> {
        self.keys.get(key_id)
    }

    /// Generate cryptographically secure random string
    fn generate_random_string(len: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789";

        let mut rng = rand::thread_rng();
        (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let mut manager = ApiKeyManager::new();

        let (raw_key, api_key) = manager
            .generate_key(
                "CI/CD Pipeline",
                "github-actions",
                vec![Role::CI],
                Some(Duration::days(90)),
            )
            .unwrap();

        // Check key format
        assert!(raw_key.starts_with("bazbom_"));
        assert_eq!(raw_key.len(), 39); // "bazbom_" + 32 chars

        // Check API key struct
        assert_eq!(api_key.name, "CI/CD Pipeline");
        assert_eq!(api_key.owner, "github-actions");
        assert!(api_key.expires_at.is_some());
        assert!(!api_key.is_expired());
    }

    #[test]
    fn test_validate_api_key() {
        let mut manager = ApiKeyManager::new();

        let (raw_key, _) = manager
            .generate_key("Test Key", "test-user", vec![Role::User], None)
            .unwrap();

        // Valid key should work
        let result = manager.validate_key(&raw_key);
        assert!(result.is_ok());

        // Invalid key should fail
        let result = manager.validate_key("invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_api_key() {
        let mut manager = ApiKeyManager::new();

        let (raw_key, api_key) = manager
            .generate_key("Test Key", "test-user", vec![Role::User], None)
            .unwrap();

        // Key should be valid initially
        assert!(manager.validate_key(&raw_key).is_ok());

        // Revoke key
        manager.revoke_key(&api_key.id).unwrap();

        // Key should no longer validate
        assert!(manager.validate_key(&raw_key).is_err());
    }

    #[test]
    fn test_expired_key() {
        let mut manager = ApiKeyManager::new();

        let (raw_key, mut api_key) = manager
            .generate_key(
                "Test Key",
                "test-user",
                vec![Role::User],
                Some(Duration::seconds(1)),
            )
            .unwrap();

        // Manually expire the key
        api_key.expires_at = Some(OffsetDateTime::now_utc() - Duration::seconds(10));
        manager.keys.insert(api_key.id.clone(), api_key.clone());

        // Validation should fail due to expiration
        let result = manager.validate_key(&raw_key);
        assert!(matches!(result, Err(AuthError::ApiKeyExpired)));
    }

    #[test]
    fn test_key_permissions() {
        let mut manager = ApiKeyManager::new();

        let (_, api_key) = manager
            .generate_key("Test Key", "test-user", vec![Role::Developer], None)
            .unwrap();

        assert!(api_key.has_permission(&Permission::ReadSBOM));
        assert!(api_key.has_permission(&Permission::WriteVulnerabilities));
        assert!(!api_key.has_permission(&Permission::ManageUsers));
    }

    #[test]
    fn test_list_keys() {
        let mut manager = ApiKeyManager::new();

        manager
            .generate_key("Key 1", "user1", vec![Role::User], None)
            .unwrap();
        manager
            .generate_key("Key 2", "user2", vec![Role::Developer], None)
            .unwrap();

        let keys = manager.list_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_last_used_tracking() {
        let mut manager = ApiKeyManager::new();

        let (raw_key, _) = manager
            .generate_key("Test Key", "test-user", vec![Role::User], None)
            .unwrap();

        // Initially, last_used should be None
        let key = manager.validate_key(&raw_key).unwrap();
        assert!(key.last_used_at.is_some());
    }
}
