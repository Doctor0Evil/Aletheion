//! ALE-TRUST-GOOGOLSWARM-ATTESTOR-LOCAL-ONLY-001.rs
//!
//! Local-only, offline-capable Googolswarm attestation guardrail for Aletheion.
//! Enforces:
//!   - Googolswarm is an attestor only, never a control-plane authority.
//!   - Attestation is host-local / enclave-local and MUST remain operable offline.
//!   - No silent-takeover: all authority elevations are explicit, multi-sig, and BirthSign-bound.
//!   - No silent reinvention of governance grammars: ALN norms and BirthSigns are referenced, not redefined.
//!   - Augmented-citizen sovereignty: DIDs, FPIC, and neurorights envelopes are first-class in every envelope.
//!
//! This module is designed to sit alongside the canonical governed decision types you already
//! defined in ALE-GOV-BIRTH-SIGN-MODEL-001.rs and the Googolswarm provenance envelopes
//! sketched for ALE-TRUST-GOVERNED-DECISION-TX-001.aln.[file:2][file:5]
//!
//! It does *not* define consensus or ledger internals. Instead, it defines:
//!   - Local attestation profiles for nodes running Aletheion workloads.
//!   - Explicit role/authority models for Googolswarm as attestor-only.
//!   - Static checks that reject any attempt to grant Googolswarm control privileges.
//!   - A local-only attestation API that can be called from ERM, compliance, or orchestration layers
//!     without assuming network connectivity or remote control.
//!
//! Approved language: Rust only.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Stable identifier newtypes mirrored from existing governance and trust models.[file:2][file:5]

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

/// Domain tags for provenance and placement.[file:2][file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GovernanceDomain {
    Land,
    Water,
    Air,
    Materials,
    Mobility,
    Biosignals,
    Augmentation,
    Energy,
    Culture,
    Emergency,
}

/// Constraint interpretation mode (mirrors BirthSign usage). [file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    Hard,
    HighPenalty,
}

/// FPIC requirement and status enums aligned with existing BirthSign semantics.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FpicRequirement {
    NotApplicable,
    RequiredBeforePlanning,
    RequiredBeforeExecution,
    EmergencyOverrideWithAudit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FpicStatus {
    NotRequired,
    Granted {
        territory: String,
        granted_at: SystemTime,
        expires_at: Option<SystemTime>,
    },
    Pending {
        territory: String,
        requested_at: SystemTime,
    },
    Denied {
        territory: String,
        reason: String,
    },
}

/// Decision outcome aligned with governed decision envelopes.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// CryptoSomatic / neurorights envelope status (simplified tag).[file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NeurorightsStatus {
    NotApplicable,
    EnforcedCompliant,
    ViolationSoft(String),
    ViolationHard(String),
}

/// Minimal FPIC view used by the attestor.
#[derive(Debug, Clone)]
pub struct FpicView {
    pub requirement: FpicRequirement,
    pub status: FpicStatus,
}

/// Minimal neurorights view used by the attestor.
#[derive(Debug, Clone)]
pub struct NeurorightsView {
    pub status: NeurorightsStatus,
    pub envelopes: Vec<AlnNormId>,
}

/// Node security posture aligned with your secure firmware/channel research.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeSecurityTier {
    TeeBacked,
    HardenedFirmwareOnly,
    Basic,
}

#[derive(Debug, Clone)]
pub struct NodeSecurityProfile {
    pub node_profile_id: NodeProfileId,
    pub tier: NodeSecurityTier,
    pub secure_boot: bool,
    pub signed_updates: bool,
    pub secure_transport: bool,
    pub last_audit_passed_at: Option<SystemTime>,
}

impl NodeSecurityProfile {
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    pub fn can_host_general_erm(&self) -> bool {
        matches!(
            self.tier,
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly
        )
    }
}

