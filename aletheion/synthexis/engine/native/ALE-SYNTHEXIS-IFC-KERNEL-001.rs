// PURPOSE: Synthexis habitat continuity engine with IFC taint enforcement,
//          BioticTreaty validation, LNP (Light/Noise/Pesticide) optimization,
//          and corridor-gated envelope acceptance for zero-contamination flows
// LANGUAGE: Rust (2024 Edition)
// DESTINATION: Aletheion Repository - Synthexis Engine Subsystem
// COMPLIANCE: Zero-contamination IFC, NeurorightsEnvelope, FPIC metadata,
//             BioticTreaty enforcement, Sonoran ecosystem preservation
// INTEGRATION: ALE-HIGHWAYS-CORRIDOR-KERNEL-001.rs, ALE-SYNTHEXIS-BIOTIC-TREATY-CORE-001.aln
// C-ABI: JSON-in/JSON-out only, no direct actuator commands, Lua FFI compatible

#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Import corridor kernel types (would be from crate dependency in production)
// use aletheion_highways_kernel::{CorridorId, CorridorDomain, SevenCapitalEnvelope, IFCLabel, CorridorKernel, CorridorKernelImpl, ValidationResult, PlanSnapshot, SensitivityLevel, ProvenanceType};

// ============================================================================
// SECTION 1: SYNTHESIS DOMAIN ENUMERATIONS & TYPE DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HabitatType {
    DesertScrub = 0,
    RiparianCorridor = 1,
    UrbanGreenSpace = 2,
    PollinatorCorridor = 3,
    WildlifeCrossing = 4,
    NativeRestoration = 5,
    ConservationEasement = 6,
}

