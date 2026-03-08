//! ALE-ERM-WATER-THERMAL-CORE-001
//! Canonical ERM schema for joint water–thermal state in Aletheion.
//!
//! This file is the **single source of truth** for water/thermal core
//! types used by generators targeting ALN, Lua, JavaScript, Kotlin, and C++.
//! It must contain **no business logic** (no optimization, routing, or policy
//! algorithms). Only data structures, units, and tagging enums live here.
//!
//! Cross-language generators MUST:
//! - Derive or mirror these types 1:1 (field names, semantics, and units).
//! - Treat all `*_Id` newtypes as opaque identifiers, not integers with
//!   arithmetic semantics.
//! - Preserve unit annotations when generating bindings.

use core::fmt;
use core::time::Duration;

// ---------- Shared identifier newtypes ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FacilityId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReservoirId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AquiferId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanalSegmentId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SewerSegmentId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThermalAssetId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridNodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NeighborhoodId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryRef(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BioticTreatyRef(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyEnvelopeRef(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScenarioId(pub u64);

// ---------- Time and tagging primitives ----------

/// Coarse time bucket used for ERM allocations and forecasts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeGranularity {
    Hour,
    Day,
    Week,
    Month,
}

/// Local time-of-day band for desert operations and Chrono governance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChronoBand {
    SunriseZone,
    PeakHeatZone,
    TwilightZone,
    NightCoolZone,
}

/// Generic confidence band for forecasts and vulnerability scores.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfidenceInterval {
    /// Lower bound (same units as the forecast metric).
    pub lower: f64,
    /// Upper bound (same units as the forecast metric).
    pub upper: f64,
    /// Confidence level, 0.0–1.0 (e.g., 0.9 for 90% CI).
    pub level: f32,
}

// ---------- Units and scalar wrappers ----------

/// Volume in cubic meters.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct VolumeM3(pub f64);

/// Flow rate in cubic meters per second.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FlowM3PerS(pub f64);

/// Temperature in degrees Celsius.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct TemperatureC(pub f64);

/// Heat flux in watts per square meter.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct HeatFluxWPerM2(pub f64);

/// Electrical power in kilowatts.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PowerKW(pub f64);

/// Energy in kilowatt-hours.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct EnergyKWh(pub f64);

/// Dimensionless fraction between 0.0 and 1.0.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Fraction(pub f64);

/// Percentage 0.0–100.0, stored as f64.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Percent(pub f64);

/// Scalar index (unitless) for normalized scores.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct IndexScore(pub f64);

// ---------- Source typing for water portfolio ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterSourceKind {
    SurfaceColoradoRiver,
    SurfaceSaltVerde,
    SurfaceLocalOther,
    Groundwater,
    ReclaimedAdvancedPurified,
    StormwaterCapture,
    AtmosphericHarvest,
    ImportedDesalinated,
    Other,
}

/// High-level regulatory or contractual classification of a water source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterRightClass {
    Municipal,
    Agricultural,
    Industrial,
    Tribal,
    Environmental,
    EmergencyReserve,
}

/// Quality classification for ERM routing and reuse decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterQualityClass {
    Potable,
    NonPotableIrrigation,
    IndustrialProcess,
    EnvironmentalFlow,
}

/// Portfolio entry describing a single water source in the city mix.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterSourcePortfolioEntry {
    pub source_id: FacilityId,
    pub kind: WaterSourceKind,
    pub right_class: WaterRightClass,
    /// Long-run average contribution (m³/day).
    pub avg_contribution_m3_per_day: VolumeM3,
    /// Current modeled contribution (m³/day).
    pub current_contribution_m3_per_day: VolumeM3,
    /// Maximum contract or design capacity (m³/day).
    pub max_capacity_m3_per_day: VolumeM3,
    /// Quality class for routing.
    pub quality_class: WaterQualityClass,
    /// Applicable Indigenous territory, if any.
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    /// Treaties that constrain allocations from this source.
    pub treaties: Vec<BioticTreatyRef>,
}

/// City-wide water portfolio.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterSourcePortfolio {
    pub entries: Vec<WaterSourcePortfolioEntry>,
}

