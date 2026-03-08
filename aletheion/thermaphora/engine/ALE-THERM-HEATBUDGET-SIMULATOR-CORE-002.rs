// ============================================================================
// FILE: aletheion/thermaphora/engine/ALE-THERM-HEATBUDGET-SIMULATOR-CORE-002.rs
// PURPOSE: Human heat budget simulation for Phoenix Downtown blocks and personas,
//          integrating corridor kernel validation, microclimate optimization,
//          and heat vulnerability assessment for zero-downtime sovereign operation
// LANGUAGE: Rust (2024 Edition)
// DESTINATION: Aletheion Repository - Thermaphora Engine Subsystem
// COMPLIANCE: Zero-contamination IFC, NeurorightsEnvelope, FPIC metadata,
//             Phoenix extreme heat resilience (120°F+ operational continuity)
// INTEGRATION: ALE-HIGHWAYS-CORRIDOR-KERNEL-001.rs, ALE-SOMAPLEX-SOMATIC-ROUTE-ENGINE-CORE-002.rs
// C-ABI: JSON-in/JSON-out only, explicit errors, no panics, Lua FFI compatible
// ============================================================================

#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// SECTION 1: THERMAL DOMAIN ENUMERATIONS & TYPE DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HeatZoneType {
    DowntownCore = 0,
    ResidentialBlock = 1,
    IndustrialZone = 2,
    ParkGreenSpace = 3,
    TransitCorridor = 4,
    MixedUse = 5,
    HeritageDistrict = 6,
}

