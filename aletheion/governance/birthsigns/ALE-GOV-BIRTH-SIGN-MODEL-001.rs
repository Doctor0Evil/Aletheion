// Core Birth-Sign and governed decision types for Aletheion.
// Ties territorial governance (laws, treaties, protocols) to concrete workflow actions,
// and prepares envelopes for trust-layer append and secure firmware orchestration.
//
// ERM layers: L2 State Modeling, L3 Blockchain Trust, L4 Optimization, L5 Citizen Interface.
// Languages: Rust only (no blacklisted terms, no Python).

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// -------------------------
/// Core identifiers
/// -------------------------

/// Spatial governance tile identifier (stable across repos).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

/// Stable identifier for legal / treaty ALN norms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

/// Stable identifier for Indigenous territory or protocol set.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryId(pub String);

/// Stable identifier for a BioticTreaty record (cross-species envelope).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub String);

/// Stable identifier for a LexEthos micro‑treaty or local rights grammar.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MicroTreatyId(pub String);

/// Stable identifier for a DID (citizen, device, or institution).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

/// Stable identifier for a physical or logical asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(pub String);

/// Stable identifier for a workflow event (ingest, optimization run, actuation).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowEventId(pub String);

/// Stable identifier for a workflow definition (e.g., AWP Water Allocation).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

/// Stable identifier for a firmware / node profile in the edge orchestration layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

/// Stable identifier for a trust-layer transaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

/// -------------------------
/// Territorial domain enums
/// -------------------------

/// High-level domains that Birth-Signs can govern.
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

/// Public law scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

/// Type of local overlay.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalOverlayKind {
    NeighborhoodMicroTreaty,
    WorkplacePolicy,
    SchoolOrCampusPolicy,
    FacilitySpecificProtocol,
    EventSpecificProtocol,
}

/// FPIC requirement level for a tile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FpicRequirement {
    NotApplicable,
    RequiredBeforePlanning,
    RequiredBeforeExecution,
    EmergencyOverrideWithAudit,
}

/// Result of FPIC evaluation for a concrete action.
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

/// Enforcement mode for constraints derived from Birth‑Signs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    /// Violations MUST cause immediate rejection.
    Hard,
    /// Violations SHOULD be avoided by optimization (high penalty), but MAY be allowed in
    /// carefully governed emergency scenarios with additional audit.
    HighPenalty,
}

/// -------------------------
/// Birth‑Sign components
/// -------------------------

/// Reference to a concrete law or regulation as encoded in ALN.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LawRef {
    pub scope: LawScope,
    pub aln_norm: AlnNormId,
    /// Optional human label or citation (e.g., statute reference).
    pub label: String,
}

/// Encoded Indigenous and tribal governance for a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndigenousGovernance {
    pub territory_id: IndigenousTerritoryId,
    pub fpic_requirement: FpicRequirement,
    /// ALN norms representing TEK (Traditional Ecological Knowledge) envelopes.
    pub tek_norms: Vec<AlnNormId>,
}

/// Ecological and cross‑species protections attached to a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcologicalProtections {
    /// Species‑level BioticTreaties that apply here.
    pub biotic_treaties: Vec<BioticTreatyId>,
    /// Habitat corridors and connectivity envelopes.
    pub habitat_corridor_norms: Vec<AlnNormId>,
    /// ALN norms limiting light, noise, and chemical use.
    pub light_noise_chemical_norms: Vec<AlnNormId>,
}

/// Local LexEthos overlays and citizen norms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalOverlay {
    pub kind: LocalOverlayKind,
    pub micro_treaty_id: MicroTreatyId,
    /// Optional summary, suitable for citizen interfaces.
    pub summary: String,
}

