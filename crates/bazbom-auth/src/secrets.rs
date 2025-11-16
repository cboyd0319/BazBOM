//! Secret Management
//!
//! Secure credential storage using OS keychain integration.
//!
//! # Platform Support
//!
//! - **macOS**: Keychain
//! - **Windows**: Credential Manager
//! - **Linux**: Secret Service (GNOME Keyring, KWallet)
//!
//! # Fallback
//!
//! Falls back to environment variables if OS keychain is unavailable.

use crate::{AuthError, AuthResult};
use keyring::Entry;
use std::collections::HashMap;

/// Secret manager for secure credential storage
pub struct SecretManager {
    /// Service name for keyring entries
    service_name: &'static str,
}

impl SecretManager {
    /// Create new secret manager
    pub fn new() -> Self {
        Self {
            service_name: "bazbom",
        }
    }

    /// Create secret manager with custom service name
    pub fn with_service_name(service_name: &'static str) -> Self {
        Self { service_name }
    }

    /// Store secret in OS keychain
    pub fn store_secret(&self, key: &str, value: &str) -> AuthResult<()> {
        let entry = Entry::new(self.service_name, key).map_err(|e| {
            AuthError::Internal(anyhow::anyhow!("Failed to create keyring entry: {}", e))
        })?;

        entry
            .set_password(value)
            .map_err(|e| AuthError::Internal(anyhow::anyhow!("Failed to store secret: {}", e)))?;

        Ok(())
    }

    /// Retrieve secret from OS keychain
    pub fn get_secret(&self, key: &str) -> AuthResult<String> {
        let entry = Entry::new(self.service_name, key).map_err(|e| {
            AuthError::Internal(anyhow::anyhow!("Failed to create keyring entry: {}", e))
        })?;

        entry
            .get_password()
            .map_err(|_| AuthError::SecretNotFound(key.to_string()))
    }

    /// Delete secret from OS keychain
    pub fn delete_secret(&self, key: &str) -> AuthResult<()> {
        let entry = Entry::new(self.service_name, key).map_err(|e| {
            AuthError::Internal(anyhow::anyhow!("Failed to create keyring entry: {}", e))
        })?;

        // keyring 3.x uses delete_credential instead of delete_password
        entry
            .delete_credential()
            .map_err(|e| AuthError::Internal(anyhow::anyhow!("Failed to delete secret: {}", e)))?;

        Ok(())
    }

    /// Check if secret exists in keychain
    pub fn has_secret(&self, key: &str) -> bool {
        self.get_secret(key).is_ok()
    }

    /// List all secret keys for this service (not all platforms support this)
    pub fn list_secrets(&self) -> Vec<String> {
        // Note: keyring crate doesn't provide a list function
        // This would need to be implemented with platform-specific code
        // For now, return empty vec
        Vec::new()
    }

    /// Get secret with fallback to environment variable
    pub fn get_secret_with_env_fallback(&self, key: &str) -> AuthResult<String> {
        // Try keychain first
        if let Ok(secret) = self.get_secret(key) {
            return Ok(secret);
        }

        // Fall back to environment variable
        std::env::var(key).map_err(|_| AuthError::SecretNotFound(key.to_string()))
    }

    /// Store multiple secrets at once
    pub fn store_secrets(&self, secrets: &HashMap<String, String>) -> AuthResult<()> {
        for (key, value) in secrets {
            self.store_secret(key, value)?;
        }
        Ok(())
    }

    /// Delete multiple secrets at once
    pub fn delete_secrets(&self, keys: &[String]) -> AuthResult<()> {
        for key in keys {
            // Ignore errors for non-existent keys
            let _ = self.delete_secret(key);
        }
        Ok(())
    }

    /// Migrate secret from environment variable to keychain
    pub fn migrate_from_env(&self, key: &str) -> AuthResult<bool> {
        // Check if already in keychain
        if self.has_secret(key) {
            return Ok(false); // Already migrated
        }

        // Try to get from environment variable
        if let Ok(value) = std::env::var(key) {
            self.store_secret(key, &value)?;
            return Ok(true); // Successfully migrated
        }

        Ok(false) // Not found in env
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Common secret keys used in BazBOM
pub mod keys {
    pub const DASHBOARD_TOKEN: &str = "BAZBOM_DASHBOARD_TOKEN";
    pub const GITHUB_TOKEN: &str = "BAZBOM_GITHUB_TOKEN";
    pub const JWT_SECRET: &str = "BAZBOM_JWT_SECRET";
    pub const AUDIT_HMAC_KEY: &str = "BAZBOM_AUDIT_HMAC_KEY";
    pub const API_KEY_SALT: &str = "BAZBOM_API_KEY_SALT";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_manager_creation() {
        let manager = SecretManager::new();
        assert_eq!(manager.service_name, "bazbom");
    }

    #[test]
    fn test_custom_service_name() {
        let manager = SecretManager::with_service_name("bazbom-test");
        assert_eq!(manager.service_name, "bazbom-test");
    }

    // Note: The following tests require OS keychain access and may fail in CI environments

    #[test]
    #[ignore] // Ignore by default as it requires OS keychain
    fn test_store_and_retrieve_secret() {
        let manager = SecretManager::with_service_name("bazbom-test");

        let key = "test-secret-key";
        let value = "super-secret-value-123";

        // Store secret
        manager.store_secret(key, value).unwrap();

        // Retrieve secret
        let retrieved = manager.get_secret(key).unwrap();
        assert_eq!(retrieved, value);

        // Clean up
        manager.delete_secret(key).unwrap();
    }

    #[test]
    #[ignore]
    fn test_delete_secret() {
        let manager = SecretManager::with_service_name("bazbom-test");

        let key = "test-delete-key";
        let value = "value";

        // Store secret
        manager.store_secret(key, value).unwrap();

        // Verify it exists
        assert!(manager.has_secret(key));

        // Delete secret
        manager.delete_secret(key).unwrap();

        // Verify it's gone
        assert!(!manager.has_secret(key));
    }

    #[test]
    fn test_env_fallback() {
        let manager = SecretManager::new();

        std::env::set_var("BAZBOM_TEST_ENV_SECRET", "env-value-123");

        let value = manager
            .get_secret_with_env_fallback("BAZBOM_TEST_ENV_SECRET")
            .unwrap();

        assert_eq!(value, "env-value-123");

        std::env::remove_var("BAZBOM_TEST_ENV_SECRET");
    }

    #[test]
    fn test_secret_not_found() {
        let manager = SecretManager::new();

        let result = manager.get_secret("non-existent-key-xyz");
        assert!(result.is_err());
        assert!(matches!(result, Err(AuthError::SecretNotFound(_))));
    }

    #[test]
    #[ignore]
    fn test_store_multiple_secrets() {
        let manager = SecretManager::with_service_name("bazbom-test");

        let mut secrets = HashMap::new();
        secrets.insert("key1".to_string(), "value1".to_string());
        secrets.insert("key2".to_string(), "value2".to_string());

        manager.store_secrets(&secrets).unwrap();

        assert_eq!(manager.get_secret("key1").unwrap(), "value1");
        assert_eq!(manager.get_secret("key2").unwrap(), "value2");

        // Clean up
        manager
            .delete_secrets(&["key1".to_string(), "key2".to_string()])
            .unwrap();
    }
}
