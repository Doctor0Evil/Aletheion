// aletheion/governance/birthsigns/core/ALE-GOV-BIRTH-SIGN-MODEL-002.rs
// Core Birth-Sign, governance envelope, and node-security types for Aletheion Phoenix.
// This file is designed to (a) prevent silent-takeovers via explicit governance envelopes,
// (b) constrain Googolswarm to a local, offline-capable attestation role only,
// (c) expose immutable provenance hooks for ALN / DID / KYC without hidden authorities,
// and (d) keep augmented-citizen sovereignty central in all governed decisions.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// -------------------------
// Core stable identifiers
// -------------------------

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

/// DID for citizens, devices, institutions under ALN/KYC/DID.
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

/// Local-only Googolswarm transaction identifier.
/// Even though this may be mirrored to external infra later,
/// the semantics here are strictly "attested provenance", not control.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

/// Identifier for an attestation session emitted by Googolswarm attestors.
/// This is never used as a control token — only as an immutable proof of evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestationSessionId(pub String);

// -------------------------
// Territorial + governance enums
// -------------------------

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

/// Public law scopes encoded as ALN norms and linked to Birth-Signs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

/// Local overlays for LexEthos micro-treaties and neighborhood rules.
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
/// Pending/Denied always block actuation for standard workflows.
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

/// Enforcement mode for constraints derived from Birth-Signs.
/// Hard == inviolate, HighPenalty == optimization can explore but never silently ignore.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    Hard,
    HighPenalty,
}

// -------------------------
// Birth-Sign components
// -------------------------

/// Reference to a concrete law or regulation as encoded in ALN.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LawRef {
    pub scope: LawScope,
    pub aln_norm: AlnNormId,
    pub label: String,
}

/// Encoded Indigenous and tribal governance for a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndigenousGovernance {
    pub territory_id: IndigenousTerritoryId,
    pub fpic_requirement: FpicRequirement,
    /// TEK envelopes encoded as ALN norms.
    pub tek_norms: Vec<AlnNormId>,
}

/// Ecological and cross-species protections attached to a tile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcologicalProtections {
    pub biotic_treaties: Vec<BioticTreatyId>,
    pub habitat_corridor_norms: Vec<AlnNormId>,
    pub light_noise_chemical_norms: Vec<AlnNormId>,
}

/// Local LexEthos overlays and citizen norms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalOverlay {
    pub kind: LocalOverlayKind,
    pub micro_treaty_id: MicroTreatyId,
    /// Optional short summary, suitable for citizen surfaces.
    pub summary: String,
}

/// Unified Birth-Sign for a spatial tile and time range.
/// This is the core "jurisdictional signature" object used across the ERM stack. [file:2]
#[derive(Debug, Clone)]
pub struct BirthSign {
    pub id: BirthSignId,
    /// Reference to tile geometry stored in a geospatial DB.
    pub tile_ref: String,
    /// Domains where this Birth-Sign is authoritative.
    pub domains: Vec<GovernanceDomain>,
    /// Governing public law references.
    pub laws: Vec<LawRef>,
    /// Indigenous and tribal governance overlays.
    pub indigenous: Vec<IndigenousGovernance>,
    /// Ecological and cross-species protections.
    pub ecological: EcologicalProtections,
    /// Local policy overlays LexEthos, neighborhood norms, etc.
    pub local_overlays: Vec<LocalOverlay>,
    /// Default enforcement mode for constraints derived from this Birth-Sign.
    pub default_constraint_mode: ConstraintMode,
    /// Versioning + temporal validity for audit and forward-only evolution. [file:2]
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

/// Binding between Birth-Signs and assets / events, used by edge and state-model layers. [file:2]
#[derive(Debug, Clone)]
pub struct BirthSignBinding {
    pub birth_sign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    pub metadata: HashMap<String, String>,
}

// -------------------------
// Constraint evaluation
// -------------------------

/// Individual constraint outcome for a given ALN norm.
#[derive(Debug, Clone)]
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation { aln_norm: AlnNormId, message: String },
    HardViolation { aln_norm: AlnNormId, message: String },
}

/// Aggregate governance evaluation for a proposed action.
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

// -------------------------
// Googolswarm-governed envelope
// -------------------------

/// Role that Googolswarm is allowed to play: strictly "AttestorLocalHost".
/// Anything else is a compile-time violation if added to this enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestationRole {
    /// Local / host-only, offline-capable attestation of envelopes and transaction ordering. [file:2]
    AttestorLocalHost,
}

