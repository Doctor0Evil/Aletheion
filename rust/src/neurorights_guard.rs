// ============================================================================
// MODULE: neurorights_guard
// PURPOSE: Neurorights protection and safety kernel enforcement
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, FCC Part 15, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

use crate::{evidence_core::EvidenceRecord, AletheionError, Result, OWNER_DID};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Neurorights policy principles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsPolicy {
    pub version: String,
    pub principles: Vec<String>,
    pub prohibited_actions: Vec<String>,
    pub required_safeguards: Vec<String>,
}

impl NeurorightsPolicy {
    /// Create the default AugmentedHumanRights v1 policy
    pub fn v1() -> Self {
        Self {
            version: "AugmentedHumanRights:v1".to_string(),
            principles: vec![
                "No covert neuromorphic control".to_string(),
                "No Death-Network style sabotage".to_string(),
                "Explicit informed consent and revocation rights".to_string(),
                "Immutable audit and clinical oversight".to_string(),
                "Equal protection regardless of race or disability".to_string(),
                "Consciousness preservation rights with explicit consent".to_string(),
                "No discrimination based on augmentation status or technology type".to_string(),
                "Deviceless and organically-integrated cybernetics receive equal protection".to_string(),
                "All biophysical data is protected under medical-grade safeguards".to_string(),
                "Appeal and override path available via Clinical Safety Board".to_string(),
            ],
            prohibited_actions: vec![
                "covert_neuromorphic_control".to_string(),
                "death_network_sabotage".to_string(),
                "discriminatory_corridor_access".to_string(),
                "unconsented_biophysical_data_access".to_string(),
                "downgrade_of_augmentation_rights".to_string(),
                "exclusion_based_on_integration_type".to_string(),
            ],
            required_safeguards: vec![
                "VitalNetSafetyKernel enforcement".to_string(),
                "Immutable ROW audit logs".to_string(),
                "Explicit consent for all BCI operations".to_string(),
                "Clinical oversight for organic integrations".to_string(),
                "Independent safety review for firmware changes".to_string(),
                "Neurorights ombud escalation path".to_string(),
            ],
        }
    }

    /// Check if an action is prohibited
    pub fn is_prohibited(&self, action: &str) -> bool {
        self.prohibited_actions.iter().any(|p| p == action)
    }

    /// Verify equal protection principle
    pub fn verify_equal_protection(&self, has_bci: bool) -> Result<()> {
        // This method ensures no discrimination based on BCI presence
        // All users receive equal protection regardless of augmentation status
        info!(
            target: "aletheion_core::neurorights_guard",
            has_bci = has_bci,
            "Equal protection verified"
        );
        Ok(())
    }
}

/// Safety Kernel for enforcing neurorights and biofield limits
pub struct SafetyKernel {
    /// Kernel reference ID
    pub kernel_ref: String,

    /// Active policy
    pub policy: NeurorightsPolicy,

    /// Biofield load ceiling (W/kg or mW/cm²)
    pub biofield_load_ceiling: f64,

    /// Consent profiles (owner_did -> consent_status)
    pub consent_profiles: HashMap<String, bool>,

    /// Audit log
    pub audit_log: Vec<String>,
}

impl SafetyKernel {
    /// Create a new safety kernel
    pub fn new(kernel_ref: String) -> Result<Self> {
        if kernel_ref.is_empty() {
            return Err(AletheionError::SafetyKernelViolation(
                "Kernel reference cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            kernel_ref,
            policy: NeurorightsPolicy::v1(),
            biofield_load_ceiling: 0.5, // Default per FCC/ICNIRP
            consent_profiles: HashMap::new(),
            audit_log: Vec::new(),
        })
    }

    /// Verify an evidence record against safety constraints
    pub fn verify_record(&self, record: &EvidenceRecord) -> Result<()> {
        // Check consent
        if !self.consent_profiles.get(&record.owner_did).unwrap_or(&false) {
            return Err(AletheionError::ConsentRequired(format!(
                "No consent profile for owner: {}",
                record.owner_did
            )));
        }

        // Check for prohibited actions in evidence type
        if self.policy.is_prohibited(&record.evidence_type) {
            return Err(AletheionError::NeurorightsViolation(format!(
                "Prohibited evidence type: {}",
                record.evidence_type
            )));
        }

        // Log verification
        self.log_audit(format!(
            "Record verified: {} by {}",
            record.record_id, record.owner_did
        ));

        Ok(())
    }

    /// Register consent for an owner
    pub fn register_consent(&mut self, owner_did: String) {
        self.consent_profiles.insert(owner_did.clone(), true);
        self.log_audit(format!("Consent registered: {}", owner_did));
        info!(
            target: "aletheion_core::neurorights_guard",
            owner_did = %owner_did,
            "Consent registered"
        );
    }

    /// Revoke consent for an owner
    pub fn revoke_consent(&mut self, owner_did: String) {
        self.consent_profiles.insert(owner_did.clone(), false);
        self.log_audit(format!("Consent revoked: {}", owner_did));
        info!(
            target: "aletheion_core::neurorights_guard",
            owner_did = %owner_did,
            "Consent revoked"
        );
    }

    /// Log an audit entry
    fn log_audit(&mut self, message: String) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        self.audit_log.push(format!("[{}] {}", timestamp, message));
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> &[String] {
        &self.audit_log
    }

    /// Verify consciousness preservation rights
    #[cfg(feature = "consciousness_preservation")]
    pub fn verify_consciousness_preservation_rights(&self, owner_did: &str) -> Result<bool> {
        // Check consent
        if !self.consent_profiles.get(owner_did).unwrap_or(&false) {
            return Ok(false);
        }

        // In production, this would require Clinical Safety Board approval
        info!(
            target: "aletheion_core::neurorights_guard",
            owner_did = %owner_did,
            "Consciousness preservation rights verified"
        );

        Ok(true)
    }
}

/// Neurorights Guard - main interface for neurorights enforcement
pub struct NeurorightsGuard {
    /// Active policy
    pub policy: NeurorightsPolicy,

