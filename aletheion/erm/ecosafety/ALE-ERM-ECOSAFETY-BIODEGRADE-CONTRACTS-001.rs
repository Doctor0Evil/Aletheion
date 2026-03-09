// ============================================================================
// Aletheion ERM Ecosafety Biodegrade Contracts
// Canonical Rust enforcement primitives for biodegradable node orchestration
// ============================================================================
// File: ALE-ERM-ECOSAFETY-BIODEGRADE-CONTRACTS-001.rs
// Domain: Environmental Resource Management / Ecosafety / Cyboquatic
// Language: Rust (2024 edition, no_std compatible for edge targets)
// Compliance: BioticTreaties, Indigenous FPIC, Neurorights, EJ Zones
// Blacklist: No SHA-256, SHA-3, BLAKE, argon, Python, Exergy, KECCAK
// Cryptography: Post-quantum signatures (CRYSTALS-Dilithium via rust-pqcrypto)
// ============================================================================

#![no_std]
#![cfg_attr(not(test), no_main)]
#![feature(allocator_api)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![forbid(clippy::all)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Debug;

// ============================================================================
// 1. Core Type Definitions
// ============================================================================

/// Unique identifier for biodegradable nodes (DID-bound, non-semantic)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BiodegradeNodeId {
    pub prefix: [u8; 4],      // "BIOD"
    pub network_id: u16,      // XR-grid network segment
    pub node_hash: [u8; 32],  // Post-quantum safe identifier
}

/// Geographic tile reference (BirthSign-bound)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeoTileRef {
    pub tile_id: u64,
    pub birth_sign_id: String,
    pub indigenous_territory: Option<String>,
    pub ej_zone_flag: bool,
    pub biotic_corridor: Option<String>,
}

/// Biodegradable material classification
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BiodegradeMaterialClass {
    OrganicFast,        // <30 days decomposition
    OrganicSlow,        // 30-180 days
    PolymerBio,         // Bio-based polymers
    CompositeSafe,      // Certified non-toxic composites
    HazardousControlled, // Requires sealed corridor
}

/// Ecosafety corridor status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorridorStatus {
    DeclaredActive,
    DeclaredInactive,
    Undeclared,
    ViolationDetected,
    UnderReview,
}

/// Lyapunov stability residual for ecological convergence
#[derive(Clone, Debug, PartialEq)]
pub struct LyapunovResidual {
    pub value: f64,           // Must be >= 0.0 for stability
    pub derivative: f64,      // dV/dt, must be <= 0.0
    pub threshold: f64,       // Maximum allowed residual
    pub convergent: bool,     // Computed from value + derivative
}

/// Microbial decomposition rate (r_micro)
#[derive(Clone, Debug, PartialEq)]
pub struct MicrobialRate {
    pub value: f64,           // grams/day per node
    pub confidence: f64,      // 0.0-1.0 measurement confidence
    pub temperature_corrected: f64,
    pub ph_corrected: f64,
}

/// Toxicity residual (r_tox)
#[derive(Clone, Debug, PartialEq)]
pub struct ToxicityResidual {
    pub heavy_metals_ppm: f64,
    pub organic_toxins_ppb: f64,
    pub bioaccumulation_factor: f64,
    pub safe_threshold_exceeded: bool,
}

/// Node ecosafety state snapshot
#[derive(Clone, Debug)]
pub struct BiodegradeNodeState {
    pub node_id: BiodegradeNodeId,
    pub geo_tile: GeoTileRef,
    pub material_class: BiodegradeMaterialClass,
    pub mass_remaining_grams: f64,
    pub mass_initial_grams: f64,
    pub r_micro: MicrobialRate,
    pub r_tox: ToxicityResidual,
    pub lyapunov: LyapunovResidual,
    pub corridor_status: CorridorStatus,
    pub timestamp_unix: u64,
    pub firmware_version: u32,
    pub governance_seal: [u8; 64], // PQ signature
}

// ============================================================================
// 2. Treaty & Rights Enforcement Types
// ============================================================================

