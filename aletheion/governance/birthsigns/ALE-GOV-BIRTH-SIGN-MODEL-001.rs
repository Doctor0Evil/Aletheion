// Role:
//   Core Birth-Sign and governed-decision model for Aletheion Phoenix.
//   Encodes territorial governance (laws, Indigenous protocols, BioticTreaties,
//   micro-treaties) and the canonical GovernedDecisionEnvelope that bridges
//   optimization, actuation, and Googolswarm provenance.[file:2][file:5]
//
// ERM layers: L2 State Modeling, L3 Blockchain Trust, L4 Optimization, L5 Citizen Interface.[file:2]
// Language: Rust only (no blacklisted cryptographic primitives or Python).
//
// This module is intentionally generic: it carries IDs and references, but
// does not embed persistence, geospatial, or ledger-client logic. Those live
// in adjacent crates (geospatial index, trust-append core, etc.).[file:2][file:5]

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Spatial governance tile identifier, stable across repos and ledgers.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

/// Stable identifier for a legal norm encoded in ALN (law, treaty clause, etc.).[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

/// Stable identifier for an Indigenous territory or protocol set.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryId(pub String);

/// Stable identifier for a BioticTreaty (cross-species envelope).[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub String);

/// Stable identifier for a LexEthos micro-treaty or local rights grammar.[file:2][file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MicroTreatyId(pub String);

/// Stable identifier for a DID (citizen, device, institution).[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

/// Stable identifier for a physical or logical asset.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(pub String);

/// Stable identifier for a workflow event (ingest, optimization run, actuation).[file:2][file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowEventId(pub String);

/// Stable identifier for a workflow definition (e.g., AWP water allocation).[file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

/// Stable identifier for a firmware node profile in the edge orchestration layer.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

/// Stable identifier for a trust-layer transaction.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

/// High-level domains that Birth-Signs can govern.[file:2]
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

/// Public-law scope for a law or regulation.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

/// Type of local overlay (LexEthos, workplace, etc.).[file:2][file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalOverlayKind {
    NeighborhoodMicroTreaty,
    WorkplacePolicy,
    SchoolOrCampusPolicy,
    FacilitySpecificProtocol,
    EventSpecificProtocol,
}

/// FPIC requirement level for a tile.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FpicRequirement {
    NotApplicable,
    RequiredBeforePlanning,
    RequiredBeforeExecution,
    EmergencyOverrideWithAudit,
}

/// Result of FPIC evaluation for a concrete action.[file:2]
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

/// Enforcement mode for constraints derived from Birth-Signs.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    /// Violations MUST cause immediate rejection.
    Hard,
    /// Violations SHOULD be avoided via high penalties, but MAY be allowed in
    /// carefully governed emergency scenarios with additional audit.[file:2]
    HighPenalty,
}

/// Reference to a concrete law or regulation as encoded in ALN.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LawRef {
    pub scope: LawScope,
    pub aln_norm: AlnNormId,
    /// Optional human label or citation (e.g., statute reference).
    pub label: String,
}

/// Encoded Indigenous and tribal governance for a tile.[file:2]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndigenousGovernance {
    pub territory_id: IndigenousTerritoryId,
    pub fpic_requirement: FpicRequirement,
    /// ALN norms representing Traditional Ecological Knowledge envelopes.[file:2]
    pub tek_norms: Vec<AlnNormId>,
}

/// Ecological and cross-species protections attached to a tile.[file:2]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcologicalProtections {
    /// Species-level BioticTreaties that apply here.
    pub biotic_treaties: Vec<BioticTreatyId>,
    /// Habitat corridors and connectivity envelopes.
    pub habitat_corridor_norms: Vec<AlnNormId>,
    /// ALN norms limiting light, noise, and chemical use.[file:2]
    pub light_noise_chemical_norms: Vec<AlnNormId>,
}

/// Local LexEthos overlays and citizen norms.[file:2][file:5]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalOverlay {
    pub kind: LocalOverlayKind,
    pub micro_treaty_id: MicroTreatyId,
    /// Optional summary suitable for citizen interfaces.
    pub summary: String,
}

/// Unified Birth-Sign record for a spatial tile and time range.[file:2]
#[derive(Debug, Clone)]
pub struct BirthSign {
    pub id: BirthSignId,
    /// Tile geometry reference (opaque key to geospatial DB, not raw geometry). [file:2]
    pub tile_ref: String,
    /// Domains where this Birth-Sign is authoritative.
    pub domains: Vec<GovernanceDomain>,
    /// Governing public law references.
    pub laws: Vec<LawRef>,
    /// Indigenous and tribal governance overlays.
    pub indigenous: Vec<IndigenousGovernance>,
    /// Ecological and cross-species protections.
    pub ecological: EcologicalProtections,
    /// Local policy overlays (LexEthos, neighborhood norms, etc.).[file:2]
    pub local_overlays: Vec<LocalOverlay>,
    /// Default enforcement mode for constraints derived from this Birth-Sign.[file:2]
    pub default_constraint_mode: ConstraintMode,
    /// Version and temporal validity for audit and safe evolution.
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

/// Binding between Birth-Signs and assets/events.[file:2]
#[derive(Debug, Clone)]
pub struct BirthSignBinding {
    pub birth_sign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    /// Optional freeform metadata (labels, debug tags, etc.).
    pub metadata: HashMap<String, String>,
}

/// Outcome for an individual ALN constraint.[file:2]
#[derive(Debug, Clone)]
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation { aln_norm: AlnNormId, message: String },
    HardViolation { aln_norm: AlnNormId, message: String },
}

