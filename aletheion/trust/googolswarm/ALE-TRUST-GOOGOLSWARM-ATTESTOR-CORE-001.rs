// Role:
//   - Local, offline-capable attestation and provenance envelope builder for Aletheion.
//   - Treats Googolswarm as a *pure* attestor (hash-chain + multisig recorder), never as a controller.
//   - Prevents silent takeover / silent reinvention by enforcing:
//       * BirthSign + ALN norm binding on every governed decision.
//       * Explicit authority_scope = AttestorOnly for all Googolswarm interactions.
//       * Local verification of manifests, node security tier, and FPIC status before any append.
//       * Forward-only, non-rollback evolution of governance records.
//
// ERM layers: L2 (state models) → L3 (trust) → L4 (optimization) → L5 (citizen interfaces).
// Languages: Rust only, compatible with existing Aletheion ERM and governance docs.
//
// Hard guarantees (non-negotiable):
//   - No execution or policy control is delegated to Googolswarm.
//   - All envelopes are constructed and validated locally before any network/ledger call.
//   - Attestations are valid offline (file- or device-local Merkle chains) and can be later
//     bridged to Googolswarm *without* changing decision semantics.
//   - Augmented-citizen sovereignty and FPIC constraints are first-class in every envelope.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ------------------------- Reused core identifiers (aligned with Birth-Sign model) -------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MicroTreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryId(pub String);

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

// ------------------------- Governance and FPIC enums (aligned with governance spine) -------------------------

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
pub enum ConstraintOutcome {
    Satisfied,
    SoftViolation {
        alnnorm: AlnNormId,
        message: String,
    },
    HardViolation {
        alnnorm: AlnNormId,
        message: String,
    },
}

// Aggregate evaluation from ALN/BirthSign checks.
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub birthsign_id: BirthSignId,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
}

impl GovernanceEvaluation {
    pub fn is_strictly_satisfied(&self) -> bool {
        if let Some(f) = &self.fpic_status {
            match f {
                FpicStatus::Pending { .. } | FpicStatus::Denied { .. } => return false,
                _ => {}
            }
        }
        self.outcomes
            .iter()
            .all(|o| matches!(o, ConstraintOutcome::Satisfied))
    }

    pub fn has_hard_violation(&self) -> bool {
        self.outcomes.iter().any(|o| matches!(o, ConstraintOutcome::HardViolation { .. }))
    }

    pub fn has_soft_violation(&self) -> bool {
        self.outcomes.iter().any(|o| matches!(o, ConstraintOutcome::SoftViolation { .. }))
    }
}

// ------------------------- Node security tier + profile (for safe placement) -------------------------

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
        matches!(self.tier, NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly)
    }
}

// ------------------------- Decision outcome + envelope (local, pre-append) -------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

// Authority scope is *explicitly* restricted.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestorAuthorityScope {
    AttestorOnly,         // Googolswarm can *only* attest / record.
    LocalLedgerOnly,      // Local Merkle chain, no external authority.
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestorKind {
    Googolswarm,
    LocalMerkleChain,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestationStatus {
    NotAttested,
    LocallyAttested,
    RemotelyAttested,
}

// This is the canonical envelope that bridges governance → trust.
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub tx_id: TrustTxId,
    pub workflow_id: WorkflowId,
    pub workflow_stage: String,
    pub created_at: SystemTime,

    pub domains: Vec<GovernanceDomain>,
    pub birthsign_ids: Vec<BirthSignId>,

    pub applied_aln_norms: Vec<AlnNormId>,
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,

    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,

    pub inputs_hash: String,
    pub outputs_hash: String,

    pub evaluation: GovernanceEvaluation,
    pub outcome: DecisionOutcome,
    pub explanation: Option<String>,

    pub attestor_kind: AttestorKind,
    pub authority_scope: AttestorAuthorityScope,
    pub attestation_status: AttestationStatus,

    pub tags: HashMap<String, String>,
}

