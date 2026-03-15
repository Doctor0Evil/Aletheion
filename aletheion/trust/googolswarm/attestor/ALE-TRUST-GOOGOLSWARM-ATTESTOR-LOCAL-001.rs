// aletheion/trust/googolswarm/attestor/ALE-TRUST-GOOGOLSWARM-ATTESTOR-LOCAL-001.rs
//
// Local-only Googolswarm attestation core for Aletheion.
// Enforces: (1) host-only, offline-capable verification, (2) zero control
// over workflows or policy, (3) explicit augmented-citizen sovereignty hooks,
// and (4) structural resistance to silent-takeovers or silent reinvention.
//
// ERM layers: L3 Blockchain Trust, L5 Citizen Interface (via summaries).
// Languages: Rust only. No forbidden hashes, no DT semantics, no rollbacks.
//
// This module does NOT own consensus, scheduling, or policy. It only:
// - Verifies that a proposed transaction or decision envelope
//   matches local provenance, ALN norms, BirthSign bindings, and
//   multi-sig attestors defined by Aletheion's own schemas.
// - Emits an AttestationRecord that other modules may record on Googolswarm,
//   but Googolswarm itself never decides or mutates Aletheion behavior.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

/// Copy of stable identifiers used across governance and trust layers.
/// These mirror shapes described in governance research for BirthSigns,
/// ALN norms, DIDs, and workflow identifiers.[file:2][file:5]

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

/// Local-only identifier for a Googolswarm ledger namespace.
/// This is intentionally a passive label; it does not grant any
/// active control over Aletheion workflows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GoogolswarmNamespaceId(pub String);

/// Local attestor node identifier (host, enclave, or hardened firmware profile).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestorNodeId(pub String);

/// Domain tags for provenance records.
/// These mirror the ERM domains (water, thermal, mobility, etc.).[file:5]
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

/// Encodes whether this attestation is executed strictly offline,
/// or in a mode where limited, pre-approved network paths exist.
/// Aletheion restricts Googolswarm to LOCAL_ONLY for core governance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestationConnectivityMode {
    LocalOnly,
    LocalFirstWithStagedSync,
}

/// Encodes the role of Googolswarm in Aletheion.
/// By design, Googolswarm is restricted to ATTESTOR_ONLY and must
/// never escalate to any stronger authority.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoogolswarmAuthorityScope {
    /// May only record and verify provenance records under
    /// ALN-governed schemas; cannot schedule, optimize, or actuate.
    AttestorOnly,
    /// Reserved for explicit rejection of any wider scope encountered
    /// in configs; used as a safety tripwire.
    RejectedEscalation(String),
}

/// High-level outcome for a single governed decision, as derived by
/// governance evaluation logic described in the Birth-Sign model.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// Summarizes constraint evaluation for a decision: which ALN norms
/// were consulted, which were violated, and whether FPIC gates are open.[file:2]
#[derive(Debug, Clone)]
pub struct GovernanceEvaluationSummary {
    pub birthsign_ids: Vec<BirthSignId>,
    pub consulted_norms: Vec<AlnNormId>,
    pub hard_violations: Vec<AlnNormId>,
    pub soft_violations: Vec<AlnNormId>,
    pub has_fpic_block: bool,
}

/// Captures augmented-citizen sovereignty footprint for a decision:
/// who is the subject, who operated the workflow, and which consent
/// channels were used.[file:2][file:5]
#[derive(Debug, Clone)]
pub struct SovereigntyFootprint {
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub consent_channel_ids: Vec<String>,
    pub grievance_channel_ids: Vec<String>,
    pub crypto_somatic_enforced: bool,
}

/// Local representation of a governed decision envelope that is
/// about to be written to Googolswarm. This must match the canonical
/// ALN-governed decision schema used across workflows.[file:2]
#[derive(Debug, Clone)]
pub struct GovernedDecisionEnvelopeLocal {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_id: WorkflowId,
    pub workflow_stage: String,
    pub domains: Vec<GovernanceDomain>,
    pub birthsign_ids: Vec<BirthSignId>,
    pub applied_aln_norms: Vec<AlnNormId>,
    pub subject_did: Option<Did>,
    pub operator_did: Option<Did>,
    pub node_profile_id: Option<NodeProfileId>,
    pub inputs_hash: String,
    pub outputs_hash: String,
    pub evaluation_outcome: DecisionOutcome,
    pub evaluation_summary: GovernanceEvaluationSummary,
    pub sovereignty: SovereigntyFootprint,
    pub tags: HashMap<String, String>,
}