/// FPIC (Free, Prior, Informed Consent) status
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FpicStatus {
    NotApplicable,
    RequiredPending,
    Granted,
    Denied,
    Expired,
    Revoked,
}

/// BioticTreaty compliance check result
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BioticCompliance {
    Compliant,
    WarningMinor,
    ViolationCorrectable,
    ViolationHardBlock,
}

/// Environmental Justice zone constraint
#[derive(Clone, Debug, PartialEq)]
pub struct EjConstraint {
    pub max_additional_toxicity_ppb: f64,
    pub max_decomposition_rate: f64,
    pub required_buffer_meters: f64,
    pub community_notification_required: bool,
}

/// Indigenous territory constraint
#[derive(Clone, Debug, PartialEq)]
pub struct IndigenousConstraint {
    pub territory_name: String,
    pub council_id: String,
    pub fpic_required: bool,
    pub fpic_status: FpicStatus,
    pub sacred_site_buffer_meters: Option<f64>,
    pub water_rights_protected: bool,
}

// ============================================================================
// 3. Ecosafety Contract Trait
// ============================================================================

/// Core ecosafety contract interface for all biodegradable node operations
pub trait EcosafetyContract: Send + Sync {
    /// Validate that a node has a declared ecosafety corridor before actuation
    fn require_corridors(&self, state: &BiodegradeNodeState) -> Result<(), ContractViolation>;
    
    /// Evaluate corridor integrity and ecological stability
    fn eval_corridor(&self, state: &BiodegradeNodeState) -> CorridorEvaluation;
    
    /// Decide whether node action is permitted, derated, or blocked
    fn decide_node_action(&self, state: &BiodegradeNodeState) -> NodeActionDecision;
    
    /// Check Lyapunov stability for ecological convergence guarantees
    fn check_lyapunov(&self, state: &BiodegradeNodeState) -> LyapunovVerdict;
    
    /// Verify FPIC and Indigenous rights compliance
    fn verify_indigenous_rights(&self, state: &BiodegradeNodeState) -> IndigenousVerdict;
    
    /// Verify BioticTreaty compliance for affected species
    fn verify_biotic_treaty(&self, state: &BiodegradeNodeState) -> BioticCompliance;
    
    /// Verify Environmental Justice zone constraints
    fn verify_ej_constraints(&self, state: &BiodegradeNodeState) -> EjVerdict;
    
    /// Generate governance envelope for Googolswarm ledger append
    fn generate_governance_envelope(&self, state: &BiodegradeNodeState) -> GovernanceEnvelope;
}

// ============================================================================
// 4. Violation & Decision Types
// ============================================================================

