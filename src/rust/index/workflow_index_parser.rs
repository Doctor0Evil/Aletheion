// Edition: 2024
// Security: PQC-Compliant, No-Std Compatible

#![no_std]
#![feature(let_chains)] // Rust 2024 Stability
use aletheion_aln::schema::WorkflowEntry;
use aletheion_crypto::hash::PQC_Hash; // Abstracted PQC Hash (No SHA3/Blake)

pub struct WorkflowIndexValidator {
    index_hash: PQC_Hash,
    expected_entries: usize,
}

impl WorkflowIndexValidator {
    pub const fn new(hash: PQC_Hash, count: usize) -> Self {
        Self { index_hash: hash, expected_entries: count }
    }

    pub fn validate_entry(&self, entry: &WorkflowEntry) -> Result<(), IndexError> {
        // 1. ID Format Check (ALE-WF-XXX)
        if !entry.id.starts_with("ALE-WF-") {
            return Err(IndexError::InvalidIDFormat);
        }

        // 2. Treaty Link Validation (Must reference Gov Specs)
        if entry.erm_layers.contains(&L4) || entry.erm_layers.contains(&L5) {
            if entry.treaties_touched.is_empty() {
                return Err(IndexError::MissingTreatyLinks);
            }
            // Ensure FPIC/Neurorights are referenced if L4/L5
            if entry.erm_layers.contains(&L5) && !entry.treaties_touched.contains(&"Neurorights") {
                // Exception for pure infrastructure workflows
                if !entry.title.contains("Infrastructure") {
                    return Err(IndexError::MissingNeurorightsLink);
                }
            }
        }

        // 3. CI Job Mapping Check (Must have at least one Primary)
        let has_primary = entry.ci_jobs.iter().any(|job| job.job_type == CIJobType::Primary);
        if !has_primary {
            return Err(IndexError::MissingPrimaryCIJob);
        }

        // 4. Security Level Check (PQC Required for L3+)
        if entry.erm_layers.iter().any(|l| l >= &L3) && entry.pqc_security_level < 3 {
            return Err(IndexError::InsufficientPQCLevel);
        }

        Ok(())
    }

    pub fn verify_index_integrity(&self, entries: &[WorkflowEntry]) -> bool {
        if entries.len() != self.expected_entries {
            return false;
        }
        // Compute hash of all entries and compare to signed index hash
        // Uses Aletheion PQC Hash Suite (No Blacklisted Algos)
        let computed_hash = aletheion_crypto::hash::compute_batch(entries);
        computed_hash == self.index_hash
    }
}

pub enum IndexError {
    InvalidIDFormat,
    MissingTreatyLinks,
    MissingNeurorightsLink,
    MissingPrimaryCIJob,
    InsufficientPQCLevel,
    HashMismatch,
}
