//! Cryptographically secure random data generation

use rand::RngCore;

/// Generate cryptographically secure random bytes
pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    bytes
}

/// Generate random hex string
pub fn generate_random_hex(len: usize) -> String {
    let bytes = generate_random_bytes(len);
    hex::encode(bytes)
}

/// Generate random alphanumeric string
pub fn generate_random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_bytes() {
        let bytes1 = generate_random_bytes(32);
        let bytes2 = generate_random_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_generate_random_hex() {
        let hex1 = generate_random_hex(16);
        let hex2 = generate_random_hex(16);

        assert_eq!(hex1.len(), 32); // 16 bytes = 32 hex chars
        assert_eq!(hex2.len(), 32);
        assert_ne!(hex1, hex2);

        // Should be valid hex
        assert!(hex1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_random_string() {
        let str1 = generate_random_string(20);
        let str2 = generate_random_string(20);

        assert_eq!(str1.len(), 20);
        assert_eq!(str2.len(), 20);
        assert_ne!(str1, str2);

        // Should be alphanumeric
        assert!(str1.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
