// Notebook Cell Merkle Tree — Tamper Detection & Cell Chain Integrity
// Implements SEC-005: Detect notebook cell modification, reordering, deletion

use serde::{Deserialize, Serialize};
use sha2::{Sha512, Digest};
use std::collections::HashMap;

/// Single notebook cell with hash-based identity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotebookCell {
    /// Cell index in notebook (0-indexed, immutable)
    pub cell_index: u32,

    /// Cell type (code, markdown, raw, etc.)
    pub cell_type: String,

    /// Source code (cell content)
    pub source: String,

    /// Execution count (None = not executed, Some(n) = executed n times)
    pub execution_count: Option<u32>,

    /// Cell outputs (concatenated)
    pub outputs: String,

    /// Cell metadata (JSON-serialized)
    pub metadata: String,

    /// Execution timestamp (Unix seconds)
    pub executed_at: u64,

    /// SHA-512 hash of this cell (deterministic)
    pub cell_hash: String,

    /// SHA-512 hash of previous cell in chain (chain linkage)
    pub previous_cell_hash: String,

    /// Ed25519 signature of cell_hash (cryptographic proof)
    pub signature: Option<String>,

    /// Public key that signed this cell
    pub signing_public_key: Option<String>,
}

impl NotebookCell {
    /// Create new cell (hash computed deterministically)
    pub fn new(
        cell_index: u32,
        cell_type: String,
        source: String,
        execution_count: Option<u32>,
        outputs: String,
        metadata: String,
        executed_at: u64,
        previous_cell_hash: String,
    ) -> Self {
        let cell_hash = Self::compute_hash(
            cell_index,
            &cell_type,
            &source,
            execution_count,
            &outputs,
            &metadata,
            executed_at,
            &previous_cell_hash,
        );

        NotebookCell {
            cell_index,
            cell_type,
            source,
            execution_count,
            outputs,
            metadata,
            executed_at,
            cell_hash,
            previous_cell_hash,
            signature: None,
            signing_public_key: None,
        }
    }

    /// Compute deterministic SHA-512 hash of cell
    fn compute_hash(
        cell_index: u32,
        cell_type: &str,
        source: &str,
        execution_count: Option<u32>,
        outputs: &str,
        metadata: &str,
        executed_at: u64,
        previous_cell_hash: &str,
    ) -> String {
        // Canonical order for hash computation
        let payload = format!(
            "idx:{}|type:{}|src:{}|exec_count:{}|out:{}|meta:{}|time:{}|prev:{}",
            cell_index,
            cell_type,
            source,
            execution_count.unwrap_or(0),
            outputs,
            metadata,
            executed_at,
            previous_cell_hash
        );

        let mut hasher = Sha512::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify cell hash integrity
    pub fn verify_hash(&self) -> bool {
        let computed = Self::compute_hash(
            self.cell_index,
            &self.cell_type,
            &self.source,
            self.execution_count,
            &self.outputs,
            &self.metadata,
            self.executed_at,
            &self.previous_cell_hash,
        );

        computed == self.cell_hash
    }

    /// Sign this cell with Ed25519 private key
    pub fn sign(&mut self, private_key_hex: &str, public_key_hex: &str) -> Result<(), String> {
        use ed25519_dalek::SigningKey;
        use signature::Signer;

        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Failed to decode private key: {}", e))?;

        if private_key_bytes.len() != 32 {
            return Err(format!("Private key must be 32 bytes, got {}", private_key_bytes.len()));
        }

        let signing_key = SigningKey::from_bytes(
            &private_key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| "Invalid private key bytes".to_string())?
        );

        let sig = signing_key.sign(self.cell_hash.as_bytes());
        self.signature = Some(hex::encode(sig.to_bytes()));
        self.signing_public_key = Some(public_key_hex.to_string());

        Ok(())
    }

    /// Verify cell signature
    pub fn verify_signature(&self) -> Result<bool, String> {
        use ed25519_dalek::VerifyingKey;
        use signature::Verifier;

        let sig_hex = self.signature.as_ref()
            .ok_or("Cell is not signed")?;
        let pubkey_hex = self.signing_public_key.as_ref()
            .ok_or("No public key provided")?;

        let sig_bytes = hex::decode(sig_hex)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;
        let pubkey_bytes = hex::decode(pubkey_hex)
            .map_err(|e| format!("Failed to decode public key: {}", e))?;

        let sig_array: [u8; 64] = sig_bytes.try_into()
            .map_err(|_| "Invalid signature bytes".to_string())?;
        let pk_array: [u8; 32] = pubkey_bytes.try_into()
            .map_err(|_| "Invalid public key bytes".to_string())?;

        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pk_array)
            .map_err(|e| format!("Invalid verifying key: {}", e))?;

