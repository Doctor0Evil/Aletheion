//! Aletheion Phoenix – Birth-Sign, Governed Decision, and Local-Only Googolswarm Attestation Model v1
//! Path: aletheion/governance/birthsigns/offline_attestation/ALE-GOV-BIRTH-SIGN-GOVERNED-DECISION-LOCAL-ATTESTOR-001.rs
//!
//! Purpose
//! - Harden Birth-Sign governance, constraint evaluation, and decision envelopes against silent-takeovers
//!   or hidden reinvention of prior work by making all control surfaces explicit, typed, and auditable. [file:3][file:6]
//! - Constrain Googolswarm to a **local_attestor** role only: offline-capable, host-local validation with no
//!   global control authority over Aletheion workflows or policy logic. [file:3][file:6]
//! - Preserve augmented-citizen sovereignty by requiring DID/consent provenance and by enforcing forward-only,
//!   non-silent evolution semantics on governance-relevant states. [file:3][file:6]
//!
//! Scope
//! - ERM Layers: L2 State Modeling, L3 Trust, L4 Optimization, L5 Citizen Interface. [file:6]
//! - Domains: water, thermal, mobility, materials, biosignals, augmentation, culture, emergency. [file:6]
//!
//! Design constraints
//! - No Digital Twin semantics; use state_model / operational_mirror terminology only. [file:6]
//! - Rust-only; no blacklisted languages; no precommit hooks in this file. [file:6]
//! - All superpowers (LexEthos, Synthexis, Thermaphora, CryptoSomatic Shield, etc.) must be mediated through
//!   explicit, non-human-controlled governance envelopes – no hidden knobs here. [file:6]
//!
//! Integration
//! - Upstream types: ALN IDs, BioticTreaties, MicroTreaties, DIDs, Node profiles, etc. are modelled as
//!   opaque string-newtypes to avoid schema drift while tier1–tier3 research consolidates. [file:3]
//! - Downstream: this module should be imported by trust-layer appenders and compliance preflight, but must
//!   *never* be silently replaced; version fields + forward-only semantics enforce visible evolution. [file:3][file:6]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Stable governance identifier for a spatial tile (Birth-Sign). [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

/// Stable identifier for an ALN norm (law, treaty, right, or protocol). [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

/// Stable identifier for an Indigenous territory or protocol bundle. [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryId(pub String);

/// Stable identifier for a cross-species BioticTreaty. [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub String);

/// Stable identifier for a LexEthos or local MicroTreaty. [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MicroTreatyId(pub String);

/// Stable DID for a citizen, device, or institution (ALN/KYC/DID compliant). [file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

/// Stable identifier for a physical or logical asset. [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(pub String);

/// Stable identifier for a workflow definition (e.g., AWP water allocation). [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

/// Stable identifier for a workflow event (ingest, optimization run, actuation). [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowEventId(pub String);

/// Stable identifier for a firmware / node profile used in ERM edge orchestration. [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

/// Stable identifier for a trust-layer transaction (provenance record). [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

/// Domain areas where Birth-Signs and governance apply. [file:3][file:6]
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

/// Scope for public law references inside a Birth-Sign. [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

/// Type of overlay for local, LexEthos-style governance. [file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalOverlayKind {
    NeighborhoodMicroTreaty,
    WorkplacePolicy,
    SchoolOrCampusPolicy,
    FacilitySpecificProtocol,
    EventSpecificProtocol,
}

/// FPIC requirement level attached to a tile. [file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FpicRequirement {
    NotApplicable,
    RequiredBeforePlanning,
    RequiredBeforeExecution,
    EmergencyOverrideWithAudit,
}

/// Result of FPIC evaluation for a specific action. [file:3][file:6]
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

/// Enforcement mode for constraints derived from Birth-Signs. [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    /// Violations MUST cause immediate rejection.
    Hard,
    /// Violations SHOULD be avoided (high penalty) but MAY be allowed in constrained emergency
    /// scenarios with additional audit and explicit override. [file:3]
    HighPenalty,
}

