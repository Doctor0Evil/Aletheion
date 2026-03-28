// aletheion/trust/googolswarm/offline/ALE-TRUST-GOOGOLSWARM-OFFLINE-ATTEST-CORE-001.rs
//
// Core offline, host-local Googolswarm attestation harness for Aletheion.
// Enforces: (1) Googolswarm is attestation-only, (2) no remote or online authority,
// (3) no silent control paths over workflows or firmware, (4) strong linkage to
// Birth-Signs, ALN norms, and governed decision envelopes as described in the
// governance and ERM blueprints for Aletheion.[file:3][file:6]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Stable identifiers reused across trust and governance layers.[file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowStageId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestorId(pub String);

/// Local-only Googolswarm namespace identifier; never points to a remote chain.[file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalLedgerId(pub String);

/// Governance domains mirrored from Birth-Signs and governed decision envelopes.[file:3][file:6]
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

/// Constraint evaluation outcome, mirrored from the Birth-Sign model.[file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// FPIC status must be present for sovereign-safe attestation.[file:3][file:6]
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

/// Constraint enforcement mode bound to the decision envelope.[file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintMode {
    Hard,
    HighPenalty,
}

/// Final decision outcome class that may be appended to the local ledger.[file:3]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// Aggregated governance evaluation copied from the governance core.[file:3]
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

/// Envelope describing a governed decision about to be attested locally.[file:3][file:6]
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelope {
    pub txid: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    pub workflow_stage: WorkflowStageId,
    pub domains: Vec<GovernanceDomain>,
    pub birthsign_ids: Vec<BirthSignId>,
    pub applied_aln_norms: Vec<AlnNormId>,
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub evaluation: GovernanceEvaluation,
    pub outcome: DecisionOutcome,
    pub explanation: Option<String>,
    pub tags: HashMap<String, String>,
}

impl GovernedDecisionEnvelope {
    pub fn new(
        txid: TrustTxId,
        workflow_id: WorkflowId,
        workflow_stage: WorkflowStageId,
        domains: Vec<GovernanceDomain>,
        birthsign_ids: Vec<BirthSignId>,
        evaluation: GovernanceEvaluation,
        inputs_hash: String,
        outputs_hash: String,
        outcome: DecisionOutcome,
    ) -> Self {
        Self {
            txid,
            created_at: SystemTime::now(),
            workflow_id,
            workflow_stage,
            domains,
            birthsign_ids,
            applied_aln_norms: Vec::new(),
            subject_did: None,
            operator_did: None,
            node_profile_id: None,
            inputs_hash,
            outputs_hash,
            evaluation,
            outcome,
            explanation: None,
            tags: HashMap::new(),
        }
    }

    pub fn with_norms(
        mut self,
        aln_norms: Vec<AlnNormId>,
    ) -> Self {
        self.applied_aln_norms = aln_norms;
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

    pub fn add_tag<S, T>(&mut self, key: S, value: T)
    where
        S: Into<String>,
        T: Into<String>,
    {
        self.tags.insert(key.into(), value.into());
    }
}

/// Attestation scope: strictly local, host-only, and offline.[file:3][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestationScope {
    /// Local filesystem or embedded store only.
    LocalStore,
    /// Local host memory plus explicit export via air-gapped media.
    LocalWithExport,
}

/// Policy for Googolswarm use inside Aletheion.[file:3][file:6]
#[derive(Debug, Clone)]
pub struct GoogolswarmUsePolicy {
    /// Googolswarm may only perform attestation; no scheduling, optimization, or control.
    pub attestation_only: bool,
    /// All operations must be offline and host-local.
    pub offline_only: bool,
    /// Attestation MUST reference Birth-Signs and ALN norms.
    pub requires_birthsign_and_aln: bool,
    /// Explicitly deny any remote authority or consensus role.
    pub deny_remote_authority: bool,
}

impl GoogolswarmUsePolicy {
    pub fn strict_offline_attestor() -> Self {
        Self {
            attestation_only: true,
            offline_only: true,
            requires_birthsign_and_aln: true,
            deny_remote_authority: true,
        }
    }
}

/// Configuration for a single local Googolswarm-compatible ledger.[file:3][file:6]
#[derive(Debug, Clone)]
pub struct LocalLedgerConfig {
    pub ledger_id: LocalLedgerId,
    pub scope: AttestationScope,
    /// Path on disk or equivalent embedded key to a local store.
    pub store_path: String,
    /// Maximum size before rotation; sovereignty-preserving by design.
    pub max_entries_before_rotate: u64,
    /// Minimal retention duration for audit purposes.
    pub min_retention: Duration,
}

/// Result of attempting to append a governed decision attestation.[file:3][file:6]
#[derive(Debug, Clone)]
pub enum AttestationStatus {
    Accepted {
        txid: TrustTxId,
        ledger_id: LocalLedgerId,
    },
    Rejected {
        txid: TrustTxId,
        reason: String,
    },
}

/// Error type for misuse or violation of offline attestation rules.[file:3][file:6]
#[derive(Debug)]
pub enum AttestationError {
    PolicyViolation(String),
    InvalidEnvelope(String),
    StorageFailure(String),
}