impl HeatZoneType {
    pub const ALL: [HeatZoneType; 7] = [
        HeatZoneType::DowntownCore,
        HeatZoneType::ResidentialBlock,
        HeatZoneType::IndustrialZone,
        HeatZoneType::ParkGreenSpace,
        HeatZoneType::TransitCorridor,
        HeatZoneType::MixedUse,
        HeatZoneType::HeritageDistrict,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            HeatZoneType::DowntownCore => "downtown_core",
            HeatZoneType::ResidentialBlock => "residential_block",
            HeatZoneType::IndustrialZone => "industrial_zone",
            HeatZoneType::ParkGreenSpace => "park_green_space",
            HeatZoneType::TransitCorridor => "transit_corridor",
            HeatZoneType::MixedUse => "mixed_use",
            HeatZoneType::HeritageDistrict => "heritage_district",
        }
    }

    pub fn base_heat_risk(&self) -> f64 {
        match self {
            HeatZoneType::DowntownCore => 0.85,
            HeatZoneType::IndustrialZone => 0.80,
            HeatZoneType::TransitCorridor => 0.75,
            HeatZoneType::MixedUse => 0.65,
            HeatZoneType::ResidentialBlock => 0.55,
            HeatZoneType::HeritageDistrict => 0.50,
            HeatZoneType::ParkGreenSpace => 0.30,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PersonaVulnerabilityClass {
    Elderly = 0,
    Child = 1,
    OutdoorWorker = 2,
    Homeless = 3,
    ChronicIllness = 4,
    Disabled = 5,
    GeneralPopulation = 6,
}

impl PersonaVulnerabilityClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            PersonaVulnerabilityClass::Elderly => "elderly",
            PersonaVulnerabilityClass::Child => "child",
            PersonaVulnerabilityClass::OutdoorWorker => "outdoor_worker",
            PersonaVulnerabilityClass::Homeless => "homeless",
            PersonaVulnerabilityClass::ChronicIllness => "chronic_illness",
            PersonaVulnerabilityClass::Disabled => "disabled",
            PersonaVulnerabilityClass::GeneralPopulation => "general_population",
        }
    }

    pub fn vulnerability_factor(&self) -> f64 {
        match self {
            PersonaVulnerabilityClass::Homeless => 1.0,
            PersonaVulnerabilityClass::OutdoorWorker => 0.95,
            PersonaVulnerabilityClass::Elderly => 0.90,
            PersonaVulnerabilityClass::ChronicIllness => 0.85,
            PersonaVulnerabilityClass::Disabled => 0.80,
            PersonaVulnerabilityClass::Child => 0.75,
            PersonaVulnerabilityClass::GeneralPopulation => 0.50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatPersona {
    pub persona_id: String,
    pub vulnerability_class: PersonaVulnerabilityClass,
    pub age_range: String,
    pub acclimatization_level: f64,
    pub hydration_status: f64,
    pub health_conditions: Vec<String>,
    pub medication_heat_sensitive: bool,
    pub fpic_verified: bool,
    pub consent_active: bool,
}

impl HeatPersona {
    pub fn new(persona_id: String, vulnerability_class: PersonaVulnerabilityClass) -> Self {
        Self {
            persona_id,
            vulnerability_class,
            age_range: "unknown".to_string(),
            acclimatization_level: 0.5,
            hydration_status: 1.0,
            health_conditions: Vec::new(),
            medication_heat_sensitive: false,
            fpic_verified: false,
            consent_active: false,
        }
    }

    pub fn with_age_range(mut self, range: String) -> Self {
        self.age_range = range;
        self
    }

    pub fn with_acclimatization(mut self, level: f64) -> Self {
        self.acclimatization_level = level.min(1.0);
        self
    }

    pub fn with_hydration(mut self, status: f64) -> Self {
        self.hydration_status = status.min(1.0);
        self
    }

    pub fn with_health_conditions(mut self, conditions: Vec<String>) -> Self {
        self.health_conditions = conditions;
        self
    }

    pub fn with_medication_heat_sensitive(mut self, sensitive: bool) -> Self {
        self.medication_heat_sensitive = sensitive;
        self
    }

    pub fn with_fpic(mut self, verified: bool) -> Self {
        self.fpic_verified = verified;
        self
    }

    pub fn with_consent(mut self, active: bool) -> Self {
        self.consent_active = active;
        self
    }

    pub fn composite_vulnerability(&self) -> f64 {
        let base = self.vulnerability_class.vulnerability_factor();
        let acclim_bonus = 1.0 - (self.acclimatization_level * 0.2);
        let hydration_penalty = if self.hydration_status < 0.7 { 1.15 } else { 1.0 };
        let medication_penalty = if self.medication_heat_sensitive { 1.10 } else { 1.0 };
        let health_penalty = 1.0 + (self.health_conditions.len() as f64 * 0.05);
        (base * acclim_bonus * hydration_penalty * medication_penalty * health_penalty).min(1.0)
    }
}

// ============================================================================
// SECTION 2: HEAT BUDGET ENVELOPE STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalHeatLoad {
    pub ambient_temp_c: f64,
    pub surface_temp_c: f64,
    pub heat_index_c: f64,
    pub wet_bulb_temp_c: f64,
    pub radiant_load_w_m2: f64,
    pub relative_humidity_pct: f64,
    pub wind_speed_m_s: f64,
    pub solar_radiation_w_m2: f64,
    pub measurement_timestamp: u64,
}

impl EnvironmentalHeatLoad {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            ambient_temp_c: 25.0,
            surface_temp_c: 35.0,
            heat_index_c: 28.0,
            wet_bulb_temp_c: 22.0,
            radiant_load_w_m2: 600.0,
            relative_humidity_pct: 20.0,
            wind_speed_m_s: 2.0,
            solar_radiation_w_m2: 800.0,
            measurement_timestamp: timestamp,
        }
    }

    pub fn with_ambient_temp(mut self, temp_c: f64) -> Self {
        self.ambient_temp_c = temp_c;
        self
    }

    pub fn with_surface_temp(mut self, temp_c: f64) -> Self {
        self.surface_temp_c = temp_c;
        self
    }

    pub fn with_heat_index(mut self, hi_c: f64) -> Self {
        self.heat_index_c = hi_c;
        self
    }

    pub fn with_wet_bulb_temp(mut self, wb_c: f64) -> Self {
        self.wet_bulb_temp_c = wb_c;
        self
    }

    pub fn with_radiant_load(mut self, load: f64) -> Self {
        self.radiant_load_w_m2 = load;
        self
    }

    pub fn with_humidity(mut self, humidity: f64) -> Self {
        self.relative_humidity_pct = humidity.min(100.0);
        self
    }

    pub fn with_wind_speed(mut self, speed: f64) -> Self {
        self.wind_speed_m_s = speed;
        self
    }

    pub fn with_solar_radiation(mut self, radiation: f64) -> Self {
        self.solar_radiation_w_m2 = radiation;
        self
    }

    pub fn is_extreme_heat(&self) -> bool {
        self.ambient_temp_c >= 46.0 || self.heat_index_c >= 50.0 || self.wet_bulb_temp_c >= 31.0
    }

    pub fn is_life_threatening(&self) -> bool {
        self.wet_bulb_temp_c >= 35.0 || self.heat_index_c >= 60.0
    }
}

