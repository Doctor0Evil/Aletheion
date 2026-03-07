// Role: Core Synthexis state-model types for SpeciesAgent and BioticTreaty,
//       grounded in Sonoran Desert / Phoenix ecology and Aletheion ERM architecture.

#![allow(clippy::derive_partial_eq_without_eq)]

use serde::{Deserialize, Serialize};

/// Basic numeric interval type for continuous ranges.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct RangeF64 {
    pub min: f64,
    pub max: f64,
}

/// Categorical taxonomic information for a species.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Taxon {
    pub kingdom: String,
    pub phylum: String,
    pub class_name: String,
    pub order: String,
    pub family: String,
    pub genus: String,
    pub species: String,
    pub common_name: String,
}

/// Habitat structure requirements (canopy, nesting, riparian dependency).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HabitatRequirements {
    /// Preferred canopy height in meters (for bats and birds roosting / foraging).
    pub canopy_height_range_m: RangeF64,
    pub cavity_nesting: bool,
    pub ground_nesting: bool,
    /// 0–1, where 1.0 = strictly riparian dependent.
    pub riparian_dependency: f64,
}

/// Water access requirements for a species (bats, pollinators, birds).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WaterRequirements {
    /// Minimum required proximity to open water (meters) for safe corridor design.
    pub min_open_water_proximity_m: f64,
    /// Acceptable water source types: "drip", "birdbath", "riparian", "irrigation".
    pub acceptable_water_sources: Vec<String>,
}

/// Vegetation requirements, grounded in Phoenix pollinator / urban forestry guidance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VegetationRequirements {
    /// IDs referencing a city-wide NativePlant registry (pollinator host plants, canopy trees, etc.).
    pub required_native_plants: Vec<String>,
    /// Minimum fraction (0–1) of landscaping that should be native / pollinator-supportive.
    pub min_native_fraction: f64,
}

/// Light-at-night tolerance profile (ALAN) for a species.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LightTolerance {
    /// True if species avoids lit corridors and prefers dark routes.
    pub light_averse: bool,
    /// Maximum acceptable illuminance along commuting corridors (lux).
    pub max_corridor_alan_lux: f64,
    /// Maximum acceptable illuminance in foraging zones (lux).
    pub max_forage_alan_lux: f64,
    /// Spectral bands to minimize / avoid, represented as (start_nm, end_nm).
    pub spectral_sensitivity_bands_nm: Vec<(u16, u16)>,
}

/// Noise tolerance profile for a species.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NoiseTolerance {
    /// Maximum nightly equivalent sound level (A-weighted dB).
    pub max_night_db_a: f64,
    /// Penalty weight for intermittent / impulsive noise (leaf blowers, impacts).
    pub impulsive_noise_penalty: f64,
    /// Penalty weight for continuous noise (traffic, HVAC).
    pub continuous_noise_penalty: f64,
}

/// Chemical tolerance (pesticides, dust) for a species.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChemicalTolerance {
    /// Pesticide classes that are prohibited within relevant corridors/habitats.
    pub pesticide_classes_prohibited: Vec<String>,
    /// Sensitivity to dust / particulates (0–1, where 1 = highly sensitive).
    pub dust_sensitivity: f64,
}

/// Seasonal activity and critical periods.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SeasonalSensitivity {
    /// Seasons or months where the species is active (e.g., "spring", "summer", "monsoon").
    pub active_seasons: Vec<String>,
    /// Windows critical for breeding, e.g., ISO 8601 intervals.
    pub breeding_windows: Vec<(String, String)>,
    /// Migration windows for migratory species.
    pub migration_windows: Vec<(String, String)>,
    /// Periods requiring dark commuting corridors (e.g., "May-01T18:00Z" to "Aug-31T06:00Z").
    pub critical_dark_corridor_periods: Vec<(String, String)>,
}

/// Priority weights for multi-objective optimization (conservation, cultural, ecosystem services).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PriorityWeights {
    /// Conservation priority (e.g., IUCN status encoded as 0–1).
    pub conservation_priority: f64,
    /// Cultural significance / treaty priority (0–1).
    pub cultural_priority: f64,
    /// Ecosystem service weight (e.g., pollination, pest control importance) (0–1).
    pub ecosystem_service_weight: f64,
}

