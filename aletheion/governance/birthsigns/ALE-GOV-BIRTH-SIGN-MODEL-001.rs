// Core Birth-Sign and governed-decision model for Aletheion.
// Ties territorial governance (laws, Indigenous protocols, ecological protections,
// local overlays) to concrete workflow actions, and prepares envelopes for
// Googolswarm-compatible governed decision transactions.
// Layers: L2 State Modeling, L3 Trust, L4 Optimization, L5 Citizen Interface.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// -------------------------
/// Core identifier newtypes
/// -------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndigenousTerritoryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BioticTreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MicroTreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowEventId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeProfileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrustTxId(pub String);

/// -------------------------
/// Territorial domain enums
/// -------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocalOverlayKind {
    NeighborhoodMicroTreaty,
    WorkplacePolicy,
    SchoolOrCampusPolicy,
    FacilitySpecificProtocol,
    EventSpecificProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FpicRequirement {
    NotApplicable,
    RequiredBeforePlanning,
    RequiredBeforeExecution,
    EmergencyOverrideWithAudit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FpicStatus {
    NotRequired,
    Granted {
        territory: IndigenousTerritoryId,
        granted_at: SystemTime,
        expires_at: Option<SystemTime>,
    },
    Pending {
        territory: IndigenousTerritoryId,
        requested_at: SystemTime,
    },
    Denied {
        territory: IndigenousTerritoryId,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintMode {
    /// Violations MUST cause immediate rejection.
    Hard,
    /// Violations SHOULD be avoided (high penalty), MAY be allowed under
    /// governed emergency scenarios with additional audit.
    HighPenalty,
}

/// -------------------------
/// BirthSign components
/// -------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LawRef {
    pub scope: LawScope,
    pub aln_norm: AlnNormId,
    /// Optional human label or citation (statute reference, ordinance id).
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndigenousGovernance {
    pub territory_id: IndigenousTerritoryId,
    pub fpic_requirement: FpicRequirement,
    /// TEK envelopes as ALN norms attached to this tile.
    pub tek_norms: Vec<AlnNormId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EcologicalProtections {
    /// Species-level BioticTreaties that apply here.
    pub biotic_treaties: Vec<BioticTreatyId>,
    /// Habitat corridor continuity envelopes (ALN norms).
    pub habitat_corridor_norms: Vec<AlnNormId>,
    /// Light, noise, and chemical use limits (ALN norms).
    pub light_noise_chemical_norms: Vec<AlnNormId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalOverlay {
    pub kind: LocalOverlayKind,
    pub micro_treaty_id: MicroTreatyId,
    /// Short human-readable summary for citizen surfaces.
    pub summary: String,
}

/// Unified Birth-Sign record for a spatial tile and time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthSign {
    pub id: BirthSignId,
    /// Opaque reference into the geospatial DB (tile id, not raw geometry).
    pub tile_ref: String,
    /// Domains where this Birth-Sign is authoritative.
    pub domains: Vec<GovernanceDomain>,
    /// Governing public law references.
    pub laws: Vec<LawRef>,
    /// Indigenous territories and protocols on this tile.
    pub indigenous: Vec<IndigenousGovernance>,
    /// Ecological and cross-species protections.
    pub ecological: EcologicalProtections,
    /// Local overlays: LexEthos micro-treaties, neighborhood norms, etc.
    pub local_overlays: Vec<LocalOverlay>,
    /// Default enforcement mode for constraints derived from this Birth-Sign.
    pub default_constraint_mode: ConstraintMode,
    /// Versioned, time-bounded validity for audit and forward-only evolution.
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

impl BirthSign {
    /// Returns true if this Birth-Sign is valid at the given time.
    pub fn is_active_at(&self, t: SystemTime) -> bool {
        if t < self.valid_from {
            return false;
        }
        if let Some(end) = self.valid_until {
            if t > end {
                return false;
            }
        }
        true
    }

    /// Returns true if this Birth-Sign governs the given domain.
    pub fn covers_domain(&self, domain: &GovernanceDomain) -> bool {
        self.domains.contains(domain)
    }
}

/// Binding between BirthSigns and assets/events in the ERM pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthSignBinding {
    pub birth_sign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    /// Optional free-form metadata (labels, debug tags, implementation hints).
    pub metadata: HashMap<String, String>,
}

/// -------------------------
/// Constraint evaluation
/// -------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation {
        aln_norm: AlnNormId,
        message: String,
    },
    HardViolation {
        aln_norm: AlnNormId,
        message: String,
    },
}

/// Aggregate evaluation for a Birth-Sign–scoped action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

impl GovernanceEvaluation {
    /// True if all outcomes are satisfied and FPIC status is non-blocking.
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(fpic) = &self.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } | FpicStatus::Denied { .. } => return false,
                FpicStatus::NotRequired | FpicStatus::Granted { .. } => {}
            }
        }
        self.outcomes
            .iter()
            .all(|o| matches!(o, ConstraintOutcome::Satisfied))
    }

    pub fn has_hard_violation(&self) -> bool {
        self.outcomes
            .iter()
            .any(|o| matches!(o, ConstraintOutcome::HardViolation { .. }))
    }

    pub fn has_soft_violation(&self) -> bool {
        self.outcomes
            .iter()
            .any(|o| matches!(o, ConstraintOutcome::SoftViolation { .. }))
    }
}