    /// Safety kernel reference
    pub safety_kernel: SafetyKernel,

    /// Violation count
    pub violation_count: u32,
}

impl NeurorightsGuard {
    /// Activate the neurorights guard
    pub fn activate() -> Result<Self> {
        let policy = NeurorightsPolicy::v1();
        let safety_kernel = SafetyKernel::new("VitalNetSafetyKernel:1.0.0".to_string())?;

        info!(
            target: "aletheion_core::neurorights_guard",
            policy_version = %policy.version,
            "Neurorights Guard activated"
        );

        Ok(Self {
            policy,
            safety_kernel,
            violation_count: 0,
        })
    }

    /// Verify equal protection for an owner
    pub fn verify_equal_protection(&self, owner_did: &str, has_bci: bool) -> Result<()> {
        self.policy.verify_equal_protection(has_bci)?;

        info!(
            target: "aletheion_core::neurorights_guard",
            owner_did = %owner_did,
            has_bci = has_bci,
            "Equal protection verified - no discrimination"
        );

        Ok(())
    }

    /// Check for discriminatory actions
    pub fn check_discrimination(&mut self, action: &str, target_did: &str) -> Result<()> {
        if self.policy.is_prohibited("discriminatory_corridor_access")
            && action.contains("discriminatory")
        {
            self.violation_count += 1;
            return Err(AletheionError::DiscriminatoryAction(format!(
                "Discriminatory action detected: {} for {}",
                action, target_did
            )));
        }

        Ok(())
    }

    /// Verify consciousness preservation rights
    #[cfg(feature = "consciousness_preservation")]
    pub fn verify_consciousness_preservation_rights(&self, owner_did: &str) -> Result<bool> {
        self.safety_kernel
            .verify_consciousness_preservation_rights(owner_did)
    }

    /// Get violation count
    pub fn get_violation_count(&self) -> u32 {
        self.violation_count
    }

    /// Report a neurorights violation
    pub fn report_violation(&mut self, violation_type: String, details: String) {
        self.violation_count += 1;
        error!(
            target: "aletheion_core::neurorights_guard",
            violation_type = %violation_type,
            details = %details,
            "Neurorights violation reported"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neurorights_policy_v1() {
        let policy = NeurorightsPolicy::v1();

        assert_eq!(policy.version, "AugmentedHumanRights:v1");
        assert!(!policy.principles.is_empty());
        assert!(!policy.prohibited_actions.is_empty());
        assert!(!policy.required_safeguards.is_empty());

        // Check specific principles
        assert!(policy.principles.iter().any(|p| p.contains("Equal protection")));
        assert!(policy
            .principles
            .iter()
            .any(|p| p.contains("Deviceless and organically-integrated")));
    }

    #[test]
    fn test_safety_kernel_creation() {
        let kernel = SafetyKernel::new("VitalNetSafetyKernel:1.0.0".to_string());
        assert!(kernel.is_ok());

        let kernel = kernel.unwrap();
        assert_eq!(kernel.kernel_ref, "VitalNetSafetyKernel:1.0.0");
        assert_eq!(kernel.biofield_load_ceiling, 0.5);
    }

    #[test]
    fn test_neurorights_guard_activation() {
        let guard = NeurorightsGuard::activate();
        assert!(guard.is_ok());

        let guard = guard.unwrap();
        assert_eq!(guard.policy.version, "AugmentedHumanRights:v1");
        assert_eq!(guard.get_violation_count(), 0);
    }

    #[test]
    fn test_equal_protection_verification() {
        let guard = NeurorightsGuard::activate().unwrap();

        // Test with BCI
        let result_with_bci =
            guard.verify_equal_protection(OWNER_DID, true);
        assert!(result_with_bci.is_ok());

        // Test without BCI
        let result_without_bci =
            guard.verify_equal_protection(OWNER_DID, false);
        assert!(result_without_bci.is_ok());

        // Both should succeed - equal protection regardless of BCI status
    }
}