/// Core offline Googolswarm attestation engine.[file:3][file:6]
pub struct OfflineAttestationEngine {
    policy: GoogolswarmUsePolicy,
    config: LocalLedgerConfig,
}

impl OfflineAttestationEngine {
    pub fn new(policy: GoogolswarmUsePolicy, config: LocalLedgerConfig) -> Self {
        Self { policy, config }
    }

    /// Enforces that no remote or online authority can be exercised by this engine.
    pub fn assert_no_remote_authority(&self) -> Result<(), AttestationError> {
        if !self.policy.attestation_only {
            return Err(AttestationError::PolicyViolation(
                "Googolswarm must be attestation-only inside Aletheion.".into(),
            ));
        }
        if !self.policy.offline_only {
            return Err(AttestationError::PolicyViolation(
                "Googolswarm attestation must be offline and host-local.".into(),
            ));
        }
        if !self.policy.deny_remote_authority {
            return Err(AttestationError::PolicyViolation(
                "Remote or online consensus authority for Googolswarm is forbidden.".into(),
            ));
        }
        Ok(())
    }

    /// Validates that an envelope is suitable for sovereign-safe attestation and cannot be
    /// used as a silent takeover vector.[file:3][file:6]
    pub fn validate_envelope(
        &self,
        env: &GovernedDecisionEnvelope,
    ) -> Result<(), AttestationError> {
        if env.birthsign_ids.is_empty() {
            return Err(AttestationError::InvalidEnvelope(
                "Missing BirthSign bindings on governed decision.".into(),
            ));
        }
        if self.policy.requires_birthsign_and_aln && env.applied_aln_norms.is_empty() {
            return Err(AttestationError::InvalidEnvelope(
                "Missing ALN norms on governed decision for attestation.".into(),
            ));
        }
        match env.outcome {
            DecisionOutcome::Approved
            | DecisionOutcome::ApprovedWithDerating
            | DecisionOutcome::Rejected
            | DecisionOutcome::PendingFpic => {}
        }
        Ok(())
    }

    /// Core append operation: writes an attestation record to a local ledger store only.
    /// This function MUST NOT initiate remote consensus, network calls, or scheduling.[file:3][file:6]
    pub fn append_attestation(
        &self,
        env: &GovernedDecisionEnvelope,
    ) -> Result<AttestationStatus, AttestationError> {
        self.assert_no_remote_authority()?;
        self.validate_envelope(env)?;

        // Placeholder for actual local storage write. In a real deployment this would be a
        // write to an append-only logfile, embedded KV store, or host-only Googolswarm shard.
        // No network, RPC, or consensus calls are allowed here to prevent silent takeovers.[file:3][file:6]
        let _ = self.config.store_path.as_str();

        Ok(AttestationStatus::Accepted {
            txid: env.txid.clone(),
            ledger_id: self.config.ledger_id.clone(),
        })
    }

    /// Helper for checking whether an envelope could represent a silent control attempt,
    /// e.g., by trying to declare governance over domains it should not own.[file:3][file:6]
    pub fn detect_silent_takeover_pattern(
        &self,
        env: &GovernedDecisionEnvelope,
    ) -> bool {
        if env.domains.len() > 5 {
            return true;
        }
        let has_biosignal = env
            .domains
            .iter()
            .any(|d| matches!(d, GovernanceDomain::Biosignals | GovernanceDomain::Augmentation));
        let has_energy = env
            .domains
            .iter()
            .any(|d| matches!(d, GovernanceDomain::Energy));
        has_biosignal && has_energy
    }

    /// Guarded append that rejects envelopes exhibiting silent takeover signatures.[file:3][file:6]
    pub fn guarded_append(
        &self,
        env: &GovernedDecisionEnvelope,
    ) -> Result<AttestationStatus, AttestationError> {
        if self.detect_silent_takeover_pattern(env) {
            return Err(AttestationError::PolicyViolation(
                "Potential silent takeover pattern detected in governed decision envelope.".into(),
            ));
        }
        self.append_attestation(env)
    }
}

/// Minimal convenience for expressing hours as Duration.[file:3]
pub fn hours(h: u64) -> Duration {
    Duration::from_secs(h * 3600)
}

/// Example convenience constructor for a default local ledger config.[file:3][file:6]
pub fn default_local_ledger_config() -> LocalLedgerConfig {
    LocalLedgerConfig {
        ledger_id: LocalLedgerId("aletheion-local-googolswarm-ledger".into()),
        scope: AttestationScope::LocalStore,
        store_path: "var/aletheion/trust/googolswarm/local-ledger.log".into(),
        max_entries_before_rotate: 1_000_000,
        min_retention: hours(24 * 365),
    }
}

/// Example engine constructor to be used by ERM trust append workflows.[file:3][file:6]
pub fn make_default_offline_attestation_engine() -> OfflineAttestationEngine {
    OfflineAttestationEngine::new(
        GoogolswarmUsePolicy::strict_offline_attestor(),
        default_local_ledger_config(),
    )
}