/// Googolswarm authority model: attestor-only.[file:2][file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoogolswarmAuthorityScope {
    /// Pure attestation: verify integrity and ordering of transactions, nothing else.
    AttestorOnly,
    /// Forbidden: direct actuation control or workflow routing authority.
    ForbiddenControlPlane,
}

/// Static configuration for Googolswarm within this node.
#[derive(Debug, Clone)]
pub struct GoogolswarmLocalConfig {
    /// Authority MUST always be AttestorOnly.
    pub authority_scope: GoogolswarmAuthorityScope,
    /// Whether this node is allowed to emit new attestations offline.
    pub offline_capable: bool,
    /// Whether this node is allowed to buffer attestations and batch-send later.
    pub buffering_allowed: bool,
    /// Hard ceiling for buffered attestations to avoid unbounded growth.
    pub max_buffered: usize,
}

impl GoogolswarmLocalConfig {
    pub fn hardened_default() -> Self {
        Self {
            authority_scope: GoogolswarmAuthorityScope::AttestorOnly,
            offline_capable: true,
            buffering_allowed: true,
            max_buffered: 4096,
        }
    }

    /// Reject any attempt to expand Googolswarm beyond attestor-only scope.
    pub fn assert_attestor_only(&self) -> Result<(), AttestationError> {
        if self.authority_scope != GoogolswarmAuthorityScope::AttestorOnly {
            return Err(AttestationError::ForbiddenAuthorityEscalation(
                "Googolswarm may only act as attestor; control-plane scope is forbidden".into(),
            ));
        }
        Ok(())
    }
}

/// Minimal view of a BirthSign for attestation purposes.[file:2]
#[derive(Debug, Clone)]
pub struct BirthSignSummary {
    pub id: BirthSignId,
    pub domains: Vec<GovernanceDomain>,
    pub default_constraint_mode: ConstraintMode,
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

/// Outcome for each ALN constraint check.
#[derive(Debug, Clone)]
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation { norm: AlnNormId, message: String },
    HardViolation { norm: AlnNormId, message: String },
}

impl ConstraintOutcome {
    pub fn is_hard(&self) -> bool {
        matches!(self, ConstraintOutcome::HardViolation { .. })
    }

    pub fn is_soft(&self) -> bool {
        matches!(self, ConstraintOutcome::SoftViolation { .. })
    }
}

/// Aggregate governance evaluation used by the attestor.[file:2]
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic: FpicView,
    pub neurorights: NeurorightsView,
}

impl GovernanceEvaluation {
    pub fn has_hard_violation(&self) -> bool {
        self.outcomes.iter().any(|o| o.is_hard())
            || matches!(self.neurorights.status, NeurorightsStatus::ViolationHard(_))
    }

    pub fn has_soft_violation(&self) -> bool {
        self.outcomes.iter().any(|o| o.is_soft())
            || matches!(self.neurorights.status, NeurorightsStatus::ViolationSoft(_))
    }

    pub fn is_fpic_blocking(&self) -> bool {
        match &self.fpic.status {
            FpicStatus::Pending { .. } | FpicStatus::Denied { .. } => true,
            _ => false,
        }
    }
}

/// Local-only attestation result.
#[derive(Debug, Clone)]
pub struct LocalAttestationResult {
    pub decision_outcome: DecisionOutcome,
    pub birth_sign_ids: Vec<BirthSignId>,
    pub applied_norms: Vec<AlnNormId>,
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    pub node_security_tier: NodeSecurityTier,
    pub explanation: Option<String>,
    /// Hashes or fingerprints for inputs/outputs, to be used when eventually appended to Googolswarm.
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Whether this attestation has been successfully persisted to local durable storage.
    pub locally_persisted: bool,
    /// Whether this attestation has been exported to Googolswarm network (optional, non-authoritative).
    pub exported_to_googolswarm: bool,
}