impl HabitatType {
    pub const ALL: [HabitatType; 7] = [
        HabitatType::DesertScrub,
        HabitatType::RiparianCorridor,
        HabitatType::UrbanGreenSpace,
        HabitatType::PollinatorCorridor,
        HabitatType::WildlifeCrossing,
        HabitatType::NativeRestoration,
        HabitatType::ConservationEasement,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            HabitatType::DesertScrub => "desert_scrub",
            HabitatType::RiparianCorridor => "riparian_corridor",
            HabitatType::UrbanGreenSpace => "urban_green_space",
            HabitatType::PollinatorCorridor => "pollinator_corridor",
            HabitatType::WildlifeCrossing => "wildlife_crossing",
            HabitatType::NativeRestoration => "native_restoration",
            HabitatType::ConservationEasement => "conservation_easement",
        }
    }

    pub fn priority_weight(&self) -> f64 {
        match self {
            HabitatType::RiparianCorridor => 1.0,
            HabitatType::WildlifeCrossing => 0.95,
            HabitatType::PollinatorCorridor => 0.9,
            HabitatType::NativeRestoration => 0.85,
            HabitatType::ConservationEasement => 0.8,
            HabitatType::DesertScrub => 0.75,
            HabitatType::UrbanGreenSpace => 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SpeciesAgentClass {
    Pollinator = 0,
    Avian = 1,
    Mammalian = 2,
    Reptilian = 3,
    Amphibian = 4,
    Keystone = 5,
    Indicator = 6,
}

impl SpeciesAgentClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpeciesAgentClass::Pollinator => "pollinator",
            SpeciesAgentClass::Avian => "avian",
            SpeciesAgentClass::Mammalian => "mammalian",
            SpeciesAgentClass::Reptilian => "reptilian",
            SpeciesAgentClass::Amphibian => "amphibian",
            SpeciesAgentClass::Keystone => "keystone",
            SpeciesAgentClass::Indicator => "indicator",
        }
    }

    pub fn conservation_priority(&self) -> f64 {
        match self {
            SpeciesAgentClass::Keystone => 1.0,
            SpeciesAgentClass::Pollinator => 0.95,
            SpeciesAgentClass::Indicator => 0.9,
            SpeciesAgentClass::Amphibian => 0.85,
            SpeciesAgentClass::Avian => 0.8,
            SpeciesAgentClass::Mammalian => 0.75,
            SpeciesAgentClass::Reptilian => 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesAgent {
    pub agent_id: String,
    pub species_name: String,
    pub class: SpeciesAgentClass,
    pub population_estimate: u64,
    pub habitat_requirements: Vec<HabitatType>,
    pub threat_level: f64,
    pub corridor_dependency: f64,
    pub fpic_verified: bool,
    pub indigenous_knowledge_integrated: bool,
}

impl SpeciesAgent {
    pub fn new(agent_id: String, species_name: String, class: SpeciesAgentClass) -> Self {
        Self {
            agent_id,
            species_name,
            class,
            population_estimate: 0,
            habitat_requirements: Vec::new(),
            threat_level: 0.0,
            corridor_dependency: 0.0,
            fpic_verified: false,
            indigenous_knowledge_integrated: false,
        }
    }

    pub fn with_population(mut self, pop: u64) -> Self {
        self.population_estimate = pop;
        self
    }

    pub fn with_habitat(mut self, habitats: Vec<HabitatType>) -> Self {
        self.habitat_requirements = habitats;
        self
    }

    pub fn with_threat_level(mut self, threat: f64) -> Self {
        self.threat_level = threat.min(1.0);
        self
    }

    pub fn with_corridor_dependency(mut self, dep: f64) -> Self {
        self.corridor_dependency = dep.min(1.0);
        self
    }

    pub fn with_fpic(mut self, verified: bool) -> Self {
        self.fpic_verified = verified;
        self
    }

    pub fn with_indigenous_knowledge(mut self, integrated: bool) -> Self {
        self.indigenous_knowledge_integrated = integrated;
        self
    }

    pub fn conservation_score(&self) -> f64 {
        let class_priority = self.class.conservation_priority();
        let threat_factor = 1.0 - self.threat_level;
        let corridor_factor = self.corridor_dependency;
        let fpic_bonus = if self.fpic_verified { 1.1 } else { 1.0 };
        let indigenous_bonus = if self.indigenous_knowledge_integrated { 1.05 } else { 1.0 };
        (class_priority * threat_factor * corridor_factor * fpic_bonus * indigenous_bonus).min(1.0)
    }
}

// ============================================================================
// SECTION 2: LIGHT/NOISE/PESTICIDE (LNP) ENVELOPE STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightPollutionEnvelope {
    pub ambient_lux: f64,
    pub sky_glow_index: f64,
    pub spectral_distribution: HashMap<String, f64>,
    pub dark_sky_compliant: bool,
    pub nocturnal_species_safe: bool,
    pub max_allowed_lux: f64,
    pub measurement_timestamp: u64,
}

impl LightPollutionEnvelope {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            ambient_lux: 0.0,
            sky_glow_index: 0.0,
            spectral_distribution: HashMap::new(),
            dark_sky_compliant: true,
            nocturnal_species_safe: true,
            max_allowed_lux: 10.0,
            measurement_timestamp: timestamp,
        }
    }

    pub fn with_ambient_lux(mut self, lux: f64) -> Self {
        self.ambient_lux = lux;
        self.dark_sky_compliant = lux <= self.max_allowed_lux;
        self.nocturnal_species_safe = lux <= 5.0;
        self
    }

    pub fn with_sky_glow(mut self, index: f64) -> Self {
        self.sky_glow_index = index.min(1.0);
        self
    }

    pub fn is_compliant(&self) -> bool {
        self.dark_sky_compliant && self.nocturnal_species_safe && self.ambient_lux <= self.max_allowed_lux
    }
}

impl Default for LightPollutionEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoisePollutionEnvelope {
    pub ambient_db: f64,
    pub frequency_spectrum: HashMap<String, f64>,
    pub peak_events_per_hour: u32,
    pub wildlife_disturbance_index: f64,
    pub human_health_impact: f64,
    pub max_allowed_db: f64,
    pub measurement_timestamp: u64,
}