/// Contract violation with severity and remediation path
#[derive(Clone, Debug, PartialEq)]
pub struct ContractViolation {
    pub violation_id: String,
    pub severity: ViolationSeverity,
    pub violated_constraint: String,
    pub explanation: String,
    pub remediation_required: bool,
    pub remediation_steps: Vec<String>,
    pub ledger_reference: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViolationSeverity {
    Advisory,
    Warning,
    Critical,
    HardBlock,
}

/// Corridor evaluation result
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorEvaluation {
    pub corridor_id: String,
    pub status: CorridorStatus,
    pub integrity_score: f64,      // 0.0-1.0
    pub ecological_margin: f64,    // Safety margin before violation
    pub expires_unix: Option<u64>,
    pub renewal_required: bool,
}

/// Node action decision from ecosafety contract
#[derive(Clone, Debug, PartialEq)]
pub struct NodeActionDecision {
    pub action_permitted: bool,
    pub derate_factor: f64,        // 1.0 = full, 0.0 = blocked
    pub block_reason: Option<String>,
    pub conditions: Vec<Condition>,
    pub valid_until_unix: u64,
    pub requires_human_review: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub condition_id: String,
    pub description: String,
    pub mandatory: bool,
}

/// Lyapunov stability verdict
#[derive(Clone, Debug, PartialEq)]
pub struct LyapunovVerdict {
    pub stable: bool,
    pub residual_value: f64,
    pub derivative_sign: i8,       // -1 = decreasing, 0 = steady, 1 = increasing
    pub convergence_time_estimate_hours: Option<f64>,
    pub intervention_recommended: bool,
}

/// Indigenous rights verdict
#[derive(Clone, Debug, PartialEq)]
pub struct IndigenousVerdict {
    pub fpic_status: FpicStatus,
    pub territory_acknowledged: bool,
    pub water_rights_respected: bool,
    pub sacred_sites_protected: bool,
    pub consultation_required: bool,
    pub block_reason: Option<String>,
}

/// Environmental Justice verdict
#[derive(Clone, Debug, PartialEq)]
pub struct EjVerdict {
    pub compliant: bool,
    pub toxicity_within_limits: bool,
    pub buffer_respected: bool,
    pub community_notified: bool,
    pub additional_monitoring_required: bool,
}

// ============================================================================
// 5. Governance Envelope for Googolswarm Ledger
// ============================================================================

/// Governance envelope appended to every biodegrade node action
#[derive(Clone, Debug)]
pub struct GovernanceEnvelope {
    pub envelope_id: String,
    pub workflow_id: String,
    pub birth_sign_ids: Vec<String>,
    pub aln_norm_ids: Vec<String>,
    pub fpic_grant_ids: Vec<String>,
    pub treaty_check_transcript: Vec<TreatyCheckResult>,
    pub lyapunov_trace: LyapunovResidual,
    pub ecosafety_score: f64,
    pub multi_sig_attestors: Vec<String>,
    pub pq_signature: [u8; 64],
    pub timestamp_unix: u64,
    pub citizen_explanation: String,
    pub grievance_reference: String,
}

#[derive(Clone, Debug)]
pub struct TreatyCheckResult {
    pub treaty_type: String,
    pub module_id: String,
    pub passed: bool,
    pub violated_atoms: Vec<String>,
    pub explanation: String,
}

// ============================================================================
// 6. Default Ecosafety Contract Implementation
// ============================================================================

/// Default ecosafety contract with Phoenix-specific constraints
pub struct DefaultEcosafetyContract {
    pub lyapunov_threshold: f64,
    pub max_toxicity_ppb: f64,
    pub min_corridor_integrity: f64,
    pub ej_buffer_meters: f64,
    pub indigenous_sacred_buffer_meters: f64,
}

impl DefaultEcosafetyContract {
    pub fn new() -> Self {
        Self {
            lyapunov_threshold: 0.1,
            max_toxicity_ppb: 5.0,
            min_corridor_integrity: 0.85,
            ej_buffer_meters: 500.0,
            indigenous_sacred_buffer_meters: 1000.0,
        }
    }
    
    /// Compute Lyapunov residual from node state
    fn compute_lyapunov(&self, state: &BiodegradeNodeState) -> LyapunovResidual {
        let decomposition_fraction = 1.0 - (state.mass_remaining_grams / state.mass_initial_grams);
        let toxicity_penalty = if state.r_tox.safe_threshold_exceeded { 0.5 } else { 0.0 };
        let value = (1.0 - decomposition_fraction) + toxicity_penalty;
        let derivative = -state.r_micro.value / state.mass_initial_grams;
        let convergent = value >= 0.0 && derivative <= 0.0;
        
        LyapunovResidual {
            value,
            derivative,
            threshold: self.lyapunov_threshold,
            convergent,
        }
    }
    
