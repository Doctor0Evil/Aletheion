// Path: aletheion/trust/attestors/ALE-TRUST-GOOGOLSWARM-LOCAL-ATTESTOR-001.rs
//
// Role:
// - Define Aletheion's canonical, host-only Googolswarm attestor model.
// - Enforce "attestation-only, no-control" semantics at the type and API level.
// - Provide immutable structures for multi-sig, ALN/KYC/DID, and quantum-safe governance tags.
// - Expose only offline-capable, append-style functions; no remote triggers, no remote exec.
// - Provide hooks for CI / ALN schemas to verify there is no silent-takeover or hidden control.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// -------------------------------------------------------------------------
/// Core identifiers and enums
/// -------------------------------------------------------------------------

/// Logical identifier for a Googolswarm-local attestor binding to this host / node.
/// This is not a network address; it is a local configuration handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalAttestorId(pub String);

impl LocalAttestorId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        LocalAttestorId(s.into())
    }
}

/// Identifier for an attestation session, scoped to a single host and time window.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestationSessionId(pub String);

impl AttestationSessionId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        AttestationSessionId(s.into())
    }
}

/// Identifier of an attested artifact (binary, config snapshot, ALN schema, workflow manifest).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestedArtifactId(pub String);

impl AttestedArtifactId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        AttestedArtifactId(s.into())
    }
}

/// Identifier for an Aletheion node profile in infra/orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

impl NodeProfileId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        NodeProfileId(s.into())
    }
}

/// Decentralized identifier of a human, institution, or device key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

impl Did {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Did(s.into())
    }
}

/// Minimal ALN norm identifier (rights, treaties, ecosafety contracts) used in attestations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

impl AlnNormId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        AlnNormId(s.into())
    }
}

/// BirthSign identifier: jurisdictional tile that governs the attested workflow or binary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

impl BirthSignId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        BirthSignId(s.into())
    }
}

/// Domain of governance for which an artifact is being attested.
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

/// Cryptographic / governance profile of an attestation.
/// These are descriptive tags; actual key material lives in secure storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceCryptoProfile {
    /// Classical-only keys; not suitable for long-lived, high-value treaties.
    ClassicalOnly,
    /// Hybrid classical + PQ; preferred for long-lived or high-impact records.
    HybridPostQuantum,
    /// PQ-only envelope where supported.
    PostQuantumPreferred,
}

/// Result of the local validation checks performed before an append is allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalValidationOutcome {
    Passed,
    Failed(String),
}

/// Multi-sig attestation decision outcome for this host-only attest action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalAttestationDecision {
    Approved,
    Rejected(String),
}

/// -------------------------------------------------------------------------
/// Multi-sig and ALN / KYC / DID anchors
/// -------------------------------------------------------------------------

/// A single signer participating in a local attestation decision.
#[derive(Debug, Clone)]
pub struct LocalAttestorSigner {
    pub did: Did,
    pub role: String,           // e.g., "infra-maintainer", "governance-auditor"
    pub weight: u8,             // voting weight for threshold schemes
}

/// Multi-sig policy required to accept an attestation on this host.
#[derive(Debug, Clone)]
pub struct LocalMultiSigPolicy {
    /// Minimum total weight required to approve.
    pub min_approval_weight: u16,
    /// Minimum distinct signers required.
    pub min_signers: u8,
    /// Whether at least one governance/audit role is mandatory.
    pub require_governance_auditor: bool,
}

impl LocalMultiSigPolicy {
    pub fn satisfied_by(&self, signers: &[LocalAttestorSigner]) -> bool {
        let mut total_weight: u16 = 0;
        let mut distinct_signers: u16 = 0;
        let mut has_auditor = false;

        let mut seen: HashMap<String, ()> = HashMap::new();
        for s in signers {
            let key = s.did.0.clone();
            if !seen.contains_key(&key) {
                seen.insert(key, ());
                distinct_signers += 1;
            }
            total_weight = total_weight.saturating_add(s.weight as u16);
            if s.role.contains("auditor") || s.role.contains("governance") {
                has_auditor = true;
            }
        }

        if distinct_signers < self.min_signers as u16 {
            return false;
        }
        if total_weight < self.min_approval_weight {
            return false;
        }
        if self.require_governance_auditor && !has_auditor {
            return false;
        }
        true
    }
}

/// Local KYC / DID compliance flags for an attestation subject.
#[derive(Debug, Clone)]
pub struct LocalKycDidStatus {
    pub subject_did: Did,
    pub kyc_verified: bool,
    pub did_bound_to_device: bool,
    pub did_bound_to_institution: bool,
    pub last_checked_at: SystemTime,
}

