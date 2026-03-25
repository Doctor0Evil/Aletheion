// aletheion/governance/birthsigns/core/ALE-GOV-BIRTH-SIGN-MODEL-002.rs

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MicroTreatyId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowEventId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalOverlayKind {
    NeighborhoodMicroTreaty,
    WorkplacePolicy,
    SchoolOrCampusPolicy,
    FacilitySpecificProtocol,
    EventSpecificProtocol,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    Hard,
    HighPenalty,
}

#[derive(Debug, Clone)]
pub struct LawRef {
    pub scope: LawScope,
    pub aln_norm: AlnNormId,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct IndigenousGovernance {
    pub territory_id: IndigenousTerritoryId,
    pub fpic_requirement: FpicRequirement,
    pub tek_norms: Vec<AlnNormId>,
}

#[derive(Debug, Clone)]
pub struct EcologicalProtections {
    pub biotic_treaties: Vec<BioticTreatyId>,
    pub habitat_corridor_norms: Vec<AlnNormId>,
    pub light_noise_chemical_norms: Vec<AlnNormId>,
}

#[derive(Debug, Clone)]
pub struct LocalOverlay {
    pub kind: LocalOverlayKind,
    pub microtreaty_id: MicroTreatyId,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct BirthSign {
    pub id: BirthSignId,
    pub tile_ref: String,
    pub domains: Vec<GovernanceDomain>,
    pub laws: Vec<LawRef>,
    pub indigenous: Vec<IndigenousGovernance>,
    pub ecological: EcologicalProtections,
    pub local_overlays: Vec<LocalOverlay>,
    pub default_constraint_mode: ConstraintMode,
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct BirthSignBinding {
    pub birthsign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation { aln_norm: AlnNormId, message: String },
    HardViolation { aln_norm: AlnNormId, message: String },
}

#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birthsign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

impl GovernanceEvaluation {
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(fpic) = &self.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } => return false,
                FpicStatus::Denied { .. } => return false,
                _ => {}
            }
        }
        self.outcomes.iter().all(|o| matches!(o, ConstraintOutcome::Satisfied))
    }

    pub fn has_hard_violation(&self) -> bool {
        self.outcomes.iter().any(|o| matches!(o, ConstraintOutcome::HardViolation { .. }))
    }

    pub fn has_soft_violation(&self) -> bool {
        self.outcomes.iter().any(|o| matches!(o, ConstraintOutcome::SoftViolation { .. }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// Scope of a decision with respect to Googolswarm.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrustScope {
    /// Record is kept locally only, never propagated off host.
    LocalOnly,
    /// Record is eligible for Googolswarm append under multisig.
    LocalAndGoogolswarm,
}

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
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    pub workflow_stage: String,
    pub domains: Vec<GovernanceDomain>,
    pub birthsign_ids: Vec<BirthSignId>,
    pub applied_aln_norms: Vec<AlnNormId>,
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_microtreaties: Vec<MicroTreatyId>,
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub evaluation: GovernanceEvaluation,
    pub outcome: DecisionOutcome,
    pub explanation: Option<String>,
    pub tags: HashMap<String, String>,
    pub trust_scope: TrustScope,
}

impl GovernedDecisionEnvelope {
    pub fn new(
        tx_id: TrustTxId,
        workflow_id: WorkflowId,
        workflow_stage: String,
        domains: Vec<GovernanceDomain>,
        birthsign_ids: Vec<BirthSignId>,
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
            birthsign_ids,
            applied_aln_norms: Vec::new(),
            applied_biotic_treaties: Vec::new(),
            applied_microtreaties: Vec::new(),
            subject_did: None,
            operator_did: None,
            node_profile_id: None,
            inputs_hash,
            outputs_hash,
            evaluation,
            outcome: DecisionOutcome::PendingFpic,
            explanation: None,
            tags: HashMap::new(),
            trust_scope: TrustScope::LocalAndGoogolswarm,
        }
    }

    pub fn with_norms(
        mut self,
        aln_norms: Vec<AlnNormId>,
        biotic: Vec<BioticTreatyId>,
        micro: Vec<MicroTreatyId>,
    ) -> Self {
        self.applied_aln_norms = aln_norms;
        self.applied_biotic_treaties = biotic;
        self.applied_microtreaties = micro;
        self
    }

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

    pub fn with_explanation(mut self, explanation: Option<String>) -> Self {
        self.explanation = explanation;
        self
    }

    pub fn add_tag<S: Into<String>, T: Into<String>>(
        mut self,
        key: S,
        value: T,
    ) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    pub fn set_trust_scope(mut self, scope: TrustScope) -> Self {
        self.trust_scope = scope;
        self
    }

    /// Derive outcome in a way that prevents silent weakening of constraints.
    pub fn derive_outcome(&mut self) {
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
                _ => {}
            }
        }

        if self.evaluation.has_hard_violation() {
            self.outcome = DecisionOutcome::Rejected;
            return;
        }

        if self.evaluation.has_soft_violation() {
            match self.evaluation.constraint_mode {
                ConstraintMode::Hard => {
                    self.outcome = DecisionOutcome::Rejected;
                }
                ConstraintMode::HighPenalty => {
                    self.outcome = DecisionOutcome::ApprovedWithDerating;
                }
            }
            return;
        }

        self.outcome = DecisionOutcome::Approved;
    }

    /// Hard guard: this decision may not be appended to Googolswarm unless all
    /// sovereignty and locality requirements are satisfied.
    pub fn may_append_to_googolswarm(
        &self,
        node_profile: &NodeSecurityProfile,
    ) -> bool {
        if self.trust_scope == TrustScope::LocalOnly {
            return false;
        }
        if !node_profile.can_host_general_erm() {
            return false;
        }
        if self.birthsign_ids.is_empty() {
            return false;
        }
        matches!(
            self.outcome,
            DecisionOutcome::Approved | DecisionOutcome::ApprovedWithDerating
        )
    }
}

pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

impl BirthSign {
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

    pub fn covers_domain(&self, domain: &GovernanceDomain) -> bool {
        self.domains.contains(domain)
    }
}