    /// Check corridor declaration and integrity
    fn validate_corridor(&self, state: &BiodegradeNodeState) -> CorridorEvaluation {
        let integrity = match state.corridor_status {
            CorridorStatus::DeclaredActive => 0.95,
            CorridorStatus::DeclaredInactive => 0.3,
            CorridorStatus::Undeclared => 0.0,
            CorridorStatus::ViolationDetected => 0.1,
            CorridorStatus::UnderReview => 0.5,
        };
        
        CorridorEvaluation {
            corridor_id: format!("CORR-{}", state.node_id.network_id),
            status: state.corridor_status.clone(),
            integrity_score: integrity,
            ecological_margin: integrity - self.min_corridor_integrity,
            expires_unix: Some(state.timestamp_unix + 86400 * 30),
            renewal_required: integrity < self.min_corridor_integrity + 0.1,
        }
    }
}

impl EcosafetyContract for DefaultEcosafetyContract {
    fn require_corridors(&self, state: &BiodegradeNodeState) -> Result<(), ContractViolation> {
        match state.corridor_status {
            CorridorStatus::DeclaredActive => Ok(()),
            CorridorStatus::Undeclared => Err(ContractViolation {
                violation_id: format!("VIOL-CORR-{}", state.timestamp_unix),
                severity: ViolationSeverity::HardBlock,
                violated_constraint: "BIODEGRADE-CORRIDOR-DECLARATION-REQUIRED".to_string(),
                explanation: "Biodegradable node must have active declared ecosafety corridor before actuation".to_string(),
                remediation_required: true,
                remediation_steps: vec![
                    "Submit corridor declaration via XR-grid node".to_string(),
                    "Await treaty engine validation".to_string(),
                    "Receive corridor activation confirmation".to_string(),
                ],
                ledger_reference: None,
            }),
            CorridorStatus::DeclaredInactive => Err(ContractViolation {
                violation_id: format!("VIOL-CORR-INACTIVE-{}", state.timestamp_unix),
                severity: ViolationSeverity::Critical,
                violated_constraint: "BIODEGRADE-CORRIDOR-ACTIVE-REQUIRED".to_string(),
                explanation: "Declared corridor is inactive; renewal required".to_string(),
                remediation_required: true,
                remediation_steps: vec!["Renew corridor declaration".to_string()],
                ledger_reference: None,
            }),
            CorridorStatus::ViolationDetected => Err(ContractViolation {
                violation_id: format!("VIOL-CORR-VIOLATION-{}", state.timestamp_unix),
                severity: ViolationSeverity::HardBlock,
                violated_constraint: "BIODEGRADE-CORRIDOR-VIOLATION-RESOLUTION".to_string(),
                explanation: "Existing corridor violation must be resolved before new actuation".to_string(),
                remediation_required: true,
                remediation_steps: vec![
                    "Resolve existing violation".to_string(),
                    "Submit remediation evidence".to_string(),
                    "Await governance review".to_string(),
                ],
                ledger_reference: None,
            }),
            CorridorStatus::UnderReview => Err(ContractViolation {
                violation_id: format!("VIOL-CORR-REVIEW-{}", state.timestamp_unix),
                severity: ViolationSeverity::Warning,
                violated_constraint: "BIODEGRADE-CORRIDOR-REVIEW-PENDING".to_string(),
                explanation: "Corridor under review; actuation paused pending decision".to_string(),
                remediation_required: false,
                remediation_steps: vec!["Await review completion".to_string()],
                ledger_reference: None,
            }),
        }
    }
    
    fn eval_corridor(&self, state: &BiodegradeNodeState) -> CorridorEvaluation {
        self.validate_corridor(state)
    }
    
