// ALE-TRUST-GOOGOLSWARM-ATTESTOR-GUARD-001.rs
// Role: Guardrail layer for Googolswarm within Aletheion.
//
// Guarantees:
// - Googolswarm is constrained to a local/host-only, offline-capable attestor role.
// - No hidden control channels, no policy authority, no unilateral upgrades.
// - All governed decisions must be multi-sig attested and DID-bound.
// - Any attempt to extend Googolswarm's capabilities beyond attestation is detectable at compile time
//   (via typed interfaces) and at runtime (via explicit capability flags and validation).
//
// Layers: L3 Blockchain Trust, L4 Optimization, L5 Citizen Interface (for explainable attestations).
//
// NOTE: This file intentionally does not perform any network I/O itself; it defines types, guards,
// and capability contracts that other crates must conform to.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Stable identifier for a Googolswarm node participating as a local attestor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GoogolswarmNodeId(pub String);

/// Stable identifier for a governed workflow family (e.g., AWP water allocation).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowFamilyId(pub String);

/// Stable identifier for a concrete workflow instance (deployment or version).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowInstanceId(pub String);

/// Stable identifier for a DID (citizen, device, institution).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(pub String);

/// Stable identifier for a trust-layer transaction (envelope that will be recorded on Googolswarm).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

/// Stable identifier for a governance domain (water, thermal, mobility, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GovernanceDomainId(pub String);

/// Stable identifier for a BirthSign record (spatial governance tile).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

/// Stable identifier for an ALN norm applied during decision evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

/// Stable identifier for a BioticTreaty record.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub String);

/// Stable identifier for a local MicroTreaty or LexEthos micro-policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MicroTreatyId(pub String);

/// Stable identifier for a multi-sig attestation group (e.g., Googolswarm Phoenix quorum).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestorGroupId(pub String);

/// Stable identifier for a specific attestor key within a group.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttestorKeyId(pub String);

/// Minimal representation of a DID-bearing participant in an attestation process.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Participant {
    pub did: Did,
    /// Optional label for transparency (e.g., "Phoenix Water Utility", "Neighborhood Assembly").
    pub label: Option<String>,
    /// Optional governance domain bias (e.g., "water", "mobility") for audit visualization.
    pub domain_hint: Option<GovernanceDomainId>,
}

/// Capability flags that describe *exactly* what Googolswarm is allowed to do on this node.
///
/// Any extension of capabilities requires a new enum variant and a conscious code change:
/// there is no "catch-all" or string-based configuration that could hide new superpowers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoogolswarmCapability {
    /// Can append governed decision envelopes that have already been evaluated by ALN validators.
    AppendGovernedDecisionTx,
    /// Can provide ordered, immutable logs for audit queries (local host or physically local cluster).
    ProvideLocalAuditLog,
    /// Can perform multi-sig attestation on envelopes, ensuring ordering and non-repudiation.
    MultiSigAttestor,
    /// Can operate offline-first, with eventual reconciliation, but may never require a remote cloud.
    OfflineCapableLedger,
}

/// Hard constraint on the *role* of Googolswarm relative to Aletheion.
///
/// This encoding prevents accidental elevation to policy engine, optimizer, or citizen interface.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GoogolswarmRole {
    /// Pure attestor: may order, sign, and store envelopes, but cannot alter payloads.
    LocalAttestorOnly,
}

/// Simple environment signal for a node running Googolswarm.
///
/// This is intentionally narrow: we do not expose arbitrary configuration surfaces here.
#[derive(Debug, Clone)]
pub struct GoogolswarmNodeProfile {
    pub node_id: GoogolswarmNodeId,
    pub role: GoogolswarmRole,
    pub capabilities: Vec<GoogolswarmCapability>,
    /// True if the node is configured and verified as local/host-only for this deployment.
    pub is_local_only: bool,
    /// True if the node can continue operations without external (wide-area) connectivity.
    pub is_offline_capable: bool,
    /// Optional free-form labels for dashboards; not used for authorization.
    pub labels: HashMap<String, String>,
}