impl NoisePollutionEnvelope {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            ambient_db: 0.0,
            frequency_spectrum: HashMap::new(),
            peak_events_per_hour: 0,
            wildlife_disturbance_index: 0.0,
            human_health_impact: 0.0,
            max_allowed_db: 55.0,
            measurement_timestamp: timestamp,
        }
    }

    pub fn with_ambient_db(mut self, db: f64) -> Self {
        self.ambient_db = db;
        self.wildlife_disturbance_index = if db > 45.0 { (db - 45.0) / 35.0 } else { 0.0 };
        self.human_health_impact = if db > 50.0 { (db - 50.0) / 40.0 } else { 0.0 };
        self
    }

    pub fn with_peak_events(mut self, events: u32) -> Self {
        self.peak_events_per_hour = events;
        self.wildlife_disturbance_index = (self.wildlife_disturbance_index + (events as f64) * 0.05).min(1.0);
        self
    }

    pub fn is_compliant(&self) -> bool {
        self.ambient_db <= self.max_allowed_db && self.wildlife_disturbance_index <= 0.5
    }
}

impl Default for NoisePollutionEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PesticideEnvelope {
    pub concentration_ppb: f64,
    pub compound_types: Vec<String>,
    pub bioaccumulation_risk: f64,
    pub pollinator_toxicity: f64,
    pub aquatic_toxicity: f64,
    pub zero_tolerance_violation: bool,
    pub measurement_timestamp: u64,
}

impl PesticideEnvelope {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            concentration_ppb: 0.0,
            compound_types: Vec::new(),
            bioaccumulation_risk: 0.0,
            pollinator_toxicity: 0.0,
            aquatic_toxicity: 0.0,
            zero_tolerance_violation: false,
            measurement_timestamp: timestamp,
        }
    }

    pub fn with_concentration(mut self, ppb: f64) -> Self {
        self.concentration_ppb = ppb;
        self.zero_tolerance_violation = ppb > 0.0;
        self.pollinator_toxicity = if ppb > 0.0 { (ppb / 100.0).min(1.0) } else { 0.0 };
        self.aquatic_toxicity = if ppb > 0.0 { (ppb / 50.0).min(1.0) } else { 0.0 };
        self
    }

    pub fn with_compounds(mut self, compounds: Vec<String>) -> Self {
        self.compound_types = compounds;
        self
    }

    pub fn is_compliant(&self) -> bool {
        !self.zero_tolerance_violation && self.concentration_ppb == 0.0
    }
}

impl Default for PesticideEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LNPEnvelope {
    pub light: LightPollutionEnvelope,
    pub noise: NoisePollutionEnvelope,
    pub pesticide: PesticideEnvelope,
    pub aggregate_compliance: bool,
    pub habitat_continuity_score: f64,
}

impl LNPEnvelope {
    pub fn new() -> Self {
        Self {
            light: LightPollutionEnvelope::new(),
            noise: NoisePollutionEnvelope::new(),
            pesticide: PesticideEnvelope::new(),
            aggregate_compliance: true,
            habitat_continuity_score: 1.0,
        }
    }

    pub fn with_light(mut self, env: LightPollutionEnvelope) -> Self {
        self.light = env;
        self.update_compliance();
        self
    }

    pub fn with_noise(mut self, env: NoisePollutionEnvelope) -> Self {
        self.noise = env;
        self.update_compliance();
        self
    }

    pub fn with_pesticide(mut self, env: PesticideEnvelope) -> Self {
        self.pesticide = env;
        self.update_compliance();
        self
    }

    fn update_compliance(&mut self) {
        self.aggregate_compliance = self.light.is_compliant() 
            && self.noise.is_compliant() 
            && self.pesticide.is_compliant();
        self.habitat_continuity_score = self.compute_continuity_score();
    }