/// Quantum governance flags: this is metadata only; real crypto is out-of-band.
#[derive(Debug, Clone)]
pub struct QuantumGovernanceFlags {
    pub profile: GovernanceCryptoProfile,
    /// True if key lifecycle policies (rotation, escrow) meet Aletheion standards.
    pub lifecycle_compliant: bool,
    /// True if quantum-safe algorithms are used for new treaties and long-lived records.
    pub pq_algorithms_required: bool,
}

/// -------------------------------------------------------------------------
/// Local, host-only Googolswarm append envelope
/// -------------------------------------------------------------------------

/// Local-only record describing *what* Aletheion is asking Googolswarm to attest.
#[derive(Debug, Clone)]
pub struct LocalAttestationSubject {
    pub artifact_id: AttestedArtifactId,
    pub artifact_kind: String,          // e.g., "binary", "config", "workflow-manifest", "schema-aln"
    pub artifact_hash: String,         // hash of artifact contents, precomputed by Aletheion
    pub node_profile_id: NodeProfileId,
    pub domains: Vec<GovernanceDomain>,
    pub birthsign_ids: Vec<BirthSignId>,
    pub applied_aln_norms: Vec<AlnNormId>,
    /// Arbitrary metadata: version, build-id, repo, commit hash, etc.
    pub tags: HashMap<String, String>,
}

/// The only structure that is serialized and appended into Googolswarm.
/// It is *append-only* and *offline-first*; it carries no instructions for remote control.
#[derive(Debug, Clone)]
pub struct GoogolswarmLocalAttestationEnvelope {
    pub attestor_id: LocalAttestorId,
    pub session_id: AttestationSessionId,
    pub subject: LocalAttestationSubject,
    pub created_at: SystemTime,
    pub kyc_status: LocalKycDidStatus,
    pub quantum_governance: QuantumGovernanceFlags,
    pub signers: Vec<LocalAttestorSigner>,
    pub decision: LocalAttestationDecision,
    /// Local validation outcome (pre-commit checks).
    pub local_validation: LocalValidationOutcome,
    /// Opaque map for domain-specific, non-control metadata.
    pub tags: HashMap<String, String>,
}

impl GoogolswarmLocalAttestationEnvelope {
    /// Helper to construct an envelope with default tags and pending decision.
    pub fn new(
        attestor_id: LocalAttestorId,
        session_id: AttestationSessionId,
        subject: LocalAttestationSubject,
        kyc_status: LocalKycDidStatus,
        quantum_governance: QuantumGovernanceFlags,
        signers: Vec<LocalAttestorSigner>,
    ) -> Self {
        GoogolswarmLocalAttestationEnvelope {
            attestor_id,
            session_id,
            subject,
            created_at: SystemTime::now(),
            kyc_status,
            quantum_governance,
            signers,
            decision: LocalAttestationDecision::Rejected(
                "UNDECIDED_INITIAL_STATE".to_string(),
            ),
            local_validation: LocalValidationOutcome::Failed(
                "UNVALIDATED_INITIAL_STATE".to_string(),
            ),
            tags: HashMap::new(),
        }
    }

    pub fn with_tag<S: Into<String>, T: Into<String>>(mut self, k: S, v: T) -> Self {
        self.tags.insert(k.into(), v.into());
        self
    }
}

/// -------------------------------------------------------------------------
/// Local validation logic (offline, no remote control surface)
/// -------------------------------------------------------------------------

/// Static checklist to ensure the attestation does NOT embed silent-takeover capabilities.
/// This runs entirely on the host and must pass before any append is allowed.
#[derive(Debug, Clone)]
pub struct SilentControlCheckConfig {
    /// Forbid any artifact kind that could encode remote execution scripts.
    pub forbidden_artifact_kinds: Vec<String>,
    /// Forbid any tag keys that might be misused as control channels.
    pub forbidden_tag_keys: Vec<String>,
    /// Forbid explicit "control" or "exec" semantics in artifact tags.
    pub forbid_control_semantics: bool,
}

impl SilentControlCheckConfig {
    pub fn aletheion_default() -> Self {
        SilentControlCheckConfig {
            forbidden_artifact_kinds: vec![
                "remote-control-script".to_string(),
                "agent-autonomy-kernel".to_string(),
                "cloud-orchestrator".to_string(),
            ],
            forbidden_tag_keys: vec![
                "remote_exec".to_string(),
                "control_channel".to_string(),
                "override_url".to_string(),
            ],
            forbid_control_semantics: true,
        }
    }
}