        match verifying_key.verify_strict(self.cell_hash.as_bytes(), &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Merkle tree of all notebook cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMerkleTree {
    /// All cells in order
    cells: Vec<NotebookCell>,

    /// Merkle tree nodes (indexed by (level, index))
    tree_nodes: HashMap<(u32, u32), String>,

    /// Root hash (Merkle root)
    root_hash: String,

    /// Is tree sealed (immutable)
    sealed: bool,
}

impl NotebookMerkleTree {
    pub fn new() -> Self {
        NotebookMerkleTree {
            cells: Vec::new(),
            tree_nodes: HashMap::new(),
            root_hash: "0".repeat(128), // All-zero initial
            sealed: false,
        }
    }

    /// Add cell to tree (rebuilds Merkle root)
    pub fn add_cell(&mut self, cell: NotebookCell) -> Result<(), String> {
        if self.sealed {
            return Err("Notebook is sealed (WORM)".to_string());
        }

        // Verify cell hash
        if !cell.verify_hash() {
            return Err("Cell hash verification failed".to_string());
        }

        // Verify cell is in order (correct index)
        if cell.cell_index != self.cells.len() as u32 {
            return Err(format!(
                "Cell index mismatch: expected {}, got {}",
                self.cells.len(),
                cell.cell_index
            ));
        }

        // Verify chain linkage (previous hash)
        if let Some(last_cell) = self.cells.last() {
            if cell.previous_cell_hash != last_cell.cell_hash {
                return Err("Cell chain linkage broken".to_string());
            }
        } else {
            // First cell must have zero previous hash
            if cell.previous_cell_hash != "0".repeat(128) {
                return Err("First cell must have all-zero previous hash".to_string());
            }
        }

        self.cells.push(cell);
        self.rebuild_merkle_tree();
        Ok(())
    }

    /// Rebuild Merkle tree from leaf hashes
    fn rebuild_merkle_tree(&mut self) {
        self.tree_nodes.clear();

        // Level 0: leaf nodes (cell hashes)
        for (i, cell) in self.cells.iter().enumerate() {
            self.tree_nodes.insert((0, i as u32), cell.cell_hash.clone());
        }

        // Build upper levels
        let mut level = 0;
        let mut level_size = self.cells.len();

        while level_size > 1 {
            let next_level_size = (level_size + 1) / 2;

            for i in 0..next_level_size {
                let left_idx = i * 2;
                let right_idx = left_idx + 1;

                let left = self.tree_nodes.get(&(level, left_idx as u32))
                    .cloned()
                    .unwrap_or_default();

                let right = if right_idx < level_size {
                    self.tree_nodes.get(&(level, right_idx as u32))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    left.clone() // Hash odd leaf with itself
                };

                let parent_hash = Self::hash_pair(&left, &right);
                self.tree_nodes.insert((level + 1, i as u32), parent_hash);
            }

            level += 1;
            level_size = next_level_size;
        }

        // Root hash
        self.root_hash = self.tree_nodes.get(&(level, 0))
            .cloned()
            .unwrap_or_else(|| "0".repeat(128));
    }

    /// Hash two nodes together
    fn hash_pair(left: &str, right: &str) -> String {
        let payload = format!("{}|{}", left, right);
        let mut hasher = Sha512::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get root hash (Merkle root of all cells)
    pub fn root_hash(&self) -> &str {
        &self.root_hash
    }

    /// Get Merkle proof for cell at index
    pub fn get_merkle_proof(&self, cell_index: u32) -> Result<Vec<(u32, u32, String)>, String> {
        if cell_index >= self.cells.len() as u32 {
            return Err(format!("Cell index out of bounds: {}", cell_index));
        }

        let mut proof = Vec::new();
        let mut current_index = cell_index;
        let mut level = 0;

        while level < 32 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if let Some(sibling_hash) = self.tree_nodes.get(&(level, sibling_index)) {
                proof.push((level, sibling_index, sibling_hash.clone()));
            }

            current_index /= 2;
            level += 1;

            // Stop if we've reached the root
            if current_index == 0 && level > 1 {
                break;
            }
        }

        Ok(proof)
    }

    /// Verify Merkle proof for a cell
    pub fn verify_merkle_proof(
        cell_hash: &str,
        cell_index: u32,
        proof: &[(u32, u32, String)],
    ) -> Result<String, String> {
        let mut current_hash = cell_hash.to_string();
        let mut current_index = cell_index;

        for (level, sibling_index, sibling_hash) in proof {
            if *level as u32 != current_index.trailing_zeros() {
                return Err("Invalid proof structure".to_string());
            }

            if current_index % 2 == 0 {
                current_hash = Self::hash_pair(&current_hash, sibling_hash);
            } else {
                current_hash = Self::hash_pair(sibling_hash, &current_hash);
            }

            current_index /= 2;
        }

        Ok(current_hash)
    }

    /// Seal the notebook (WORM)
    pub fn seal(&mut self) -> Result<(), String> {
        if self.cells.is_empty() {
            return Err("Cannot seal empty notebook".to_string());
        }
        self.sealed = true;
        Ok(())
    }

    /// Get all cells
    pub fn cells(&self) -> &[NotebookCell] {
        &self.cells
    }

    /// Get specific cell
    pub fn get_cell(&self, index: u32) -> Option<&NotebookCell> {
        self.cells.get(index as usize)
    }

    /// Verify entire tree integrity
    pub fn verify_integrity(&self) -> bool {
        if self.cells.is_empty() {
            return true;
        }

        // Verify each cell hash
        for cell in &self.cells {
            if !cell.verify_hash() {
                return false;
            }
        }

        // Verify cell chain linkage
        for i in 1..self.cells.len() {
            if self.cells[i].previous_cell_hash != self.cells[i - 1].cell_hash {
                return false;
            }
        }

        // Verify Merkle root
        let mut test_tree = NotebookMerkleTree::new();
        for cell in self.cells.iter().cloned() {
            if test_tree.add_cell(cell).is_err() {
                return false;
            }
        }

        test_tree.root_hash == self.root_hash
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

impl Default for NotebookMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_cell_hash_deterministic() {
        let cell1 = NotebookCell::new(
            0,
            "code".to_string(),
            "print('hello')".to_string(),
            Some(1),
            "hello\n".to_string(),
            "{}".to_string(),
            1000,
            "0".repeat(128),
        );

        let cell2 = NotebookCell::new(
            0,
            "code".to_string(),
            "print('hello')".to_string(),
            Some(1),
            "hello\n".to_string(),
            "{}".to_string(),
            1000,
            "0".repeat(128),
        );

        assert_eq!(cell1.cell_hash, cell2.cell_hash);
    }

    #[test]
    fn test_notebook_cell_tampering_detected() {
        let mut cell = NotebookCell::new(
            0,
            "code".to_string(),
            "print('hello')".to_string(),
            Some(1),
            "hello\n".to_string(),
            "{}".to_string(),
            1000,
            "0".repeat(128),
        );

        assert!(cell.verify_hash());

        // Tamper with source
        cell.source = "print('hacked')".to_string();
        assert!(!cell.verify_hash());
    }

    #[test]
    fn test_merkle_tree_chain_linkage() {
        let mut tree = NotebookMerkleTree::new();

        let cell0 = NotebookCell::new(
            0,
            "code".to_string(),
            "x = 1".to_string(),
            Some(1),
            "".to_string(),
            "{}".to_string(),
            1000,
            "0".repeat(128),
        );

        assert!(tree.add_cell(cell0.clone()).is_ok());

        let cell1 = NotebookCell::new(
            1,
            "code".to_string(),
            "print(x)".to_string(),
            Some(1),
            "1\n".to_string(),
            "{}".to_string(),
            1001,
            cell0.cell_hash.clone(),
        );

        assert!(tree.add_cell(cell1).is_ok());
        assert!(tree.verify_integrity());
    }

    #[test]
    fn test_merkle_tree_reordering_detected() {
        let mut tree = NotebookMerkleTree::new();

        let cell0 = NotebookCell::new(
            0,
            "code".to_string(),
            "x = 1".to_string(),
            Some(1),
            "".to_string(),
            "{}".to_string(),
            1000,
            "0".repeat(128),
        );

        assert!(tree.add_cell(cell0.clone()).is_ok());

        // Try to add a cell with wrong index
        let bad_cell = NotebookCell::new(
            0, // Should be 1
            "code".to_string(),
            "y = 2".to_string(),
            Some(1),
            "".to_string(),
            "{}".to_string(),
            1001,
            cell0.cell_hash.clone(),
        );

        assert!(tree.add_cell(bad_cell).is_err());
    }

    #[test]
    fn test_merkle_proof_generation() {
        let mut tree = NotebookMerkleTree::new();

        for i in 0..4 {
            let cell = NotebookCell::new(
                i as u32,
                "code".to_string(),
                format!("cell_{}", i),
                Some(1),
                format!("output_{}", i),
                "{}".to_string(),
                1000 + i as u64,
                if i == 0 {
                    "0".repeat(128)
                } else {
                    tree.cells[i as usize - 1].cell_hash.clone()
                },
            );

            tree.add_cell(cell).ok();
        }

        // Generate proof for cell 0
        let proof = tree.get_merkle_proof(0).unwrap();
        assert!(!proof.is_empty());

        // Proof should contain sibling hashes
        assert!(proof.len() > 0);
    }

    #[test]
    fn test_notebook_seal() {
        let mut tree = NotebookMerkleTree::new();

        let cell = NotebookCell::new(
            0,
            "code".to_string(),
            "x = 1".to_string(),
            Some(1),
            "".to_string(),
            "{}".to_string(),
            1000,
            "0".repeat(128),
        );

        tree.add_cell(cell).ok();
        tree.seal().ok();

        // Should not be able to add more cells
        let cell2 = NotebookCell::new(
            1,
            "code".to_string(),
            "y = 2".to_string(),
            Some(1),
            "".to_string(),
            "{}".to_string(),
            1001,
            tree.cells[0].cell_hash.clone(),
        );

        assert!(tree.add_cell(cell2).is_err());
    }
}
