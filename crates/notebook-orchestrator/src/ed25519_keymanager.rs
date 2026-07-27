// Ed25519 Key Management — Key Lifecycle, Rotation, Expiration, Revocation
// Implements SEC-001: Replace HMAC with Ed25519 asymmetric signatures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use signature::Signer;

/// Key version identifier
pub type KeyVersion = u32;

/// Ed25519 keypair
#[derive(Debug, Clone)]
pub struct Ed25519KeyPair {
    /// Signing key (private, 32 bytes)
    signing_key: ed25519_dalek::SigningKey,
    /// Verifying key (public, 32 bytes)
    verifying_key: ed25519_dalek::VerifyingKey,
}

impl Ed25519KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut secret_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret_bytes);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Ed25519KeyPair {
            signing_key,
            verifying_key,
        }
    }

    /// Import keypair from hex-encoded bytes
    pub fn from_hex(private_key_hex: &str) -> Result<Self, String> {
        let private_bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Failed to decode private key hex: {}", e))?;

        if private_bytes.len() != 32 {
            return Err(format!("Private key must be 32 bytes, got {}", private_bytes.len()));
        }

        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            private_bytes.as_slice().try_into()
                .map_err(|_| "Invalid private key bytes".to_string())?
        );
        let verifying_key = signing_key.verifying_key();

        Ok(Ed25519KeyPair {
            signing_key,
            verifying_key,
        })
    }

    /// Export private key as hex
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    /// Export public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        hex::encode(signature.to_bytes())
    }

    /// Verify a signature (static method for verification without private key)
    pub fn verify(public_key_hex: &str, message: &[u8], signature_hex: &str) -> Result<bool, String> {
        let pubkey_bytes = hex::decode(public_key_hex)
            .map_err(|e| format!("Failed to decode public key: {}", e))?;
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;

        if pubkey_bytes.len() != 32 {
            return Err(format!("Public key must be 32 bytes, got {}", pubkey_bytes.len()));
        }
        if sig_bytes.len() != 64 {
            return Err(format!("Signature must be 64 bytes, got {}", sig_bytes.len()));
        }

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            pubkey_bytes.as_slice().try_into()
                .map_err(|_| "Invalid public key bytes".to_string())?
        ).map_err(|e| format!("Invalid verifying key: {}", e))?;

        let sig_array: [u8; 64] = sig_bytes.try_into()
            .map_err(|_| "Invalid signature bytes".to_string())?;
        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        match verifying_key.verify_strict(message, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Key metadata: version, validity window, status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyMetadata {
    /// Key version identifier
    pub version: KeyVersion,

    /// Agent that owns this key
    pub agent_id: String,

    /// Public key (hex)
    pub public_key: String,

    /// Creation timestamp
    pub created_at: u64,

    /// Key becomes valid at this timestamp
    pub valid_from: u64,

    /// Key expires at this timestamp (exclusive)
    pub valid_until: u64,

    /// Key status
    pub status: KeyStatus,

    /// Optional revocation reason
    pub revocation_reason: Option<String>,

    /// Revocation timestamp
    pub revoked_at: Option<u64>,

    /// Key rotation target (version of key replacing this one)
    pub rotated_to: Option<KeyVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "not_yet_valid")]
    NotYetValid,
    #[serde(rename = "revoked")]
    Revoked,
}

impl KeyMetadata {
    /// Create new key metadata
    pub fn new(
        version: KeyVersion,
        agent_id: String,
        public_key: String,
        valid_from: u64,
        valid_until: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        KeyMetadata {
            version,
            agent_id,
            public_key,
            created_at: now,
            valid_from,
            valid_until,
            status: KeyStatus::Active,
            revocation_reason: None,
            revoked_at: None,
            rotated_to: None,
        }
    }

    /// Check if key is currently valid
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.status {
            KeyStatus::Revoked => false,
            KeyStatus::Expired => false,
            KeyStatus::NotYetValid => false,
            KeyStatus::Active => now >= self.valid_from && now < self.valid_until,
        }
    }

    /// Revoke this key
    pub fn revoke(&mut self, reason: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.status = KeyStatus::Revoked;
        self.revocation_reason = Some(reason);
        self.revoked_at = Some(now);
    }

    /// Mark key as rotated
    pub fn mark_rotated(&mut self, rotated_to: KeyVersion) {
        self.rotated_to = Some(rotated_to);
    }

    /// Update status based on current time
    pub fn update_status(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if self.status == KeyStatus::Revoked {
            return; // Revoked status is permanent
        }

        if now < self.valid_from {
            self.status = KeyStatus::NotYetValid;
        } else if now >= self.valid_until {
            self.status = KeyStatus::Expired;
        } else {
            self.status = KeyStatus::Active;
        }
    }
}