/// Encodes a single attestation result: pass, fail, or quarantined.
/// "Quarantined" is used when the record is structurally valid but
/// triggers sovereignty or anti-takeover alarms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestationResult {
    Passed,
    FailedStructure(String),
    FailedSovereignty(String),
    FailedAuthorityScope(String),
    Quarantined(String),
}

/// Local attestation record that can be stored on Googolswarm as a
/// proof-of-attestation, but is fully intelligible and enforceable
/// without any remote dependency.
#[derive(Debug, Clone)]
pub struct AttestationRecord {
    pub attestation_id: String,
    pub issued_at: SystemTime,
    pub attestor_node: AttestorNodeId,
    pub namespace: GoogolswarmNamespaceId,
    pub connectivity_mode: AttestationConnectivityMode,
    pub authority_scope: GoogolswarmAuthorityScope,
    pub decision_tx: TrustTxId,
    pub workflow_id: WorkflowId,
    pub domains: Vec<GovernanceDomain>,
    pub birthsign_ids: Vec<BirthSignId>,
    pub consulted_norms: Vec<AlnNormId>,
    pub decision_outcome: DecisionOutcome,
    pub sovereignty_footprint: SovereigntyFootprint,
    pub result: AttestationResult,
    pub summary_message: String,
    pub meta: HashMap<String, String>,
}

/// Configuration describing how this host is allowed to talk to
/// Googolswarm and what safety invariants must hold.
#[derive(Debug, Clone)]
pub struct LocalAttestorConfig {
    pub attestor_node: AttestorNodeId,
    pub namespace: GoogolswarmNamespaceId,
    pub connectivity_mode: AttestationConnectivityMode,
    pub authority_scope: GoogolswarmAuthorityScope,
    pub allowed_domains: HashSet<GovernanceDomain>,
    pub max_birthsign_span: usize,
    pub require_crypto_somatic_for_biosignals: bool,
    pub require_subject_for_biosignals: bool,
    pub allow_derating_only_when_soft_violations: bool,
    pub quarantine_on_unknown_norms: bool,
    pub min_multisig_attestors: usize,
}

/// Captures local multi-sig attestation evidence.
/// This is kept local; Googolswarm only sees finalized summaries.
#[derive(Debug, Clone)]
pub struct LocalMultisigEvidence {
    pub attestor_ids: Vec<AttestorNodeId>,
    pub signatures_present: usize,
    pub signatures_required: usize,
}

/// Attestation engine object, parameterized by a local configuration.
#[derive(Debug, Clone)]
pub struct GoogolswarmAttestorLocal {
    pub config: LocalAttestorConfig,
}

impl GoogolswarmAttestorLocal {
    /// Construct a new attestor with LOCAL_ONLY, ATTESTOR_ONLY defaults.
    pub fn new_default(attestor_node: AttestorNodeId, namespace: GoogolswarmNamespaceId) -> Self {
        let mut allowed_domains = HashSet::new();
        allowed_domains.insert(GovernanceDomain::Water);
        allowed_domains.insert(GovernanceDomain::Energy);
        allowed_domains.insert(GovernanceDomain::Materials);
        allowed_domains.insert(GovernanceDomain::Mobility);
        allowed_domains.insert(GovernanceDomain::Emergency);

        Self {
            config: LocalAttestorConfig {
                attestor_node,
                namespace,
                connectivity_mode: AttestationConnectivityMode::LocalOnly,
                authority_scope: GoogolswarmAuthorityScope::AttestorOnly,
                allowed_domains,
                max_birthsign_span: 16,
                require_crypto_somatic_for_biosignals: true,
                require_subject_for_biosignals: true,
                allow_derating_only_when_soft_violations: true,
                quarantine_on_unknown_norms: true,
                min_multisig_attestors: 2,
            },
        }
    }

