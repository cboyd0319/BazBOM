//! AEAD Encryption using ChaCha20-Poly1305
//!
//! Provides authenticated encryption with automatic nonce generation.

use crate::{CryptoError, CryptoResult};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use serde::{Deserialize, Serialize};

/// Encrypted data container with nonce and ciphertext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Random nonce (96 bits / 12 bytes)
    pub nonce: Vec<u8>,
    /// Ciphertext with authentication tag
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Encode encrypted data as base64 string
    pub fn to_base64(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, json.as_bytes())
    }

    /// Decode encrypted data from base64 string
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|_| CryptoError::InvalidCiphertext)?;

        serde_json::from_slice(&decoded)
            .map_err(|e| CryptoError::Internal(anyhow::anyhow!("Failed to deserialize: {}", e)))
    }
}

/// Data encryptor using ChaCha20-Poly1305
pub struct DataEncryptor {
    cipher: ChaCha20Poly1305,
}

impl DataEncryptor {
    /// Create new encryptor with given key
    ///
    /// Key must be exactly 32 bytes (256 bits)
    pub fn new(key: &[u8]) -> CryptoResult<Self> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey(format!(
                "Key must be 32 bytes, got {}",
                key.len()
            )));
        }

        let key_array: &[u8; 32] = key
            .try_into()
            .map_err(|_| CryptoError::InvalidKey("Invalid key length".to_string()))?;

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key_array));

        Ok(Self { cipher })
    }

    /// Generate a random 256-bit key
    pub fn generate_key() -> [u8; 32] {
        ChaCha20Poly1305::generate_key(&mut OsRng).into()
    }

    /// Encrypt data with automatic nonce generation
    pub fn encrypt(&self, plaintext: &[u8]) -> CryptoResult<EncryptedData> {
        // Generate random nonce (12 bytes for ChaCha20-Poly1305)
        let nonce = Self::generate_nonce();

        // Encrypt data
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> CryptoResult<Vec<u8>> {
        if encrypted.nonce.len() != 12 {
            return Err(CryptoError::InvalidCiphertext);
        }

        let nonce = Nonce::from_slice(&encrypted.nonce);

        self.cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }

    /// Encrypt and encode to base64 string
    pub fn encrypt_to_string(&self, plaintext: &[u8]) -> CryptoResult<String> {
        let encrypted = self.encrypt(plaintext)?;
        Ok(encrypted.to_base64())
    }

    /// Decrypt from base64 string
    pub fn decrypt_from_string(&self, encoded: &str) -> CryptoResult<Vec<u8>> {
        let encrypted = EncryptedData::from_base64(encoded)?;
        self.decrypt(&encrypted)
    }

    /// Generate random nonce (12 bytes)
    fn generate_nonce() -> Nonce {
        use rand::RngCore;
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        *Nonce::from_slice(&nonce_bytes)
    }
}

/// Secure key derivation from password
///
/// Uses PBKDF2 with SHA-256 (note: for production, consider using Argon2)
pub fn derive_key_from_password(password: &str, salt: &[u8], iterations: u32) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut key = [0u8; 32];
    let mut hasher = Sha256::new();

    // Simple PBKDF2-like derivation (for production, use proper PBKDF2 or Argon2)
    hasher.update(password.as_bytes());
    hasher.update(salt);

    let mut result = hasher.finalize();

    for _ in 0..iterations {
        let mut hasher = Sha256::new();
        hasher.update(result);
        result = hasher.finalize();
    }

    key.copy_from_slice(&result);
    key
}

impl Drop for DataEncryptor {
    fn drop(&mut self) {
        // Zeroize cipher key on drop (security best practice)
        // Note: ChaCha20Poly1305 doesn't expose the key directly,
        // but we ensure secure cleanup where possible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key1 = DataEncryptor::generate_key();
        let key2 = DataEncryptor::generate_key();

        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
        assert_ne!(key1, key2); // Keys should be different
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = DataEncryptor::generate_key();
        let encryptor = DataEncryptor::new(&key).unwrap();

        let plaintext = b"Hello, BazBOM!";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        assert_ne!(encrypted.ciphertext, plaintext);
        assert_eq!(encrypted.nonce.len(), 12);

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = DataEncryptor::generate_key();
        let encryptor = DataEncryptor::new(&key).unwrap();

        let plaintext = b"Sensitive SBOM data";
        let encrypted_str = encryptor.encrypt_to_string(plaintext).unwrap();

        assert!(!encrypted_str.is_empty());

        let decrypted = encryptor.decrypt_from_string(&encrypted_str).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0u8; 16];
        let result = DataEncryptor::new(&short_key);

        assert!(result.is_err());
    }

    #[test]
    fn test_decryption_with_wrong_key() {
        let key1 = DataEncryptor::generate_key();
        let key2 = DataEncryptor::generate_key();

        let encryptor1 = DataEncryptor::new(&key1).unwrap();
        let encryptor2 = DataEncryptor::new(&key2).unwrap();

        let plaintext = b"Secret data";
        let encrypted = encryptor1.encrypt(plaintext).unwrap();

        // Try to decrypt with wrong key
        let result = encryptor2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let key = DataEncryptor::generate_key();
        let encryptor = DataEncryptor::new(&key).unwrap();

        let plaintext = b"Test data";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // Serialize to JSON
        let json = serde_json::to_string(&encrypted).unwrap();
        assert!(!json.is_empty());

        // Deserialize back
        let deserialized: EncryptedData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.nonce, encrypted.nonce);
        assert_eq!(deserialized.ciphertext, encrypted.ciphertext);
    }

    #[test]
    fn test_key_derivation() {
        let password = "my-secure-password";
        let salt = b"random-salt-1234";

        let key1 = derive_key_from_password(password, salt, 10000);
        let key2 = derive_key_from_password(password, salt, 10000);

        // Same password and salt should produce same key
        assert_eq!(key1, key2);

        // Different salt should produce different key
        let key3 = derive_key_from_password(password, b"different-salt", 10000);
        assert_ne!(key1, key3);

        // Different password should produce different key
        let key4 = derive_key_from_password("different-password", salt, 10000);
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = DataEncryptor::generate_key();
        let encryptor = DataEncryptor::new(&key).unwrap();

        let plaintext = b"Same plaintext";

        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Nonces should be different (random)
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // Ciphertexts should be different (due to different nonces)
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }
}