// ---------- Physical infrastructure: water ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReservoirKind {
    SurfaceReservoir,
    UndergroundStorage,
    RechargeBasin,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reservoir {
    pub id: ReservoirId,
    pub name: String,
    pub kind: ReservoirKind,
    /// Current volume stored (m³).
    pub current_volume: VolumeM3,
    /// Maximum design storage (m³).
    pub capacity: VolumeM3,
    /// Minimum safe operational volume (m³).
    pub min_operational_volume: VolumeM3,
    /// Linked aquifer (for recharge) if applicable.
    pub linked_aquifer: Option<AquiferId>,
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Aquifer {
    pub id: AquiferId,
    pub name: String,
    /// Modeled saturated volume under management (m³).
    pub modeled_volume: VolumeM3,
    /// Safe yield limit (m³/year).
    pub safe_yield_per_year: VolumeM3,
    /// Current extraction rate (m³/day).
    pub current_extraction_m3_per_day: VolumeM3,
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AwfFacilityKind {
    /// Advanced Water Purification plant.
    AdvancedPurified,
    /// Conventional wastewater treatment.
    WastewaterTreatment,
    /// Stormwater capture or infiltration site.
    StormwaterCapture,
}

/// Advanced Water Purification / treatment facility state.
#[derive(Clone, Debug, PartialEq)]
pub struct AwfFacility {
    pub id: FacilityId,
    pub name: String,
    pub kind: AwfFacilityKind,
    /// Current inflow (m³/s).
    pub inflow_rate: FlowM3PerS,
    /// Current outflow of treated water (m³/s).
    pub outflow_treated_rate: FlowM3PerS,
    /// Reclamation efficiency (0.0–1.0).
    pub reclamation_efficiency: Fraction,
    /// Nominal design throughput (m³/day).
    pub design_throughput_m3_per_day: VolumeM3,
    /// Connection to reservoirs or aquifers.
    pub linked_reservoirs: Vec<ReservoirId>,
    pub linked_aquifers: Vec<AquiferId>,
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

/// Canal segment for water distribution and monsoon operations.
#[derive(Clone, Debug, PartialEq)]
pub struct CanalSegment {
    pub id: CanalSegmentId,
    pub name: String,
    /// Maximum safe flow (m³/s) before overtopping.
    pub max_safe_flow: FlowM3PerS,
    /// Current flow (m³/s).
    pub current_flow: FlowM3PerS,
    /// Storage capacity within segment (m³), if modeled.
    pub inline_storage_capacity: Option<VolumeM3>,
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

/// Sewer segment with pollutant monitoring hooks.
#[derive(Clone, Debug, PartialEq)]
pub struct SewerSegment {
    pub id: SewerSegmentId,
    pub name: String,
    /// Current volumetric flow (m³/s).
    pub current_flow: FlowM3PerS,
    /// Total organic carbon (mg/L) or other proxy metric.
    pub pollutant_index: IndexScore,
    /// Whether this segment is within a zero-discharge zone.
    pub is_zero_discharge_zone: bool,
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

// ---------- Physical infrastructure: thermal and grid ----------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThermalAssetKind {
    TreeCanopy,
    ShadeStructure,
    CoolPavement,
    GreenRoof,
    WaterFeature,
    MistSystem,
    HighAlbedoSurface,
    BuildingEnvelopeUpgrade,
}

/// Thermal asset that modifies microclimate.
#[derive(Clone, Debug, PartialEq)]
pub struct ThermalAsset {
    pub id: ThermalAssetId,
    pub name: String,
    pub kind: ThermalAssetKind,
    /// Approximate influence radius in meters.
    pub influence_radius_m: f32,
    /// Expected daytime surface temperature reduction (°C).
    pub expected_surface_delta_c: TemperatureC,
    /// Expected mean radiant temperature reduction (°C).
    pub expected_mrt_delta_c: TemperatureC,
    /// Linked water demand (m³/day) if any (e.g., for irrigation, mist).
    pub linked_water_demand_m3_per_day: Option<VolumeM3>,
    /// Associated grid node (for powered assets).
    pub grid_node: Option<GridNodeId>,
    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

/// Grid node for coupling thermal measures to energy availability.
#[derive(Clone, Debug, PartialEq)]
pub struct GridNode {
    pub id: GridNodeId,
    pub name: String,
    /// Available renewable power (kW).
    pub available_renewable_kw: PowerKW,
    /// Total load (kW) at the node.
    pub total_load_kw: PowerKW,
    /// Peak capacity (kW).
    pub capacity_kw: PowerKW,
}

// ---------- Joint water–thermal cell ----------

/// A spatial cell tying together water and thermal state for ERM decisions.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterThermalCell {
    /// Spatial index (implementation-specific; e.g., H3, grid row/col).
    pub cell_id: u64,
    pub neighborhood: Option<NeighborhoodId>,

    // Water metrics.
    /// Localized water demand (m³/day).
    pub local_water_demand_m3_per_day: VolumeM3,
    /// Localized water supply routed to this cell (m³/day).
    pub local_water_supply_m3_per_day: VolumeM3,
    /// Fraction of water demand currently met (0.0–1.0).
    pub water_supply_fraction: Fraction,

    // Thermal metrics.
    /// Outdoor air temperature (°C).
    pub air_temperature_c: TemperatureC,
    /// Mean radiant temperature (°C).
    pub mrt_c: TemperatureC,
    /// Heat index-like composite metric (unitless index).
    pub heat_burden_index: IndexScore,

    // Coupled assets.
    pub nearby_thermal_assets: Vec<ThermalAssetId>,
    pub nearby_canal_segments: Vec<CanalSegmentId>,

    pub indigenous_territory: Option<IndigenousTerritoryRef>,
    pub treaties: Vec<BioticTreatyRef>,
}

// ---------- Vulnerability and budgeting ----------

/// Vulnerability profile for heat, water scarcity, and compounded risk.
#[derive(Clone, Debug, PartialEq)]
pub struct ThermalVulnerabilityProfile {
    pub neighborhood: NeighborhoodId,
    /// Vulnerability index 0.0–1.0; higher means more vulnerable.
    pub vulnerability_index: IndexScore,
    /// Confidence in vulnerability estimate.
    pub confidence: ConfidenceInterval,
    /// Population-weighted exposure to extreme heat events per season.
    pub extreme_heat_events_per_season: f32,
    /// Share of population with limited access to cooling (0.0–1.0).
    pub limited_cooling_fraction: Fraction,
}

/// Heat budget index for a given population cohort or cell.
#[derive(Clone, Debug, PartialEq)]
pub struct HeatBudgetIndex {
    /// Underlying spatial cell.
    pub cell: u64,
    /// Computed budget score 0.0–1.0; higher means closer to unsafe.
    pub score: IndexScore,
    /// Time scale over which this budget is evaluated.
    pub horizon: Duration,
    /// Chrono band most critical for this budget.
    pub critical_chrono_band: ChronoBand,
}

// ---------- ERM scenario and policy envelopes ----------

/// Policy envelope tags for high-level operating modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolicyMode {
    Normal,
    DroughtAlert,
    DroughtEmergency,
    HeatEmergency,
    FloodEmergency,
}

/// Policy envelope snapshot for water–thermal decisions.
#[derive(Clone, Debug, PartialEq)]
pub struct PolicyEnvelope {
    pub id: PolicyEnvelopeRef,
    pub mode: PolicyMode,
    /// Reference to applicable Indigenous territories.
    pub indigenous_territories: Vec<IndigenousTerritoryRef>,
    /// References to all treaties that must be enforced.
    pub treaties: Vec<BioticTreatyRef>,
    /// Human-readable description for audit and governance UIs.
    pub description: String,
}

/// High-level ERM scenario tying together state, portfolio, and policy.
#[derive(Clone, Debug, PartialEq)]
pub struct ErmWaterThermalScenario {
    pub scenario_id: ScenarioId,
    pub name: String,
    pub description: String,

    pub portfolio: WaterSourcePortfolio,
    pub reservoirs: Vec<Reservoir>,
    pub aquifers: Vec<Aquifer>,
    pub awf_facilities: Vec<AwfFacility>,
    pub canals: Vec<CanalSegment>,
    pub sewers: Vec<SewerSegment>,

    pub thermal_assets: Vec<ThermalAsset>,
    pub grid_nodes: Vec<GridNode>,

    pub cells: Vec<WaterThermalCell>,
    pub vulnerability_profiles: Vec<ThermalVulnerabilityProfile>,
    pub heat_budget_indices: Vec<HeatBudgetIndex>,

    pub active_policy_envelope: PolicyEnvelope,
}

// ---------- Display helpers (no business logic) ----------

impl fmt::Display for PolicyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PolicyMode::Normal => "normal",
            PolicyMode::DroughtAlert => "drought_alert",
            PolicyMode::DroughtEmergency => "drought_emergency",
            PolicyMode::HeatEmergency => "heat_emergency",
            PolicyMode::FloodEmergency => "flood_emergency",
        };
        f.write_str(s)
    }
}