    /// Verifies structure, sovereignty, and scope for a decision envelope,
    /// plus multi-sig evidence, without contacting any remote node.
    /// Does NOT write to Googolswarm; it only creates an AttestationRecord.
    pub fn attest_local(
        &self,
        decision: &GovernedDecisionEnvelopeLocal,
        multisig: &LocalMultisigEvidence,
    ) -> AttestationRecord {
        let mut meta = HashMap::new();
        meta.insert("version".to_string(), "ALE-TRUST-GOOGOLSWARM-ATTESTOR-LOCAL-001".to_string());

        // 1. Enforce authority scope: Googolswarm must stay AttestorOnly.
        let authority_scope = match &self.config.authority_scope {
            GoogolswarmAuthorityScope::AttestorOnly => GoogolswarmAuthorityScope::AttestorOnly,
            GoogolswarmAuthorityScope::RejectedEscalation(reason) => {
                return self.record_scope_failure(decision, reason.clone(), meta);
            }
        };

        // 2. Basic structural checks: domains, birthsign span, hashes non-empty.
        if decision.domains.is_empty() {
            return self.record_failure(
                decision,
                AttestationResult::FailedStructure("missing domains".into()),
                "Decision must include at least one governance domain.",
                meta,
                authority_scope,
            );
        }
        if !decision
            .domains
            .iter()
            .all(|d| self.config.allowed_domains.contains(d))
        {
            return self.record_failure(
                decision,
                AttestationResult::FailedStructure("domain outside attestor scope".into()),
                "Decision references domains outside this attestor's configured scope.",
                meta,
                authority_scope,
            );
        }
        if decision.birthsign_ids.is_empty() {
            return self.record_failure(
                decision,
                AttestationResult::FailedStructure("missing BirthSignIds".into()),
                "Decision must carry at least one BirthSignId.",
                meta,
                authority_scope,
            );
        }
        if decision.birthsign_ids.len() > self.config.max_birthsign_span {
            return self.record_failure(
                decision,
                AttestationResult::FailedStructure("too many BirthSignIds".into()),
                "Decision spans more BirthSign tiles than allowed for a single attestation.",
                meta,
                authority_scope,
            );
        }
        if decision.inputs_hash.is_empty() || decision.outputs_hash.is_empty() {
            return self.record_failure(
                decision,
                AttestationResult::FailedStructure("missing input/output hashes".into()),
                "Decision must include non-empty hashes for inputs and outputs.",
                meta,
                authority_scope,
            );
        }

        // 3. Sovereignty enforcement for biosignal/augmentation domains.
        let touches_biosignals = decision
            .domains
            .iter()
            .any(|d| matches!(d, GovernanceDomain::Biosignals | GovernanceDomain::Augmentation));

        if touches_biosignals {
            if self.config.require_subject_for_biosignals && decision.sovereignty.subject_did.is_none()
            {
                return self.record_failure(
                    decision,
                    AttestationResult::FailedSovereignty("missing subject DID for biosignals".into()),
                    "Biosignal / augmentation decisions must include subject DID.",
                    meta,
                    authority_scope,
                );
            }
            if self.config.require_crypto_somatic_for_biosignals
                && !decision.sovereignty.crypto_somatic_enforced
            {
                return self.record_failure(
                    decision,
                    AttestationResult::FailedSovereignty(
                        "CryptoSomatic Shield not enforced for biosignals".into(),
                    ),
                    "Biosignal / augmentation decisions must enforce CryptoSomatic Shield.",
                    meta,
                    authority_scope,
                );
            }
        }

        // 4. Governance evaluation invariants: no hidden escalation.
        // - Hard violations must never be attested as Approved.
        // - Derating is only allowed when there are purely soft violations.
        let hard_violation_count = decision.evaluation_summary.hard_violations.len();
        let soft_violation_count = decision.evaluation_summary.soft_violations.len();
        let has_unknown_norms = decision.evaluation_summary.consulted_norms.is_empty()
            && (hard_violation_count > 0 || soft_violation_count > 0);

        if has_unknown_norms && self.config.quarantine_on_unknown_norms {
            return self.record_failure(
                decision,
                AttestationResult::Quarantined("violations without explicit ALN norm ids".into()),
                "Decision has constraint violations without explicit ALN norm IDs; quarantined.",
                meta,
                authority_scope,
            );
        }

        match decision.evaluation_outcome {
            DecisionOutcome::Approved | DecisionOutcome::ApprovedWithDerating => {
                if hard_violation_count > 0 {
                    return self.record_failure(
                        decision,
                        AttestationResult::FailedSovereignty(
                            "approved despite hard violations".into(),
                        ),
                        "Decision cannot be Approved while hard violations are present.",
                        meta,
                        authority_scope,
                    );
                }
                if let DecisionOutcome::ApprovedWithDerating = decision.evaluation_outcome {
                    if self.config.allow_derating_only_when_soft_violations
                        && soft_violation_count == 0
                    {
                        return self.record_failure(
                            decision,
                            AttestationResult::FailedSovereignty(
                                "derating without soft violations".into(),
                            ),
                            "ApprovedWithDerating requires at least one soft violation.",
                            meta,
                            authority_scope,
                        );
                    }
                }
            }
            DecisionOutcome::Rejected | DecisionOutcome::PendingFpic => {
                // Rejected and PendingFpic are always structurally allowed.
            }
        }

        // 5. FPIC: PendingFpic decisions must never be silently treated as Approved.
        if decision.evaluation_summary.has_fpic_block {
            match decision.evaluation_outcome {
                DecisionOutcome::PendingFpic => {
                    // OK: explicit pending state.
                }
                _ => {
                    return self.record_failure(
                        decision,
                        AttestationResult::FailedSovereignty("FPIC blocked but not PendingFpic".into()),
                        "FPIC-blocked decisions must be PendingFpic.",
                        meta,
                        authority_scope,
                    );
                }
            }
        }

        // 6. Multi-sig attestation: ensure sufficient local attestations.
        if multisig.signatures_present < self.config.min_multisig_attestors
            || multisig.signatures_present < multisig.signatures_required
        {
            return self.record_failure(
                decision,
                AttestationResult::FailedAuthorityScope("insufficient local multisig attestors".into()),
                "Not enough local attestors signed this decision.",
                meta,
                authority_scope,
            );
        }

        // 7. All invariants passed: emit a local attestation record.
        self.record_success(decision, authority_scope, meta)
    }