    fn compute_continuity_score(&self) -> f64 {
        let light_score = if self.light.is_compliant() { 1.0 } else { 1.0 - self.light.ambient_lux / self.light.max_allowed_lux };
        let noise_score = if self.noise.is_compliant() { 1.0 } else { 1.0 - self.noise.wildlife_disturbance_index };
        let pesticide_score = if self.pesticide.is_compliant() { 1.0 } else { 0.0 };
        (light_score * 0.3 + noise_score * 0.3 + pesticide_score * 0.4).min(1.0)
    }
}

impl Default for LNPEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 3: BIOTIC TREATY SCHEMA & VALIDATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioticTreatyConstraint {
    pub constraint_id: String,
    pub constraint_type: String,
    pub domain: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub required_value: Option<f64>,
    pub enforcement_level: String,
    pub citation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioticTreaty {
    pub treaty_id: String,
    pub version: String,
    pub effective_date: u64,
    pub expiration_date: Option<u64>,
    pub stake_multisig_verified: bool,
    pub indigenous_consultation_complete: bool,
    pub fpic_verified: bool,
    pub constraints: Vec<BioticTreatyConstraint>,
    pub species_agents: Vec<SpeciesAgent>,
    pub habitat_types_protected: Vec<HabitatType>,
    pub corridor_connectivity_min: f64,
    pub pesticide_zero_tolerance: bool,
    pub sovereignty_scalar: f64,
    pub roH_bound: f64,
}

impl BioticTreaty {
    pub fn new(treaty_id: String, version: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            treaty_id,
            version,
            effective_date: now,
            expiration_date: None,
            stake_multisig_verified: false,
            indigenous_consultation_complete: false,
            fpic_verified: false,
            constraints: Vec::new(),
            species_agents: Vec::new(),
            habitat_types_protected: Vec::new(),
            corridor_connectivity_min: 0.7,
            pesticide_zero_tolerance: true,
            sovereignty_scalar: 0.85,
            roH_bound: 0.15,
        }
    }

    pub fn with_multisig(mut self, verified: bool) -> Self {
        self.stake_multisig_verified = verified;
        self
    }

    pub fn with_indigenous_consultation(mut self, complete: bool) -> Self {
        self.indigenous_consultation_complete = complete;
        self
    }

    pub fn with_fpic(mut self, verified: bool) -> Self {
        self.fpic_verified = verified;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<BioticTreatyConstraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_species(mut self, species: Vec<SpeciesAgent>) -> Self {
        self.species_agents = species;
        self
    }

    pub fn with_habitats(mut self, habitats: Vec<HabitatType>) -> Self {
        self.habitat_types_protected = habitats;
        self
    }

    pub fn is_valid(&self) -> bool {
        self.stake_multisig_verified 
            && self.indigenous_consultation_complete 
            && self.fpic_verified 
            && self.roH_bound <= 0.3
    }

    pub fn validate_constraint(&self, constraint: &BioticTreatyConstraint, value: f64) -> bool {
        if let Some(min) = constraint.min_value {
            if value < min {
                return false;
            }
        }
        if let Some(max) = constraint.max_value {
            if value > max {
                return false;
            }
        }
        if let Some(required) = constraint.required_value {
            if (value - required).abs() > 1e-6 {
                return false;
            }
        }
        true
    }
}

// ============================================================================
// SECTION 4: IFC TAINT LABEL INTEGRATION FOR SYNTHESIS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthexisIFCLabel {
    pub label_id: String,
    pub sensitivity: String,
    pub domain: String,
    pub provenance: String,
    pub origin_hash: String,
    pub timestamp: u64,
    pub fpic_verified: bool,
    pub neurorights_compliant: bool,
    pub biotic_treaty_bound: bool,
    pub corridor_validated: bool,
}

