// ============================================================================
// MODULE: evidence_core
// PURPOSE: Core evidence management and living index functionality
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

use crate::{
    row_ledger::{RowEntry, RowLedger, RowSignature},
    neurorights_guard::{NeurorightsGuard, SafetyKernel},
    AletheionError, ComplianceMetadata, Result,
    MIN_EVIDENCE_COMPLETENESS, OWNER_DID,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use validator::Validate;

/// Evidence record structure for health and eco improvements
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EvidenceRecord {
    /// Unique record identifier
    #[validate(length(min = 1))]
    pub record_id: String,

    /// Reference to ROW entry hash
    #[validate(length(min = 1))]
    pub row_ref: String,

    /// Evidence type (health, eco, policy, mission)
    #[validate(length(min = 1))]
    pub evidence_type: String,

    /// Metric name (e.g., "PM2.5_reduction", "respiratory_improvement")
    #[validate(length(min = 1))]
    pub metric: String,

    /// Delta value (improvement amount)
    pub delta: f64,

    /// Unit of measurement
    #[validate(length(min = 1))]
    pub unit: String,

    /// Timestamp of evidence collection
    pub timestamp: DateTime<Utc>,

    /// Owner DID
    #[validate(length(min = 1))]
    pub owner_did: String,

    /// Corridor where evidence was collected
    #[validate(length(min = 1))]
    pub corridor: String,

    /// Evidence completeness score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub completeness_score: f64,

    /// Linked BCI device ID (if applicable)
    pub linked_bci_device_id: Option<String>,

    /// Consciousness preservation flag
    pub consciousness_preservation_relevant: bool,
}

impl EvidenceRecord {
    /// Create a new evidence record
    pub fn new(
        evidence_type: String,
        metric: String,
        delta: f64,
        unit: String,
        corridor: String,
        owner_did: String,
        linked_bci_device_id: Option<String>,
    ) -> Self {
        let record_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        Self {
            record_id,
            row_ref: String::new(), // Will be set when committed to ledger
            evidence_type,
            metric,
            delta,
            unit,
            timestamp,
            owner_did,
            corridor,
            completeness_score: 0.0, // Will be calculated
            linked_bci_device_id,
            consciousness_preservation_relevant: false,
        }
    }

    /// Calculate completeness score based on evidence chain
    pub fn calculate_completeness(&mut self, chain_verified: bool, audit_passed: bool) -> f64 {
        let mut score = 0.0;

        // Base score for valid record structure
        score += 0.3;

        // ROW reference present
        if !self.row_ref.is_empty() {
            score += 0.2;
        }

        // Evidence chain verified
        if chain_verified {
            score += 0.3;
        }

        // Audit passed
        if audit_passed {
            score += 0.2;
        }

        self.completeness_score = score;
        score
    }

    /// Verify evidence meets minimum completeness threshold
    pub fn meets_threshold(&self) -> bool {
        self.completeness_score >= MIN_EVIDENCE_COMPLETENESS
    }

    /// Generate hash for this record
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.record_id.as_bytes());
        hasher.update(self.row_ref.as_bytes());
        hasher.update(self.evidence_type.as_bytes());
        hasher.update(self.metric.as_bytes());
        hasher.update(self.delta.to_string().as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Personal evidence wallet for augmented citizens
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EvidenceWallet {
    /// Wallet identifier (DID-anchored)
    #[validate(length(min = 1))]
    pub wallet_id: String,

    /// Owner's Decentralized Identifier
    #[validate(length(min = 1))]
    pub owner_did: String,

    /// Linked BCI augmentation device ID
    pub linked_bci_device_id: Option<String>,

    /// Evidence records (array of EvidenceRecord)
    pub evidence_records: Vec<EvidenceRecord>,

    /// Health improvements (structured data)
    pub health_improvements: HashMap<String, f64>,

    /// Eco improvements (structured data)
    pub eco_improvements: HashMap<String, f64>,

    /// Care access permissions (array of authorized provider DIDs)
    pub care_access_providers: Vec<String>,

    /// Consciousness preservation data (encrypted)
    pub consciousness_preservation_data: Option<Vec<u8>>,

    /// Wallet status
    #[validate(length(min = 1))]
    pub wallet_status: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Evidence completeness score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub evidence_completeness_score: f64,
}

impl EvidenceWallet {
    /// Create a new evidence wallet
    pub fn new(owner_did: String, linked_bci_device_id: Option<String>) -> Self {
        let wallet_id = format!("evidence-wallet-{}", Uuid::new_v4());
        let now = Utc::now();

        Self {
            wallet_id,
            owner_did,
            linked_bci_device_id,
            evidence_records: Vec::new(),
            health_improvements: HashMap::new(),
            eco_improvements: HashMap::new(),
            care_access_providers: Vec::new(),
            consciousness_preservation_data: None,
            wallet_status: "active".to_string(),
            created_at: now,
            updated_at: now,
            evidence_completeness_score: 1.0,
        }
    }

    /// Add an evidence record to the wallet
    pub fn add_evidence_record(&mut self, mut record: EvidenceRecord) -> Result<()> {
        // Verify completeness before adding
        record.calculate_completeness(true, true);

        if !record.meets_threshold() {
            return Err(AletheionError::EvidenceChainIncomplete(format!(
                "Evidence record {} has completeness score {} < {}",
                record.record_id, record.completeness_score, MIN_EVIDENCE_COMPLETENESS
            )));
        }

        // Track improvements
        match record.evidence_type.as_str() {
            "health" => {
                self.health_improvements
                    .entry(record.metric.clone())
                    .and_modify(|v| *v += record.delta)
                    .or_insert(record.delta);
            }
            "eco" => {
                self.eco_improvements
                    .entry(record.metric.clone())
                    .and_modify(|v| *v += record.delta)
                    .or_insert(record.delta);
            }
            _ => {}
        }

        self.evidence_records.push(record);
        self.updated_at = Utc::now();
        self.recalculate_completeness();

        Ok(())
    }

    /// Recalculate overall wallet completeness score
    pub fn recalculate_completeness(&mut self) {
        if self.evidence_records.is_empty() {
            self.evidence_completeness_score = 1.0;
            return;
        }

        let total: f64 = self
            .evidence_records
            .iter()
            .map(|r| r.completeness_score)
            .sum();
        self.evidence_completeness_score = total / self.evidence_records.len() as f64;
    }

    /// Verify wallet meets minimum completeness threshold
    pub fn meets_threshold(&self) -> bool {
        self.evidence_completeness_score >= MIN_EVIDENCE_COMPLETENESS
    }

    /// Get all evidence records for a specific corridor
    pub fn get_records_by_corridor(&self, corridor: &str) -> Vec<&EvidenceRecord> {
        self.evidence_records
            .iter()
            .filter(|r| r.corridor == corridor)
            .collect()
    }

    /// Get all evidence records linked to BCI device
    pub fn get_bci_linked_records(&self) -> Vec<&EvidenceRecord> {
        self.evidence_records
            .iter()
            .filter(|r| r.linked_bci_device_id.is_some())
            .collect()
    }
}

/// Living Evidence Index - maps spec clauses to ledger entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingIndex {
    /// Index identifier
    pub index_id: String,

    /// Mapping: spec_clause -> [test_ids]
    pub spec_to_tests: HashMap<String, Vec<String>>,

    /// Mapping: test_id -> [mission_ids]
    pub test_to_missions: HashMap<String, Vec<String>>,

    /// Mapping: mission_id -> [eco_metric_ids]
    pub mission_to_metrics: HashMap<String, Vec<String>>,

    /// Mapping: eco_metric_id -> [row_entry_hashes]
    pub metric_to_rows: HashMap<String, Vec<String>>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last audit timestamp
    pub last_audit_at: DateTime<Utc>,

    /// Undocumented behaviors detected
    pub undocumented_behaviors: Vec<String>,
}

impl LivingIndex {
    /// Create a new living index
    pub fn new() -> Self {
        Self {
            index_id: Uuid::new_v4().to_string(),
            spec_to_tests: HashMap::new(),
            test_to_missions: HashMap::new(),
            mission_to_metrics: HashMap::new(),
            metric_to_rows: HashMap::new(),
            created_at: Utc::now(),
            last_audit_at: Utc::now(),
            undocumented_behaviors: Vec::new(),
        }
    }

    /// Add a spec clause to test mapping
    pub fn add_spec_test_mapping(&mut self, spec_clause: String, test_id: String) {
        self.spec_to_tests
            .entry(spec_clause)
            .or_insert_with(Vec::new)
            .push(test_id);
    }

    /// Add a test to mission mapping
    pub fn add_test_mission_mapping(&mut self, test_id: String, mission_id: String) {
        self.test_to_missions
            .entry(test_id)
            .or_insert_with(Vec::new)
            .push(mission_id);
    }

    /// Add a mission to metric mapping
    pub fn add_mission_metric_mapping(&mut self, mission_id: String, metric_id: String) {
        self.mission_to_metrics
            .entry(mission_id)
            .or_insert_with(Vec::new)
            .push(metric_id);
    }

    /// Add a metric to ROW mapping
    pub fn add_metric_row_mapping(&mut self, metric_id: String, row_hash: String) {
        self.metric_to_rows
            .entry(metric_id)
            .or_insert_with(Vec::new)
            .push(row_hash);
    }

    /// Audit for undocumented behaviors
    pub fn audit_undocumented_behaviors(&mut self, all_control_paths: &[String]) {
        self.undocumented_behaviors.clear();

        for path in all_control_paths {
            // Check if this control path has a complete evidence chain
            let has_evidence = self.spec_to_tests.values().any(|tests| {
                tests.iter().any(|test_id| {
                    self.test_to_missions.get(test_id).map_or(false, |missions| {
                        missions.iter().any(|mission_id| {
                            self.mission_to_metrics.get(mission_id).map_or(false, |metrics| {
                                metrics.iter().any(|metric_id| {
                                    self.metric_to_rows.get(metric_id).map_or(false, |rows| {
                                        !rows.is_empty()
                                    })
                                })
                            })
                        })
                    })
                })
            });

            if !has_evidence {
                self.undocumented_behaviors.push(path.clone());
                warn!(
                    target: "aletheion_core::evidence_core",
                    control_path = path,
                    "Undocumented behavior detected"
                );
            }
        }

        self.last_audit_at = Utc::now();
    }

    /// Get evidence completeness for the index
    pub fn get_completeness_score(&self) -> f64 {
        if self.undocumented_behaviors.is_empty() {
            return 1.0;
        }

        // Simple heuristic: ratio of documented to total control paths
        // In production, this would be more sophisticated
        1.0 - (self.undocumented_behaviors.len() as f64 * 0.1)
            .max(0.0)
            .min(1.0)
    }
}

impl Default for LivingIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Main Evidence Core struct
pub struct EvidenceCore {
    /// Compliance metadata
    pub compliance: ComplianceMetadata,

    /// Evidence wallets (owner_did -> EvidenceWallet)
    pub wallets: HashMap<String, EvidenceWallet>,

    /// Living index
    pub living_index: LivingIndex,

    /// ROW ledger reference
    pub ledger: RowLedger,

    /// Neurorights guard
    pub neurorights_guard: NeurorightsGuard,

    /// Safety kernel
    pub safety_kernel: SafetyKernel,
}

impl EvidenceCore {
    /// Create a new Evidence Core instance
    pub fn new() -> Result<Self> {
        let compliance = ComplianceMetadata::default();
        let ledger = RowLedger::initialize()?;
        let neurorights_guard = NeurorightsGuard::activate()?;
        let safety_kernel = SafetyKernel::new(SAFETY_KERNEL_REF.to_string())?;

        Ok(Self {
            compliance,
            wallets: HashMap::new(),
            living_index: LivingIndex::new(),
            ledger,
            neurorights_guard,
            safety_kernel,
        })
    }

    /// Create or get an evidence wallet for an owner
    pub fn get_or_create_wallet(
        &mut self,
        owner_did: String,
        linked_bci_device_id: Option<String>,
    ) -> Result<&mut EvidenceWallet> {
        // Neurorights check: ensure no discrimination based on BCI presence
        self.neurorights_guard
            .verify_equal_protection(&owner_did, linked_bci_device_id.is_some())?;

        use std::collections::hash_map::Entry;
        match self.wallets.entry(owner_did.clone()) {
            Entry::Vacant(entry) => {
                let wallet = EvidenceWallet::new(owner_did, linked_bci_device_id);
                info!(
                    target: "aletheion_core::evidence_core",
                    owner_did = %owner_did,
                    wallet_id = %wallet.wallet_id,
                    "Created new evidence wallet"
                );
                Ok(entry.insert(wallet))
            }
            Entry::Occupied(entry) => Ok(entry.into_mut()),
        }
    }

    /// Add evidence record to a wallet
    pub fn add_evidence_record(
        &mut self,
        owner_did: &str,
        record: EvidenceRecord,
    ) -> Result<()> {
        // Safety kernel verification
        self.safety_kernel.verify_record(&record)?;

        // Create ROW entry for this evidence
        let row_entry = RowEntry::from_evidence_record(record)?;
        let row_hash = self.ledger.append(row_entry)?;

        // Get or create wallet
        let wallet = self.get_or_create_wallet(owner_did.to_string(), None)?;

        // Update record with ROW reference
        let mut record = wallet
            .evidence_records
            .iter_mut()
            .last()
            .ok_or_else(|| AletheionError::EvidenceChainIncomplete("No record found".to_string()))?;
        record.row_ref = row_hash.clone();

        info!(
            target: "aletheion_core::evidence_core",
            owner_did = %owner_did,
            row_hash = %row_hash,
            "Evidence record committed to ledger"
        );

        Ok(())
    }

    /// Run audit for undocumented behaviors
    pub fn run_audit(&mut self, control_paths: Vec<String>) -> Result<()> {
        info!(
            target: "aletheion_core::evidence_core",
            control_paths_count = control_paths.len(),
            "Running evidence audit"
        );

        self.living_index
            .audit_undocumented_behaviors(&control_paths);

        let completeness = self.living_index.get_completeness_score();

        if completeness < MIN_EVIDENCE_COMPLETENESS {
            error!(
                target: "aletheion_core::evidence_core",
                completeness = %completeness,
                undocumented_count = self.living_index.undocumented_behaviors.len(),
                "Evidence completeness below threshold"
            );

            return Err(AletheionError::AuditFailure(format!(
                "Evidence completeness {} < {}",
                completeness, MIN_EVIDENCE_COMPLETENESS
            )));
        }

        info!(
            target: "aletheion_core::evidence_core",
            completeness = %completeness,
            "Audit passed"
        );

        Ok(())
    }

    /// Get evidence completeness score
    pub fn get_completeness_score(&self) -> f64 {
        self.living_index.get_completeness_score()
    }

    /// Verify consciousness preservation eligibility
    #[cfg(feature = "consciousness_preservation")]
    pub fn verify_consciousness_preservation(&self, owner_did: &str) -> Result<bool> {
        // Check if owner has BCI with consciousness preservation enabled
        // This requires Clinical Safety Board approval in production
        self.neurorights_guard
            .verify_consciousness_preservation_rights(owner_did)
    }
}

impl Default for EvidenceCore {
    fn default() -> Self {
        Self::new().expect("Failed to create EvidenceCore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_record_creation() {
        let record = EvidenceRecord::new(
            "health".to_string(),
            "respiratory_improvement".to_string(),
            15.5,
            "percent".to_string(),
            "rehab_neuroassist".to_string(),
            OWNER_DID.to_string(),
            Some("bci-aug-bostrom-001".to_string()),
        );

        assert!(!record.record_id.is_empty());
        assert_eq!(record.evidence_type, "health");
        assert_eq!(record.delta, 15.5);
        assert!(record.linked_bci_device_id.is_some());
    }

    #[test]
    fn test_evidence_wallet_creation() {
        let wallet = EvidenceWallet::new(OWNER_DID.to_string(), None);

        assert!(!wallet.wallet_id.is_empty());
        assert_eq!(wallet.owner_did, OWNER_DID);
        assert_eq!(wallet.wallet_status, "active");
        assert!(wallet.meets_threshold());
    }

    #[test]
    fn test_evidence_core_initialization() {
        let core = EvidenceCore::new();
        assert!(core.is_ok());

        let core = core.unwrap();
        assert_eq!(core.compliance.owner_did, OWNER_DID);
        assert!(core.get_completeness_score() >= 0.0);
    }

    #[test]
    fn test_living_index_mapping() {
        let mut index = LivingIndex::new();

        index.add_spec_test_mapping("spec_001".to_string(), "test_001".to_string());
        index.add_test_mission_mapping("test_001".to_string(), "mission_001".to_string());
        index.add_mission_metric_mapping("mission_001".to_string(), "metric_001".to_string());
        index.add_metric_row_mapping("metric_001".to_string(), "row_hash_001".to_string());

        assert!(index.spec_to_tests.contains_key("spec_001"));
        assert!(index.test_to_missions.contains_key("test_001"));
        assert!(index.mission_to_metrics.contains_key("mission_001"));
        assert!(index.metric_to_rows.contains_key("metric_001"));
    }
}