/// Key store: manages all keys for all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStore {
    /// Map of (agent_id, key_version) → KeyMetadata
    metadata: HashMap<(String, KeyVersion), KeyMetadata>,

    /// Map of (agent_id, key_version) → private key (hex)
    private_keys: HashMap<(String, KeyVersion), String>,

    /// Current active key version per agent
    current_key_version: HashMap<String, KeyVersion>,
}

impl KeyStore {
    pub fn new() -> Self {
        KeyStore {
            metadata: HashMap::new(),
            private_keys: HashMap::new(),
            current_key_version: HashMap::new(),
        }
    }

    /// Generate and store a new keypair for an agent
    pub fn generate_key(
        &mut self,
        agent_id: String,
        valid_from: u64,
        valid_until: u64,
    ) -> Result<(KeyVersion, String), String> {
        // Determine next version
        let current_version = self.current_key_version.get(&agent_id).copied().unwrap_or(0);
        let new_version = current_version + 1;

        // Generate keypair
        let keypair = Ed25519KeyPair::generate();
        let public_key = keypair.public_key_hex();
        let private_key = keypair.private_key_hex();

        // Create metadata
        let metadata = KeyMetadata::new(
            new_version,
            agent_id.clone(),
            public_key.clone(),
            valid_from,
            valid_until,
        );

        // Store
        self.metadata.insert((agent_id.clone(), new_version), metadata);
        self.private_keys.insert((agent_id.clone(), new_version), private_key);
        self.current_key_version.insert(agent_id.clone(), new_version);

        Ok((new_version, public_key))
    }

    /// Get current active key for agent
    pub fn get_current_key(&self, agent_id: &str) -> Result<(KeyVersion, String), String> {
        let version = self.current_key_version.get(agent_id)
            .ok_or(format!("No key found for agent {}", agent_id))?;

        let metadata = self.metadata.get(&(agent_id.to_string(), *version))
            .ok_or("Key metadata not found".to_string())?;

        if !metadata.is_valid() {
            return Err(format!("Key {} is not valid (status: {:?})", version, metadata.status));
        }

        Ok((*version, metadata.public_key.clone()))
    }

    /// Get specific key version
    pub fn get_key(&self, agent_id: &str, version: KeyVersion) -> Result<KeyMetadata, String> {
        self.metadata.get(&(agent_id.to_string(), version))
            .cloned()
            .ok_or(format!("Key {}/{} not found", agent_id, version))
    }

    /// Sign with agent's current key
    pub fn sign(&self, agent_id: &str, message: &[u8]) -> Result<String, String> {
        let version = self.current_key_version.get(agent_id)
            .ok_or(format!("No key for agent {}", agent_id))?;

        let private_key_hex = self.private_keys.get(&(agent_id.to_string(), *version))
            .ok_or("Private key not found".to_string())?;

        let keypair = Ed25519KeyPair::from_hex(private_key_hex)?;
        Ok(keypair.sign(message))
    }

    /// Revoke a key
    pub fn revoke_key(
        &mut self,
        agent_id: &str,
        version: KeyVersion,
        reason: String,
    ) -> Result<(), String> {
        let key = self.metadata.get_mut(&(agent_id.to_string(), version))
            .ok_or("Key not found".to_string())?;

        key.revoke(reason);
        Ok(())
    }

    /// Rotate key: revoke old, set new as current
    pub fn rotate_key(
        &mut self,
        agent_id: &str,
        valid_from: u64,
        valid_until: u64,
    ) -> Result<(KeyVersion, String), String> {
        // Get current version
        let old_version = self.current_key_version.get(agent_id)
            .ok_or(format!("No current key for agent {}", agent_id))?
            .clone();

        // Revoke old key
        self.revoke_key(agent_id, old_version, "Rotated to new key".to_string())?;

        // Generate new key
        let (new_version, public_key) = self.generate_key(
            agent_id.to_string(),
            valid_from,
            valid_until,
        )?;

        // Mark old key as rotated
        if let Some(old_metadata) = self.metadata.get_mut(&(agent_id.to_string(), old_version)) {
            old_metadata.mark_rotated(new_version);
        }

        Ok((new_version, public_key))
    }

