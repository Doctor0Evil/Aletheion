// ============================================================================
// LIBRARY: aletheion_core
// PURPOSE: Core library for Aletheion GOD-City Evidence Core
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, FCC Part 15, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![forbid(clippy::unwrap_used)]

//! # Aletheion Core
//!
//! Evidence Core and Living Index for Aletheion GOD-City smart-city infrastructure.
//! Provides immutable ROW (Record-of-Work) ledger, neurorights protection, and
//! evidence chain verification for all augmented citizens.
//!
//! ## Features
//!
//! - **Immutable Ledger:** DID-anchored, hash-chained ROW entries
//! - **Neurorights Protection:** VitalNetSafetyKernel integration
//! - **Evidence Wallets:** Personal evidence records for augmented citizens
//! - **Consciousness Preservation:** Optional preservation for event-of-Death continuity
//!
//! ## Compliance
//!
//! This library enforces GDPR, HIPAA, EU AI Act 2024, FCC Part 15, and Neurorights Charter v1.
//! All organic BCI objects are governed by augmented-human-rights policy.
//!
//! ## Example
//!
//! ```rust
//! use aletheion_core::{EvidenceCore, RowLedger, NeurorightsGuard};
//!
//! let mut core = EvidenceCore::new();
//! let ledger = RowLedger::initialize()?;
//! let guard = NeurorightsGuard::activate()?;
//!
//! // All operations are audit-logged and neurorights-compliant
//! ```

pub mod evidence_core;
pub mod row_ledger;
pub mod neurorights_guard;

pub use evidence_core::EvidenceCore;
pub use row_ledger::{RowEntry, RowLedger, RowSignature};
pub use neurorights_guard::{NeurorightsGuard, NeurorightsPolicy, SafetyKernel};

// Re-exports for convenience
pub use chrono::{DateTime, Utc};
pub use sha2::{Digest, Sha256};
pub use ed25519_dalek::{Signature, Signer, SigningKey};
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;
pub use validator::Validate;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Owner DID for this implementation
pub const OWNER_DID: &str = "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";

/// Safety kernel reference
pub const SAFETY_KERNEL_REF: &str = "VitalNetSafetyKernel:1.0.0";

/// Neurorights policy version
pub const NEURORIGHTS_POLICY_VERSION: &str = "AugmentedHumanRights:v1";

/// Minimum evidence completeness score (KER threshold)
pub const MIN_EVIDENCE_COMPLETENESS: f64 = 0.86;

/// Compliance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetadata {
    pub regulations: Vec<String>,
    pub owner_did: String,
    pub safety_kernel: String,
    pub neurorights_policy: String,
    pub audit_enabled: bool,
}

impl Default for ComplianceMetadata {
    fn default() -> Self {
        Self {
            regulations: vec![
                "GDPR".to_string(),
                "HIPAA".to_string(),
                "EU-AI-Act-2024".to_string(),
                "FCC-Part-15".to_string(),
                "Neurorights-Charter-v1".to_string(),
            ],
            owner_did: OWNER_DID.to_string(),
            safety_kernel: SAFETY_KERNEL_REF.to_string(),
            neurorights_policy: NEURORIGHTS_POLICY_VERSION.to_string(),
            audit_enabled: true,
        }
    }
}

/// Error types for Aletheion Core
#[derive(Debug, thiserror::Error)]
pub enum AletheionError {
    #[error("Neurorights violation: {0}")]
    NeurorightsViolation(String),

    #[error("Evidence chain incomplete: {0}")]
    EvidenceChainIncomplete(String),

    #[error("ROW ledger error: {0}")]
    RowLedgerError(String),

    #[error("Safety kernel violation: {0}")]
    SafetyKernelViolation(String),

    #[error("Consent required: {0}")]
    ConsentRequired(String),

    #[error("Biofield load exceeded: {0}")]
    BiofieldLoadExceeded(String),

    #[error("Discriminatory action detected: {0}")]
    DiscriminatoryAction(String),

    #[error("Audit failure: {0}")]
    AuditFailure(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, AletheionError>;

/// Initialize tracing for audit logging
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("aletheion_core=info".parse().unwrap()),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!(
        target: "aletheion_core::init",
        version = VERSION,
        owner_did = OWNER_DID,
        safety_kernel = SAFETY_KERNEL_REF,
        "Aletheion Core initialized with neurorights protection"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_metadata_default() {
        let metadata = ComplianceMetadata::default();
        assert!(metadata.regulations.contains(&"GDPR".to_string()));
        assert!(metadata.regulations.contains(&"HIPAA".to_string()));
        assert_eq!(metadata.owner_did, OWNER_DID);
        assert_eq!(metadata.safety_kernel, SAFETY_KERNEL_REF);
        assert!(metadata.audit_enabled);
    }

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!OWNER_DID.is_empty());
        assert!(MIN_EVIDENCE_COMPLETENESS >= 0.86);
    }
}
