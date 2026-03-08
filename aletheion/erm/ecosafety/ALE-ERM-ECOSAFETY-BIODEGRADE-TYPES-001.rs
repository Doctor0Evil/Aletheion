// Role
// - Define typed ecosafety objects for biodegradable cyboquatic materials and nodes.
// - Make r_degrade, r_micro, r_tox explicit, reusable, and mandatory wherever
//   biodegradable nodes appear in ERM/infra modules.
// - Bind material identities to decomposition.sim / leachate qpudatashards
//   so corridors are grounded in lab+field evidence, not ad-hoc constants.[file:23][file:25]

use std::collections::HashMap;

//////////////////////////////////////////////////////////////
// 1. Shared ecosafety primitives (imported in-code)
//////////////////////////////////////////////////////////////

/// Minimal mirror of the ecosafety spine types, to be shared from
/// ALE-ERM-ECOSAFETY-TYPES-001.rs in real wiring.[file:23]
#[derive(Debug, Clone)]
pub struct RiskCoord {
    pub name: String,   // e.g. "r_degrade", "r_micro", "r_tox_acute"
    pub value: f64,     // current normalized rx ∈ [0,1]
    pub min_safe: f64,  // safe band lower
    pub max_safe: f64,  // safe band upper
}

#[derive(Debug, Clone)]
pub struct RiskVector {
    pub id: String,             // material- or node-specific id
    pub coords: Vec<RiskCoord>, // ordered but accessed by name
}

#[derive(Debug, Clone)]
pub struct LyapunovResidual {
    pub system_id: String,
    pub t: f64,
    pub value: f64,
    pub dvalue_dt: f64,
    pub stable: bool,
}

#[derive(Debug, Clone)]
pub struct Corridor {
    pub corridor_id: String,
    pub domain: String, // "BIODEGRADE", "SOFT_ROBOT_SUBSTRATE"
    pub risk_vector: RiskVector,
    pub lyapunov_template: Option<LyapunovResidual>,
}

//////////////////////////////////////////////////////////////
// 2. Biodegradable material identity and references
//////////////////////////////////////////////////////////////

/// Logical identifier for a biodegradable material recipe.
/// Backed by lab protocols (ISO/OECD) and LC–MS leachate tests.[file:23]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BiodegradeMaterialId(pub String);

impl BiodegradeMaterialId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        BiodegradeMaterialId(s.into())
    }
}

/// Reference to a decomposition.sim shard that encodes lab/field decay curves
/// for a given material under a given test protocol.[file:23]
#[derive(Debug, Clone)]
pub struct DecompositionShardRef {
    pub shard_id: String,          // qpudatashard id
    pub protocol: String,          // e.g. "ISO-14851", "OECD-202"
    pub environment: String,       // "CANAL", "MAR_VAULT", "WETLAND_CELL"
    pub birth_sign_id: String,     // territorial context for the experiment
}

/// Reference to a leachate/toxicity shard (LC–MS, ecotox tests).
#[derive(Debug, Clone)]
pub struct LeachateShardRef {
    pub shard_id: String,
    pub protocol: String,      // e.g. "LCMS-TOX-PANEL-V1"
    pub compartment: String,   // "WATER_COLUMN", "SEDIMENT", "SOIL"
    pub birth_sign_id: String,
}

//////////////////////////////////////////////////////////////
// 3. BiodegradeProfile: core safety object for materials
//////////////////////////////////////////////////////////////

/// Normalized risk channels for biodegradable materials.
/// These are mandatory: no BiodegradeProfile without r_degrade, r_micro, r_tox.[file:23]
#[derive(Debug, Clone)]
pub struct BiodegradeRiskChannels {
    pub r_degrade: RiskCoord,        // mass loss / time vs target profile
    pub r_micro: RiskCoord,          // microplastic / fines residue risk
    pub r_tox_acute: RiskCoord,      // acute ecotoxicity
    pub r_tox_chronic: RiskCoord,    // chronic / bioaccumulation index
    pub r_shear_fragility: RiskCoord, // fragmentation under hydraulic shear
    pub r_habitat_load: RiskCoord,   // disturbance / habitat load
}

