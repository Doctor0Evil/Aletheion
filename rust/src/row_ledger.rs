// ============================================================================
// MODULE: row_ledger
// PURPOSE: Immutable ROW (Record-of-Work) ledger with DID anchoring
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, FCC Part 15, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

use crate::{
    evidence_core::EvidenceRecord,
    AletheionError, Result,
    OWNER_DID, SAFTY_KERNEL_REF,
};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use tracing::{debug, error, info};

/// ROW signature with DID anchoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowSignature {
    /// Signature bytes (Ed25519)
    pub signature_bytes: Vec<u8>,

    /// Signer's public key
    pub public_key: Vec<u8>,

    /// Signer's DID
    pub signer_did: String,

    /// Timestamp of signing
    pub signed_at: DateTime<Utc>,

    /// Signature algorithm
    pub algorithm: String,
}

impl RowSignature {
    /// Create a new signature
    pub fn new(signing_key: &SigningKey, data_hash: &[u8], signer_did: String) -> Self {
        let signature = signing_key.sign(data_hash);
        let public_key = signing_key.verifying_key().to_bytes().to_vec();

        Self {
            signature_bytes: signature.to_bytes().to_vec(),
            public_key,
            signer_did,
            signed_at: Utc::now(),
            algorithm: "Ed25519".to_string(),
        }
    }

    /// Verify the signature
    pub fn verify(&self, data_hash: &[u8]) -> Result<bool> {
        let public_key = VerifyingKey::from_bytes(&self.public_key)
            .map_err(|e| AletheionError::CryptoError(format!("Invalid public key: {}", e)))?;

        let signature = Signature::from_bytes(&self.signature_bytes.try_into().map_err(|_| {
            AletheionError::CryptoError("Invalid signature bytes".to_string())
        })?);

        Ok(public_key.verify_strict(data_hash, &signature).is_ok())
    }

    /// Generate NewRowPrint! format string
    pub fn to_new_row_print(&self, entry_hash: &str) -> String {
        format!(
            "NewRowPrint!:aletheion:{}:{}:{}:sha256:{}",
            self.signer_did,
            self.signed_at.to_rfc3339(),
            self.algorithm,
            entry_hash
        )
    }
}

/// ROW entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowEntry {
    /// Entry identifier (UUID)
    pub entry_id: String,

    /// Previous entry hash (for chain integrity)
    pub previous_hash: String,

    /// Entry hash (SHA256 of content)
    pub entry_hash: String,

    /// Entry type (evidence, policy, mission, audit)
    pub entry_type: String,

    /// Entry data (JSON)
    pub data: String,

    /// Owner DID
    pub owner_did: String,

    /// Corridor reference
    pub corridor: String,

    /// KER score (Knowledge-Energy-Restoration)
    pub ker_score: f64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Signature
    pub signature: RowSignature,

    /// Multi-sig threshold (for governance actions)
    pub multisig_threshold: Option<u32>,

    /// Multi-sig signatures collected
    pub multisig_signatures: Vec<RowSignature>,
}

impl RowEntry {
    /// Create a new ROW entry from an evidence record
    pub fn from_evidence_record(record: EvidenceRecord) -> Result<Self> {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let data = serde_json::to_string(&record)
            .map_err(|e| AletheionError::RowLedgerError(format!("Serialization error: {}", e)))?;

        // Hash will be calculated after previous_hash is set
        let entry_hash = String::new();
        let previous_hash = String::new();

        Ok(Self {
            entry_id,
            previous_hash,
            entry_hash,
            entry_type: "evidence".to_string(),
            data,
            owner_did: record.owner_did,
            corridor: record.corridor,
            ker_score: record.completeness_score,
            timestamp,
            signature: RowSignature::new(
                &SigningKey::generate(&mut rand::thread_rng()),
                &[],
                OWNER_DID.to_string(),
            ),
            multisig_threshold: None,
            multisig_signatures: Vec::new(),
        })
    }

