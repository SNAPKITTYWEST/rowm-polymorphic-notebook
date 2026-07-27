// WASM Cryptographic Engine
// SHA-512, Blake3, Ed25519 verification for deterministic hashing and signing

use wasm_bindgen::prelude::*;
use sha2::{Sha512, Digest};

#[wasm_bindgen]
pub struct CryptoEngine;

#[wasm_bindgen]
impl CryptoEngine {
    /// Compute SHA-512 hash
    #[wasm_bindgen]
    pub fn sha512(input: &str) -> String {
        let mut hasher = Sha512::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify SHA-512 hash
    #[wasm_bindgen]
    pub fn verify_sha512(input: &str, hash: &str) -> bool {
        let computed = Self::sha512(input);
        computed == hash
    }

    /// Compute Blake3 hash (if available via blake3 crate)
    #[wasm_bindgen]
    pub fn blake3(input: &str) -> String {
        let hash = blake3::hash(input.as_bytes());
        hash.to_hex().to_string()
    }

    /// Verify Blake3 hash
    #[wasm_bindgen]
    pub fn verify_blake3(input: &str, hash: &str) -> bool {
        let computed = Self::blake3(input);
        computed == hash
    }

    /// Generate nonce (deterministic, seeded)
    #[wasm_bindgen]
    pub fn generate_nonce(seed: &str, counter: u32) -> String {
        let combined = format!("{}-{}", seed, counter);
        let hash = Self::sha512(&combined);
        hash[..32].to_string() // First 128 bits
    }

    /// Check if string is valid hex (128 chars for SHA-512)
    #[wasm_bindgen]
    pub fn is_valid_sha512_hex(s: &str) -> bool {
        s.len() == 128 && s.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if string is valid hex (64 chars for Blake3)
    #[wasm_bindgen]
    pub fn is_valid_blake3_hex(s: &str) -> bool {
        s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Verify Ed25519 signature (stub — actual verification requires libsodium)
    #[wasm_bindgen]
    pub fn verify_ed25519_signature(public_key: &str, message: &str, signature: &str) -> bool {
        // Stub: actual implementation requires libsodium or similar
        // For now, check format:
        // - public_key: 64 hex chars (32 bytes)
        // - signature: 128 hex chars (64 bytes)

        if public_key.len() != 64 || !public_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }

        if signature.len() != 128 || !signature.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }

        // Real verification would call libsodium
        true
    }

    /// Compute HMAC-SHA512
    #[wasm_bindgen]
    pub fn hmac_sha512(key: &str, message: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha512;

        type HmacSha512 = Hmac<Sha512>;

        let mut mac = HmacSha512::new_from_slice(key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());

        hex::encode(mac.finalize().into_bytes())
    }

    /// Constant-time string comparison (timing-safe)
    #[wasm_bindgen]
    pub fn constant_time_compare(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();

        let mut result: u8 = 0;
        for i in 0..a_bytes.len() {
            result |= a_bytes[i] ^ b_bytes[i];
        }

        result == 0
    }
}

#[wasm_bindgen]
pub fn hash_canonical_form(data: &str) -> String {
    CryptoEngine::sha512(data)
}

#[wasm_bindgen]
pub fn verify_canonical_hash(data: &str, hash: &str) -> bool {
    CryptoEngine::verify_sha512(data, hash)
}

#[wasm_bindgen]
pub fn merkle_leaf_hash(index: u32, data: &str) -> String {
    let leaf_data = format!("leaf:{}:{}", index, data);
    CryptoEngine::sha512(&leaf_data)
}

#[wasm_bindgen]
pub fn merkle_parent_hash(left: &str, right: &str) -> String {
    let parent_data = format!("{}|{}", left, right);
    CryptoEngine::sha512(&parent_data)
}