impl BiodegradeRiskChannels {
    /// Returns a flat RiskVector suitable for Corridor construction.
    pub fn to_risk_vector(&self, id: &str) -> RiskVector {
        RiskVector {
            id: id.to_string(),
            coords: vec![
                self.r_degrade.clone(),
                self.r_micro.clone(),
                self.r_tox_acute.clone(),
                self.r_tox_chronic.clone(),
                self.r_shear_fragility.clone(),
                self.r_habitat_load.clone(),
            ],
        }
    }

    /// Quick index by name for consumers that need random access.
    pub fn as_map(&self) -> HashMap<&str, &RiskCoord> {
        let mut m = HashMap::new();
        m.insert("r_degrade", &self.r_degrade);
        m.insert("r_micro", &self.r_micro);
        m.insert("r_tox_acute", &self.r_tox_acute);
        m.insert("r_tox_chronic", &self.r_tox_chronic);
        m.insert("r_shear_fragility", &self.r_shear_fragility);
        m.insert("r_habitat_load", &self.r_habitat_load);
        m
    }
}

/// Biodegradation corridor + shard evidence for a single material recipe.
/// This is the canonical object that ties lab data to operational corridors.[file:23]
#[derive(Debug, Clone)]
pub struct BiodegradeProfile {
    pub material_id: BiodegradeMaterialId,
    /// Risk channels with normalized rx values and safe bands.
    pub channels: BiodegradeRiskChannels,
    /// Corridor definition in the shared ecosafety grammar.
    pub corridor: Corridor,
    /// Lab/field decomposition shards that support this profile.
    pub decomposition_shards: Vec<DecompositionShardRef>,
    /// Leachate/toxicity shards backing r_tox and r_micro.
    pub leachate_shards: Vec<LeachateShardRef>,
    /// Optional Lyapunov residual template for long-term stability of this material
    /// under its intended deployment environment.[file:23]
    pub vt_template: Option<LyapunovResidual>,
}

impl BiodegradeProfile {
    /// Helper to construct a Corridor that focuses on biodegradable channels.
    pub fn to_corridor(&self, corridor_id: &str, domain: &str) -> Corridor {
        Corridor {
            corridor_id: corridor_id.to_string(),
            domain: domain.to_string(),
            risk_vector: self.channels.to_risk_vector(corridor_id),
            lyapunov_template: self.vt_template.clone(),
        }
    }
}

//////////////////////////////////////////////////////////////
// 4. Node-level binding: biodegradable cyboquatic nodes
//////////////////////////////////////////////////////////////

/// Identifier for a specific cyboquatic node instance (soft-robot or Flow-vac substrate).[file:23]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BiodegradeNodeId(pub String);

impl BiodegradeNodeId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        BiodegradeNodeId(s.into())
    }
}

/// Operational state slice for a biodegradable node, designed to be embedded
/// into CanalSegmentState, WetlandState, MARVaultState, and sewer/Flow-vac state
/// structs in infra modules.[file:23]
#[derive(Debug, Clone)]
pub struct BiodegradeNodeState {
    pub node_id: BiodegradeNodeId,
    pub material_id: BiodegradeMaterialId,
    /// Deployment age in days since first immersion/activation.
    pub age_days: f64,
    /// Estimated residual mass fraction (0–1) from decomposition.sim shards.
    pub residual_mass_fraction: f64,
    /// Measured/estimated microplastic residue concentration (normalized).
    pub r_micro_current: f64,
    /// Measured/estimated toxicity index (normalized).
    pub r_tox_current: f64,
    /// Latest composite Lyapunov residual for this node in its environment.[file:23]
    pub vt_current: Option<LyapunovResidual>,
    /// Geography binding for governance and Birth-Sign queries.
    pub corridor_id: String,   // e.g. canal segment id
    pub birth_sign_id: String, // territorial tile
}

impl BiodegradeNodeState {
    /// Update fast-changing rx values from sensors / model outputs.
    pub fn update_risks(&mut self, r_micro: f64, r_tox: f64) {
        self.r_micro_current = r_micro;
        self.r_tox_current = r_tox;
    }
}

//////////////////////////////////////////////////////////////
// 5. Registry helpers for profiles and nodes
//////////////////////////////////////////////////////////////

/// In-memory registry for BiodegradeProfiles, to be backed by
/// ALN configs + qpudatashard ingestion in higher layers.[file:23]
#[derive(Debug, Default)]
pub struct BiodegradeProfileRegistry {
    by_material: HashMap<BiodegradeMaterialId, BiodegradeProfile>,
}