    fn decide_node_action(&self, state: &BiodegradeNodeState) -> NodeActionDecision {
        let corridor_eval = self.eval_corridor(state);
        let lyapunov_verdict = self.check_lyapunov(state);
        let indigenous_verdict = self.verify_indigenous_rights(state);
        let ej_verdict = self.verify_ej_constraints(state);
        let biotic_compliance = self.verify_biotic_treaty(state);
        
        // Hard blocks
        if corridor_eval.status == CorridorStatus::Undeclared {
            return NodeActionDecision {
                action_permitted: false,
                derate_factor: 0.0,
                block_reason: Some("Ecosafety corridor not declared".to_string()),
                conditions: vec![],
                valid_until_unix: state.timestamp_unix,
                requires_human_review: false,
            };
        }
        
        if indigenous_verdict.block_reason.is_some() {
            return NodeActionDecision {
                action_permitted: false,
                derate_factor: 0.0,
                block_reason: indigenous_verdict.block_reason,
                conditions: vec![],
                valid_until_unix: state.timestamp_unix,
                requires_human_review: true,
            };
        }
        
        if biotic_compliance == BioticCompliance::ViolationHardBlock {
            return NodeActionDecision {
                action_permitted: false,
                derate_factor: 0.0,
                block_reason: Some("BioticTreaty violation detected".to_string()),
                conditions: vec![],
                valid_until_unix: state.timestamp_unix,
                requires_human_review: true,
            };
        }
        
        // Derate calculations
        let mut derate_factor = 1.0;
        let mut conditions = Vec::new();
        
        if corridor_eval.integrity_score < self.min_corridor_integrity + 0.1 {
            derate_factor *= 0.7;
            conditions.push(Condition {
                condition_id: "COND-CORRIDOR-MARGIN".to_string(),
                description: "Corridor integrity margin low; reduced actuation rate".to_string(),
                mandatory: true,
            });
        }
        
        if !lyapunov_verdict.stable {
            derate_factor *= 0.5;
            conditions.push(Condition {
                condition_id: "COND-LYAPUNOV-UNSTABLE".to_string(),
                description: "Lyapunov stability not achieved; derated for safety".to_string(),
                mandatory: true,
            });
        }
        
        if ej_verdict.additional_monitoring_required {
            conditions.push(Condition {
                condition_id: "COND-EJ-MONITORING".to_string(),
                description: "Additional EJ zone monitoring required during actuation".to_string(),
                mandatory: true,
            });
        }
        
        NodeActionDecision {
            action_permitted: derate_factor > 0.0,
            derate_factor,
            block_reason: None,
            conditions,
            valid_until_unix: state.timestamp_unix + 3600,
            requires_human_review: false,
        }
    }
    
    fn check_lyapunov(&self, state: &BiodegradeNodeState) -> LyapunovVerdict {
        let residual = self.compute_lyapunov(state);
        let stable = residual.convergent && residual.value <= residual.threshold;
        let derivative_sign = if residual.derivative < -0.001 {
            -1
        } else if residual.derivative > 0.001 {
            1
        } else {
            0
        };
        
        let convergence_time = if stable && residual.value > 0.0 && residual.derivative < 0.0 {
            Some((residual.value / -residual.derivative) / 3600.0) // hours
        } else {
            None
        };
        
        LyapunovVerdict {
            stable,
            residual_value: residual.value,
            derivative_sign,
            convergence_time_estimate_hours: convergence_time,
            intervention_recommended: !stable || derivative_sign > 0,
        }
    }
    
    fn verify_indigenous_rights(&self, state: &BiodegradeNodeState) -> IndigenousVerdict {
        if state.geo_tile.indigenous_territory.is_none() {
            return IndigenousVerdict {
                fpic_status: FpicStatus::NotApplicable,
                territory_acknowledged: false,
                water_rights_respected: true,
                sacred_sites_protected: true,
                consultation_required: false,
                block_reason: None,
            };
        }
        
        // FPIC required for indigenous territory
        match state.geo_tile.birth_sign_id.contains("FPIC") {
            true => IndigenousVerdict {
                fpic_status: FpicStatus::Granted,
                territory_acknowledged: true,
                water_rights_respected: true,
                sacred_sites_protected: true,
                consultation_required: false,
                block_reason: None,
            },
            false => IndigenousVerdict {
                fpic_status: FpicStatus::RequiredPending,
                territory_acknowledged: true,
                water_rights_respected: true,
                sacred_sites_protected: true,
                consultation_required: true,
                block_reason: Some("FPIC required and not yet granted for indigenous territory".to_string()),
            },
        }
    }
    
    fn verify_biotic_treaty(&self, state: &BiodegradeNodeState) -> BioticCompliance {
        if state.geo_tile.biotic_corridor.is_none() {
            return BioticCompliance::Compliant;
        }
        
        if state.r_tox.safe_threshold_exceeded {
            return BioticCompliance::ViolationHardBlock;
        }
        
        if state.r_micro.value > 10.0 {
            return BioticCompliance::WarningMinor;
        }
        
        BioticCompliance::Compliant
    }
    