/// High-level decision outcome used by optimization and actuation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// -------------------------
/// Governed decision envelope
/// -------------------------

/// A governed decision is the bridge between optimization, governance,
/// and the Googolswarm trust layer. Designed to serialize into the
/// canonical governed-decision Tx schema. [file:2][file:5]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    /// e.g., "Optimization", "GovernancePreflight", "Actuation"
    pub workflow_stage: String,
    pub domains: Vec<GovernanceDomain>,
    /// Spatial context: one or more Birth-Signs that applied.
    pub birth_sign_ids: Vec<BirthSignId>,
    /// ALN norms actually consulted (laws, treaties, overlays).
    pub applied_aln_norms: Vec<AlnNormId>,
    /// Additional ALN contracts directly applied (BioticTreaties, micro-treaties).
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,
    /// Participants.
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    /// Hashes/references to inputs and outputs for replay/audit.
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Aggregate governance evaluation.
    pub evaluation: GovernanceEvaluation,
    /// Final outcome derived from evaluation + constraint mode.
    pub outcome: DecisionOutcome,
    /// Optional explanation suitable for citizen surfaces.
    pub explanation: Option<String>,
    /// Opaque map for domain-specific metadata (e.g., volumes, route ids).
    pub tags: HashMap<String, String>,
}

impl GovernedDecisionEnvelope {
    /// Create a new envelope with default outcome PendingFpic and empty tags.
    pub fn new(
        tx_id: TrustTxId,
        workflow_id: WorkflowId,
        workflow_stage: String,
        domains: Vec<GovernanceDomain>,
        birth_sign_ids: Vec<BirthSignId>,
        evaluation: GovernanceEvaluation,
        inputs_hash: String,
        outputs_hash: String,
    ) -> Self {
        Self {
            tx_id,
            created_at: SystemTime::now(),
            workflow_id,
            workflow_stage,
            domains,
            birth_sign_ids,
            applied_aln_norms: Vec::new(),
            applied_biotic_treaties: Vec::new(),
            applied_micro_treaties: Vec::new(),
            subject_did: None,
            operator_did: None,
            node_profile_id: None,
            inputs_hash,
            outputs_hash,
            evaluation,
            outcome: DecisionOutcome::PendingFpic,
            explanation: None,
            tags: HashMap::new(),
        }
    }

    /// Attach provenance about norms actually consulted.
    pub fn with_norms(
        mut self,
        aln_norms: Vec<AlnNormId>,
        biotic: Vec<BioticTreatyId>,
        micro: Vec<MicroTreatyId>,
    ) -> Self {
        self.applied_aln_norms = aln_norms;
        self.applied_biotic_treaties = biotic;
        self.applied_micro_treaties = micro;
        self
    }

    /// Attach DIDs and node profile info.
    pub fn with_participants(
        mut self,
        subject: Option<Did>,
        operator: Option<Did>,
        node_profile: Option<NodeProfileId>,
    ) -> Self {
        self.subject_did = subject;
        self.operator_did = operator;
        self.node_profile_id = node_profile;
        self
    }

    /// Attach a human-readable explanation.
    pub fn with_explanation(mut self, explanation: Option<String>) -> Self {
        self.explanation = explanation;
        self
    }