/// Reference to a concrete law or regulation as encoded in ALN. [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LawRef {
    pub scope: LawScope,
    pub aln_norm: AlnNormId,
    /// Optional human label or statutory citation.
    pub label: String,
}

/// Encoded Indigenous and tribal governance for a tile. [file:6]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndigenousGovernance {
    pub territory_id: IndigenousTerritoryId,
    pub fpic_requirement: FpicRequirement,
    /// ALN norms representing TEK envelopes. [file:6]
    pub tek_norms: Vec<AlnNormId>,
}

/// Ecological and cross-species protections for a tile. [file:6]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcologicalProtections {
    /// Species-level BioticTreaties that apply here. [file:6]
    pub biotic_treaties: Vec<BioticTreatyId>,
    /// Habitat corridors and connectivity envelopes.
    pub habitat_corridor_norms: Vec<AlnNormId>,
    /// ALN norms limiting light, noise, and chemical use.
    pub light_noise_chemical_norms: Vec<AlnNormId>,
}

/// Local overlays such as LexEthos microtreaties and citizen norms. [file:6]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalOverlay {
    pub kind: LocalOverlayKind,
    pub micro_treaty_id: MicroTreatyId,
    /// Optional summary for citizen interfaces.
    pub summary: String,
}

/// Unified Birth-Sign record for a spatial tile and time interval. [file:3][file:6]
#[derive(Debug, Clone)]
pub struct BirthSign {
    pub id: BirthSignId,
    /// Reference to geometry stored in a geospatial service, not raw coordinates. [file:3]
    pub tile_ref: String,
    /// Domains where this Birth-Sign is authoritative.
    pub domains: Vec<GovernanceDomain>,
    /// Governing public law references.
    pub laws: Vec<LawRef>,
    /// Indigenous and tribal governance overlays.
    pub indigenous: Vec<IndigenousGovernance>,
    /// Ecological and cross-species protections.
    pub ecological: EcologicalProtections,
    /// Local policy overlays and norms.
    pub local_overlays: Vec<LocalOverlay>,
    /// Default enforcement mode for constraints derived from this Birth-Sign.
    pub default_constraint_mode: ConstraintMode,
    /// Version and temporal validity for audit and forward-only evolution. [file:3]
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

/// Binding between a Birth-Sign and assets/events. [file:3]
#[derive(Debug, Clone)]
pub struct BirthSignBinding {
    pub birth_sign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    /// Freeform metadata for debug / audit.
    pub metadata: HashMap<String, String>,
}

/// Individual constraint outcome for a given ALN norm. [file:3]
#[derive(Debug, Clone)]
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation { aln_norm: AlnNormId, message: String },
    HardViolation { aln_norm: AlnNormId, message: String },
}

/// Aggregate governance evaluation for a proposed action. [file:3]
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

/// Canonical outcome mapping for a governed decision. [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// Googolswarm role scoping: strictly local attestation only. [file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoogolswarmAttestorRole {
    /// Local, offline-capable, host-bound attestor – allowed. [file:3]
    LocalHostOnly,
    /// Any remote, centralized, or cross-tenant authority – disallowed in this module.
    ForbiddenRemote,
}

/// Offline/host attestation policy for Googolswarm in Aletheion. [file:3][file:6]
#[derive(Debug, Clone)]
pub struct GoogolswarmLocalAttestorPolicy {
    /// Role must *always* be LocalHostOnly for valid use. [file:3]
    pub role: GoogolswarmAttestorRole,
    /// Indicates whether this attestor can run completely offline using locally cached consensus
    /// roots and hash-linked logs. [file:6]
    pub offline_capable: bool,
    /// Attestation scope is strictly limited to verifying integrity and ordering of audit logs and
    /// governed decisions – not to changing policy, constraints, or optimization logic. [file:3][file:6]
    pub scope_attestation_only: bool,
    /// Explicit flag that this attestor has *no* authority to mutate ALN grammars, Birth-Signs,
    /// or workflow source code. [file:3]
    pub no_control_over_governance_logic: bool,
    /// Version for forward-only evolution of attestor policy.
    pub version: u32,
}