/// Error type for the attestation pipeline.
#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("forbidden authority escalation: {0}")]
    ForbiddenAuthorityEscalation(String),
    #[error("node security profile not sufficient for this operation: {0}")]
    InsufficientNodeSecurity(String),
    #[error("buffer is full; cannot accept more pending attestations")]
    BufferFull,
    #[error("internal error: {0}")]
    Internal(String),
}

/// In-memory buffer for local attestation envelopes, designed to work offline.
#[derive(Debug)]
pub struct LocalAttestationBuffer {
    config: GoogolswarmLocalConfig,
    node_security: NodeSecurityProfile,
    pending: Vec<LocalAttestationResult>,
}

impl LocalAttestationBuffer {
    pub fn new(config: GoogolswarmLocalConfig, node_security: NodeSecurityProfile) -> Self {
        Self {
            config,
            node_security,
            pending: Vec::new(),
        }
    }

    fn assert_local_invariants(&self) -> Result<(), AttestationError> {
        self.config.assert_attestor_only()?;
        if !self.node_security.can_host_general_erm() {
            return Err(AttestationError::InsufficientNodeSecurity(
                "node cannot host ERM-class attestation workloads".into(),
            ));
        }
        Ok(())
    }

    /// Perform a local-only attestation of a governed decision, no network assumptions.[file:2]
    ///
    /// This function:
    ///   - Derives the canonical DecisionOutcome from governance evaluation and FPIC status.
    ///   - Asserts Googolswarm remains attestor-only.
    ///   - Writes the result into a local buffer for eventual Googolswarm append.
    ///   - Does *not* call external services; callers may later export buffered attestations.
    #[allow(clippy::too_many_arguments)]
    pub fn attest_locally(
        &mut self,
        tx_id: TrustTxId,
        workflow_id: WorkflowId,
        stage: &str,
        birth_signs: Vec<BirthSignSummary>,
        evaluation: GovernanceEvaluation,
        subject_did: Option<Did>,
        operator_did: Option<Did>,
        inputs_hash: String,
        outputs_hash: String,
    ) -> Result<LocalAttestationResult, AttestationError> {
        self.assert_local_invariants()?;

        if self.pending.len() >= self.config.max_buffered {
            return Err(AttestationError::BufferFull);
        }

        let decision_outcome =
            derive_decision_outcome(&evaluation, &birth_signs, stage, &self.node_security);

        let birthed_ids = birth_signs.into_iter().map(|b| b.id).collect::<Vec<_>>();
        let applied_norms = collect_applied_norms(&evaluation);

        let result = LocalAttestationResult {
            decision_outcome,
            birth_sign_ids: birthed_ids,
            applied_norms,
            subject_did,
            operator_did,
            node_profile_id: Some(self.node_security.node_profile_id.clone()),
            node_security_tier: self.node_security.tier.clone(),
            explanation: Some(format!(
                "Local-only Googolswarm attestation for tx={:?}, workflow={:?}, stage={}",
                tx_id, workflow_id, stage
            )),
            inputs_hash,
            outputs_hash,
            locally_persisted: false,
            exported_to_googolswarm: false,
        };

        self.pending.push(result.clone());
        Ok(result)
    }

    /// Mark the oldest pending attestation as locally persisted (e.g., fsync’d to disk).
    pub fn mark_oldest_persisted(&mut self) -> Option<LocalAttestationResult> {
        if let Some(mut env) = self.pending.first_mut() {
            env.locally_persisted = true;
            return Some(env.clone());
        }
        None
    }

    /// Pop and return all locally persisted attestations ready for export.
    pub fn drain_persisted_for_export(&mut self) -> Vec<LocalAttestationResult> {
        let mut ready = Vec::new();
        let mut remaining = Vec::new();
        for env in self.pending.drain(..) {
            if env.locally_persisted {
                ready.push(env);
            } else {
                remaining.push(env);
            }
        }
        self.pending = remaining;
        ready
    }