impl BiodegradeProfileRegistry {
    pub fn new() -> Self {
        Self {
            by_material: HashMap::new(),
        }
    }

    pub fn insert(&mut self, profile: BiodegradeProfile) {
        self.by_material.insert(profile.material_id.clone(), profile);
    }

    pub fn get(&self, id: &BiodegradeMaterialId) -> Option<&BiodegradeProfile> {
        self.by_material.get(id)
    }

    /// Convenience: fetch the Corridor for a material, or None if missing.
    pub fn corridor_for(
        &self,
        id: &BiodegradeMaterialId,
        corridor_id: &str,
        domain: &str,
    ) -> Option<Corridor> {
        self.by_material.get(id).map(|p| p.to_corridor(corridor_id, domain))
    }
}

/// Lightweight bundle tying a node state to its material profile;
/// this is what Service/FOG routing layers can pass into ecosafety contracts
/// like safe_step() without redoing lookups on every call.[file:23]
#[derive(Debug, Clone)]
pub struct BiodegradeNodeContext<'a> {
    pub state: &'a BiodegradeNodeState,
    pub profile: &'a BiodegradeProfile,
}

impl<'a> BiodegradeNodeContext<'a> {
    pub fn new(state: &'a BiodegradeNodeState, profile: &'a BiodegradeProfile) -> Self {
        Self { state, profile }
    }
}

//////////////////////////////////////////////////////////////
// 6. Minimal tests (profile wiring, not enforcement)
//////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_coord(name: &str, value: f64, min: f64, max: f64) -> RiskCoord {
        RiskCoord {
            name: name.to_string(),
            value,
            min_safe: min,
            max_safe: max,
        }
    }

    #[test]
    fn biodegrade_channels_to_vector_has_all_coords() {
        let channels = BiodegradeRiskChannels {
            r_degrade: mk_coord("r_degrade", 0.3, 0.0, 0.8),
            r_micro: mk_coord("r_micro", 0.2, 0.0, 0.7),
            r_tox_acute: mk_coord("r_tox_acute", 0.1, 0.0, 0.6),
            r_tox_chronic: mk_coord("r_tox_chronic", 0.1, 0.0, 0.5),
            r_shear_fragility: mk_coord("r_shear_fragility", 0.4, 0.0, 0.9),
            r_habitat_load: mk_coord("r_habitat_load", 0.2, 0.0, 0.8),
        };
        let rv = channels.to_risk_vector("BIO_MAT_V1");
        assert_eq!(rv.coords.len(), 6);
        let names: Vec<_> = rv.coords.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"r_degrade"));
        assert!(names.contains(&"r_micro"));
        assert!(names.contains(&"r_tox_acute"));
        assert!(names.contains(&"r_tox_chronic"));
    }

    #[test]
    fn registry_returns_corridor_for_material() {
        let channels = BiodegradeRiskChannels {
            r_degrade: mk_coord("r_degrade", 0.3, 0.0, 0.8),
            r_micro: mk_coord("r_micro", 0.2, 0.0, 0.7),
            r_tox_acute: mk_coord("r_tox_acute", 0.1, 0.0, 0.6),
            r_tox_chronic: mk_coord("r_tox_chronic", 0.1, 0.0, 0.5),
            r_shear_fragility: mk_coord("r_shear_fragility", 0.4, 0.0, 0.9),
            r_habitat_load: mk_coord("r_habitat_load", 0.2, 0.0, 0.8),
        };
        let mat_id = BiodegradeMaterialId::new("BIO_MAT_V1");
        let corridor = Corridor {
            corridor_id: "BIODEGRADE_DEFAULT".to_string(),
            domain: "BIODEGRADE".to_string(),
            risk_vector: channels.to_risk_vector("BIODEGRADE_DEFAULT"),
            lyapunov_template: None,
        };
        let profile = BiodegradeProfile {
            material_id: mat_id.clone(),
            channels,
            corridor,
            decomposition_shards: vec![],
            leachate_shards: vec![],
            vt_template: None,
        };

        let mut reg = BiodegradeProfileRegistry::new();
        reg.insert(profile);

        let c = reg.corridor_for(&mat_id, "RUN1", "BIODEGRADE").unwrap();
        assert_eq!(c.domain, "BIODEGRADE");
        assert_eq!(c.risk_vector.id, "RUN1");
    }
}
