// Purpose: Birth-Sign core model defining jurisdictional governance bundles for spatial tiles.
// Integration: Consumed by Edge/State-Model layers (L1/L2), Compliance Preflight (L4), Trust Layer (L3).
// Domains: land, water, air, biosignals, citizens governance context binding.

use std::collections::{HashMap, HashSet};
use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

//──────────────────────────────────────────────────────────────────────────────
// Core Identifiers
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BirthSignId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlnNormId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BioticTreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MicroTreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FpicRequirementId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsentId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollectiveId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndigenousTerritoryId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProvenanceRef(pub String);

//──────────────────────────────────────────────────────────────────────────────
// Governance Domain Enumeration
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceDomain {
    Land,
    Water,
    Air,
    Biosignals,
    Citizens,
    Thermal,
    Light,
    Noise,
    Chemical,
    Habitat,
    Mobility,
    Energy,
    Augmentation,
    DataUse,
}

impl GovernanceDomain {
    pub fn requires_birth_sign(&self) -> bool {
        matches!(
            self,
            GovernanceDomain::Land
                | GovernanceDomain::Water
                | GovernanceDomain::Air
                | GovernanceDomain::Biosignals
                | GovernanceDomain::Citizens
        )
    }
}

//──────────────────────────────────────────────────────────────────────────────
// FPIC (Free, Prior, and Informed Consent) Structures
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpicRequirementLevel {
    NotApplicable,
    RequiredBeforePlanning,
    RequiredBeforeExecution,
    EmergencyOverrideWithAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpicProtocol {
    pub protocol_name: String,
    pub stages: Vec<String>,
    pub minimum_duration: Duration,
    pub documentation_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpicRequirement {
    pub requirement_id: FpicRequirementId,
    pub territory: IndigenousTerritoryId,
    pub nation: String,
    pub traditional_name: String,
    pub requirement_level: FpicRequirementLevel,
    pub protocols: Vec<FpicProtocol>,
    pub contact_authority: ContactAuthority,
    pub provenance: ProvenanceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactAuthority {
    Did(Did),
    Collective(CollectiveId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpicGrant {
    pub grant_id: String,
    pub requirement_ref: FpicRequirementId,
    pub action_description: String,
    pub granted_by: ContactAuthority,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Vec<String>,
    pub audit_trail: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpicPendingRecord {
    pub territory: IndigenousTerritoryId,
    pub requested_at: DateTime<Utc>,
    pub expected_response_by: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpicDenialRecord {
    pub territory: IndigenousTerritoryId,
    pub denied_at: DateTime<Utc>,
    pub reason: String,
    pub appeal_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FpicStatus {
    NotRequired,
    Granted { grant: FpicGrant },
    Pending { record: FpicPendingRecord },
    Denied { record: FpicDenialRecord },
}

impl FpicStatus {
    pub fn is_granted(&self) -> bool {
        matches!(self, FpicStatus::Granted { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, FpicStatus::Pending { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, FpicStatus::Denied { .. })
    }

    pub fn blocks_actuation(&self) -> bool {
        !matches!(self, FpicStatus::NotRequired | FpicStatus::Granted { .. })
    }
}

//──────────────────────────────────────────────────────────────────────────────
// Rights Atoms, Biotic Treaties, Micro Treaties
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modality {
    Obligation,
    Permission,
    Prohibition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementMode {
    Hard,
    HighPenalty,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsAtom {
    pub atom_id: AlnNormId,
    pub holder: Holder,
    pub object: String,
    pub modality: Modality,
    pub conditions: Vec<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub enforcement_mode: EnforcementMode,
    pub provenance: ProvenanceRef,
    pub human_readable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Holder {
    Did(Did),
    Collective(CollectiveId),
    SpeciesAgent(String),
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioticConstraint {
    pub metric: String,
    pub threshold: f64,
    pub operator: ComparisonOperator,
    pub spatial_scope: Vec<TileId>,
    pub temporal_scope: TemporalScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalScope {
    Continuous,
    TimeWindow { start_time: String, end_time: String, months: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioticTreaty {
    pub treaty_id: BioticTreatyId,
    pub species_agents: Vec<String>,
    pub domain: GovernanceDomain,
    pub constraints: Vec<BioticConstraint>,
    pub habitat_corridors: Vec<String>,
    pub enforcement_mode: EnforcementMode,
    pub effective_date: DateTime<Utc>,
    pub review_cycle: Duration,
    pub provenance: ProvenanceRef,
    pub human_readable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroRule {
    pub rule_id: String,
    pub statement: String,
    pub aln_constraint: String,
    pub affected_assets: Vec<String>,
    pub override_priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTreaty {
    pub treaty_id: MicroTreatyId,
    pub scope: String,
    pub initiator: ContactAuthority,
    pub domain: GovernanceDomain,
    pub rules: Vec<MicroRule>,
    pub consensus_mechanism: String,
    pub approval_threshold: f64,
    pub active_period: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub enforcement_mode: EnforcementMode,
    pub provenance: ProvenanceRef,
}

//──────────────────────────────────────────────────────────────────────────────
// Birth-Sign Bundle: Per-Tile Governance Context
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthSign {
    pub birth_sign_id: BirthSignId,
    pub spatial_scope: SpatialScope,
    pub rights_atoms: Vec<AlnNormId>,
    pub biotic_treaties: Vec<BioticTreatyId>,
    pub micro_treaties: Vec<MicroTreatyId>,
    pub fpic_requirement: Option<FpicRequirementId>,
    pub effective_from: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub provenance: Vec<ProvenanceRef>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialScope {
    pub tiles: Vec<TileId>,
    pub geometry_refs: Vec<String>,
    pub bounding_box: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

impl BirthSign {
    pub fn is_active(&self) -> bool {
        Utc::now() >= self.effective_from
    }

    pub fn has_fpic_requirement(&self) -> bool {
        self.fpic_requirement.is_some()
    }

    pub fn all_governance_refs(&self) -> HashSet<String> {
        let mut refs = HashSet::new();
        refs.extend(self.rights_atoms.iter().map(|id| id.0.clone()));
        refs.extend(self.biotic_treaties.iter().map(|id| id.0.clone()));
        refs.extend(self.micro_treaties.iter().map(|id| id.0.clone()));
        if let Some(fpic) = &self.fpic_requirement {
            refs.insert(fpic.0.clone());
        }
        refs
    }
}

//──────────────────────────────────────────────────────────────────────────────
// Node Security Profile
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSecurityProfile {
    pub node_id: String,
    pub authorized_domains: HashSet<GovernanceDomain>,
    pub max_enforcement_level: EnforcementMode,
    pub fpic_consultation_enabled: bool,
    pub audit_retention_days: u32,
    pub cryptographic_binding: CryptographicBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographicBinding {
    pub public_key_did: Did,
    pub signature_scheme: String,
    pub quantum_resistant: bool,
}

impl NodeSecurityProfile {
    pub fn can_enforce(&self, mode: &EnforcementMode) -> bool {
        match (&self.max_enforcement_level, mode) {
            (EnforcementMode::Hard, _) => true,
            (EnforcementMode::HighPenalty, EnforcementMode::Hard) => false,
            (EnforcementMode::HighPenalty, _) => true,
            (EnforcementMode::Advisory, EnforcementMode::Advisory) => true,
            (EnforcementMode::Advisory, _) => false,
        }
    }

    pub fn can_access_domain(&self, domain: &GovernanceDomain) -> bool {
        self.authorized_domains.contains(domain)
    }
}

//──────────────────────────────────────────────────────────────────────────────
// Governance Evaluation Output
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintMode {
    AllHard,
    MixedHardAndPenalty,
    AllPenalty,
    AllAdvisory,
    NoConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintOutcome {
    pub norm_id: String,
    pub norm_type: NormType,
    pub enforcement_mode: EnforcementMode,
    pub satisfied: bool,
    pub violation_reason: Option<String>,
    pub penalty_weight: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormType {
    RightsAtom,
    BioticTreaty,
    MicroTreaty,
    FpicRequirement,
    ConsentAtom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvaluation {
    pub birth_sign_id: BirthSignId,
    pub evaluated_at: DateTime<Utc>,
    pub constraint_mode: ConstraintMode,
    pub outcomes: Vec<ConstraintOutcome>,
    pub fpic_status: Option<FpicStatus>,
    pub total_penalty: f64,
    pub decision_outcome: DecisionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionOutcome {
    Approved,
    ApprovedWithDerating { derating_factor: u8 },
    Rejected { reason: String },
    PendingFpic { territory: String },
    PendingConsent { subject_did: String },
}

impl GovernanceEvaluation {
    pub fn new(birth_sign_id: BirthSignId) -> Self {
        Self {
            birth_sign_id,
            evaluated_at: Utc::now(),
            constraint_mode: ConstraintMode::NoConstraints,
            outcomes: Vec::new(),
            fpic_status: None,
            total_penalty: 0.0,
            decision_outcome: DecisionOutcome::Approved,
        }
    }

    pub fn add_outcome(&mut self, outcome: ConstraintOutcome) {
        if let Some(penalty) = outcome.penalty_weight {
            self.total_penalty += penalty;
        }
        self.outcomes.push(outcome);
    }

    pub fn finalize(&mut self) {
        self.constraint_mode = self.compute_constraint_mode();
        self.decision_outcome = self.derive_decision();
    }

    fn compute_constraint_mode(&self) -> ConstraintMode {
        if self.outcomes.is_empty() {
            return ConstraintMode::NoConstraints;
        }

        let has_hard = self.outcomes.iter().any(|o| matches!(o.enforcement_mode, EnforcementMode::Hard));
        let has_penalty = self.outcomes.iter().any(|o| matches!(o.enforcement_mode, EnforcementMode::HighPenalty));
        let has_advisory = self.outcomes.iter().any(|o| matches!(o.enforcement_mode, EnforcementMode::Advisory));

        match (has_hard, has_penalty, has_advisory) {
            (true, false, false) => ConstraintMode::AllHard,
            (true, true, _) => ConstraintMode::MixedHardAndPenalty,
            (false, true, _) => ConstraintMode::AllPenalty,
            (false, false, true) => ConstraintMode::AllAdvisory,
            _ => ConstraintMode::NoConstraints,
        }
    }

    fn derive_decision(&self) -> DecisionOutcome {
        if let Some(fpic) = &self.fpic_status {
            if fpic.is_pending() {
                if let FpicStatus::Pending { record } = fpic {
                    return DecisionOutcome::PendingFpic {
                        territory: record.territory.0.clone(),
                    };
                }
            }
            if fpic.is_denied() {
                return DecisionOutcome::Rejected {
                    reason: "FPIC denied for Indigenous territory".to_string(),
                };
            }
        }

        let hard_violations: Vec<_> = self.outcomes.iter()
            .filter(|o| matches!(o.enforcement_mode, EnforcementMode::Hard) && !o.satisfied)
            .collect();

        if !hard_violations.is_empty() {
            let reasons: Vec<_> = hard_violations.iter()
                .filter_map(|o| o.violation_reason.as_ref())
                .cloned()
                .collect();
            return DecisionOutcome::Rejected {
                reason: reasons.join("; "),
            };
        }

        let penalty_threshold_critical = 1_000_000.0;
        let penalty_threshold_derating = 100_000.0;

        if self.total_penalty >= penalty_threshold_critical {
            return DecisionOutcome::Rejected {
                reason: format!("Total penalty {} exceeds critical threshold", self.total_penalty),
            };
        }

        if self.total_penalty >= penalty_threshold_derating {
            let derating = ((self.total_penalty / penalty_threshold_derating) * 10.0).min(90.0) as u8;
            return DecisionOutcome::ApprovedWithDerating {
                derating_factor: derating,
            };
        }

        DecisionOutcome::Approved
    }

    pub fn is_approved(&self) -> bool {
        matches!(
            self.decision_outcome,
            DecisionOutcome::Approved | DecisionOutcome::ApprovedWithDerating { .. }
        )
    }

    pub fn requires_fpic_resolution(&self) -> bool {
        matches!(self.decision_outcome, DecisionOutcome::PendingFpic { .. })
    }
}

//──────────────────────────────────────────────────────────────────────────────
// Governed Decision Envelope: Trust-Layer Transaction Schema
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernedDecisionEnvelope {
    pub envelope_id: String,
    pub workflow_id: String,
    pub birth_sign_id: BirthSignId,
    pub governance_evaluation: GovernanceEvaluation,
    pub action_description: String,
    pub actor_did: Option<Did>,
    pub affected_domains: Vec<GovernanceDomain>,
    pub invoked_norms: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub cryptographic_signature: String,
    pub provenance_chain: Vec<ProvenanceRef>,
}

impl GovernedDecisionEnvelope {
    pub fn new(
        workflow_id: String,
        birth_sign_id: BirthSignId,
        governance_evaluation: GovernanceEvaluation,
        action_description: String,
    ) -> Self {
        let invoked_norms = governance_evaluation.outcomes.iter()
            .map(|o| o.norm_id.clone())
            .collect();

        Self {
            envelope_id: Self::generate_envelope_id(&workflow_id, &birth_sign_id),
            workflow_id,
            birth_sign_id,
            governance_evaluation,
            action_description,
            actor_did: None,
            affected_domains: Vec::new(),
            invoked_norms,
            timestamp: Utc::now(),
            cryptographic_signature: String::new(),
            provenance_chain: Vec::new(),
        }
    }

    fn generate_envelope_id(workflow_id: &str, birth_sign_id: &BirthSignId) -> String {
        format!(
            "GOV-ENVELOPE-{}-{}-{}",
            workflow_id,
            birth_sign_id.0,
            Utc::now().timestamp_millis()
        )
    }

    pub fn with_actor(mut self, did: Did) -> Self {
        self.actor_did = Some(did);
        self
    }

    pub fn with_domains(mut self, domains: Vec<GovernanceDomain>) -> Self {
        self.affected_domains = domains;
        self
    }

    pub fn sign(&mut self, signature: String) {
        self.cryptographic_signature = signature;
    }

    pub fn append_provenance(&mut self, provenance: ProvenanceRef) {
        self.provenance_chain.push(provenance);
    }

    pub fn to_audit_record(&self) -> AuditRecord {
        AuditRecord {
            envelope_id: self.envelope_id.clone(),
            timestamp: self.timestamp,
            birth_sign_id: self.birth_sign_id.clone(),
            decision_outcome: self.governance_evaluation.decision_outcome.clone(),
            actor_did: self.actor_did.clone(),
            invoked_norms: self.invoked_norms.clone(),
            signature: self.cryptographic_signature.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub envelope_id: String,
    pub timestamp: DateTime<Utc>,
    pub birth_sign_id: BirthSignId,
    pub decision_outcome: DecisionOutcome,
    pub actor_did: Option<Did>,
    pub invoked_norms: Vec<String>,
    pub signature: String,
}

//──────────────────────────────────────────────────────────────────────────────
// Birth-Sign Registry Interface
//──────────────────────────────────────────────────────────────────────────────

pub trait BirthSignRegistry {
    fn get_birth_sign(&self, id: &BirthSignId) -> Result<BirthSign, RegistryError>;
    fn get_rights_atom(&self, id: &AlnNormId) -> Result<RightsAtom, RegistryError>;
    fn get_biotic_treaty(&self, id: &BioticTreatyId) -> Result<BioticTreaty, RegistryError>;
    fn get_micro_treaty(&self, id: &MicroTreatyId) -> Result<MicroTreaty, RegistryError>;
    fn get_fpic_requirement(&self, id: &FpicRequirementId) -> Result<FpicRequirement, RegistryError>;
    fn get_fpic_status(&self, requirement_id: &FpicRequirementId, action: &str) -> Result<FpicStatus, RegistryError>;
    fn resolve_birth_sign_for_location(&self, lat: f64, lon: f64, time: DateTime<Utc>) -> Result<BirthSignId, RegistryError>;
}

#[derive(Debug, Clone)]
pub enum RegistryError {
    NotFound { id: String },
    InvalidFormat { reason: String },
    NetworkError { details: String },
    AuthorizationFailure { required_role: String },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegistryError::NotFound { id } => write!(f, "Birth-Sign or norm not found: {}", id),
            RegistryError::InvalidFormat { reason } => write!(f, "Invalid format: {}", reason),
            RegistryError::NetworkError { details } => write!(f, "Network error: {}", details),
            RegistryError::AuthorizationFailure { required_role } => {
                write!(f, "Authorization failure, required role: {}", required_role)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

//──────────────────────────────────────────────────────────────────────────────
// Governance Evaluator: Core Preflight Logic
//──────────────────────────────────────────────────────────────────────────────

pub struct GovernanceEvaluator<R: BirthSignRegistry> {
    registry: R,
}

impl<R: BirthSignRegistry> GovernanceEvaluator<R> {
    pub fn new(registry: R) -> Self {
        Self { registry }
    }

    pub fn evaluate_action(
        &self,
        birth_sign_id: &BirthSignId,
        action_context: &ActionContext,
    ) -> Result<GovernanceEvaluation, RegistryError> {
        let birth_sign = self.registry.get_birth_sign(birth_sign_id)?;
        let mut evaluation = GovernanceEvaluation::new(birth_sign_id.clone());

        for rights_id in &birth_sign.rights_atoms {
            let atom = self.registry.get_rights_atom(rights_id)?;
            let outcome = self.evaluate_rights_atom(&atom, action_context);
            evaluation.add_outcome(outcome);
        }

        for treaty_id in &birth_sign.biotic_treaties {
            let treaty = self.registry.get_biotic_treaty(treaty_id)?;
            let outcome = self.evaluate_biotic_treaty(&treaty, action_context);
            evaluation.add_outcome(outcome);
        }

        for micro_id in &birth_sign.micro_treaties {
            let micro = self.registry.get_micro_treaty(micro_id)?;
            let outcome = self.evaluate_micro_treaty(&micro, action_context);
            evaluation.add_outcome(outcome);
        }

        if let Some(fpic_req_id) = &birth_sign.fpic_requirement {
            let fpic_status = self.registry.get_fpic_status(fpic_req_id, &action_context.description)?;
            evaluation.fpic_status = Some(fpic_status);
        }

        evaluation.finalize();
        Ok(evaluation)
    }

    fn evaluate_rights_atom(&self, atom: &RightsAtom, _context: &ActionContext) -> ConstraintOutcome {
        let satisfied = true;
        let penalty_weight = if matches!(atom.enforcement_mode, EnforcementMode::HighPenalty) {
            Some(10_000.0)
        } else {
            None
        };

        ConstraintOutcome {
            norm_id: atom.atom_id.0.clone(),
            norm_type: NormType::RightsAtom,
            enforcement_mode: atom.enforcement_mode.clone(),
            satisfied,
            violation_reason: None,
            penalty_weight,
        }
    }

    fn evaluate_biotic_treaty(&self, treaty: &BioticTreaty, _context: &ActionContext) -> ConstraintOutcome {
        let satisfied = true;
        let penalty_weight = if matches!(treaty.enforcement_mode, EnforcementMode::HighPenalty) {
            Some(50_000.0)
        } else {
            None
        };

        ConstraintOutcome {
            norm_id: treaty.treaty_id.0.clone(),
            norm_type: NormType::BioticTreaty,
            enforcement_mode: treaty.enforcement_mode.clone(),
            satisfied,
            violation_reason: None,
            penalty_weight,
        }
    }

    fn evaluate_micro_treaty(&self, micro: &MicroTreaty, _context: &ActionContext) -> ConstraintOutcome {
        let satisfied = true;
        let penalty_weight = if matches!(micro.enforcement_mode, EnforcementMode::HighPenalty) {
            Some(5_000.0)
        } else {
            None
        };

        ConstraintOutcome {
            norm_id: micro.treaty_id.0.clone(),
            norm_type: NormType::MicroTreaty,
            enforcement_mode: micro.enforcement_mode.clone(),
            satisfied,
            violation_reason: None,
            penalty_weight,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    pub description: String,
    pub spatial_footprint: Vec<TileId>,
    pub domains: Vec<GovernanceDomain>,
    pub proposed_parameters: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
}

//──────────────────────────────────────────────────────────────────────────────
// Compliance Error Types
//──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ComplianceError {
    MissingBirthSign { domain: GovernanceDomain },
    FpicBlocked { territory: String, reason: String },
    HardConstraintViolation { norm_id: String, reason: String },
    RegistryUnavailable { details: String },
    InvalidActionContext { reason: String },
}

impl fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComplianceError::MissingBirthSign { domain } => {
                write!(f, "Missing Birth-Sign for governed domain: {:?}", domain)
            }
            ComplianceError::FpicBlocked { territory, reason } => {
                write!(f, "FPIC blocked for territory {}: {}", territory, reason)
            }
            ComplianceError::HardConstraintViolation { norm_id, reason } => {
                write!(f, "Hard constraint violation [{}]: {}", norm_id, reason)
            }
            ComplianceError::RegistryUnavailable { details } => {
                write!(f, "Governance registry unavailable: {}", details)
            }
            ComplianceError::InvalidActionContext { reason } => {
                write!(f, "Invalid action context: {}", reason)
            }
        }
    }
}

impl std::error::Error for ComplianceError {}

//──────────────────────────────────────────────────────────────────────────────
// Module Tests
//──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_birth_sign_creation() {
        let birth_sign = BirthSign {
            birth_sign_id: BirthSignId("phoenix_downtown_001".to_string()),
            spatial_scope: SpatialScope {
                tiles: vec![TileId("tile_001".to_string())],
                geometry_refs: vec![],
                bounding_box: None,
            },
            rights_atoms: vec![AlnNormId("water_rights_001".to_string())],
            biotic_treaties: vec![],
            micro_treaties: vec![],
            fpic_requirement: None,
            effective_from: Utc::now(),
            last_updated: Utc::now(),
            provenance: vec![],
            version: 1,
        };

        assert!(birth_sign.is_active());
        assert!(!birth_sign.has_fpic_requirement());
    }

    #[test]
    fn test_fpic_status_logic() {
        let granted = FpicStatus::Granted {
            grant: FpicGrant {
                grant_id: "grant_001".to_string(),
                requirement_ref: FpicRequirementId("fpic_001".to_string()),
                action_description: "Water allocation".to_string(),
                granted_by: ContactAuthority::Did(Did("did:test:123".to_string())),
                granted_at: Utc::now(),
                expires_at: None,
                conditions: vec![],
                audit_trail: vec![],
            },
        };

        assert!(granted.is_granted());
        assert!(!granted.blocks_actuation());

        let pending = FpicStatus::Pending {
            record: FpicPendingRecord {
                territory: IndigenousTerritoryId("territory_001".to_string()),
                requested_at: Utc::now(),
                expected_response_by: Utc::now() + Duration::days(30),
            },
        };

        assert!(pending.is_pending());
        assert!(pending.blocks_actuation());
    }

    #[test]
    fn test_governance_evaluation_decision_derivation() {
        let mut eval = GovernanceEvaluation::new(BirthSignId("test_birth_sign".to_string()));

        eval.add_outcome(ConstraintOutcome {
            norm_id: "norm_001".to_string(),
            norm_type: NormType::RightsAtom,
            enforcement_mode: EnforcementMode::Hard,
            satisfied: false,
            violation_reason: Some("Test violation".to_string()),
            penalty_weight: None,
        });

        eval.finalize();

        assert!(!eval.is_approved());
        assert!(matches!(eval.decision_outcome, DecisionOutcome::Rejected { .. }));
    }

    #[test]
    fn test_node_security_profile_enforcement() {
        let profile = NodeSecurityProfile {
            node_id: "edge_node_001".to_string(),
            authorized_domains: vec![GovernanceDomain::Water, GovernanceDomain::Land].into_iter().collect(),
            max_enforcement_level: EnforcementMode::HighPenalty,
            fpic_consultation_enabled: true,
            audit_retention_days: 365,
            cryptographic_binding: CryptographicBinding {
                public_key_did: Did("did:test:node001".to_string()),
                signature_scheme: "Ed25519".to_string(),
                quantum_resistant: false,
            },
        };

        assert!(profile.can_access_domain(&GovernanceDomain::Water));
        assert!(!profile.can_access_domain(&GovernanceDomain::Biosignals));
        assert!(profile.can_enforce(&EnforcementMode::HighPenalty));
        assert!(!profile.can_enforce(&EnforcementMode::Hard));
    }
}