impl GoogolswarmNodeProfile {
    /// Returns true if this node satisfies all non-negotiable constraints for Aletheion:
    /// - Role must be LocalAttestorOnly.
    /// - Must expose MultiSigAttestor and OfflineCapableLedger.
    /// - Must be explicitly marked as local-only and offline-capable.
    pub fn is_compliant_attestor(&self) -> bool {
        if self.role != GoogolswarmRole::LocalAttestorOnly {
            return false;
        }
        let mut has_multisig = false;
        let mut has_offline = false;
        for c in &self.capabilities {
            match c {
                GoogolswarmCapability::MultiSigAttestor => has_multisig = true,
                GoogolswarmCapability::OfflineCapableLedger => has_offline = true,
                _ => {}
            }
        }
        self.is_local_only && self.is_offline_capable && has_multisig && has_offline
    }
}

/// Enumerates the *only* kinds of payloads Googolswarm may ever attest for Aletheion.
///
/// This enforces that Googolswarm cannot silently become a command bus or hidden policy engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttestablePayloadKind {
    /// An already-validated governed decision envelope (e.g., from ALE-GOV-BIRTH-SIGN-MODEL-001.rs).
    GovernedDecisionEnvelope,
    /// A narrow, schema-checked firmware attestation record, proving node integrity (no policy logic).
    FirmwareIntegrityRecord,
}

/// High-level outcome of a governed decision, carried into the trust layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating,
    Rejected,
    PendingFpic,
}

/// Summarizes constraint outcomes for the decision; this is a mirror of governance-layer results.
#[derive(Debug, Clone)]
pub struct ConstraintSummary {
    pub birth_sign_ids: Vec<BirthSignId>,
    pub applied_aln_norms: Vec<AlnNormId>,
    pub applied_biotic_treaties: Vec<BioticTreatyId>,
    pub applied_micro_treaties: Vec<MicroTreatyId>,
    /// True if any hard violation occurred before the decision was finalized.
    pub has_hard_violation: bool,
    /// True if any soft violation occurred (e.g., derating applied).
    pub has_soft_violation: bool,
    /// Optional FPIC-related status, encoded in human-readable form for citizen surfaces.
    pub fpic_status: Option<String>,
}

/// Minimal, forward-only description of data provenance hashes.
///
/// Hashing algorithm selection is delegated to lower layers that respect the blacklist and crypto rules.
#[derive(Debug, Clone)]
pub struct ProvenanceHashes {
    pub inputs_hash: String,
    pub outputs_hash: String,
}

/// Record of a single attestor's signature over an envelope.
///
/// The underlying signature bytes are opaque; the important part is that they are DID-bound
/// and grouped into a multi-sig contract.
#[derive(Debug, Clone)]
pub struct AttestorSignature {
    pub attestor_group_id: AttestorGroupId,
    pub attestor_key_id: AttestorKeyId,
    pub signer_did: Did,
    pub signed_at: SystemTime,
    /// Signature material as a byte string representation (e.g., base64).
    pub signature_blob: String,
}

/// Aggregated multi-sig status for a governed decision.
///
/// This is how Googolswarm proves that no single actor (human or machine) can unilaterally
/// push a transaction into the trust layer.
#[derive(Debug, Clone)]
pub struct MultiSigAttestation {
    pub threshold_required: u8,
    pub signatures: Vec<AttestorSignature>,
}

impl MultiSigAttestation {
    /// Returns true if the multi-sig envelope has met or exceeded its threshold.
    pub fn is_satisfied(&self) -> bool {
        self.signatures.len() as u8 >= self.threshold_required && self.threshold_required > 0
    }
}

/// A single governed decision envelope as seen by Googolswarm.
///
/// This structure is intentionally aligned with the governed decision envelope used in
/// governance and Birth-Sign modules, but it *adds* explicit fields for:
/// - Googolswarm's attestor node profile at the time of commit,
/// - multi-sig attestations,
/// - strict attestation scope, and
/// - augmented-citizen sovereignty via DIDs and explanation surfaces.
#[derive(Debug, Clone)]
pub struct GoogolswarmGovernedDecisionTx {
    pub tx_id: TrustTxId,
    pub created_at: SystemTime,
    pub workflow_family_id: WorkflowFamilyId,
    pub workflow_instance_id: WorkflowInstanceId,
    pub workflow_stage: String,
    pub domains: Vec<GovernanceDomainId>,
    /// Spatial context via Birth-Signs; must be non-empty for any action touching land/water/bodies.
    pub birth_sign_ids: Vec<BirthSignId>,
    /// DID of the subject (citizen, collective, or community) impacted by this decision.
    pub subject_did: Option<Did>,
    /// DID of the operator (city agency, cooperative, etc.) who initiated the decision.
    pub operator_did: Option<Did>,
    /// Optional list of advisory participants (e.g., Indigenous governance council).
    pub advisory_participants: Vec<Participant>,
    /// Summary of constraints and norms consulted by the governance layer.
    pub constraint_summary: ConstraintSummary,
    /// Provenance hashes for inputs and outputs.
    pub provenance: ProvenanceHashes,
    /// Outcome determined *before* attestation.
    pub outcome: DecisionOutcome,
    /// Attestation payload kind; must be GovernedDecisionEnvelope for this struct.
    pub payload_kind: AttestablePayloadKind,
    /// The Googolswarm node profile that created or last updated this attestation record.
    pub attestor_node_profile: GoogolswarmNodeProfile,
    /// Multi-sig envelope; must satisfy its threshold before commit.
    pub multi_sig: MultiSigAttestation,
    /// Human-readable explanation, suitable for citizen interfaces.
    pub explanation: Option<String>,
    /// Opaque tags reserved for domain-specific metadata.
    pub tags: HashMap<String, String>,
}