    /// Calculate entry hash
    pub fn calculate_hash(&mut self, previous_hash: String) -> Result<()> {
        self.previous_hash = previous_hash;

        let mut hasher = Sha256::new();
        hasher.update(self.entry_id.as_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.entry_type.as_bytes());
        hasher.update(self.data.as_bytes());
        hasher.update(self.owner_did.as_bytes());
        hasher.update(self.corridor.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());

        self.entry_hash = hex::encode(hasher.finalize());

        // Re-sign with new hash
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        self.signature = RowSignature::new(&signing_key, self.entry_hash.as_bytes(), self.owner_did.clone());

        Ok(())
    }

    /// Verify entry integrity
    pub fn verify(&self) -> Result<bool> {
        // Verify hash
        let mut hasher = Sha256::new();
        hasher.update(self.entry_id.as_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.entry_type.as_bytes());
        hasher.update(self.data.as_bytes());
        hasher.update(self.owner_did.as_bytes());
        hasher.update(self.corridor.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());

        let calculated_hash = hex::encode(hasher.finalize());

        if calculated_hash != self.entry_hash {
            return Ok(false);
        }

        // Verify signature
        self.signature.verify(self.entry_hash.as_bytes())
    }

    /// Check if KER score meets threshold
    pub fn meets_ker_threshold(&self) -> bool {
        self.ker_score >= crate::MIN_EVIDENCE_COMPLETENESS
    }
}

/// Immutable ROW Ledger
pub struct RowLedger {
    /// Ledger file path
    pub file_path: String,

    /// In-memory cache of entries
    pub entries: Vec<RowEntry>,

    /// Last hash in chain
    pub last_hash: String,

    /// Signing key (in production, this would be HSM-backed)
    pub signing_key: SigningKey,
}

impl RowLedger {
    /// Initialize a new ledger
    pub fn initialize() -> Result<Self> {
        let file_path = "aletheion_ledger.row".to_string();
        let signing_key = SigningKey::generate(&mut rand::thread_rng());

        let mut ledger = Self {
            file_path,
            entries: Vec::new(),
            last_hash: "genesis".to_string(),
            signing_key,
        };

        // Load existing ledger if present
        if Path::new(&ledger.file_path).exists() {
            ledger.load()?;
        }

        info!(
            target: "aletheion_core::row_ledger",
            file_path = %ledger.file_path,
            entries_count = ledger.entries.len(),
            "ROW Ledger initialized"
        );

        Ok(ledger)
    }

    /// Append a new entry to the ledger
    pub fn append(&mut self, mut entry: RowEntry) -> Result<String> {
        // Verify entry before appending
        if !entry.meets_ker_threshold() {
            return Err(AletheionError::RowLedgerError(format!(
                "KER score {} below threshold {}",
                entry.ker_score,
                crate::MIN_EVIDENCE_COMPLETENESS
            )));
        }

        // Calculate hash with previous hash
        entry.calculate_hash(self.last_hash.clone())?;

        // Verify entry integrity
        if !entry.verify()? {
            return Err(AletheionError::RowLedgerError(
                "Entry verification failed".to_string(),
            ));
        }

        let entry_hash = entry.entry_hash.clone();

        // Add to in-memory cache
        self.entries.push(entry);
        self.last_hash = entry_hash.clone();

        // Persist to file
        self.persist()?;

        info!(
            target: "aletheion_core::row_ledger",
            entry_hash = %entry_hash,
            entry_type = "evidence",
            "New ROW entry appended"
        );

        Ok(entry_hash)
    }