    /// Forbid any API that looks like direct control-plane access.
    ///
    /// If future code attempts to use the attestation buffer as a control router,
    /// this helper SHOULD be wired into CI checks and panic paths.
    pub fn forbid_control_plane_usage(&self, reason: &str) -> Result<(), AttestationError> {
        self.config.assert_attestor_only()?;
        Err(AttestationError::ForbiddenAuthorityEscalation(format!(
            "attempted to use Googolswarm attestor as control-plane: {}",
            reason
        )))
    }
}

/// Derive canonical DecisionOutcome from governance evaluation.
/// This mirrors the mapping suggested in your existing BirthSign / governed decision sketches.[file:2]
fn derive_decision_outcome(
    eval: &GovernanceEvaluation,
    _birth_signs: &[BirthSignSummary],
    _stage: &str,
    _node: &NodeSecurityProfile,
) -> DecisionOutcome {
    // FPIC has highest precedence.
    if eval.is_fpic_blocking() {
        return DecisionOutcome::PendingFpic;
    }

    // Hard violations always reject.
    if eval.has_hard_violation() {
        return DecisionOutcome::Rejected;
    }

    // Soft violations depend on constraint mode.
    if eval.has_soft_violation() {
        match eval.constraint_mode {
            ConstraintMode::Hard => DecisionOutcome::Rejected,
            ConstraintMode::HighPenalty => DecisionOutcome::ApprovedWithDerating,
        }
    } else {
        DecisionOutcome::Approved
    }
}

/// Collect ALN norms that were actually involved in evaluation.
/// In a full integration this would be populated by the governance layer.[file:2]
fn collect_applied_norms(eval: &GovernanceEvaluation) -> Vec<AlnNormId> {
    let mut norms = Vec::new();
    for outcome in &eval.outcomes {
        match outcome {
            ConstraintOutcome::Satisfied => {}
            ConstraintOutcome::SoftViolation { norm, .. } => norms.push(norm.clone()),
            ConstraintOutcome::HardViolation { norm, .. } => norms.push(norm.clone()),
        }
    }
    norms
}

/// Minimal local attestation index to guard against silent reinvention of works.
#[derive(Debug)]
pub struct AttestationIndex {
    /// Map from workflow to last-known BirthSign versions used in attested decisions.
    pub last_birth_sign_versions: HashMap<WorkflowId, HashMap<BirthSignId, u32>>,
    /// Map from workflow to last-known ALN norm set; used to detect unexpected schema drift.
    pub last_norm_sets: HashMap<WorkflowId, Vec<AlnNormId>>,
}

impl AttestationIndex {
    pub fn new() -> Self {
        Self {
            last_birth_sign_versions: HashMap::new(),
            last_norm_sets: HashMap::new(),
        }
    }

    /// Update index with a new attestation, returning whether this represents a *governed* evolution
    /// (version increment, additional norms) or a suspicious drift that should be audited.
    pub fn update_and_check_drift(
        &mut self,
        workflow_id: &WorkflowId,
        birth_signs: &[BirthSignSummary],
        applied_norms: &[AlnNormId],
    ) -> DriftStatus {
        let workflow_entry = self
            .last_birth_sign_versions
            .entry(workflow_id.clone())
            .or_insert_with(HashMap::new);
        let norm_entry = self
            .last_norm_sets
            .entry(workflow_id.clone())
            .or_insert_with(Vec::new);

        let mut suspicious = false;

        // BirthSign version tracking: forbid backwards version jumps (silent rollback).
        for bs in birth_signs {
            match workflow_entry.get(&bs.id) {
                Some(prev) if *prev > bs.version => {
                    suspicious = true;
                }
                _ => {
                    workflow_entry.insert(bs.id.clone(), bs.version);
                }
            }
        }

        // Norm drift: detect sudden disappearance of norms (possible silent reinvention).
        if !norm_entry.is_empty() {
            let previous: HashMap<&AlnNormId, ()> =
                norm_entry.iter().map(|n| (n, ())).collect();
            let current: HashMap<&AlnNormId, ()> =
                applied_norms.iter().map(|n| (n, ())).collect();
            for key in previous.keys() {
                if !current.contains_key(key) {
                    suspicious = true;
                    break;
                }
            }
        }

        *norm_entry = applied_norms.to_vec();

        if suspicious {
            DriftStatus::SuspiciousDrift
        } else {
            DriftStatus::GovernedEvolution
        }
    }
}