/// Core Synthexis nonhuman interlocutor, tuned for Sonoran Desert / Phoenix.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpeciesAgent {
    /// Stable identifier (e.g., "myotis_velifer", "native_bee_guild_X").
    pub species_id: String,
    pub taxon: Taxon,
    pub habitat_requirements: HabitatRequirements,
    pub water_requirements: WaterRequirements,
    pub vegetation_requirements: VegetationRequirements,
    pub light_tolerance: LightTolerance,
    pub noise_tolerance: NoiseTolerance,
    pub chemical_tolerance: ChemicalTolerance,
    pub seasonal_sensitivities: SeasonalSensitivity,
    pub priority_weights: PriorityWeights,
}

/// Spatial scope for a BioticTreaty (habitat patches, corridors, riparian areas).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpatialScope {
    /// Polygon IDs referencing the city’s geospatial layer (parks, canals, riparian buffers, etc.).
    pub polygon_ids: Vec<String>,
    /// Network segment IDs representing corridors (streets, canal paths, utility easements).
    pub corridor_segment_ids: Vec<String>,
}

/// Temporal scope for a BioticTreaty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemporalScope {
    /// ISO 8601 start and end of the treaty’s active period.
    pub start: String,
    pub end: String,
    /// Optional recurrence rules (e.g., "YEARLY_MAY_AUG_NIGHT_ONLY").
    pub recurrence_rules: Vec<String>,
}

/// Lighting domain constraints for a treaty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LightingConstraints {
    pub max_lux: f64,
    /// Allowed spectra (e.g., "amber", "low_blue") for fixtures in scope.
    pub allowed_spectra: Vec<String>,
    pub shielding_required: bool,
    /// Segment IDs that must remain as dark corridors when the treaty is active.
    pub dark_corridor_segments: Vec<String>,
}

/// Noise domain constraints for a treaty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NoiseConstraints {
    /// Maximum dB per time band (e.g., "night", "dawn", "evening").
    pub max_db_by_time_band: Vec<(String, f64)>,
    /// Explicitly forbidden impulsive noise sources (e.g., "leaf_blower", "pile_driver").
    pub forbidden_impulsive_sources: Vec<String>,
}

/// Chemical domain constraints for a treaty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChemicalConstraints {
    /// Banned pesticide classes (e.g., "neonicotinoid", "pyrethroid").
    pub banned_pesticide_classes: Vec<String>,
    /// Time windows where spraying is disallowed (e.g., pollinator peak hours).
    pub spray_blackout_windows: Vec<(String, String)>,
    /// No-spray buffer distance (meters) around key habitats / corridors.
    pub no_spray_buffers_m: f64,
}

/// Vegetation and structure constraints for a treaty.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VegetationStructureConstraints {
    /// Minimum fraction of landscaping that must be native / pollinator-supportive.
    pub required_native_plant_fraction: f64,
    /// Host plant IDs that must be present within scope.
    pub required_host_plants: Vec<String>,
    /// Requirements for snags / cavities for bats & birds (e.g., min snags per hectare).
    pub snag_roost_requirements: Vec<String>,
}

/// Enforcement mode for how strictly a BioticTreaty is applied.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EnforcementMode {
    HardLimits,
    Advisory,
}

/// Metrics to be tracked for treaty monitoring.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TreatyMetricsConfig {
    pub track_corridor_connectivity_score: bool,
    pub track_alan_violation_count: bool,
    pub track_pesticide_violation_count: bool,
    pub track_species_activity_index: bool,
}

/// Core treaty binding infrastructure behaviour to one or more SpeciesAgents.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BioticTreaty {
    pub treaty_id: String,
    /// Species IDs this treaty protects (resolved to SpeciesAgent in the state model).
    pub species_ids: Vec<String>,
    pub spatial_scope: SpatialScope,
    pub temporal_scope: TemporalScope,
    pub lighting: Option<LightingConstraints>,
    pub noise: Option<NoiseConstraints>,
    pub chemicals: Option<ChemicalConstraints>,
    pub vegetation_structure: Option<VegetationStructureConstraints>,
    pub enforcement_mode: EnforcementMode,
    pub metrics_to_track: TreatyMetricsConfig,
}