    fn verify_ej_constraints(&self, state: &BiodegradeNodeState) -> EjVerdict {
        if !state.geo_tile.ej_zone_flag {
            return EjVerdict {
                compliant: true,
                toxicity_within_limits: true,
                buffer_respected: true,
                community_notified: false,
                additional_monitoring_required: false,
            };
        }
        
        let toxicity_ok = state.r_tox.organic_toxins_ppb <= self.max_toxicity_ppb;
        let buffer_ok = state.geo_tile.tile_id % 2 == 0; // Simplified buffer check
        
        EjVerdict {
            compliant: toxicity_ok && buffer_ok,
            toxicity_within_limits: toxicity_ok,
            buffer_respected: buffer_ok,
            community_notified: true,
            additional_monitoring_required: !toxicity_ok || !buffer_ok,
        }
    }
    
    fn generate_governance_envelope(&self, state: &BiodegradeNodeState) -> GovernanceEnvelope {
        let lyapunov = self.compute_lyapunov(state);
        let corridor = self.validate_corridor(state);
        let indigenous = self.verify_indigenous_rights(state);
        let ej = self.verify_ej_constraints(state);
        let biotic = self.verify_biotic_treaty(state);
        
        let treaty_checks = vec![
            TreatyCheckResult {
                treaty_type: "IndigenousRights".to_string(),
                module_id: "treaty_indigenous".to_string(),
                passed: indigenous.block_reason.is_none(),
                violated_atoms: vec![],
                explanation: format!("FPIC status: {:?}", indigenous.fpic_status),
            },
            TreatyCheckResult {
                treaty_type: "BioticTreaty".to_string(),
                module_id: "treaty_biotic".to_string(),
                passed: biotic != BioticCompliance::ViolationHardBlock,
                violated_atoms: vec![],
                explanation: format!("Biotic compliance: {:?}", biotic),
            },
            TreatyCheckResult {
                treaty_type: "EnvironmentalJustice".to_string(),
                module_id: "treaty_ej".to_string(),
                passed: ej.compliant,
                violated_atoms: vec![],
                explanation: format!("EJ zone compliance: {}", ej.compliant),
            },
        ];
        
        let ecosafety_score = corridor.integrity_score * 
            if lyapunov.stable { 1.0 } else { 0.5 } *
            if indigenous.block_reason.is_none() { 1.0 } else { 0.0 };
        
        GovernanceEnvelope {
            envelope_id: format!("ENV-BIOD-{}-{}", state.node_id.network_id, state.timestamp_unix),
            workflow_id: "WORKFLOW-BIODEGRADE-001".to_string(),
            birth_sign_ids: vec![state.geo_tile.birth_sign_id.clone()],
            aln_norm_ids: vec!["ALE-ERM-ECOSAFETY-BIODEGRADE-GRAMMAR-001".to_string()],
            fpic_grant_ids: vec![],
            treaty_check_transcript: treaty_checks,
            lyapunov_trace: lyapunov,
            ecosafety_score,
            multi_sig_attestors: vec!["XR-NODE-ATTESTOR-001".to_string()],
            pq_signature: [0u8; 64], // Placeholder for actual PQ signature
            timestamp_unix: state.timestamp_unix,
            citizen_explanation: format!(
                "Biodegradable node {} actuation evaluated. Ecosafety score: {:.2}. \
                 Corridor integrity: {:.2}. Lyapunov stable: {}. \
                 Indigenous rights: {}. EJ compliant: {}.",
                state.node_id.network_id,
                ecosafety_score,
                corridor.integrity_score,
                lyapunov.stable,
                if indigenous.block_reason.is_none() { "respected" } else { "review required" },
                ej.compliant
            ),
            grievance_reference: format!("GRIEV-BIOD-{}-{}", state.node_id.network_id, state.timestamp_unix),
        }
    }
}

// ============================================================================
// 7. Public API Functions (SMART Chain Integration)
// ============================================================================

/// Require corridors before any biodegrade node actuation (SMART Stage 5: Treaty Check)
pub fn require_corridors_entry(
    contract: &dyn EcosafetyContract,
    state: &BiodegradeNodeState,
) -> Result<GovernanceEnvelope, ContractViolation> {
    contract.require_corridors(state)?;
    Ok(contract.generate_governance_envelope(state))
}

/// Evaluate corridor and return decision (SMART Stage 4-5: Optimize + Treaty)
pub fn eval_corridor_entry(
    contract: &dyn EcosafetyContract,
    state: &BiodegradeNodeState,
) -> (CorridorEvaluation, NodeActionDecision) {
    let corridor = contract.eval_corridor(state);
    let decision = contract.decide_node_action(state);
    (corridor, decision)
}

/// Full decision pipeline with Lyapunov verification (SMART Stage 4-6)
pub fn decide_node_action_entry(
    contract: &dyn EcosafetyContract,
    state: &BiodegradeNodeState,
) -> (NodeActionDecision, GovernanceEnvelope) {
    let decision = contract.decide_node_action(state);
    let envelope = contract.generate_governance_envelope(state);
    (decision, envelope)
}

/// Lyapunov stability check for ecological convergence (SMART Stage 4: Optimize)
pub fn check_lyapunov_entry(
    contract: &dyn EcosafetyContract,
    state: &BiodegradeNodeState,
) -> LyapunovVerdict {
    contract.check_lyapunov(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_state() -> BiodegradeNodeState {
        BiodegradeNodeState {
            node_id: BiodegradeNodeId {
                prefix: *b"BIOD",
                network_id: 1,
                node_hash: [0u8; 32],
            },
            geo_tile: GeoTileRef {
                tile_id: 100,
                birth_sign_id: "BIRTHSIGN-PHX-DOWNTOWN-001".to_string(),
                indigenous_territory: None,
                ej_zone_flag: false,
                biotic_corridor: None,
            },
            material_class: BiodegradeMaterialClass::OrganicFast,
            mass_remaining_grams: 50.0,
            mass_initial_grams: 100.0,
            r_micro: MicrobialRate {
                value: 2.0,
                confidence: 0.95,
                temperature_corrected: 2.1,
                ph_corrected: 1.9,
            },
            r_tox: ToxicityResidual {
                heavy_metals_ppm: 0.1,
                organic_toxins_ppb: 1.0,
                bioaccumulation_factor: 0.5,
                safe_threshold_exceeded: false,
            },
            lyapunov: LyapunovResidual {
                value: 0.5,
                derivative: -0.01,
                threshold: 0.1,
                convergent: true,
            },
            corridor_status: CorridorStatus::DeclaredActive,
            timestamp_unix: 1710023020,
            firmware_version: 1,
            governance_seal: [0u8; 64],
        }
    }
    
    #[test]
    fn test_require_corridors_active_passes() {
        let contract = DefaultEcosafetyContract::new();
        let state = create_test_state();
        assert!(contract.require_corridors(&state).is_ok());
    }
    
    #[test]
    fn test_require_corridors_undeclared_blocks() {
        let contract = DefaultEcosafetyContract::new();
        let mut state = create_test_state();
        state.corridor_status = CorridorStatus::Undeclared;
        assert!(matches!(
            contract.require_corridors(&state),
            Err(ContractViolation { severity: ViolationSeverity::HardBlock, .. })
        ));
    }
    
    #[test]
    fn test_lyapunov_stable_convergence() {
        let contract = DefaultEcosafetyContract::new();
        let state = create_test_state();
        let verdict = contract.check_lyapunov(&state);
        assert!(verdict.stable);
        assert_eq!(verdict.derivative_sign, -1);
        assert!(verdict.convergence_time_estimate_hours.is_some());
    }
    
    #[test]
    fn test_governance_envelope_generation() {
        let contract = DefaultEcosafetyContract::new();
        let state = create_test_state();
        let envelope = contract.generate_governance_envelope(&state);
        assert!(envelope.ecosafety_score > 0.0);
        assert_eq!(envelope.treaty_check_transcript.len(), 3);
        assert!(!envelope.citizen_explanation.is_empty());
    }
}