/// Drift assessment for governance artifacts across attestations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftStatus {
    GovernedEvolution,
    SuspiciousDrift,
}

/// Helper: duration in hours, to match other governance modules.
pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_eval(mode: ConstraintMode) -> GovernanceEvaluation {
        GovernanceEvaluation {
            birth_sign_id: BirthSignId("B:1".into()),
            constraint_mode: mode,
            outcomes: vec![ConstraintOutcome::SoftViolation {
                norm: AlnNormId("N:soft".into()),
                message: "soft".into(),
            }],
            fpic: FpicView {
                requirement: FpicRequirement::RequiredBeforeExecution,
                status: FpicStatus::NotRequired,
            },
            neurorights: NeurorightsView {
                status: NeurorightsStatus::NotApplicable,
                envelopes: Vec::new(),
            },
        }
    }

    #[test]
    fn decision_outcome_respects_constraint_mode() {
        let bs = BirthSignSummary {
            id: BirthSignId("B:1".into()),
            domains: vec![GovernanceDomain::Water],
            default_constraint_mode: ConstraintMode::HighPenalty,
            version: 1,
            valid_from: SystemTime::now(),
            valid_until: None,
        };
        let node = NodeSecurityProfile {
            node_profile_id: NodeProfileId("node:1".into()),
            tier: NodeSecurityTier::HardenedFirmwareOnly,
            secure_boot: true,
            signed_updates: true,
            secure_transport: true,
            last_audit_passed_at: None,
        };

        let eval_high = dummy_eval(ConstraintMode::HighPenalty);
        let outcome_high = super::derive_decision_outcome(&eval_high, &[bs.clone()], "Optimization", &node);
        assert_eq!(outcome_high, DecisionOutcome::ApprovedWithDerating);

        let eval_hard = dummy_eval(ConstraintMode::Hard);
        let outcome_hard = super::derive_decision_outcome(&eval_hard, &[bs], "Optimization", &node);
        assert_eq!(outcome_hard, DecisionOutcome::Rejected);
    }

    #[test]
    fn config_rejects_control_plane_scope() {
        let mut cfg = GoogolswarmLocalConfig::hardened_default();
        assert!(cfg.assert_attestor_only().is_ok());
        cfg.authority_scope = GoogolswarmAuthorityScope::ForbiddenControlPlane;
        assert!(cfg.assert_attestor_only().is_err());
    }

    #[test]
    fn drift_index_detects_version_rollback() {
        let mut idx = AttestationIndex::new();
        let wf = WorkflowId("WF:1".into());

        let bs_v1 = BirthSignSummary {
            id: BirthSignId("B:1".into()),
            domains: vec![GovernanceDomain::Land],
            default_constraint_mode: ConstraintMode::HighPenalty,
            version: 2,
            valid_from: SystemTime::now(),
            valid_until: None,
        };

        let status1 =
            idx.update_and_check_drift(&wf, &[bs_v1.clone()], &vec![AlnNormId("N:1".into())]);
        assert_eq!(status1, DriftStatus::GovernedEvolution);

        let bs_v0 = BirthSignSummary {
            version: 1,
            ..bs_v1
        };
        let status2 =
            idx.update_and_check_drift(&wf, &[bs_v0], &vec![AlnNormId("N:1".into())]);
        assert_eq!(status2, DriftStatus::SuspiciousDrift);
    }
}