    /// Tag the envelope with domain-specific metadata.
    pub fn add_tag<S: Into<String>, T: Into<String>>(
        &mut self,
        key: S,
        value: T,
    ) {
        self.tags.insert(key.into(), value.into());
    }

    /// Canonical mapping from evaluation to DecisionOutcome.
    /// Optimization and actuation should call this once after ALN checks.
    pub fn derive_outcome(&mut self) {
        // FPIC takes precedence.
        if let Some(fpic) = &self.evaluation.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } => {
                    self.outcome = DecisionOutcome::PendingFpic;
                    return;
                }
                FpicStatus::Denied { .. } => {
                    self.outcome = DecisionOutcome::Rejected;
                    return;
                }
                FpicStatus::NotRequired | FpicStatus::Granted { .. } => {}
            }
        }

        // Hard violations always reject.
        if self.evaluation.has_hard_violation() {
            self.outcome = DecisionOutcome::Rejected;
            return;
        }

        // Soft violations: behavior depends on constraint mode.
        if self.evaluation.has_soft_violation() {
            match self.evaluation.constraint_mode {
                ConstraintMode::Hard => {
                    // In strict-hard mode, treat soft as reject as well.
                    self.outcome = DecisionOutcome::Rejected;
                }
                ConstraintMode::HighPenalty => {
                    self.outcome = DecisionOutcome::ApprovedWithDerating;
                }
            }
            return;
        }

        // No violations and no FPIC blockers.
        self.outcome = DecisionOutcome::Approved;
    }
}

/// -------------------------
/// Secure firmware / node tier
/// -------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeSecurityTier {
    /// TEE-backed node, suitable for sensitive governance/consent.
    TeeBacked,
    /// Hardened firmware channel, signed manifests, but no TEE.
    HardenedFirmwareOnly,
    /// Basic node, only for low-risk sensing / noncritical tasks.
    Basic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSecurityProfile {
    pub node_profile_id: NodeProfileId,
    pub tier: NodeSecurityTier,
    pub secure_boot: bool,
    pub signed_updates: bool,
    pub secure_transport: bool,
    pub last_audit_passed_at: Option<SystemTime>,
}

impl NodeSecurityProfile {
    /// True if this node may host highly sensitive governance workloads
    /// (consent processing, DID handling, treaty evaluation).
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// True if this node is suitable for general ERM logic
    /// that is not biosignal- or identity-sensitive.
    pub fn can_host_general_erm(&self) -> bool {
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

/// -------------------------
/// Time helpers
/// -------------------------

pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

/// -------------------------
/// Minimal tests
/// -------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn birth_sign_active_window() {
        let now = SystemTime::now();
        let past = now - hours(2);
        let future = now + hours(2);

        let bs = BirthSign {
            id: BirthSignId("tile-1".into()),
            tile_ref: "TILE:1".into(),
            domains: vec![GovernanceDomain::Water],
            laws: vec![],
            indigenous: vec![],
            ecological: EcologicalProtections {
                biotic_treaties: vec![],
                habitat_corridor_norms: vec![],
                light_noise_chemical_norms: vec![],
            },
            local_overlays: vec![],
            default_constraint_mode: ConstraintMode::HighPenalty,
            version: 1,
            valid_from: past,
            valid_until: Some(future),
        };

        assert!(bs.is_active_at(now));
    }

    #[test]
    fn outcome_derives_from_soft_violation_mode() {
        let eval = GovernanceEvaluation {
            birth_sign_id: BirthSignId("b1".into()),
            constraint_mode: ConstraintMode::HighPenalty,
            outcomes: vec![ConstraintOutcome::SoftViolation {
                aln_norm: AlnNormId("N1".into()),
                message: "minor conflict".into(),
            }],
            fpic_status: Some(FpicStatus::NotRequired),
        };

        let mut env = GovernedDecisionEnvelope::new(
            TrustTxId("tx1".into()),
            WorkflowId("wf1".into()),
            "Optimization".into(),
            vec![GovernanceDomain::Water],
            vec![BirthSignId("b1".into())],
            eval,
            "in".into(),
            "out".into(),
        );

        env.derive_outcome();
        assert_eq!(env.outcome, DecisionOutcome::ApprovedWithDerating);
    }
}