impl SynthexisIFCLabel {
    pub fn new(label_id: String, sensitivity: String, domain: String, provenance: String, origin_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            label_id,
            sensitivity,
            domain,
            provenance,
            origin_hash,
            timestamp,
            fpic_verified: false,
            neurorights_compliant: false,
            biotic_treaty_bound: false,
            corridor_validated: false,
        }
    }

    pub fn with_fpic(mut self, verified: bool) -> Self {
        self.fpic_verified = verified;
        self
    }

    pub fn with_neurorights(mut self, compliant: bool) -> Self {
        self.neurorights_compliant = compliant;
        self
    }

    pub fn with_biotic_treaty(mut self, bound: bool) -> Self {
        self.biotic_treaty_bound = bound;
        self
    }

    pub fn with_corridor_validation(mut self, validated: bool) -> Self {
        self.corridor_validated = validated;
        self
    }

    pub fn can_flow_to(&self, target: &SynthexisIFCLabel) -> bool {
        let sensitivity_order = ["public", "internal", "confidential", "sovereign"];
        let source_idx = sensitivity_order.iter().position(|&s| s == self.sensitivity.as_str()).unwrap_or(0);
        let target_idx = sensitivity_order.iter().position(|&s| s == target.sensitivity.as_str()).unwrap_or(0);
        if source_idx > target_idx {
            return false;
        }
        if self.domain != target.domain && source_idx >= 2 {
            return false;
        }
        if !self.fpic_verified && target.fpic_verified {
            return false;
        }
        true
    }
}

// ============================================================================
// SECTION 5: HABITAT CONTINUITY ENGINE CORE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatContinuityInput {
    pub request_id: String,
    pub region_hash: u64,
    pub habitat_types: Vec<HabitatType>,
    pub species_agents: Vec<SpeciesAgent>,
    pub lnp_envelope: LNPEnvelope,
    pub biotic_treaty: BioticTreaty,
    pub ifc_labels: Vec<SynthexisIFCLabel>,
    pub corridor_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatContinuityOutput {
    pub request_id: String,
    pub continuity_score: f64,
    pub lnp_compliance: bool,
    pub treaty_compliance: bool,
    pub ifc_valid: bool,
    pub corridor_valid: bool,
    pub species_risk_assessment: HashMap<String, f64>,
    pub recommended_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub can_proceed: bool,
    pub output_ifc_label: SynthexisIFCLabel,
    pub timestamp: u64,
}

impl HabitatContinuityOutput {
    pub fn new(request_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            request_id,
            continuity_score: 0.0,
            lnp_compliance: false,
            treaty_compliance: false,
            ifc_valid: false,
            corridor_valid: false,
            species_risk_assessment: HashMap::new(),
            recommended_actions: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            can_proceed: false,
            output_ifc_label: SynthexisIFCLabel::new(
                format!("IFC-{}-SYNX-0001", timestamp),
                "internal".to_string(),
                "biotic".to_string(),
                "inference".to_string(),
                format!("hash_{}", request_id),
            ),
            timestamp,
        }
    }

    pub fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
        self.can_proceed = false;
    }

    pub fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }

    pub fn add_action(&mut self, action: String) {
        self.recommended_actions.push(action);
    }
}

#[derive(Debug, Clone)]
pub struct SynthexisIFCKernel {
    pub active_treaties: Arc<HashMap<String, BioticTreaty>>,
    pub kernel_version: String,
    pub policy_version: String,
}

impl SynthexisIFCKernel {
    pub fn new(kernel_version: String, policy_version: String) -> Self {
        Self {
            active_treaties: Arc::new(HashMap::new()),
            kernel_version,
            policy_version,
        }
    }

    pub fn validate_ifc_labels(&self, labels: &[SynthexisIFCLabel]) -> Result<bool, String> {
        if labels.is_empty() {
            return Err("IFC labels required for all Synthexis operations".to_string());
        }
        for label in labels {
            if label.sensitivity == "sovereign" && !label.fpic_verified {
                return Err(format!("Sovereign IFC label {} requires FPIC verification", label.label_id));
            }
            if label.domain != "biotic" && label.domain != "treaty" {
                return Err(format!("Invalid domain {} for Synthexis IFC label", label.domain));
            }
            if label.origin_hash.is_empty() {
                return Err(format!("IFC label {} missing origin hash", label.label_id));
            }
        }
        Ok(true)
    }

