//! BazBOM Cryptographic Primitives
//!
//! This crate provides cryptographic operations for BazBOM:
//!
//! - **AEAD Encryption**: ChaCha20-Poly1305 for data encryption
//! - **Key Derivation**: Secure key derivation from passwords
//! - **Hashing**: SHA-256 for integrity checks
//! - **Random Generation**: Cryptographically secure random data
//!
//! # Security Features
//!
//! - AEAD (Authenticated Encryption with Associated Data)
//! - Automatic nonce generation
//! - Secure memory cleanup (zeroize)
//! - Constant-time operations
//!
//! # Example: Encrypt and Decrypt Data
//!
//! ```no_run
//! use bazbom_crypto::encryption::DataEncryptor;
//!
//! # fn example() -> anyhow::Result<()> {
//! // Create encryptor with a key
//! let key = DataEncryptor::generate_key();
//! let encryptor = DataEncryptor::new(&key)?;
//!
//! // Encrypt data
//! let plaintext = b"sensitive data";
//! let ciphertext = encryptor.encrypt(plaintext)?;
//!
//! // Decrypt data
//! let decrypted = encryptor.decrypt(&ciphertext)?;
//! assert_eq!(decrypted, plaintext);
//! # Ok(())
//! # }
//! ```

pub mod encryption;
pub mod hashing;
pub mod random;

pub use encryption::{DataEncryptor, EncryptedData};
pub use hashing::{hash_data, verify_hash};
pub use random::generate_random_bytes;

/// Cryptographic error types
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type CryptoResult<T> = Result<T, CryptoError>;