    fn record_scope_failure(
        &self,
        decision: &GovernedDecisionEnvelopeLocal,
        reason: String,
        mut meta: HashMap<String, String>,
    ) -> AttestationRecord {
        meta.insert("scope_failure".into(), reason.clone());
        AttestationRecord {
            attestation_id: self.make_attestation_id(decision),
            issued_at: SystemTime::now(),
            attestor_node: self.config.attestor_node.clone(),
            namespace: self.config.namespace.clone(),
            connectivity_mode: self.config.connectivity_mode.clone(),
            authority_scope: GoogolswarmAuthorityScope::RejectedEscalation(reason.clone()),
            decision_tx: decision.tx_id.clone(),
            workflow_id: decision.workflow_id.clone(),
            domains: decision.domains.clone(),
            birthsign_ids: decision.birthsign_ids.clone(),
            consulted_norms: decision.evaluation_summary.consulted_norms.clone(),
            decision_outcome: decision.evaluation_outcome.clone(),
            sovereignty_footprint: decision.sovereignty.clone(),
            result: AttestationResult::FailedAuthorityScope(reason.clone()),
            summary_message: "Googolswarm authority scope escalation rejected by local policy."
                .to_string(),
            meta,
        }
    }

    fn record_failure(
        &self,
        decision: &GovernedDecisionEnvelopeLocal,
        result: AttestationResult,
        message: &str,
        mut meta: HashMap<String, String>,
        authority_scope: GoogolswarmAuthorityScope,
    ) -> AttestationRecord {
        meta.insert("failure_reason".into(), format!("{:?}", result));
        AttestationRecord {
            attestation_id: self.make_attestation_id(decision),
            issued_at: SystemTime::now(),
            attestor_node: self.config.attestor_node.clone(),
            namespace: self.config.namespace.clone(),
            connectivity_mode: self.config.connectivity_mode.clone(),
            authority_scope,
            decision_tx: decision.tx_id.clone(),
            workflow_id: decision.workflow_id.clone(),
            domains: decision.domains.clone(),
            birthsign_ids: decision.birthsign_ids.clone(),
            consulted_norms: decision.evaluation_summary.consulted_norms.clone(),
            decision_outcome: decision.evaluation_outcome.clone(),
            sovereignty_footprint: decision.sovereignty.clone(),
            result,
            summary_message: message.to_string(),
            meta,
        }
    }