/// Unified Birth‑Sign for a spatial tile and time range.
#[derive(Debug, Clone)]
pub struct BirthSign {
    pub id: BirthSignId,
    /// Tile geometry reference (opaque key to geospatial DB, not raw geometry here).
    pub tile_ref: String,
    /// Domains where this Birth‑Sign is authoritative.
    pub domains: Vec<GovernanceDomain>,
    /// Governing public law references.
    pub laws: Vec<LawRef>,
    /// Indigenous and tribal governance overlays.
    pub indigenous: Vec<IndigenousGovernance>,
    /// Ecological and cross‑species protections.
    pub ecological: EcologicalProtections,
    /// Local policy overlays (LexEthos, neighborhood norms, etc.).
    pub local_overlays: Vec<LocalOverlay>,
    /// Default enforcement mode for constraints derived from this Birth‑Sign.
    pub default_constraint_mode: ConstraintMode,
    /// Version and temporal validity for audit and rollback‑safe evolution.
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

/// Binding between Birth‑Signs and assets / events.
#[derive(Debug, Clone)]
pub struct BirthSignBinding {
    pub birth_sign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    /// Optional free‑form metadata (e.g., labels, debug tags).
    pub metadata: HashMap<String, String>,
}

/// -------------------------
/// Constraint evaluation results
/// -------------------------

/// Individual constraint outcome for a given ALN norm.
#[derive(Debug, Clone)]
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

/// Aggregate governance evaluation for a proposed action.
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

/// High‑level decision outcome used by optimization and actuation stages.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// -------------------------
/// Governed decision envelope
/// -------------------------

/// A governed decision is the bridge between optimization, governance, and the trust layer.
/// It is designed to be serialized into ALN/Googolswarm transactions.
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    pub workflow_stage: String, // e.g., "Optimization", "Actuation", "EmergencyOverride"
    pub domains: Vec<GovernanceDomain>,
    /// Spatial context.
    pub birth_sign_ids: Vec<BirthSignId>,
    /// Norms actually consulted for this decision (laws, treaties, overlays).
    pub applied_aln_norms: Vec<AlnNormId>,
    /// Additional ALN contracts directly applied (e.g., BioticTreaties, LexEthos micro‑treaties).
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,
    /// Participants.
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    /// Hashes / references to inputs and outputs for replay and audit.
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Aggregate governance evaluation.
    pub evaluation: GovernanceEvaluation,
    /// Final decision outcome.
    pub outcome: DecisionOutcome,
    /// Optional explanation string suitable for citizen interfaces.
    pub explanation: Option<String>,
    /// Opaque map for domain‑specific data (e.g., water volumes, routing IDs).
    pub tags: HashMap<String, String>,
}

/// -------------------------
/// Helper functions
/// -------------------------

impl BirthSign {
    /// Returns true if this Birth‑Sign is valid at the given time.
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

    /// Returns true if this Birth‑Sign covers the given domain.
    pub fn covers_domain(&self, domain: &GovernanceDomain) -> bool {
        self.domains.contains(domain)
    }
}

impl GovernanceEvaluation {
    /// Returns true if all outcomes are satisfied and FPIC status is non‑blocking.
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(fpic) = &self.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } | FpicStatus::Denied { .. } => return false,
                _ => {}
            }
        }
        self.outcomes.iter().all(|o| matches!(o, ConstraintOutcome::Satisfied))
    }

    /// Returns true if any hard violation is present.
    pub fn has_hard_violation(&self) -> bool {
        self.outcomes
            .iter()
            .any(|o| matches!(o, ConstraintOutcome::HardViolation { .. }))
    }

    /// Returns true if any soft violation is present.
    pub fn has_soft_violation(&self) -> bool {
        self.outcomes
            .iter()
            .any(|o| matches!(o, ConstraintOutcome::SoftViolation { .. }))
    }
}

impl GovernedDecisionEnvelope {
    /// Decide the canonical DecisionOutcome from the evaluation and constraint mode.
    /// This function SHOULD be used by optimization and actuation stages as the single
    /// mapping point from ALN checks to workflow behavior.
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
                _ => {}
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
                    // In strict‑hard mode, treat soft as reject as well.
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
/// Secure firmware and node‑placement ties
/// -------------------------

/// Security posture of a node profile, used by orchestration and governance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeSecurityTier {
    /// Trusted Execution Environment available; suitable for sensitive governance and consent logic.
    TeeBacked,
    /// No TEE, but hardened firmware channel (e.g., minimal OpenWrt image, signed manifests).
    HardenedFirmwareOnly,
    /// Basic node; suitable only for low‑risk sensing or non‑critical tasks.
    Basic,
}

/// Summary of security features available on a node.
#[derive(Debug, Clone)]
pub struct NodeSecurityProfile {
    pub node_profile_id: NodeProfileId,
    pub tier: NodeSecurityTier,
    /// Whether secure boot is enabled and enforced.
    pub secure_boot: bool,
    /// Whether firmware updates are signed and verified.
    pub signed_updates: bool,
    /// Whether TLS/DTLS with modern ciphers is mandatory on this node.
    pub secure_transport: bool,
    /// Whether this node currently passes vulnerability scans and audits.
    pub last_audit_passed_at: Option<SystemTime>,
}

/// Minimal policy helper for scheduling sensitive workloads.
impl NodeSecurityProfile {
    /// Returns true if this node is allowed to host highly sensitive governance workloads
    /// (e.g., consent processing, DID handling, treaty evaluation).
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// Returns true if this node is suitable for generic ERM logic that is not
    /// biosignal‑ or identity‑sensitive.
    pub fn can_host_general_erm(&self) -> bool {
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

/// -------------------------
/// Example constructor helpers
/// -------------------------

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

    /// Attach a human‑readable explanation.
    pub fn with_explanation(mut self, explanation: Option<String>) -> Self {
        self.explanation = explanation;
        self
    }

    /// Tag the envelope with domain‑specific metadata.
    pub fn add_tag<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) {
        self.tags.insert(key.into(), value.into());
    }
}

/// -------------------------
/// Time helpers
/// -------------------------

/// Convenience for expressing durations in hours.
pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}