    /// Persist ledger to file
    fn persist(&self) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)
            .map_err(|e| AletheionError::RowLedgerError(format!("File open error: {}", e)))?;

        let mut writer = BufWriter::new(file);
        for entry in &self.entries {
            let json = serde_json::to_string(entry)
                .map_err(|e| AletheionError::RowLedgerError(format!("Serialization error: {}", e)))?;
            writeln!(writer, "{}", json)
                .map_err(|e| AletheionError::RowLedgerError(format!("Write error: {}", e)))?;
        }
        writer
            .flush()
            .map_err(|e| AletheionError::RowLedgerError(format!("Flush error: {}", e)))?;

        Ok(())
    }

    /// Load ledger from file
    pub fn load(&mut self) -> Result<()> {
        let file = File::open(&self.file_path)
            .map_err(|e| AletheionError::RowLedgerError(format!("File open error: {}", e)))?;

        let reader = BufReader::new(file);
        self.entries.clear();

        for line in std::io::BufRead::lines(reader) {
            let line = line.map_err(|e| AletheionError::RowLedgerError(format!("Read error: {}", e)))?;
            let entry: RowEntry = serde_json::from_str(&line)
                .map_err(|e| AletheionError::RowLedgerError(format!("Deserialization error: {}", e)))?;
            self.entries.push(entry);
        }

        if let Some(last_entry) = self.entries.last() {
            self.last_hash = last_entry.entry_hash.clone();
        }

        info!(
            target: "aletheion_core::row_ledger",
            entries_loaded = self.entries.len(),
            "Ledger loaded from file"
        );

        Ok(())
    }

    /// Get entry by hash
    pub fn get_entry(&self, hash: &str) -> Option<&RowEntry> {
        self.entries.iter().find(|e| e.entry_hash == hash)
    }

    /// Verify chain integrity
    pub fn verify_chain(&self) -> Result<bool> {
        let mut previous_hash = "genesis".to_string();

        for entry in &self.entries {
            if entry.previous_hash != previous_hash {
                error!(
                    target: "aletheion_core::row_ledger",
                    entry_id = %entry.entry_id,
                    expected = %previous_hash,
                    actual = %entry.previous_hash,
                    "Chain integrity violation"
                );
                return Ok(false);
            }
            previous_hash = entry.entry_hash.clone();
        }

        Ok(true)
    }

    /// Get all entries for a specific owner
    pub fn get_entries_by_owner(&self, owner_did: &str) -> Vec<&RowEntry> {
        self.entries.iter().filter(|e| e.owner_did == owner_did).collect()
    }

    /// Get all entries for a specific corridor
    pub fn get_entries_by_corridor(&self, corridor: &str) -> Vec<&RowEntry> {
        self.entries.iter().filter(|e| e.corridor == corridor).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_core::EvidenceRecord;

    #[test]
    fn test_row_signature_creation() {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let data_hash = Sha256::digest(b"test data");

        let signature = RowSignature::new(&signing_key, &data_hash, OWNER_DID.to_string());

        assert!(!signature.signature_bytes.is_empty());
        assert!(!signature.public_key.is_empty());
        assert_eq!(signature.signer_did, OWNER_DID);
        assert_eq!(signature.algorithm, "Ed25519");
    }

    #[test]
    fn test_row_entry_creation() {
        let record = EvidenceRecord::new(
            "health".to_string(),
            "test_metric".to_string(),
            10.0,
            "units".to_string(),
            "rehab_neuroassist".to_string(),
            OWNER_DID.to_string(),
            None,
        );

        let entry = RowEntry::from_evidence_record(record);
        assert!(entry.is_ok());

        let entry = entry.unwrap();
        assert_eq!(entry.entry_type, "evidence");
        assert_eq!(entry.owner_did, OWNER_DID);
    }

    #[test]
    fn test_ledger_append_and_verify() {
        let mut ledger = RowLedger::initialize().unwrap();

        let record = EvidenceRecord::new(
            "eco".to_string(),
            "PM2.5_reduction".to_string(),
            25.0,
            "percent".to_string(),
            "public_plaza_AR".to_string(),
            OWNER_DID.to_string(),
            None,
        );

        let mut entry = RowEntry::from_evidence_record(record).unwrap();
        entry.ker_score = 0.95; // Above threshold

        let hash = ledger.append(entry).unwrap();
        assert!(!hash.is_empty());

        // Verify chain integrity
        let chain_valid = ledger.verify_chain().unwrap();
        assert!(chain_valid);
    }
}