/// Security posture of a node profile; used by orchestration and governance. [file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeSecurityTier {
    TeeBacked,
    HardenedFirmwareOnly,
    Basic,
}

/// Summary of security features available on a node. [file:3]
#[derive(Debug, Clone)]
pub struct NodeSecurityProfile {
    pub node_profile_id: NodeProfileId,
    pub tier: NodeSecurityTier,
    /// Whether secure boot is enabled and enforced.
    pub secure_boot: bool,
    /// Whether firmware updates are signed and verified.
    pub signed_updates: bool,
    /// Whether TLSDTLS with modern ciphers is mandatory on this node.
    pub secure_transport: bool,
    /// Whether this node currently passes vulnerability scans and audits.
    pub last_audit_passed_at: Option<SystemTime>,
}

/// Governed decision envelope that bridges optimization, governance, and the trust-layer. [file:3]
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    pub workflow_stage: String,
    /// Domains involved in this decision.
    pub domains: Vec<GovernanceDomain>,
    /// Spatial context: which Birth-Signs governed this decision.
    pub birth_sign_ids: Vec<BirthSignId>,
    /// Norms actually consulted (laws, treaties, overlays).
    pub applied_aln_norms: Vec<AlnNormId>,
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,
    /// Participants (citizen, operator, node).
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    /// Hashes for inputs / outputs – immutable provenance references. [file:3]
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Governance evaluation result.
    pub evaluation: GovernanceEvaluation,
    /// Final decision outcome.
    pub outcome: DecisionOutcome,
    /// Narrative explanation for citizen interfaces.
    pub explanation: Option<String>,
    /// Domain-specific tags for extensibility.
    pub tags: HashMap<String, String>,
    /// Local-only Googolswarm attestation policy snapshot used when appending to trust-layer. [file:3]
    pub local_attestor_policy: GoogolswarmLocalAttestorPolicy,
}

/// Forward-only evolution marker for Birth-Sign versions and governance logic. [file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForwardOnlyChangeKind {
    BirthSignVersionIncrement,
    AlnNormAddition,
    ConstraintTightening,
    ConstraintRelaxationWithAudit,
    AttestorPolicyTightening,
}

/// Record describing a governance evolution step; used for PM and audit dashboards. [file:6]
#[derive(Debug, Clone)]
pub struct GovernanceEvolutionRecord {
    pub change_id: String,
    pub kind: ForwardOnlyChangeKind,
    pub created_at: SystemTime,
    pub author_did: Option<Did>,
    pub from_version: u32,
    pub to_version: u32,
    pub rationale: String,
}

/// Helper functions (impl blocks)

impl BirthSign {
    /// Returns true if this Birth-Sign covers the given domain. [file:3]
    pub fn covers_domain(&self, domain: GovernanceDomain) -> bool {
        self.domains.contains(&domain)
    }

    /// Returns true if this Birth-Sign is active at a given time instant. [file:3]
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
}

impl GovernanceEvaluation {
    /// Returns true if all outcomes are satisfied and FPIC status is non-blocking. [file:3]
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(fpic) = &self.fpic_status {
            match fpic {
                FpicStatus::Pending { .. } | FpicStatus::Denied { .. } => {
                    return false;
                }
                FpicStatus::NotRequired | FpicStatus::Granted { .. } => {}
            }
        }
        self.outcomes
            .iter()
            .all(|o| matches!(o, ConstraintOutcome::Satisfied))
    }

    /// Returns true if any hard violation is present. [file:3]
    pub fn has_hard_violation(&self) -> bool {
        self.outcomes.iter().any(|o| matches!(o, ConstraintOutcome::HardViolation { .. }))
    }

    /// Returns true if any soft violation is present. [file:3]
    pub fn has_soft_violation(&self) -> bool {
        self.outcomes.iter().any(|o| matches!(o, ConstraintOutcome::SoftViolation { .. }))
    }
}