/// Run local, deterministic checks to ensure this envelope cannot be used as a control surface.
pub fn run_silent_control_checks(
    env: &GoogolswarmLocalAttestationEnvelope,
    cfg: &SilentControlCheckConfig,
) -> LocalValidationOutcome {
    // 1. Artifact-kind blacklist.
    if cfg
        .forbidden_artifact_kinds
        .iter()
        .any(|k| k == &env.subject.artifact_kind)
    {
        return LocalValidationOutcome::Failed(format!(
            "Forbidden artifact_kind for Googolswarm attestation: {}",
            env.subject.artifact_kind
        ));
    }

    // 2. Tag-key blacklist.
    for bad_key in &cfg.forbidden_tag_keys {
        if env.subject.tags.contains_key(bad_key) || env.tags.contains_key(bad_key) {
            return LocalValidationOutcome::Failed(format!(
                "Forbidden tag key present in attestation: {}",
                bad_key
            ));
        }
    }

    // 3. Simple semantic filter on tag values (no "exec:", "ssh://", "http://" control URIs).
    if cfg.forbid_control_semantics {
        let suspicious_prefixes = ["exec:", "ssh://", "tcp://", "udp://", "http://", "https://"];
        for (k, v) in env.subject.tags.iter().chain(env.tags.iter()) {
            for prefix in &suspicious_prefixes {
                if v.trim_start().starts_with(prefix) {
                    return LocalValidationOutcome::Failed(format!(
                        "Suspicious control-like semantic in tag {}: {}",
                        k, v
                    ));
                }
            }
        }
    }

    LocalValidationOutcome::Passed
}

/// Enforce multi-sig, KYC/DID, and local checks.
/// No network calls here; this is fully offline.
///
/// If everything passes, the envelope decision becomes `Approved`.
/// Otherwise it becomes `Rejected` with a reason.
pub fn finalize_local_attestation(
    mut env: GoogolswarmLocalAttestationEnvelope,
    policy: &LocalMultiSigPolicy,
    silent_cfg: &SilentControlCheckConfig,
) -> GoogolswarmLocalAttestationEnvelope {
    // 1. KYC / DID checks: subject must be verified and bound at least to device or institution.
    if !env.kyc_status.kyc_verified {
        env.local_validation =
            LocalValidationOutcome::Failed("KYC verification missing".to_string());
        env.decision = LocalAttestationDecision::Rejected("KYC_FAILED".to_string());
        return env;
    }
    if !(env.kyc_status.did_bound_to_device || env.kyc_status.did_bound_to_institution) {
        env.local_validation = LocalValidationOutcome::Failed(
            "DID not bound to any trusted anchor (device or institution)".to_string(),
        );
        env.decision = LocalAttestationDecision::Rejected("DID_ANCHOR_MISSING".to_string());
        return env;
    }

    // 2. Multi-sig policy.
    if !policy.satisfied_by(&env.signers) {
        env.local_validation =
            LocalValidationOutcome::Failed("Local multi-sig policy not satisfied".to_string());
        env.decision = LocalAttestationDecision::Rejected("MULTISIG_FAILED".to_string());
        return env;
    }

    // 3. Silent-control checks.
    let sc = run_silent_control_checks(&env, silent_cfg);
    match &sc {
        LocalValidationOutcome::Failed(reason) => {
            env.local_validation = sc;
            env.decision = LocalAttestationDecision::Rejected(format!(
                "SILENT_CONTROL_CHECK_FAILED: {}",
                reason
            ));
            return env;
        }
        LocalValidationOutcome::Passed => {
            env.local_validation = sc;
        }
    }

    // 4. Quantum governance flags sanity: only allow append when lifecycle is compliant.
    if !env.quantum_governance.lifecycle_compliant {
        env.local_validation = LocalValidationOutcome::Failed(
            "Quantum governance lifecycle not compliant".to_string(),
        );
        env.decision = LocalAttestationDecision::Rejected("QGOV_LIFECYCLE_FAILED".to_string());
        return env;
    }

    // If we reach here, local validation is successful.
    env.decision = LocalAttestationDecision::Approved;
    env
}

/// -------------------------------------------------------------------------
/// Offline-only append stub (no control, no remote exec)
/// -------------------------------------------------------------------------

/// Local representation of an "append" into Googolswarm.
/// Note: this does not perform any network I/O. It just prepares the payload
/// that a separate, audited, offline-capable process will serialize and write.
#[derive(Debug, Clone)]
pub struct LocalGoogolswarmAppendRecord {
    pub tx_id: String,
    pub created_at: SystemTime,
    pub envelope: GoogolswarmLocalAttestationEnvelope,
}