    pub fn validate_biotic_treaty(&self, treaty: &BioticTreaty) -> Result<bool, String> {
        if !treaty.is_valid() {
            return Err(format!(
                "BioticTreaty {} invalid: multisig={}, indigenous={}, fpic={}, roH={}",
                treaty.treaty_id,
                treaty.stake_multisig_verified,
                treaty.indigenous_consultation_complete,
                treaty.fpic_verified,
                treaty.roH_bound
            ));
        }
        if !treaty.pesticide_zero_tolerance {
            return Err("Pesticide zero-tolerance required per BioticTreaty".to_string());
        }
        if treaty.corridor_connectivity_min < 0.7 {
            return Err("Corridor connectivity minimum must be >= 0.7".to_string());
        }
        Ok(true)
    }

    pub fn validate_lnp_envelope(&self, envelope: &LNPEnvelope, treaty: &BioticTreaty) -> Result<bool, String> {
        if !envelope.pesticide.is_compliant() && treaty.pesticide_zero_tolerance {
            return Err(format!(
                "Pesticide concentration {} ppb violates zero-tolerance policy",
                envelope.pesticide.concentration_ppb
            ));
        }
        if !envelope.light.is_compliant() {
            return Err(format!(
                "Light pollution {} lux exceeds maximum {} lux",
                envelope.light.ambient_lux, envelope.light.max_allowed_lux
            ));
        }
        if !envelope.noise.is_compliant() {
            return Err(format!(
                "Noise pollution {} dB exceeds maximum {} dB",
                envelope.noise.ambient_db, envelope.noise.max_allowed_db
            ));
        }
        if envelope.habitat_continuity_score < 0.7 {
            return Err(format!(
                "Habitat continuity score {} below minimum 0.7 threshold",
                envelope.habitat_continuity_score
            ));
        }
        Ok(true)
    }

    pub fn compute_species_risk(&self, species: &[SpeciesAgent], lnp: &LNPEnvelope, treaty: &BioticTreaty) -> HashMap<String, f64> {
        let mut risk_map = HashMap::new();
        for agent in species {
            let base_risk = agent.threat_level;
            let pesticide_risk = if lnp.pesticide.zero_tolerance_violation { 0.3 } else { 0.0 };
            let noise_risk = lnp.noise.wildlife_disturbance_index * 0.2;
            let light_risk = if !lnp.light.nocturnal_species_safe && agent.class == SpeciesAgentClass::Pollinator { 0.25 } else { 0.0 };
            let corridor_risk = (1.0 - agent.corridor_dependency) * 0.15;
            let treaty_protection = if treaty.stake_multisig_verified { 0.1 } else { 0.0 };
            let total_risk = (base_risk + pesticide_risk + noise_risk + light_risk + corridor_risk - treaty_protection).max(0.0).min(1.0);
            risk_map.insert(agent.species_name.clone(), total_risk);
        }
        risk_map
    }

    pub fn generate_recommendations(&self, lnp: &LNPEnvelope, treaty: &BioticTreaty, species_risk: &HashMap<String, f64>) -> Vec<String> {
        let mut actions = Vec::new();
        if lnp.light.ambient_lux > 5.0 {
            actions.push("Install dark-sky compliant lighting fixtures in nocturnal corridors".to_string());
        }
        if lnp.noise.ambient_db > 45.0 {
            actions.push("Deploy noise barriers along wildlife crossing points".to_string());
        }
        if lnp.pesticide.concentration_ppb > 0.0 {
            actions.push("Implement pesticide buffer zones per BeeCorridorRouter standards".to_string());
        }
        for (species, risk) in species_risk {
            if *risk > 0.6 {
                actions.push(format!("Priority habitat restoration for {} (risk score: {:.2})", species, risk));
            }
        }
        if treaty.corridor_connectivity_min < 0.85 {
            actions.push("Enhance corridor connectivity to minimum 0.85 for optimal wildlife passage".to_string());
        }
        actions
    }