/// Minimal description of an attestation result produced by Googolswarm.
/// This struct is intentionally narrow: it cannot embed control hooks or policy changes. [file:2]
#[derive(Debug, Clone)]
pub struct GoogolswarmAttestation {
    pub session_id: AttestationSessionId,
    pub role: AttestationRole,
    /// Which transaction envelope was attested.
    pub tx_id: TrustTxId,
    /// Hash of the serialized envelope payload as seen by Googolswarm.
    pub envelope_hash: String,
    /// Time of local attestation, for ordering and audit.
    pub attested_at: SystemTime,
    /// Multi-sig participant DIDs (local) for this attestation.
    pub attestor_dids: Vec<Did>,
    /// Optional human-readable note for audit logs.
    pub note: Option<String>,
}

/// A governed decision envelope is the bridge between optimization, governance,
/// and the trust layer, and the *only* structure that Googolswarm may attest. [file:2][file:5]
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    /// e.g., "Optimization", "GovernancePreflight", "Actuation", "EmergencyOverride".
    pub workflow_stage: String,
    /// Spatial and thematic context.
    pub domains: Vec<GovernanceDomain>,
    pub birth_sign_ids: Vec<BirthSignId>,
    /// Norms actually consulted for this decision (laws, treaties, overlays).
    pub applied_aln_norms: Vec<AlnNormId>,
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,
    /// Participants.
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    /// Hashes over inputs/outputs for reproducibility and tamper detection.
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Aggregate governance evaluation.
    pub evaluation: GovernanceEvaluation,
    /// Final decision outcome.
    pub outcome: DecisionOutcome,
    /// Optional explanation string suitable for citizen interfaces.
    pub explanation: Option<String>,
    /// Domain-specific metadata (water volumes, route IDs, etc.).
    pub tags: HashMap<String, String>,
    /// Optional local Googolswarm attestation record.
    /// Presence of this field never changes outcome semantics.
    pub attestation: Option<GoogolswarmAttestation>,
}

// -------------------------
// Node security profile
// -------------------------

/// Security posture of a node profile, used by orchestration and governance. [file:2]
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

// -------------------------
// Impl blocks - core logic
// -------------------------

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

    /// Returns true if this Birth-Sign covers the given domain.
    pub fn covers_domain(&self, domain: &GovernanceDomain) -> bool {
        self.domains.contains(domain)
    }
}

impl GovernanceEvaluation {
    /// Returns true if all outcomes are satisfied and FPIC status is non-blocking.
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(fpic) = &self.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } => return false,
                FpicStatus::Denied { .. } => return false,
                FpicStatus::Granted { .. } | FpicStatus::NotRequired => {}
            }
        }
        self.outcomes
            .iter()
            .all(|o| matches!(o, ConstraintOutcome::Satisfied))
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

impl NodeSecurityProfile {
    /// Returns true if this node is allowed to host highly sensitive governance workloads
    /// such as consent processing, DID handling, or treaty evaluation. [file:2]
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// Returns true if this node is suitable for generic ERM logic
    /// that is not biosignal- or identity-sensitive.
    pub fn can_host_general_erm(&self) -> bool {
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

impl GoogolswarmAttestation {
    /// Returns true if this attestation conforms to the local/host-only role.
    /// Any future role additions must be explicit here, preventing silent capability creep. [file:2]
    pub fn is_local_host_only(&self) -> bool {
        matches!(self.role, AttestationRole::AttestorLocalHost)
    }
}

impl GovernedDecisionEnvelope {
    /// Create a new envelope with default outcome PendingFpic and no attestation.
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
            attestation: None,
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

    /// Attach a Googolswarm attestation, but only if it is strictly local/host-only.
    /// This prevents any silent escalation of Googolswarm into a control-plane role. [file:2]
    pub fn attach_local_attestation(
        mut self,
        attestation: GoogolswarmAttestation,
    ) -> Self {
        if attestation.is_local_host_only() && attestation.tx_id == self.tx_id {
            self.attestation = Some(attestation);
        }
        self
    }

    /// Decide the canonical DecisionOutcome from the evaluation and constraint mode.
    /// FPIC status always takes precedence, then hard/soft violations. [file:2]
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
                FpicStatus::Granted { .. } | FpicStatus::NotRequired => {}
            }
        }

        // Hard violations always reject.
        if self.evaluation.has_hard_violation() {
            self.outcome = DecisionOutcome::Rejected;
            return;
        }

        // Soft violations depend on constraint mode.
        if self.evaluation.has_soft_violation() {
            match self.evaluation.constraint_mode {
                ConstraintMode::Hard => {
                    // In hard mode, treat soft as reject as well (fail-safe).
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

// -------------------------
// Time helpers
// -------------------------

/// Convenience for expressing durations in hours.
pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}