/// Aggregate governance evaluation for a proposed action.[file:2]
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

/// High-level decision outcome used by optimization and actuation stages.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// Security posture of a node profile, used by orchestration and governance.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeSecurityTier {
    /// Trusted Execution Environment available; suitable for sensitive governance/consent logic.[file:2]
    TeeBacked,
    /// No TEE, but hardened firmware channel (e.g., minimal OpenWrt, signed manifests).[file:2]
    HardenedFirmwareOnly,
    /// Basic node; only for low-risk sensing or non-critical tasks.[file:2]
    Basic,
}

/// Summary of security features available on a node profile.[file:2]
#[derive(Debug, Clone)]
pub struct NodeSecurityProfile {
    pub node_profile_id: NodeProfileId,
    pub tier: NodeSecurityTier,
    /// Whether secure boot is enabled and enforced.
    pub secure_boot: bool,
    /// Whether firmware updates are signed and verified.
    pub signed_updates: bool,
    /// Whether secure transport (e.g., TLSDTLS) is mandatory.
    pub secure_transport: bool,
    /// Whether this node currently passes vulnerability scans and audits.[file:2]
    pub last_audit_passed_at: Option<SystemTime>,
}

impl NodeSecurityProfile {
    /// Returns true if this node may host highly sensitive governance workloads
    /// (consent processing, DID handling, treaty evaluation).[file:2]
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// Returns true if this node is suitable for generic ERM logic that is not
    /// biosignal- or identity-sensitive.[file:2]
    pub fn can_host_general_erm(&self) -> bool {
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

/// A governed decision is the bridge between optimization, governance, and the trust layer.[file:2]
/// It is designed to be serialized into ALN/Googolswarm transactions.[file:2]
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    /// e.g., "Optimization", "Actuation", "EmergencyOverride".
    pub workflow_stage: String,
    /// Spatial context.
    pub domains: Vec<GovernanceDomain>,
    pub birth_sign_ids: Vec<BirthSignId>,
    /// Norms actually consulted for this decision (laws, treaties, overlays).[file:2]
    pub applied_aln_norms: Vec<AlnNormId>,
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,
    /// Participants (subject and operator).
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    /// Node profile that executed this decision, if applicable.[file:2]
    pub node_profile_id: Option<NodeProfileId>,
    /// Hashes of inputs and outputs for replay and audit.[file:2]
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Aggregate governance evaluation.
    pub evaluation: GovernanceEvaluation,
    /// Final decision outcome (canonical mapping for optimizers/actuators).[file:2]
    pub outcome: DecisionOutcome,
    /// Optional explanation string for citizen interfaces.[file:2]
    pub explanation: Option<String>,
    /// Opaque map for domain-specific data (water volumes, routing IDs, etc.).[file:2]
    pub tags: HashMap<String, String>,
}

impl BirthSign {
    /// Returns true if this Birth-Sign is active at the given time.[file:2]
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

    /// Returns true if this Birth-Sign covers the given domain.[file:2]
    pub fn covers_domain(&self, domain: &GovernanceDomain) -> bool {
        self.domains.contains(domain)
    }
}

impl GovernanceEvaluation {
    /// Returns true if all constraint outcomes are satisfied and FPIC status is non-blocking.[file:2]
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(fpic) = &self.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } | FpicStatus::Denied { .. } => return false,
                FpicStatus::NotRequired | FpicStatus::Granted { .. } => {}
            }
        }
        self.outcomes.iter().all(|o| matches!(o, ConstraintOutcome::Satisfied))
    }

    /// Returns true if any hard violation is present.[file:2]
    pub fn has_hard_violation(&self) -> bool {
        self.outcomes
            .iter()
            .any(|o| matches!(o, ConstraintOutcome::HardViolation { .. }))
    }

    /// Returns true if any soft violation is present.[file:2]
    pub fn has_soft_violation(&self) -> bool {
        self.outcomes
            .iter()
            .any(|o| matches!(o, ConstraintOutcome::SoftViolation { .. }))
    }
}

impl GovernedDecisionEnvelope {
    /// Create a new envelope with default outcome PendingFpic and empty tags.[file:2]
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

    /// Attach provenance about norms actually consulted.[file:2]
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

    /// Attach participants (subject, operator, node profile).[file:2]
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

    /// Attach a human-readable explanation.[file:2]
    pub fn with_explanation(mut self, explanation: Option<String>) -> Self {
        self.explanation = explanation;
        self
    }

    /// Tag the envelope with domain-specific metadata.[file:2]
    pub fn add_tag<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) {
        self.tags.insert(key.into(), value.into());
    }

    /// Decide the canonical DecisionOutcome from the evaluation and constraint mode.[file:2]
    /// This should be used by optimization and actuation stages as the single
    /// mapping from ALN checks to workflow behavior.[file:2]
    pub fn derive_outcome(&mut self) {
        // FPIC precedence.
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

        // Soft violations may derate depending on constraint mode.
        if self.evaluation.has_soft_violation() {
            match self.evaluation.constraint_mode {
                ConstraintMode::Hard => {
                    // In strict hard mode, treat soft as reject as well.[file:2]
                    self.outcome = DecisionOutcome::Rejected;
                    return;
                }
                ConstraintMode::HighPenalty => {
                    self.outcome = DecisionOutcome::ApprovedWithDerating;
                    return;
                }
            }
        }

        // No violations and no FPIC blockers.
        self.outcome = DecisionOutcome::Approved;
    }
}

/// Convenience for expressing durations in hours.[file:2]
pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}