impl LocalGoogolswarmAppendRecord {
    pub fn new(envelope: GoogolswarmLocalAttestationEnvelope) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let tx_id = format!(
            "ALE_GSW_LOCAL_TX_{}_{}",
            now,
            envelope.session_id.0.as_str()
        );
        LocalGoogolswarmAppendRecord {
            tx_id,
            created_at: SystemTime::now(),
            envelope,
        }
    }
}

/// Prepare an offline-only append record.
/// This is the *only* way this module talks about Googolswarm writes.
///
/// Callers MUST:
/// - run `finalize_local_attestation` first,
/// - ensure `decision == Approved`,
/// - serialize and persist the record using an audited, host-only process
///   (e.g., a batch job or air-gapped export).
pub fn prepare_local_append_record(
    env: GoogolswarmLocalAttestationEnvelope,
) -> Result<LocalGoogolswarmAppendRecord, String> {
    match env.decision {
        LocalAttestationDecision::Approved => Ok(LocalGoogolswarmAppendRecord::new(env)),
        LocalAttestationDecision::Rejected(reason) => Err(format!(
            "Cannot prepare append: local decision rejected ({})",
            reason
        )),
    }
}

/// -------------------------------------------------------------------------
/// Tests (no network, no control surfaces)
/// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_default_env() -> GoogolswarmLocalAttestationEnvelope {
        let attestor_id = LocalAttestorId::new("local-attestor-phoenix-01");
        let session_id = AttestationSessionId::new("session-123");
        let subject = LocalAttestationSubject {
            artifact_id: AttestedArtifactId::new("artifact-awp-allocator-001"),
            artifact_kind: "binary".to_string(),
            artifact_hash: "deadbeefcafebabe".to_string(),
            node_profile_id: NodeProfileId::new("NODE-PROFILE-LOCAL-001"),
            domains: vec![GovernanceDomain::Water, GovernanceDomain::Energy],
            birthsign_ids: vec![BirthSignId::new("BIRTHSIGN-DOWNTOWN-CANAL-01")],
            applied_aln_norms: vec![AlnNormId::new("ALN-NORM-WATER-TREATY-001")],
            tags: HashMap::new(),
        };
        let kyc_status = LocalKycDidStatus {
            subject_did: Did::new("did:aletheion:local-operator-001"),
            kyc_verified: true,
            did_bound_to_device: true,
            did_bound_to_institution: false,
            last_checked_at: SystemTime::now(),
        };
        let quantum_governance = QuantumGovernanceFlags {
            profile: GovernanceCryptoProfile::HybridPostQuantum,
            lifecycle_compliant: true,
            pq_algorithms_required: true,
        };
        let signers = vec![
            LocalAttestorSigner {
                did: Did::new("did:aletheion:infra-maintainer-001"),
                role: "infra-maintainer".to_string(),
                weight: 60,
            },
            LocalAttestorSigner {
                did: Did::new("did:aletheion:governance-auditor-001"),
                role: "governance-auditor".to_string(),
                weight: 60,
            },
        ];

        GoogolswarmLocalAttestationEnvelope::new(
            attestor_id,
            session_id,
            subject,
            kyc_status,
            quantum_governance,
            signers,
        )
    }

    #[test]
    fn test_silent_control_check_blocks_forbidden_kind() {
        let mut env = mk_default_env();
        env.subject.artifact_kind = "remote-control-script".to_string();
        let cfg = SilentControlCheckConfig::aletheion_default();
        let res = run_silent_control_checks(&env, &cfg);
        match res {
            LocalValidationOutcome::Failed(_) => {}
            _ => panic!("Expected failure for forbidden artifact kind"),
        }
    }

    #[test]
    fn test_finalize_local_attestation_success() {
        let env = mk_default_env();
        let policy = LocalMultiSigPolicy {
            min_approval_weight: 100,
            min_signers: 2,
            require_governance_auditor: true,
        };
        let cfg = SilentControlCheckConfig::aletheion_default();
        let env2 = finalize_local_attestation(env, &policy, &cfg);
        match env2.decision {
            LocalAttestationDecision::Approved => {}
            _ => panic!("Expected approved decision for valid env"),
        }
        match env2.local_validation {
            LocalValidationOutcome::Passed => {}
            _ => panic!("Expected local validation to pass"),
        }
    }

    #[test]
    fn test_prepare_local_append_record_rejects_if_not_approved() {
        let env = mk_default_env();
        let res = prepare_local_append_record(env);
        assert!(res.is_err(), "Append should fail if not approved");
    }
}