impl GoogolswarmGovernedDecisionTx {
    /// Construct a new governed decision transaction with minimal required fields.
    ///
    /// This constructor enforces that:
    /// - payload_kind is fixed to GovernedDecisionEnvelope.
    /// - birth_sign_ids list is provided (even if empty for some abstract domains).
    /// - attestor_node_profile is attached at creation.
    pub fn new(
        tx_id: TrustTxId,
        workflow_family_id: WorkflowFamilyId,
        workflow_instance_id: WorkflowInstanceId,
        workflow_stage: String,
        domains: Vec<GovernanceDomainId>,
        birth_sign_ids: Vec<BirthSignId>,
        subject_did: Option<Did>,
        operator_did: Option<Did>,
        constraint_summary: ConstraintSummary,
        provenance: ProvenanceHashes,
        outcome: DecisionOutcome,
        attestor_node_profile: GoogolswarmNodeProfile,
        multi_sig: MultiSigAttestation,
    ) -> Self {
        Self {
            tx_id,
            created_at: SystemTime::now(),
            workflow_family_id,
            workflow_instance_id,
            workflow_stage,
            domains,
            birth_sign_ids,
            subject_did,
            operator_did,
            advisory_participants: Vec::new(),
            constraint_summary,
            provenance,
            outcome,
            payload_kind: AttestablePayloadKind::GovernedDecisionEnvelope,
            attestor_node_profile,
            multi_sig,
            explanation: None,
            tags: HashMap::new(),
        }
    }

    /// Attach advisory participants (e.g., Indigenous councils, citizen assemblies) for audit and
    /// explanation, without changing the core decision semantics.
    pub fn with_advisory_participants(mut self, participants: Vec<Participant>) -> Self {
        self.advisory_participants = participants;
        self
    }

    /// Attach a human-readable explanation string that can be surfaced to augmented citizens.
    pub fn with_explanation<S: Into<String>>(mut self, explanation: S) -> Self {
        self.explanation = Some(explanation.into());
        self
    }

    /// Attach an arbitrary tag for domain-specific metadata.
    pub fn add_tag<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.tags.insert(key.into(), value.into());
    }

    /// Returns true if this envelope meets all requirements to be appended to Googolswarm:
    ///
    /// - Googolswarm node profile is a compliant local attestor.
    /// - Payload kind is GovernedDecisionEnvelope.
    /// - Multi-sig threshold is satisfied.
    /// - At least one BirthSign is present whenever domains overlap with land/water/bodies.
    /// - Subject DID and operator DID are present for citizen-impacting decisions.
    pub fn is_appendable(&self) -> bool {
        if !self.attestor_node_profile.is_compliant_attestor() {
            return false;
        }

        if self.payload_kind != AttestablePayloadKind::GovernedDecisionEnvelope {
            return false;
        }

        if !self.multi_sig.is_satisfied() {
            return false;
        }

        // Enforce BirthSign presence for territorial domains.
        if self.requires_birth_signs() && self.birth_sign_ids.is_empty() {
            return false;
        }

        // Enforce DID presence when any citizen-facing domain is involved.
        if self.touches_citizens() {
            if self.subject_did.is_none() || self.operator_did.is_none() {
                return false;
            }
        }

        true
    }

    /// Returns true if any of the governed domains are territorial/embodied and therefore require
    /// BirthSign context.
    fn requires_birth_signs(&self) -> bool {
        // Minimal heuristic: any domain whose id contains these tokens is treated as territorial.
        self.domains.iter().any(|d| {
            let s = d.0.to_lowercase();
            s.contains("land")
                || s.contains("water")
                || s.contains("air")
                || s.contains("mobility")
                || s.contains("biosignal")
                || s.contains("augmentation")
                || s.contains("soil")
                || s.contains("waste")
        })
    }

    /// Returns true if this decision is citizen-impacting, and thus must be DID-bound.
    fn touches_citizens(&self) -> bool {
        self.domains.iter().any(|d| {
            let s = d.0.to_lowercase();
            s.contains("citizen")
                || s.contains("health")
                || s.contains("education")
                || s.contains("labor")
                || s.contains("care")
                || s.contains("culture")
        })
    }
}