    /// Get all keys for an agent
    pub fn get_agent_keys(&self, agent_id: &str) -> Vec<KeyMetadata> {
        self.metadata
            .iter()
            .filter(|(key, _)| key.0 == agent_id)
            .map(|(_, metadata)| metadata.clone())
            .collect()
    }

    /// Verify signature with any key version (for receipt verification)
    pub fn verify_signature(
        &self,
        agent_id: &str,
        key_version: KeyVersion,
        message: &[u8],
        signature_hex: &str,
    ) -> Result<bool, String> {
        let metadata = self.get_key(agent_id, key_version)?;

        // Check if key was valid at signature time (lenient: allow expired keys to verify old signatures)
        if metadata.status == KeyStatus::Revoked {
            return Err("Key is revoked (cannot verify any signatures with revoked key)".to_string());
        }

        Ed25519KeyPair::verify(&metadata.public_key, message, signature_hex)
    }

    /// Export all metadata (for persistence)
    pub fn export_metadata(&self) -> HashMap<(String, KeyVersion), KeyMetadata> {
        self.metadata.clone()
    }

    /// Import metadata (from persistence)
    pub fn import_metadata(
        &mut self,
        metadata: HashMap<(String, KeyVersion), KeyMetadata>,
    ) -> Result<(), String> {
        for (key, value) in metadata {
            self.metadata.insert(key.clone(), value.clone());
            self.current_key_version.insert(key.0.clone(), key.1);
        }
        Ok(())
    }
}

impl Default for KeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = Ed25519KeyPair::generate();
        let private_hex = kp.private_key_hex();
        let public_hex = kp.public_key_hex();

        assert_eq!(private_hex.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(public_hex.len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = Ed25519KeyPair::generate();
        let message = b"Test message";
        let signature = kp.sign(message);

        let verified = Ed25519KeyPair::verify(&kp.public_key_hex(), message, &signature)
            .expect("Verification failed");
        assert!(verified);
    }

    #[test]
    fn test_signature_verification_fails_with_wrong_message() {
        let kp = Ed25519KeyPair::generate();
        let message = b"Test message";
        let signature = kp.sign(message);

        let wrong_message = b"Wrong message";
        let verified = Ed25519KeyPair::verify(&kp.public_key_hex(), wrong_message, &signature)
            .expect("Verification check failed");
        assert!(!verified);
    }

    #[test]
    fn test_key_metadata_validity() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let kp = Ed25519KeyPair::generate();
        let mut metadata = KeyMetadata::new(
            1,
            "agent1".to_string(),
            kp.public_key_hex(),
            now,
            now + 3600,
        );

        assert!(metadata.is_valid());

        metadata.revoke("Test revocation".to_string());
        assert!(!metadata.is_valid());
    }

    #[test]
    fn test_keystore_rotation() {
        let mut store = KeyStore::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate initial key
        let (v1, pk1) = store.generate_key(
            "agent1".to_string(),
            now,
            now + 3600,
        ).expect("Generate key failed");

        assert_eq!(v1, 1);

        // Rotate to new key
        let (v2, pk2) = store.rotate_key(
            "agent1",
            now,
            now + 7200,
        ).expect("Rotate key failed");

        assert_eq!(v2, 2);
        assert_ne!(pk1, pk2);

        // New key should be current
        let (current_version, _) = store.get_current_key("agent1").expect("Get current key failed");
        assert_eq!(current_version, v2);

        // Old key should be revoked
        let old_key = store.get_key("agent1", v1).expect("Get old key failed");
        assert_eq!(old_key.status, KeyStatus::Revoked);
    }

    #[test]
    fn test_keystore_sign_and_verify() {
        let mut store = KeyStore::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        store.generate_key(
            "agent1".to_string(),
            now,
            now + 3600,
        ).expect("Generate key failed");

        let message = b"Receipt to sign";
        let signature = store.sign("agent1", message).expect("Sign failed");

        let verified = store.verify_signature("agent1", 1, message, &signature)
            .expect("Verify failed");
        assert!(verified);
    }
}
