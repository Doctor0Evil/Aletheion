#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Core identifiers shared across governance, ERM, and trust layers.

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

/// Territorial domains and scopes.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LawScope {
    City,
    County,
    State,
    National,
    CrossBorderTreaty,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalOverlayKind {
    NeighborhoodMicroTreaty,
    WorkplaceNorms,
    EventOverlay,
    CoolingNorms,
    MobilityNorms,
    DataUseNorms,
}

/// FPIC requirement modeling for Indigenous governance.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FpicRequirement {
    NotApplicable,
    InformOnly,
    Consult,
    ConsentRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FpicStatus {
    Unknown,
    Pending,
    Granted {
        granted_at: SystemTime,
        expires_at: Option<SystemTime>,
        grant_norm: Option<AlnNormId>,
    },
    Refused {
        refused_at: SystemTime,
    },
}

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
    pub fpic_status: FpicStatus,
    /// ALN norms representing TEK envelopes that apply here.
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
    pub summary: String,
}

/// Canonical Birth-Sign bundle: the per-tile governance signature.

#[derive(Debug, Clone)]
pub struct BirthSign {
    pub id: BirthSignId,
    /// Active laws and regulations for this tile.
    pub laws: Vec<LawRef>,
    /// Indigenous governance bundle.
    pub indigenous: Option<IndigenousGovernance>,
    /// Ecological protections and BioticTreaties.
    pub ecological: EcologicalProtections,
    /// Local overlays such as LexEthos micro-treaties and norms.
    pub overlays: Vec<LocalOverlay>,
    /// Versioning and temporal validity.
    pub version: u32,
    pub valid_from: SystemTime,
    pub valid_until: Option<SystemTime>,
}

/// Binding a Birth-Sign to assets, events, and nodes.

#[derive(Debug, Clone)]
pub struct BirthSignBinding {
    pub birth_sign_id: BirthSignId,
    pub asset_ids: Vec<AssetId>,
    pub workflow_event_ids: Vec<WorkflowEventId>,
    pub node_profile_ids: Vec<NodeProfileId>,
}

/// Minimal metadata envelope every ERM event should carry.

#[derive(Debug, Clone)]
pub struct GovernanceContext {
    pub birth_sign_id: BirthSignId,
    pub workflow_id: WorkflowId,
    pub event_id: WorkflowEventId,
    pub subject_did: Option<Did>,
}

/// Node security tier, used by edge orchestration to host workloads safely.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeSecurityTier {
    Basic,
    HardenedFirmwareOnly,
    TeeBacked,
}

/// Summary of security features available on a node.

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
    /// True if this node may host sensitive governance workloads (consent, DIDs, treaty evaluation).
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// True if this node may host general ERM logic that is not biosignal- or identity-sensitive.
    pub fn can_host_general_erm(&self) -> bool {
        matches!(
            self.tier,
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly
        )
    }
}

/// Canonical governance decision outcome for trust-layer records.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionOutcome {
    Approved,
    Rejected,
    Derated,
    PendingFpic,
}

/// Envelope used when appending governed decisions to Googolswarm.

#[derive(Debug, Clone)]
pub struct GovernedDecision {
    pub trust_tx_id: TrustTxId,
    pub workflow_id: WorkflowId,
    pub event_id: WorkflowEventId,
    pub birth_sign_id: BirthSignId,
    pub applied_aln_norms: Vec<AlnNormId>,
    pub subject_did: Option<Did>,
    pub operator_dids: Vec<Did>,
    pub outcome: DecisionOutcome,
    pub occurred_at: SystemTime,
    /// Hashes of normalized input and output states (algorithm defined elsewhere).
    pub inputs_hash: String,
    pub outputs_hash: String,
    /// Optional list of violated norms when outcome != Approved.
    pub violations: Vec<AlnNormId>,
}

/// In-memory registry interface for Birth-Signs (backed by a geospatial DB elsewhere).

#[derive(Default)]
pub struct BirthSignRegistry {
    by_id: HashMap<BirthSignId, BirthSign>,
}

impl BirthSignRegistry {
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
        }
    }

    pub fn insert(&mut self, bs: BirthSign) {
        self.by_id.insert(bs.id.clone(), bs);
    }

    pub fn get(&self, id: &BirthSignId) -> Option<&BirthSign> {
        self.by_id.get(id)
    }
}

/// Placeholder for geospatial lookup; real implementation will call an external service.

pub fn birth_sign_for_point(lat: f64, lon: f64, timestamp: SystemTime) -> Option<BirthSignId> {
    let _ = (lat, lon, timestamp);
    None
}

/// Attach a Birth-Sign to arbitrary metadata, used by edge and state-model layers.

pub fn attach_birth_sign_metadata(
    mut meta: HashMap<String, String>,
    birth_sign_id: &BirthSignId,
) -> HashMap<String, String> {
    meta.insert("birth_sign_id".to_string(), birth_sign_id.0.clone());
    meta
}