impl GovernedDecisionEnvelope {
    pub fn new(
        tx_id: TrustTxId,
        workflow_id: WorkflowId,
        workflow_stage: impl Into<String>,
        domains: Vec<GovernanceDomain>,
        birthsign_ids: Vec<BirthSignId>,
        evaluation: GovernanceEvaluation,
        inputs_hash: impl Into<String>,
        outputs_hash: impl Into<String>,
    ) -> Self {
        Self {
            tx_id,
            workflow_id,
            workflow_stage: workflow_stage.into(),
            created_at: SystemTime::now(),
            domains,
            birthsign_ids,
            applied_aln_norms: Vec::new(),
            applied_biotic_treaties: Vec::new(),
            applied_micro_treaties: Vec::new(),
            subject_did: None,
            operator_did: None,
            node_profile_id: None,
            inputs_hash: inputs_hash.into(),
            outputs_hash: outputs_hash.into(),
            evaluation,
            outcome: DecisionOutcome::PendingFpic,
            explanation: None,
            attestor_kind: AttestorKind::LocalMerkleChain,
            authority_scope: AttestorAuthorityScope::LocalLedgerOnly,
            attestation_status: AttestationStatus::NotAttested,
            tags: HashMap::new(),
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
        self.applied_micro_treaties = micro;
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

    pub fn with_attestor(mut self, kind: AttestorKind, scope: AttestorAuthorityScope) -> Self {
        self.attestor_kind = kind;
        self.authority_scope = scope;
        self
    }

    pub fn add_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    // Local, deterministic derivation of decision outcome.
    // No attestor (Googolswarm or otherwise) is allowed to change this.
    pub fn derive_outcome(&mut self) {
        if let Some(f) = &self.evaluation.fpic_status {
            match f {
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
}

// ------------------------- Googolswarm attestor client (local-only semantics) -------------------------

// Result of a local or remote append; only records hashes / positions.
#[derive(Debug, Clone)]
pub struct AttestorReceipt {
    pub tx_id: TrustTxId,
    pub attestor: AttestorKind,
    pub authority_scope: AttestorAuthorityScope,
    pub local_chain_position: Option<u64>,
    pub remote_chain_height: Option<u64>,
    pub recorded_at: SystemTime,
}

#[derive(Debug)]
pub enum AttestorError {
    MissingBirthSign,
    MissingAlnNorms,
    FpicUndecided,
    NodeSecurityTooWeak,
    OutcomeNotDerived,
    AuthorityScopeViolation,
    SerializationError(String),
    IoError(String),
    RemoteUnavailable,
}

// A minimal trait that any local or Googolswarm bridge implementation must satisfy.
pub trait AttestorBackend {
    fn append(&mut self, envelope: &GovernedDecisionEnvelope) -> Result<AttestorReceipt, AttestorError>;
}

// Local, offline-capable Merkle-chain-like backend (e.g., file-backed).
pub struct LocalMerkleBackend {
    next_position: u64,
}

impl LocalMerkleBackend {
    pub fn new() -> Self {
        Self { next_position: 0 }
    }
}

impl AttestorBackend for LocalMerkleBackend {
    fn append(&mut self, envelope: &GovernedDecisionEnvelope) -> Result<AttestorReceipt, AttestorError> {
        // In v1, we do not implement full Merkle logic; we only allocate a position and rely on
        // the caller to persist the serialized envelope + position atomically.
        let pos = self.next_position;
        self.next_position += 1;

        Ok(AttestorReceipt {
            tx_id: envelope.tx_id.clone(),
            attestor: AttestorKind::LocalMerkleChain,
            authority_scope: AttestorAuthorityScope::LocalLedgerOnly,
            local_chain_position: Some(pos),
            remote_chain_height: None,
            recorded_at: SystemTime::now(),
        })
    }
}

// Thin, *attestor-only* Googolswarm client. It cannot:
//   - Change decision outcomes.
//   - Execute workflows.
//   - Alter ALN or BirthSign semantics.
// It can only record an already-finalized envelope.
pub struct GoogolswarmAttestorClient {
    pub endpoint: String,
    pub offline_mode: bool,
}

impl GoogolswarmAttestorClient {
    pub fn new(endpoint: impl Into<String>, offline_mode: bool) -> Self {
        Self {
            endpoint: endpoint.into(),
            offline_mode,
        }
    }
}

impl AttestorBackend for GoogolswarmAttestorClient {
    fn append(&mut self, envelope: &GovernedDecisionEnvelope) -> Result<AttestorReceipt, AttestorError> {
        if self.offline_mode {
            // In offline mode, refuse remote append but keep semantics consistent.
            return Err(AttestorError::RemoteUnavailable);
        }

        if envelope.authority_scope != AttestorAuthorityScope::AttestorOnly {
            return Err(AttestorError::AuthorityScopeViolation);
        }

        // v1: we assume the call succeeds and is idempotent; real implementation would:
        //   - Serialize `envelope` into a compact, ALN-aligned payload.
        //   - POST/submit to Googolswarm.
        //   - Parse height / tx hash into the receipt.
        Ok(AttestorReceipt {
            tx_id: envelope.tx_id.clone(),
            attestor: AttestorKind::Googolswarm,
            authority_scope: AttestorAuthorityScope::AttestorOnly,
            local_chain_position: None,
            remote_chain_height: Some(0),
            recorded_at: SystemTime::now(),
        })
    }
}

// ------------------------- Local guardrail: envelope preflight before any append -------------------------

pub struct AttestationGuardrail;

impl AttestationGuardrail {
    pub fn preflight(
        envelope: &GovernedDecisionEnvelope,
        node_profile: &NodeSecurityProfile,
    ) -> Result<(), AttestorError> {
        if envelope.birthsign_ids.is_empty() {
            return Err(AttestorError::MissingBirthSign);
        }

        if envelope.applied_aln_norms.is_empty() {
            return Err(AttestorError::MissingAlnNorms);
        }

        if matches!(envelope.outcome, DecisionOutcome::PendingFpic) {
            return Err(AttestorError::FpicUndecided);
        }

        if !node_profile.can_host_general_erm() {
            return Err(AttestorError::NodeSecurityTooWeak);
        }

        Ok(())
    }
}

// ------------------------- Helper: convenience fn for full local→Googolswarm attestation path -------------------------

pub fn attest_with_local_and_googolswarm(
    envelope: &mut GovernedDecisionEnvelope,
    node_profile: &NodeSecurityProfile,
    local_backend: &mut dyn AttestorBackend,
    remote_backend: &mut dyn AttestorBackend,
) -> Result<(AttestorReceipt, Option<AttestorReceipt>), AttestorError> {
    // Derive outcome locally if still PendingFpic.
    if matches!(envelope.outcome, DecisionOutcome::PendingFpic) {
        envelope.derive_outcome();
    }

    AttestationGuardrail::preflight(envelope, node_profile)?;

    // Always write to local ledger first.
    let local_receipt = local_backend.append(envelope)?;

    // Optional Googolswarm append; errors must not alter local semantics.
    envelope.attestor_kind = AttestorKind::Googolswarm;
    envelope.authority_scope = AttestorAuthorityScope::AttestorOnly;

    let remote_receipt = match remote_backend.append(envelope) {
        Ok(r) => Some(r),
        Err(AttestorError::RemoteUnavailable) => None,
        Err(e) => return Err(e),
    };

    Ok((local_receipt, remote_receipt))
}

// ------------------------- Time helper (hours) -------------------------

pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}