    pub fn process_habitat_continuity(&self, input: HabitatContinuityInput) -> HabitatContinuityOutput {
        let mut output = HabitatContinuityOutput::new(input.request_id.clone());

        if let Err(e) = self.validate_ifc_labels(&input.ifc_labels) {
            output.add_error(e);
            return output;
        }
        output.ifc_valid = true;

        if let Err(e) = self.validate_biotic_treaty(&input.biotic_treaty) {
            output.add_error(e);
            return output;
        }
        output.treaty_compliance = true;

        if let Err(e) = self.validate_lnp_envelope(&input.lnp_envelope, &input.biotic_treaty) {
            output.add_error(e);
            return output;
        }
        output.lnp_compliance = true;

        output.corridor_valid = !input.corridor_id.is_empty();
        if !output.corridor_valid {
            output.add_error("Corridor ID required for habitat continuity validation".to_string());
            return output;
        }

        output.continuity_score = input.lnp_envelope.habitat_continuity_score;
        output.species_risk_assessment = self.compute_species_risk(&input.species_agents, &input.lnp_envelope, &input.biotic_treaty);
        output.recommended_actions = self.generate_recommendations(&input.lnp_envelope, &input.biotic_treaty, &output.species_risk_assessment);

        let high_risk_species: Vec<_> = output.species_risk_assessment.iter().filter(|(_, &risk)| risk > 0.5).collect();
        if !high_risk_species.is_empty() {
            output.add_warning(format!("{} species with elevated risk scores detected", high_risk_species.len()));
        }

        output.can_proceed = output.lnp_compliance && output.treaty_compliance && output.ifc_valid && output.corridor_valid;
        output.output_ifc_label = output.output_ifc_label
            .with_fpic(input.biotic_treaty.fpic_verified)
            .with_biotic_treaty(true)
            .with_corridor_validation(output.corridor_valid);

        output
    }
}

// ============================================================================
// SECTION 6: JSON C-ABI EXPORT FOR LUA FFI INTEGRATION
// ============================================================================