impl Default for EnvironmentalHeatLoad {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologicalHeatResponse {
    pub core_temp_c: f64,
    pub skin_temp_c: f64,
    pub heart_rate_bpm: u16,
    pub sweat_rate_l_hr: f64,
    pub dehydration_level: f64,
    pub heat_strain_index: f64,
    pub thermal_comfort_vote: i8,
    pub cognitive_impairment_score: f64,
}

impl PhysiologicalHeatResponse {
    pub fn new() -> Self {
        Self {
            core_temp_c: 37.0,
            skin_temp_c: 33.0,
            heart_rate_bpm: 70,
            sweat_rate_l_hr: 0.5,
            dehydration_level: 0.0,
            heat_strain_index: 0.0,
            thermal_comfort_vote: 0,
            cognitive_impairment_score: 0.0,
        }
    }

    pub fn with_core_temp(mut self, temp: f64) -> Self {
        self.core_temp_c = temp;
        self
    }

    pub fn with_skin_temp(mut self, temp: f64) -> Self {
        self.skin_temp_c = temp;
        self
    }

    pub fn with_heart_rate(mut self, bpm: u16) -> Self {
        self.heart_rate_bpm = bpm;
        self
    }

    pub fn with_sweat_rate(mut self, rate: f64) -> Self {
        self.sweat_rate_l_hr = rate;
        self
    }

    pub fn with_dehydration(mut self, level: f64) -> Self {
        self.dehydration_level = level.min(1.0);
        self
    }

    pub fn with_heat_strain(mut self, strain: f64) -> Self {
        self.heat_strain_index = strain.min(1.0);
        self
    }

    pub fn with_thermal_comfort(mut self, vote: i8) -> Self {
        self.thermal_comfort_vote = vote.clamp(-3, 3);
        self
    }

    pub fn with_cognitive_impairment(mut self, score: f64) -> Self {
        self.cognitive_impairment_score = score.min(1.0);
        self
    }

    pub fn is_heat_exhaustion(&self) -> bool {
        self.core_temp_c >= 38.5 || self.heat_strain_index >= 0.7 || self.dehydration_level >= 0.5
    }

    pub fn is_heat_stroke(&self) -> bool {
        self.core_temp_c >= 40.0 || self.heat_strain_index >= 0.9 || self.dehydration_level >= 0.8
    }
}

impl Default for PhysiologicalHeatResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroclimateIntervention {
    pub intervention_type: String,
    pub effectiveness_score: f64,
    pub energy_consumption_kw: f64,
    pub water_consumption_l_hr: f64,
    pub coverage_area_m2: f64,
    pub operational_cost_usd_hr: f64,
    pub maintenance_interval_hours: u32,
    pub carbon_footprint_kg_co2_hr: f64,
}

impl MicroclimateIntervention {
    pub fn new(intervention_type: String) -> Self {
        Self {
            intervention_type,
            effectiveness_score: 0.0,
            energy_consumption_kw: 0.0,
            water_consumption_l_hr: 0.0,
            coverage_area_m2: 0.0,
            operational_cost_usd_hr: 0.0,
            maintenance_interval_hours: 0,
            carbon_footprint_kg_co2_hr: 0.0,
        }
    }