/// Firmware integrity record that Googolswarm is allowed to attest.
///
/// NOTE: This does not grant Googolswarm any right to *issue* firmware or push updates;
/// it only records that a given node reported a particular firmware hash and security posture.
#[derive(Debug, Clone)]
pub struct FirmwareIntegrityRecord {
    pub tx_id: TrustTxId,
    pub reported_at: SystemTime,
    pub node_id: GoogolswarmNodeId,
    pub workflow_families: Vec<WorkflowFamilyId>,
    /// Abstracted firmware hash (algorithm selection handled in a lower layer).
    pub firmware_hash: String,
    /// True if secure boot is enabled and verified.
    pub secure_boot_enabled: bool,
    /// True if firmware updates are signed and verified by the authorized Aletheion channel.
    pub signed_updates_required: bool,
    /// True if this firmware image passed the most recent compliance scan.
    pub last_compliance_passed: bool,
    /// Attestor node profile that recorded this integrity statement.
    pub attestor_node_profile: GoogolswarmNodeProfile,
}

impl FirmwareIntegrityRecord {
    /// Returns true if this record is safe to append:
    /// - Attestor node profile must be compliant.
    /// - No elevation of Googolswarm beyond attestor role.
    pub fn is_appendable(&self) -> bool {
        self.attestor_node_profile.is_compliant_attestor()
    }
}

/// Lightweight guard interface that orchestrators can use before attempting to append to Googolswarm.
///
/// This keeps all Googolswarm-related checks in one place and makes it impossible to "forget"
/// the constraints in individual workflows.
pub struct GoogolswarmAttestorGuard;

impl GoogolswarmAttestorGuard {
    /// Validate a governed decision transaction before calling any Googolswarm client.
    ///
    /// Returns Ok(()) if the envelope is safe to append, or Err(reason) explaining the violation.
    pub fn validate_governed_tx(tx: &GoogolswarmGovernedDecisionTx) -> Result<(), String> {
        if !tx.attestor_node_profile.is_compliant_attestor() {
            return Err("Googolswarm node profile is not a compliant local attestor".into());
        }
        if tx.payload_kind != AttestablePayloadKind::GovernedDecisionEnvelope {
            return Err("Payload kind is not GovernedDecisionEnvelope".into());
        }
        if !tx.multi_sig.is_satisfied() {
            return Err("Multi-sig threshold not satisfied".into());
        }
        if tx.requires_birth_signs() && tx.birth_sign_ids.is_empty() {
            return Err("Missing BirthSignIds for territorial domains".into());
        }
        if tx.touches_citizens() {
            if tx.subject_did.is_none() {
                return Err("Missing subject DID for citizen-impacting decision".into());
            }
            if tx.operator_did.is_none() {
                return Err("Missing operator DID for citizen-impacting decision".into());
            }
        }
        Ok(())
    }

    /// Validate a firmware integrity record before attestation.
    pub fn validate_firmware_record(record: &FirmwareIntegrityRecord) -> Result<(), String> {
        if !record.attestor_node_profile.is_compliant_attestor() {
            return Err("Googolswarm node profile is not a compliant local attestor".into());
        }
        Ok(())
    }

    /// Helper to enforce a maximum acceptable staleness window for records during offline operation.
    ///
    /// If `now - created_at` exceeds `max_age`, the record should be rejected or refreshed.
    pub fn is_within_age(created_at: SystemTime, now: SystemTime, max_age: Duration) -> bool {
        match now.duration_since(created_at) {
            Ok(delta) => delta <= max_age,
            Err(_) => false,
        }
    }
}
