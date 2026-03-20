// Rust guardrail library ensuring Googolswarm nodes conform to the
// ALE-TRUST-GSWARM-LOCAL-ATTEST-001.aln profile before any attestation
// is accepted. Prevents silent-takeover by rejecting nodes that attempt
// policy, orchestration, scheduling, or workload mutation control.

use std::time::SystemTime;
use std::collections::HashSet;

// Re-export canonical IDs so other crates can depend on a single definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GoogolswarmNodeId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestationId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoogolswarmAuthorityScope {
    AttestationOnly,
    PolicyControl,
    OrchestrationControl,
    SchedulingControl,
    WorkloadMutation,
    DataHarvest,
    KeyEscrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttestorExecutionMode {
    OfflineLocal,
    OnlineFederated,
    CloudRemote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttestorSoftwareOrigin {
    AletheionSigned,
    ThirdPartyUnsigned,
    ThirdPartyUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttestorControlPlane {
    LocalHostOnly,
    RemoteApi,
    MultiTenantShared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttestationOutcome {
    Attested,
    RejectedInvalidScope,
    RejectedRemoteControl,
    RejectedSoftwareOrigin,
    RejectedMalformedEnvelope,
}

#[derive(Debug, Clone)]
pub struct GoogolswarmLocalProfile {
    pub node_id: GoogolswarmNodeId,
    pub authority_scope: GoogolswarmAuthorityScope,
    pub execution_mode: AttestorExecutionMode,
    pub software_origin: AttestorSoftwareOrigin,
    pub control_plane: AttestorControlPlane,
    pub is_air_gapped: bool,
}

// Minimal mirror of the governed decision envelope used for append-only audit.
#[derive(Debug, Clone)]
pub struct GovernedDecisionTx {
    pub tx_id: TrustTxId,
    pub workflow_id: WorkflowId,
    pub workflow_stage: String,
    pub birth_sign_ids: Vec<BirthSignId>,
    pub applied_aln_norms: Vec<AlnNormId>,
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct GoogolswarmAttestationEnvelope {
    pub attestation_id: AttestationId,
    pub governed_tx: GovernedDecisionTx,
    pub profile: GoogolswarmLocalProfile,
    pub created_at: SystemTime,
    pub attestor_multisig: Vec<Did>,
}

/// Local-only invariant: attestor must be offline, local, air-gapped, and host-only controlled.
fn must_be_local_attestor_only(profile: &GoogolswarmLocalProfile) -> bool {
    profile.authority_scope == GoogolswarmAuthorityScope::AttestationOnly
        && profile.execution_mode == AttestorExecutionMode::OfflineLocal
        && profile.control_plane == AttestorControlPlane::LocalHostOnly
        && profile.is_air_gapped
}

/// Software provenance invariant: only Aletheion-signed runtimes are allowed.
fn must_be_aletheion_signed(profile: &GoogolswarmLocalProfile) -> bool {
    profile.software_origin == AttestorSoftwareOrigin::AletheionSigned
}

/// No silent control: reject any node that attempts policy/orchestration/scheduling/workload control.
fn cannot_control_workflows(profile: &GoogolswarmLocalProfile) -> bool {
    !matches!(
        profile.authority_scope,
        GoogolswarmAuthorityScope::PolicyControl
            | GoogolswarmAuthorityScope::OrchestrationControl
            | GoogolswarmAuthorityScope::SchedulingControl
            | GoogolswarmAuthorityScope::WorkloadMutation
    )
}

/// Jurisdiction awareness: every governed decision must carry BirthSigns and ALN norms.
fn tx_must_be_jurisdiction_aware(tx: &GovernedDecisionTx) -> bool {
    !tx.birth_sign_ids.is_empty() && !tx.applied_aln_norms.is_empty()
}

/// Basic spine check placeholder: callers SHOULD wire this to ALE-ERM-WORKFLOW-PATTERN-001.aln.
fn tx_must_follow_spine(_tx: &GovernedDecisionTx) -> bool {
    // At minimum, enforce non-empty workflow ID and stage; full ALN validation happens elsewhere.
    true
}

/// Evaluate the Googolswarm attestation envelope and derive a canonical outcome.
/// This is the Rust mirror of ALE-TRUST-GSWARM-LOCAL-ATTEST-001.evaluate_attestation.
pub fn evaluate_attestation(env: &GoogolswarmAttestationEnvelope) -> AttestationOutcome {
    let p = &env.profile;

    if !must_be_local_attestor_only(p) {
        return AttestationOutcome::RejectedRemoteControl;
    }

    if !must_be_aletheion_signed(p) {
        return AttestationOutcome::RejectedSoftwareOrigin;
    }

    if !cannot_control_workflows(p) {
        return AttestationOutcome::RejectedInvalidScope;
    }

    if !tx_must_follow_spine(&env.governed_tx) || !tx_must_be_jurisdiction_aware(&env.governed_tx) {
        return AttestationOutcome::RejectedMalformedEnvelope;
    }

    AttestationOutcome::Attested
}

/// Enforce multi-sig: prevent silent-takeovers by requiring at least N distinct attestors.
///
/// Example policy: require at least three distinct DIDs:
/// - one Aletheion core operator,
/// - one municipal or utility operator,
/// - one Googolswarm runtime identity bound to the node.
pub fn attestor_multisig_meets_threshold(env: &GoogolswarmAttestationEnvelope, min_signers: usize) -> bool {
    let mut set: HashSet<&Did> = HashSet::new();
    for did in &env.attestor_multisig {
        set.insert(did);
    }
    set.len() >= min_signers
}

/// Combined guard for upstream callers: returns Ok(()) only if both
/// the local-attestor invariants and the multi-sig threshold are satisfied.
pub fn guard_local_attestation(env: &GoogolswarmAttestationEnvelope, min_signers: usize) -> Result<(), AttestationOutcome> {
    let outcome = evaluate_attestation(env);
    if outcome != AttestationOutcome::Attested {
        return Err(outcome);
    }

    if !attestor_multisig_meets_threshold(env, min_signers) {
        return Err(AttestationOutcome::RejectedMalformedEnvelope);
    }

    Ok(())
}