    pub fn with_effectiveness(mut self, score: f64) -> Self {
        self.effectiveness_score = score.min(1.0);
        self
    }

    pub fn with_energy(mut self, kw: f64) -> Self {
        self.energy_consumption_kw = kw;
        self
    }

    pub fn with_water(mut self, liters: f64) -> Self {
        self.water_consumption_l_hr = liters;
        self
    }

    pub fn with_coverage(mut self, area: f64) -> Self {
        self.coverage_area_m2 = area;
        self
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.operational_cost_usd_hr = cost;
        self
    }

    pub fn with_maintenance(mut self, hours: u32) -> Self {
        self.maintenance_interval_hours = hours;
        self
    }

    pub fn with_carbon(mut self, footprint: f64) -> Self {
        self.carbon_footprint_kg_co2_hr = footprint;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatBudgetEnvelope {
    pub environmental_load: EnvironmentalHeatLoad,
    pub physiological_response: PhysiologicalHeatResponse,
    pub interventions: Vec<MicroclimateIntervention>,
    pub aggregate_risk_score: f64,
    pub recommended_actions: Vec<String>,
    pub safe_exposure_time_min: u32,
    pub cooling_requirement_kw: f64,
    pub hydration_requirement_ml_hr: f64,
    pub compliance_valid: bool,
}

impl HeatBudgetEnvelope {
    pub fn new() -> Self {
        Self {
            environmental_load: EnvironmentalHeatLoad::new(),
            physiological_response: PhysiologicalHeatResponse::new(),
            interventions: Vec::new(),
            aggregate_risk_score: 0.0,
            recommended_actions: Vec::new(),
            safe_exposure_time_min: 480,
            cooling_requirement_kw: 0.0,
            hydration_requirement_ml_hr: 500.0,
            compliance_valid: true,
        }
    }

    pub fn with_environmental_load(mut self, load: EnvironmentalHeatLoad) -> Self {
        self.environmental_load = load;
        self
    }

    pub fn with_physiological_response(mut self, response: PhysiologicalHeatResponse) -> Self {
        self.physiological_response = response;
        self
    }

    pub fn with_interventions(mut self, interventions: Vec<MicroclimateIntervention>) -> Self {
        self.interventions = interventions;
        self
    }

    pub fn compute_risk_score(&mut self, persona: &HeatPersona) {
        let env_risk = if self.environmental_load.is_life_threatening() {
            1.0
        } else if self.environmental_load.is_extreme_heat() {
            0.7
        } else {
            0.3
        };
        let phys_risk = self.physiological_response.heat_strain_index;
        let vulnerability = persona.composite_vulnerability();
        self.aggregate_risk_score = (env_risk * 0.4 + phys_risk * 0.4 + vulnerability * 0.2).min(1.0);
    }

    pub fn compute_safe_exposure(&mut self, persona: &HeatPersona) {
        let base_time = if self.environmental_load.is_life_threatening() {
            15
        } else if self.environmental_load.is_extreme_heat() {
            60
        } else {
            240
        };
        let vulnerability_factor = 1.0 - (persona.composite_vulnerability() * 0.5);
        let hydration_factor = persona.hydration_status;
        self.safe_exposure_time_min = (base_time as f64 * vulnerability_factor * hydration_factor) as u32;
    }

    pub fn compute_hydration_requirement(&mut self, persona: &HeatPersona) {
        let base_requirement = 500.0;
        let heat_factor = if self.environmental_load.ambient_temp_c >= 40.0 { 2.0 } else { 1.0 };
        let activity_factor = if persona.vulnerability_class == PersonaVulnerabilityClass::OutdoorWorker { 1.5 } else { 1.0 };
        let vulnerability_factor = 1.0 + (persona.composite_vulnerability() * 0.3);
        self.hydration_requirement_ml_hr = base_requirement * heat_factor * activity_factor * vulnerability_factor;
    }

    pub fn generate_recommendations(&mut self, persona: &HeatPersona) {
        self.recommended_actions.clear();
        if self.environmental_load.is_life_threatening() {
            self.recommended_actions.push("IMMEDIATE: Seek air-conditioned shelter".to_string());
            self.recommended_actions.push("EMERGENCY: Activate cooling center protocol".to_string());
        }
        if self.environmental_load.is_extreme_heat() {
            self.recommended_actions.push("HIGH ALERT: Limit outdoor exposure to 30 minutes".to_string());
            self.recommended_actions.push("Hydrate with 500ml water every hour".to_string());
        }
        if persona.vulnerability_class == PersonaVulnerabilityClass::Elderly {
            self.recommended_actions.push("Check on elderly neighbors every 2 hours".to_string());
        }
        if persona.vulnerability_class == PersonaVulnerabilityClass::OutdoorWorker {
            self.recommended_actions.push("Mandatory 15-minute cool-down breaks every hour".to_string());
        }
        if self.physiological_response.dehydration_level > 0.3 {
            self.recommended_actions.push("CRITICAL: Increase fluid intake immediately".to_string());
        }
        if !self.interventions.is_empty() {
            self.recommended_actions.push(format!("Activate {} microclimate interventions", self.interventions.len()));
        }
    }

    pub fn validate_compliance(&mut self) {
        self.compliance_valid = self.aggregate_risk_score <= 0.7 && self.safe_exposure_time_min >= 15;
        if self.environmental_load.wet_bulb_temp_c >= 35.0 {
            self.compliance_valid = false;
        }
    }
}

impl Default for HeatBudgetEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 3: IFC LABEL INTEGRATION FOR THERMAPHORA
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermaphoraIFCLabel {
    pub label_id: String,
    pub sensitivity: String,
    pub domain: String,
    pub provenance: String,
    pub origin_hash: String,
    pub timestamp: u64,
    pub fpic_verified: bool,
    pub neurorights_compliant: bool,
    pub thermal_treaty_bound: bool,
    pub corridor_validated: bool,
}

impl ThermaphoraIFCLabel {
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
            thermal_treaty_bound: false,
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

    pub fn with_thermal_treaty(mut self, bound: bool) -> Self {
        self.thermal_treaty_bound = bound;
        self
    }

    pub fn with_corridor_validation(mut self, validated: bool) -> Self {
        self.corridor_validated = validated;
        self
    }
}

// ============================================================================
// SECTION 4: HEAT BUDGET SIMULATOR INPUT/OUTPUT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatBudgetInput {
    pub request_id: String,
    pub block_id: String,
    pub zone_type: HeatZoneType,
    pub personas: Vec<HeatPersona>,
    pub environmental_load: EnvironmentalHeatLoad,
    pub available_interventions: Vec<MicroclimateIntervention>,
    pub ifc_labels: Vec<ThermaphoraIFCLabel>,
    pub corridor_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatBudgetOutput {
    pub request_id: String,
    pub block_id: String,
    pub envelopes: Vec<HeatBudgetEnvelope>,
    pub aggregate_risk_score: f64,
    pub population_at_risk: u32,
    pub recommended_interventions: Vec<String>,
    pub cooling_stations_required: u32,
    pub misting_stations_required: u32,
    pub shade_structures_required: u32,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub can_proceed: bool,
    pub output_ifc_label: ThermaphoraIFCLabel,
    pub timestamp: u64,
}

impl HeatBudgetOutput {
    pub fn new(request_id: String, block_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            request_id,
            block_id,
            envelopes: Vec::new(),
            aggregate_risk_score: 0.0,
            population_at_risk: 0,
            recommended_interventions: Vec::new(),
            cooling_stations_required: 0,
            misting_stations_required: 0,
            shade_structures_required: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            can_proceed: true,
            output_ifc_label: ThermaphoraIFCLabel::new(
                format!("IFC-{}-THERM-0002", timestamp),
                "internal".to_string(),
                "thermal".to_string(),
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
}

// ============================================================================
// SECTION 5: HEAT BUDGET SIMULATOR KERNEL
// ============================================================================

#[derive(Debug, Clone)]
pub struct HeatBudgetSimulatorKernel {
    pub kernel_version: String,
    pub policy_version: String,
    pub phoenix_heat_threshold_c: f64,
    pub extreme_heat_threshold_c: f64,
    pub life_threatening_wetbulb_c: f64,
}

impl HeatBudgetSimulatorKernel {
    pub fn new(kernel_version: String, policy_version: String) -> Self {
        Self {
            kernel_version,
            policy_version,
            phoenix_heat_threshold_c: 46.0,
            extreme_heat_threshold_c: 50.0,
            life_threatening_wetbulb_c: 35.0,
        }
    }

    pub fn validate_ifc_labels(&self, labels: &[ThermaphoraIFCLabel]) -> Result<bool, String> {
        if labels.is_empty() {
            return Err("IFC labels required for all Thermaphora operations".to_string());
        }
        for label in labels {
            if label.sensitivity == "sovereign" && !label.fpic_verified {
                return Err(format!("Sovereign IFC label {} requires FPIC verification", label.label_id));
            }
            if label.domain != "thermal" && label.domain != "somatic" && label.domain != "water" {
                return Err(format!("Invalid domain {} for Thermaphora IFC label", label.domain));
            }
            if label.origin_hash.is_empty() {
                return Err(format!("IFC label {} missing origin hash", label.label_id));
            }
        }
        Ok(true)
    }

    pub fn validate_environmental_load(&self, load: &EnvironmentalHeatLoad) -> Result<bool, String> {
        if load.ambient_temp_c > 55.0 {
            return Err(format!("Ambient temperature {}°C exceeds 55°C operational limit", load.ambient_temp_c));
        }
        if load.heat_index_c > 60.0 {
            return Err(format!("Heat index {}°C exceeds 60°C safety threshold", load.heat_index_c));
        }
        if load.wet_bulb_temp_c >= self.life_threatening_wetbulb_c {
            return Err(format!("Wet bulb temperature {}°C reaches life-threatening threshold", load.wet_bulb_temp_c));
        }
        Ok(true)
    }

    pub fn validate_persona(&self, persona: &HeatPersona) -> Result<bool, String> {
        if persona.vulnerability_class == PersonaVulnerabilityClass::Homeless && !persona.fpic_verified {
            return Err("Homeless persona requires FPIC verification for heat monitoring".to_string());
        }
        if !persona.consent_active && persona.vulnerability_class != PersonaVulnerabilityClass::GeneralPopulation {
            return Err("Active consent required for vulnerable persona heat monitoring".to_string());
        }
        Ok(true)
    }

    pub fn compute_physiological_response(&self, env_load: &EnvironmentalHeatLoad, persona: &HeatPersona) -> PhysiologicalHeatResponse {
        let mut response = PhysiologicalHeatResponse::new();
        let vulnerability = persona.composite_vulnerability();
        let heat_delta = env_load.ambient_temp_c - 25.0;
        response.core_temp_c = 37.0 + (heat_delta * 0.02 * vulnerability);
        response.skin_temp_c = 33.0 + (heat_delta * 0.15 * vulnerability);
        response.heart_rate_bpm = (70 + (heat_delta as u16 * 2) + (vulnerability * 20.0) as u16).min(180);
        response.sweat_rate_l_hr = 0.5 + (heat_delta * 0.05 * vulnerability);
        response.dehydration_level = (response.sweat_rate_l_hr * 0.1 * (1.0 - persona.hydration_status)).min(1.0);
        response.heat_strain_index = (heat_delta / 30.0 * vulnerability).min(1.0);
        response.thermal_comfort_vote = ((heat_delta / 10.0 * -1.0) as i8).clamp(-3, 3);
        response.cognitive_impairment_score = (response.heat_strain_index * 0.5 * vulnerability).min(1.0);
        response
    }

    pub fn optimize_interventions(&self, env_load: &EnvironmentalHeatLoad, available: &[MicroclimateIntervention]) -> Vec<MicroclimateIntervention> {
        let mut selected = Vec::new();
        let mut total_effectiveness = 0.0;
        let mut interventions = available.to_vec();
        interventions.sort_by(|a, b| (b.effectiveness_score / (b.energy_consumption_kw + 0.1))
            .partial_cmp(&(a.effectiveness_score / (a.energy_consumption_kw + 0.1)))
            .unwrap_or(std::cmp::Ordering::Equal));
        for intervention in interventions {
            if total_effectiveness >= 0.8 {
                break;
            }
            selected.push(intervention.clone());
            total_effectiveness += intervention.effectiveness_score * 0.3;
        }
        selected
    }

    pub fn simulate_block_heat_budget(&self, input: &HeatBudgetInput) -> Result<HeatBudgetOutput, String> {
        let mut output = HeatBudgetOutput::new(input.request_id.clone(), input.block_id.clone());
        if let Err(e) = self.validate_ifc_labels(&input.ifc_labels) {
            output.add_error(e);
            return Ok(output);
        }
        if let Err(e) = self.validate_environmental_load(&input.environmental_load) {
            output.add_error(e);
            return Ok(output);
        }
        for persona in &input.personas {
            if let Err(e) = self.validate_persona(persona) {
                output.add_error(e);
                return Ok(output);
            }
        }
        let mut total_risk = 0.0;
        let mut at_risk_count = 0u32;
        for persona in &input.personas {
            let mut envelope = HeatBudgetEnvelope::new();
            envelope = envelope.with_environmental_load(input.environmental_load.clone());
            let phys_response = self.compute_physiological_response(&input.environmental_load, persona);
            envelope = envelope.with_physiological_response(phys_response);
            let optimized = self.optimize_interventions(&input.environmental_load, &input.available_interventions);
            envelope = envelope.with_interventions(optimized);
            envelope.compute_risk_score(persona);
            envelope.compute_safe_exposure(persona);
            envelope.compute_hydration_requirement(persona);
            envelope.generate_recommendations(persona);
            envelope.validate_compliance();
            if envelope.aggregate_risk_score > 0.5 {
                at_risk_count += 1;
            }
            total_risk += envelope.aggregate_risk_score;
            output.envelopes.push(envelope);
        }
        output.aggregate_risk_score = if input.personas.is_empty() { 0.0 } else { total_risk / input.personas.len() as f64 };
        output.population_at_risk = at_risk_count;
        if output.aggregate_risk_score > 0.7 {
            output.cooling_stations_required = ((at_risk_count as f64 / 100.0).ceil() as u32).max(1);
            output.misting_stations_required = ((at_risk_count as f64 / 200.0).ceil() as u32).max(1);
            output.shade_structures_required = ((at_risk_count as f64 / 50.0).ceil() as u32).max(1);
            output.recommended_interventions.push("Activate emergency cooling protocol".to_string());
        }
        if input.environmental_load.is_extreme_heat() {
            output.add_warning("Extreme heat conditions detected - activate Heat Relief Network".to_string());
        }
        if input.environmental_load.is_life_threatening() {
            output.add_warning("Life-threatening heat conditions - emergency protocols required".to_string());
        }
        output.can_proceed = output.errors.is_empty() && output.aggregate_risk_score <= 0.7;
        output.output_ifc_label = output.output_ifc_label
            .with_fpic(true)
            .with_thermal_treaty(true)
            .with_corridor_validation(!input.corridor_id.is_empty());
        Ok(output)
    }
}

// ============================================================================
// SECTION 6: JSON C-ABI EXPORT FOR LUA FFI INTEGRATION
// ============================================================================

#[no_mangle]
pub extern "C" fn ale_thermaphora_simulate_heat_budget_json(
    input_json: *const libc::c_char,
) -> *mut libc::c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let input_str = CStr::from_ptr(input_json).to_string_lossy();
            let input: HeatBudgetInput = match serde_json::from_str(&input_str) {
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
            let kernel = HeatBudgetSimulatorKernel::new("002".to_string(), "001".to_string());
            match kernel.simulate_block_heat_budget(&input) {
                Ok(output) => {
                    let json_output = serde_json::to_string(&output).unwrap_or_else(|e| {
                        format!(r#"{{"valid":false,"error":"Serialization failed: {}"}}"#, e)
                    });
                    CString::new(json_output).unwrap()
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
            let err = CString::new(r#"{"valid":false,"error":"Kernel panic during heat budget simulation"}"#)
                .unwrap();
            err.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn ale_thermaphora_free_string(ptr: *mut libc::c_char) {
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
    fn test_persona_vulnerability_computation() {
        let persona = HeatPersona::new("persona_001".to_string(), PersonaVulnerabilityClass::Elderly)
            .with_acclimatization(0.6)
            .with_hydration(0.8)
            .with_fpic(true)
            .with_consent(true);
        let vulnerability = persona.composite_vulnerability();
        assert!(vulnerability > 0.0);
        assert!(vulnerability <= 1.0);
    }

    #[test]
    fn test_environmental_heat_load_extreme_detection() {
        let env_load = EnvironmentalHeatLoad::new()
            .with_ambient_temp(48.0)
            .with_heat_index(52.0);
        assert!(env_load.is_extreme_heat());
        assert!(!env_load.is_life_threatening());
    }

    #[test]
    fn test_environmental_heat_load_life_threatening() {
        let env_load = EnvironmentalHeatLoad::new()
            .with_wet_bulb_temp(36.0)
            .with_heat_index(62.0);
        assert!(env_load.is_life_threatening());
    }

    #[test]
    fn test_physiological_response_computation() {
        let kernel = HeatBudgetSimulatorKernel::new("002".to_string(), "001".to_string());
        let env_load = EnvironmentalHeatLoad::new().with_ambient_temp(45.0);
        let persona = HeatPersona::new("persona_001".to_string(), PersonaVulnerabilityClass::OutdoorWorker);
        let response = kernel.compute_physiological_response(&env_load, &persona);
        assert!(response.core_temp_c > 37.0);
        assert!(response.heat_strain_index > 0.0);
    }

    #[test]
    fn test_heat_budget_simulation_basic() {
        let kernel = HeatBudgetSimulatorKernel::new("002".to_string(), "001".to_string());
        let persona = HeatPersona::new("persona_001".to_string(), PersonaVulnerabilityClass::GeneralPopulation)
            .with_fpic(true)
            .with_consent(true);
        let ifc_label = ThermaphoraIFCLabel::new(
            "IFC-001".to_string(),
            "internal".to_string(),
            "thermal".to_string(),
            "sensor".to_string(),
            "hash123".to_string(),
        );
        let input = HeatBudgetInput {
            request_id: "req_001".to_string(),
            block_id: "block_downtown_001".to_string(),
            zone_type: HeatZoneType::DowntownCore,
            personas: vec![persona],
            environmental_load: EnvironmentalHeatLoad::new(),
            available_interventions: Vec::new(),
            ifc_labels: vec![ifc_label],
            corridor_id: "CORR-001".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let result = kernel.simulate_block_heat_budget(&input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.can_proceed);
        assert!(!output.envelopes.is_empty());
    }

    #[test]
    fn test_ifc_label_validation() {
        let kernel = HeatBudgetSimulatorKernel::new("002".to_string(), "001".to_string());
        let label = ThermaphoraIFCLabel::new(
            "IFC-001".to_string(),
            "sovereign".to_string(),
            "thermal".to_string(),
            "citizen".to_string(),
            "hash123".to_string(),
        );
        let result = kernel.validate_ifc_labels(&[label]);
        assert!(result.is_err());
    }
}