    fn record_success(
        &self,
        decision: &GovernedDecisionEnvelopeLocal,
        authority_scope: GoogolswarmAuthorityScope,
        mut meta: HashMap<String, String>,
    ) -> AttestationRecord {
        meta.insert("status".into(), "passed".into());
        AttestationRecord {
            attestation_id: self.make_attestation_id(decision),
            issued_at: SystemTime::now(),
            attestor_node: self.config.attestor_node.clone(),
            namespace: self.config.namespace.clone(),
            connectivity_mode: self.config.connectivity_mode.clone(),
            authority_scope,
            decision_tx: decision.tx_id.clone(),
            workflow_id: decision.workflow_id.clone(),
            domains: decision.domains.clone(),
            birthsign_ids: decision.birthsign_ids.clone(),
            consulted_norms: decision.evaluation_summary.consulted_norms.clone(),
            decision_outcome: decision.evaluation_outcome.clone(),
            sovereignty_footprint: decision.sovereignty.clone(),
            result: AttestationResult::Passed,
            summary_message: "Local Googolswarm attestation passed; record may be appended by trust layer."
                .to_string(),
            meta,
        }
    }

    fn make_attestation_id(&self, decision: &GovernedDecisionEnvelopeLocal) -> String {
        // Simple, deterministic ID composition using stable fields.
        // No cryptographic hash: actual hashing is delegated to lower layers
        // that comply with your allowed primitives.
        format!(
            "ATT-{}-{}-{}",
            self.config.attestor_node.0,
            decision.workflow_id.0,
            decision.tx_id.0
        )
    }
}

/// Convenience helpers for tests and higher-level glue.
impl LocalMultisigEvidence {
    pub fn new(attestor_ids: Vec<AttestorNodeId>, signatures_required: usize) -> Self {
        let present = attestor_ids.len();
        Self {
            attestor_ids,
            signatures_present: present,
            signatures_required,
        }
    }
}

impl GovernanceEvaluationSummary {
    pub fn satisfied(birthsign_ids: Vec<BirthSignId>, norms: Vec<AlnNormId>) -> Self {
        Self {
            birthsign_ids,
            consulted_norms: norms,
            hard_violations: Vec::new(),
            soft_violations: Vec::new(),
            has_fpic_block: false,
        }
    }
}

impl SovereigntyFootprint {
    pub fn empty() -> Self {
        Self {
            subject_did: None,
            operator_did: None,
            consent_channel_ids: Vec::new(),
            grievance_channel_ids: Vec::new(),
            crypto_somatic_enforced: false,
        }
    }
}

/// Example unit-test-like check (can be wired into your real test harness).
#[cfg(test)]
mod tests {
    use super::*;

    fn demo_decision(outcome: DecisionOutcome) -> GovernedDecisionEnvelopeLocal {
        let bs = vec![BirthSignId("TILE-001".into())];
        GovernedDecisionEnvelopeLocal {
            tx_id: TrustTxId("TX-001".into()),
            created_at: SystemTime::now(),
            workflow_id: WorkflowId("WF-WATER-ALLOCATION".into()),
            workflow_stage: "Optimization".into(),
            domains: vec![GovernanceDomain::Water],
            birthsign_ids: bs.clone(),
            applied_aln_norms: vec![AlnNormId("ALN-WATER-RIGHTS-001".into())],
            subject_did: None,
            operator_did: Some(Did("did:aletheion:operator:demo".into())),
            node_profile_id: Some(NodeProfileId("NODE-PROFILE-LOCAL".into())),
            inputs_hash: "inputs-demo".into(),
            outputs_hash: "outputs-demo".into(),
            evaluation_outcome: outcome,
            evaluation_summary: GovernanceEvaluationSummary::satisfied(
                bs,
                vec![AlnNormId("ALN-WATER-RIGHTS-001".into())],
            ),
            sovereignty: SovereigntyFootprint::empty(),
            tags: HashMap::new(),
        }
    }

    #[test]
    fn local_attestation_passes_basic_water_decision() {
        let attestor = GoogolswarmAttestorLocal::new_default(
            AttestorNodeId("NODE-ATTESTOR-001".into()),
            GoogolswarmNamespaceId("NS-PHOENIX-GOVERNANCE".into()),
        );
        let decision = demo_decision(DecisionOutcome::Approved);
        let multisig = LocalMultisigEvidence::new(
            vec![
                AttestorNodeId("NODE-ATTESTOR-001".into()),
                AttestorNodeId("NODE-ATTESTOR-002".into()),
            ],
            2,
        );
        let record = attestor.attest_local(&decision, &multisig);
        assert_eq!(record.result, AttestationResult::Passed);
        assert_eq!(record.decision_tx.0, "TX-001");
    }
}