impl NodeSecurityProfile {
    /// Returns true if this node is allowed to host highly sensitive governance workloads
    /// such as consent processing or DID handling. [file:3]
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// Returns true if this node is suitable for non-biosignal ERM logic. [file:3]
    pub fn can_host_general_erm(&self) -> bool {
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

impl GoogolswarmLocalAttestorPolicy {
    /// Returns true if this policy satisfies Aletheion's local-only, offline-capable attestor rules. [file:3][file:6]
    pub fn is_valid_local_only(&self) -> bool {
        self.role == GoogolswarmAttestorRole::LocalHostOnly
            && self.offline_capable
            && self.scope_attestation_only
            && self.no_control_over_governance_logic
    }

    /// Creates a default safe local-only policy, with explicit versioning for forward-only changes. [file:3]
    pub fn default_v1() -> Self {
        Self {
            role: GoogolswarmAttestorRole::LocalHostOnly,
            offline_capable: true,
            scope_attestation_only: true,
            no_control_over_governance_logic: true,
            version: 1,
        }
    }
}

impl GovernedDecisionEnvelope {
    /// Create a new envelope with default outcome PendingFpic and a safe local attestor policy. [file:3][file:6]
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
            local_attestor_policy: GoogolswarmLocalAttestorPolicy::default_v1(),
        }
    }

    /// Attach provenance about norms actually consulted. [file:3]
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

    /// Attach participants and node profile info, preserving augmented-citizen sovereignty via explicit DID fields. [file:6]
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

    /// Attach a human-readable explanation string for citizen interfaces. [file:6]
    pub fn with_explanation(mut self, explanation: Option<String>) -> Self {
        self.explanation = explanation;
        self
    }

    /// Tag the envelope with domain-specific metadata. [file:3]
    pub fn add_tag<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) {
        self.tags.insert(key.into(), value.into());
    }

    /// Enforce local-only Googolswarm usage before a trust-layer append.
    ///
    /// - Fails if policy is not LocalHostOnly or if scope exceeds attestation-only. [file:3][file:6]
    /// - Ensures no hidden elevation to a remote control surface.
    pub fn assert_local_attestor_only(&self) -> Result<(), String> {
        if !self.local_attestor_policy.is_valid_local_only() {
            return Err("Invalid Googolswarm attestor policy: must be local-host-only, offline-capable, attestation-only, and non-controlling.".to_string());
        }
        Ok(())
    }

    /// Decide the canonical DecisionOutcome from the evaluation and constraint mode.
    ///
    /// This function SHOULD be the single mapping point from ALN checks to workflow behavior for
    /// optimization and actuation stages, preventing silent reinvention of outcome logic. [file:3]
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
                FpicStatus::Granted { .. } | FpicStatus::NotRequired => {
                    // fall through
                }
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
                    // In strict hard mode, treat soft as reject as well.
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

/// Convenience for expressing durations in hours. [file:3]
pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

/// Minimal forward-only evolution helper.
///
/// This function checks that governance evolution steps always move versions upward and returns a
/// GovernanceEvolutionRecord for audit dashboards and Googolswarm attestation (local-only). [file:6]
pub fn record_forward_only_evolution(
    change_id: String,
    kind: ForwardOnlyChangeKind,
    author_did: Option<Did>,
    from_version: u32,
    to_version: u32,
    rationale: String,
) -> Result<GovernanceEvolutionRecord, String> {
    if to_version <= from_version {
        return Err("Forward-only evolution requires to_version > from_version; refusing silent rollback or reinvention.".to_string());
    }

    Ok(GovernanceEvolutionRecord {
        change_id,
        kind,
        created_at: SystemTime::now(),
        author_did,
        from_version,
        to_version,
        rationale,
    })
}