#[no_mangle]
pub extern "C" fn ale_synthexis_process_habitat_json(
    input_json: *const libc::c_char,
) -> *mut libc::c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let input_str = CStr::from_ptr(input_json).to_string_lossy();
            let input: HabitatContinuityInput = match serde_json::from_str(&input_str) {
                Ok(i) => i,
                Err(e) => {
                    return CString::new(format!(
                        r#"{{"valid":false,"error":"Input JSON parse failed: {}"}}"#,
                        e
                    ))
                    .unwrap()
                    .into_raw();
                }
            };

            let kernel = SynthexisIFCKernel::new("001".to_string(), "001".to_string());
            let output = kernel.process_habitat_continuity(input);
            let json_output = serde_json::to_string(&output).unwrap_or_else(|e| {
                format!(r#"{{"valid":false,"error":"Serialization failed: {}"}}"#, e)
            });

            CString::new(json_output).unwrap()
        }
    }));

    match result {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => {
            let err = CString::new(r#"{"valid":false,"error":"Kernel panic during habitat processing"}"#)
                .unwrap();
            err.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn ale_synthexis_validate_treaty_json(
    treaty_json: *const libc::c_char,
) -> *mut libc::c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let treaty_str = CStr::from_ptr(treaty_json).to_string_lossy();
            let treaty: BioticTreaty = match serde_json::from_str(&treaty_str) {
                Ok(t) => t,
                Err(e) => {
                    return CString::new(format!(
                        r#"{{"valid":false,"error":"Treaty JSON parse failed: {}"}}"#,
                        e
                    ))
                    .unwrap()
                    .into_raw();
                }
            };

            let kernel = SynthexisIFCKernel::new("001".to_string(), "001".to_string());
            match kernel.validate_biotic_treaty(&treaty) {
                Ok(valid) => {
                    let json_output = serde_json::json!({
                        "valid": valid,
                        "treaty_id": treaty.treaty_id,
                        "multisig_verified": treaty.stake_multisig_verified,
                        "indigenous_consultation": treaty.indigenous_consultation_complete,
                        "fpic_verified": treaty.fpic_verified,
                        "roH_bound": treaty.roH_bound
                    });
                    CString::new(json_output.to_string()).unwrap()
                }
                Err(e) => {
                    CString::new(format!(r#"{{"valid":false,"error":"{}"}}"#, e)).unwrap()
                }
            }
        }
    }));

    match result {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => {
            let err = CString::new(r#"{"valid":false,"error":"Kernel panic during treaty validation"}"#)
                .unwrap();
            err.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn ale_synthexis_free_string(ptr: *mut libc::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// ============================================================================
// SECTION 7: UNIT TESTS (CI-INTEGRATED)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_agent_conservation_score() {
        let agent = SpeciesAgent::new(
            "agent_001".to_string(),
            "Apis mellifera".to_string(),
            SpeciesAgentClass::Pollinator,
        )
        .with_threat_level(0.3)
        .with_corridor_dependency(0.8)
        .with_fpic(true)
        .with_indigenous_knowledge(true);
        let score = agent.conservation_score();
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_lnp_envelope_compliance() {
        let light = LightPollutionEnvelope::new().with_ambient_lux(3.0);
        let noise = NoisePollutionEnvelope::new().with_ambient_db(40.0);
        let pesticide = PesticideEnvelope::new().with_concentration(0.0);
        let lnp = LNPEnvelope::new()
            .with_light(light)
            .with_noise(noise)
            .with_pesticide(pesticide);
        assert!(lnp.aggregate_compliance);
        assert!(lnp.habitat_continuity_score >= 0.9);
    }

    #[test]
    fn test_biotic_treaty_validation() {
        let treaty = BioticTreaty::new("MT-000001-BIOT".to_string(), "001".to_string())
            .with_multisig(true)
            .with_indigenous_consultation(true)
            .with_fpic(true);
        let kernel = SynthexisIFCKernel::new("001".to_string(), "001".to_string());
        let result = kernel.validate_biotic_treaty(&treaty);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pesticide_zero_tolerance_violation() {
        let pesticide = PesticideEnvelope::new().with_concentration(5.0);
        assert!(pesticide.zero_tolerance_violation);
        assert!(!pesticide.is_compliant());
    }

    #[test]
    fn test_ifc_label_flow_permission() {
        let source = SynthexisIFCLabel::new(
            "IFC-001".to_string(),
            "public".to_string(),
            "biotic".to_string(),
            "sensor".to_string(),
            "hash123".to_string(),
        );
        let target = SynthexisIFCLabel::new(
            "IFC-002".to_string(),
            "internal".to_string(),
            "biotic".to_string(),
            "inference".to_string(),
            "hash456".to_string(),
        );
        assert!(source.can_flow_to(&target));
    }

    #[test]
    fn test_habitat_continuity_processing() {
        let kernel = SynthexisIFCKernel::new("001".to_string(), "001".to_string());
        let treaty = BioticTreaty::new("MT-000001-BIOT".to_string(), "001".to_string())
            .with_multisig(true)
            .with_indigenous_consultation(true)
            .with_fpic(true);
        let lnp = LNPEnvelope::new();
        let ifc_label = SynthexisIFCLabel::new(
            "IFC-001".to_string(),
            "internal".to_string(),
            "biotic".to_string(),
            "sensor".to_string(),
            "hash123".to_string(),
        );
        let input = HabitatContinuityInput {
            request_id: "req_001".to_string(),
            region_hash: 12345,
            habitat_types: vec![HabitatType::PollinatorCorridor],
            species_agents: Vec::new(),
            lnp_envelope: lnp,
            biotic_treaty: treaty,
            ifc_labels: vec![ifc_label],
            corridor_id: "CORR-001".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let output = kernel.process_habitat_continuity(input);
        assert!(output.can_proceed);
        assert!(output.lnp_compliance);
        assert!(output.treaty_compliance);
        assert!(output.ifc_valid);
        assert!(output.corridor_valid);
    }
}
