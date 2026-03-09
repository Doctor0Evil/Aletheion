// Phoenix Industrial Corridor 01 – Cave Creek + AWG + Cool Pavement + PHX-DUST
// Corridor-level orchestrator wiring:
// - AWP + groundwater + stormwater state (WF 6,7,8,10)
// - Rooftop AWG clusters (MOF-based AWG; WF 10,11)
// - Cool pavement + shade corridors (WF 15,16,14)
// - PHX-DUST haboob responses (WF 17)
// into a single ERM-facing control surface.
//
// ERM: L2 state aggregation, L3 trust hooks (delegated), L4-ready exports.
// Language: Rust (no blacklisted hashes or primitives).

use std::collections::HashMap;
use std::time::SystemTime;

// ---------- Shared conceptual IDs (mirrors core model, but corridor-local) ----------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CorridorWaterSourceId {
    AwpPlant(&'static str),          // e.g., "CaveCreekAWP"
    GroundwaterBlock(&'static str),  // e.g., "NorthPhxBlock01"
    AtmosphericCluster(&'static str),// e.g., "SouthPhxRooftopAWG01"
    StormCaptureBasin(&'static str), // e.g., "MonsoonRecharge01"
    ImportedPortfolio(&'static str), // e.g., "ColoradoPortfolio"
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CorridorThermalAssetId {
    CoolPavementSegment(&'static str), // e.g., "CP-McDowell-Industrial-01"
    ShadeCorridor(&'static str),       // tree + shade mesh id
    IndustrialRooftop(&'static str),   // roofs hosting AWG + PV
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CorridorDustCellId {
    GridCell(&'static str), // e.g., "PHX-DUST-CELL-15N-07W"
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CorridorDemandZoneId {
    Neighborhood(&'static str),
    CoolingCorridor(&'static str),
    IndustrialCluster(&'static str),
    AgriculturalDistrict(&'static str),
    EcologicalReserve(&'static str),
}

// ---------- Basic physical units ----------

#[derive(Debug, Clone, Copy)]
pub struct AcreFeet {
    pub value: f64,
}

impl AcreFeet {
    pub fn zero() -> Self {
        Self { value: 0.0 }
    }
    pub fn from_mgd(mgd: f64, days: f64) -> Self {
        // 1 MGD ≈ 3.068883 AF/day (same as core model) [file:5]
        Self {
            value: mgd * 3.068_883_f64 * days,
        }
    }
}

// A simple daily water yield struct for AWG, AWP, etc.
#[derive(Debug, Clone, Copy)]
pub struct DailyWaterYieldAf {
    pub value: f64,
}

impl DailyWaterYieldAf {
    pub fn zero() -> Self {
        Self { value: 0.0 }
    }
}

// ---------- Corridor water state ----------

#[derive(Debug, Clone)]
pub struct CorridorSourceState {
    pub id: CorridorWaterSourceId,
    pub last_update: Option<SystemTime>,
    pub storage_af: AcreFeet,
    pub inflow_rate_afy: f64,
    pub outflow_rate_afy: f64,
    pub quality_score: f32, // 0–1
    pub tags: Vec<String>,  // e.g., ["awp", "cave-creek"]
}

#[derive(Debug, Clone)]
pub struct CorridorDemandState {
    pub id: CorridorDemandZoneId,
    pub last_update: Option<SystemTime>,
    pub daily_demand_af: f64,
    pub vulnerability_score: f32, // heat + socio-economic 0–1
    pub equity_weight: f32,
    pub tags: Vec<String>, // e.g., ["heat-vulnerable", "industrial"]
}

// AWG cluster view – derived from RH, irradiance, MOF mass.
#[derive(Debug, Clone)]
pub struct AwgClusterSnapshot {
    pub id: CorridorWaterSourceId, // must be AtmosphericCluster
    pub timestamp: SystemTime,
    pub mofs_kg: f64,
    pub rh_fraction: f32,       // 0.10–0.30 typical in PHX [web:16]
    pub solar_irr_w_m2: f32,    // current irradiance
    pub expected_yield_af: DailyWaterYieldAf,
    pub mode: AwgMode,
}

#[derive(Debug, Clone, Copy)]
pub enum AwgMode {
    SorptionOnly,
    HybridSorptionCondensation,
}

// ---------- Thermal assets and cool pavement ----------

#[derive(Debug, Clone)]
pub struct CoolPavementSegmentState {
    pub id: CorridorThermalAssetId, // CoolPavementSegment
    pub last_update: Option<SystemTime>,
    pub albedo: f32,           // current albedo
    pub surface_temp_f: f32,   // current midday surface temp
    pub baseline_temp_f: f32,  // comparable non-coated baseline
    pub miles: f32,
    pub tags: Vec<String>,     // e.g., ["arterial", "industrial"]
}

#[derive(Debug, Clone)]
pub struct ShadeCorridorState {
    pub id: CorridorThermalAssetId, // ShadeCorridor
    pub last_update: Option<SystemTime>,
    pub canopy_cover_fraction: f32, // 0–1
    pub species_mix: Vec<String>,   // e.g., ["saguaro", "palo-verde", "ocotillo"]
    pub cooling_bonus_f: f32,       // estimated additional air temp reduction
    pub tags: Vec<String>,
}

// ---------- Dust state and PHX-DUST scale ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhxDustCategory {
    Isolated1,
    Minor2,
    Moderate3,
    Major4,
    Extreme5,
}

#[derive(Debug, Clone)]
pub struct DustCellSnapshot {
    pub id: CorridorDustCellId,
    pub timestamp: SystemTime,
    pub pm10_ug_m3: f32,
    pub pm25_ug_m3: f32,
    pub max_gust_mph: f32,
    pub category: PhxDustCategory,
}

// ---------- Corridor indicators exported to L4 ----------

#[derive(Debug, Clone)]
pub struct CorridorPortfolioIndicators {
    pub total_storage_af: f64,
    pub total_awp_storage_af: f64,
    pub total_groundwater_storage_af: f64,
    pub total_awg_yield_af_per_day: f64,
    pub colorado_exposure_fraction: f64,
    pub awp_reuse_fraction: f64, // approximate, reuse vs demand [file:5]
}

#[derive(Debug, Clone)]
pub struct ThermalIndicators {
    pub total_cool_pavement_miles: f32,
    pub avg_surface_reduction_f: f32,
    pub avg_nighttime_offset_f: f32,
}

#[derive(Debug, Clone)]
pub struct DustIndicators {
    pub max_category_seen: PhxDustCategory,
    pub cells_in_cat3_or_higher: usize,
}

// ---------- Main corridor orchestrator ----------

pub struct PhxCorridorWhdCore {
    // Water
    sources: HashMap<CorridorWaterSourceId, CorridorSourceState>,
    demands: HashMap<CorridorDemandZoneId, CorridorDemandState>,
    awg_clusters: HashMap<CorridorWaterSourceId, AwgClusterSnapshot>,

    // Thermal
    cool_pavements: HashMap<CorridorThermalAssetId, CoolPavementSegmentState>,
    shade_corridors: HashMap<CorridorThermalAssetId, ShadeCorridorState>,

    // Dust
    dust_cells: HashMap<CorridorDustCellId, DustCellSnapshot>,
}

impl PhxCorridorWhdCore {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            demands: HashMap::new(),
            awg_clusters: HashMap::new(),
            cool_pavements: HashMap::new(),
            shade_corridors: HashMap::new(),
            dust_cells: HashMap::new(),
        }
    }

    // -------- Corridor bootstrap helpers --------

    /// Register known static sources with initial tags.
    pub fn register_awp_source(
        &mut self,
        id_str: &'static str,
        initial_storage_af: f64,
        tags: Vec<String>,
    ) {
        let id = CorridorWaterSourceId::AwpPlant(id_str);
        let state = CorridorSourceState {
            id: id.clone(),
            last_update: None,
            storage_af: AcreFeet { value: initial_storage_af },
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 0.99,
            tags,
        };
        self.sources.insert(id, state);
    }

    pub fn register_groundwater_block(
        &mut self,
        id_str: &'static str,
        initial_storage_af: f64,
        tags: Vec<String>,
    ) {
        let id = CorridorWaterSourceId::GroundwaterBlock(id_str);
        let state = CorridorSourceState {
            id: id.clone(),
            last_update: None,
            storage_af: AcreFeet { value: initial_storage_af },
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 0.98,
            tags,
        };
        self.sources.insert(id, state);
    }

    pub fn register_awg_cluster(&mut self, id_str: &'static str, mofs_kg: f64) {
        let id = CorridorWaterSourceId::AtmosphericCluster(id_str);
        let snapshot = AwgClusterSnapshot {
            id: id.clone(),
            timestamp: SystemTime::now(),
            mofs_kg,
            rh_fraction: 0.15,
            solar_irr_w_m2: 0.0,
            expected_yield_af: DailyWaterYieldAf::zero(),
            mode: AwgMode::SorptionOnly,
        };
        self.awg_clusters.insert(id.clone(), snapshot);
        // Also make sure this cluster appears as a water source with 0 storage but yield info.
        self.sources.entry(id).or_insert(CorridorSourceState {
            id: CorridorWaterSourceId::AtmosphericCluster(id_str),
            last_update: None,
            storage_af: AcreFeet::zero(),
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 1.0,
            tags: vec!["awg".into(), "atmospheric".into()],
        });
    }

    pub fn register_demand_zone(
        &mut self,
        id: CorridorDemandZoneId,
        daily_demand_af: f64,
        vulnerability_score: f32,
        equity_weight: f32,
        tags: Vec<String>,
    ) {
        let state = CorridorDemandState {
            id: id.clone(),
            last_update: None,
            daily_demand_af,
            vulnerability_score,
            equity_weight,
            tags,
        };
        self.demands.insert(id, state);
    }

    pub fn register_cool_pavement_segment(
        &mut self,
        id_str: &'static str,
        miles: f32,
        albedo: f32,
        baseline_temp_f: f32,
        tags: Vec<String>,
    ) {
        let id = CorridorThermalAssetId::CoolPavementSegment(id_str);
        let state = CoolPavementSegmentState {
            id: id.clone(),
            last_update: None,
            albedo,
            surface_temp_f: baseline_temp_f,
            baseline_temp_f,
            miles,
            tags,
        };
        self.cool_pavements.insert(id, state);
    }

    pub fn register_shade_corridor(
        &mut self,
        id_str: &'static str,
        canopy_cover_fraction: f32,
        species_mix: Vec<String>,
        cooling_bonus_f: f32,
        tags: Vec<String>,
    ) {
        let id = CorridorThermalAssetId::ShadeCorridor(id_str);
        let state = ShadeCorridorState {
            id: id.clone(),
            last_update: None,
            canopy_cover_fraction,
            species_mix,
            cooling_bonus_f,
            tags,
        };
        self.shade_corridors.insert(id, state);
    }

    // -------- Update APIs fed by ingestion workflows --------

    pub fn update_source_storage(
        &mut self,
        id: &CorridorWaterSourceId,
        storage_af: f64,
        inflow_rate_afy: f64,
        outflow_rate_afy: f64,
        quality_score: f32,
        ts: SystemTime,
    ) {
        if let Some(state) = self.sources.get_mut(id) {
            state.storage_af.value = storage_af.max(0.0);
            state.inflow_rate_afy = inflow_rate_afy;
            state.outflow_rate_afy = outflow_rate_afy;
            state.quality_score = quality_score;
            state.last_update = Some(ts);
        }
    }

    pub fn update_demand_zone(
        &mut self,
        id: &CorridorDemandZoneId,
        daily_demand_af: f64,
        vulnerability_score: f32,
        equity_weight: f32,
        ts: SystemTime,
    ) {
        if let Some(state) = self.demands.get_mut(id) {
            state.daily_demand_af = daily_demand_af.max(0.0);
            state.vulnerability_score = vulnerability_score.clamp(0.0, 1.0);
            state.equity_weight = equity_weight.max(0.0);
            state.last_update = Some(ts);
        }
    }

    pub fn update_awg_cluster_from_env(
        &mut self,
        id_str: &'static str,
        rh_fraction: f32,
        solar_irr_w_m2: f32,
        mofs_kg: f64,
        ts: SystemTime,
    ) {
        let key = CorridorWaterSourceId::AtmosphericCluster(id_str);
        let rh = rh_fraction.clamp(0.0, 1.0);

        // Approximate yield: 0.7–1.3 L/kg/day between 0.10–0.30 RH [web:16].
        let base_min = 0.7_f64;
        let base_max = 1.3_f64;
        let rh10 = 0.10_f32;
        let rh30 = 0.30_f32;
        let mut liters_per_kg_day = base_min;

        if rh <= rh10 {
            liters_per_kg_day = base_min * (rh / rh10 as f32).max(0.1) as f64;
        } else if rh >= rh30 {
            liters_per_kg_day = base_max;
        } else {
            let t = (rh - rh10) / (rh30 - rh10);
            liters_per_kg_day = (base_min + (base_max - base_min) * t as f64).max(base_min);
        }

        // Adjust yield by solar availability (simplified linear factor).
        let solar_factor = (solar_irr_w_m2 / 1000.0_f32).clamp(0.0, 1.2) as f64;
        let liters_day = liters_per_kg_day * mofs_kg * solar_factor;

        // Convert to acre-feet/day: 1 AF ≈ 1,233,480 L.
        let af_day = liters_day / 1_233_480.0_f64;

        let mode = if rh > 0.25 && solar_irr_w_m2 > 500.0 {
            AwgMode::HybridSorptionCondensation
        } else {
            AwgMode::SorptionOnly
        };

        let snapshot = AwgClusterSnapshot {
            id: key.clone(),
            timestamp: ts,
            mofs_kg,
            rh_fraction: rh,
            solar_irr_w_m2,
            expected_yield_af: DailyWaterYieldAf { value: af_day.max(0.0) },
            mode,
        };

        self.awg_clusters.insert(key.clone(), snapshot);

        if let Some(src) = self.sources.get_mut(&key) {
            src.last_update = Some(ts);
            // Treat AWG yield as an effective inflow (AFY equivalent).
            src.inflow_rate_afy = af_day * 365.0;
        }
    }

    pub fn update_cool_pavement_segment(
        &mut self,
        id_str: &'static str,
        albedo: f32,
        surface_temp_f: f32,
        baseline_temp_f: f32,
        ts: SystemTime,
    ) {
        let key = CorridorThermalAssetId::CoolPavementSegment(id_str);
        if let Some(seg) = self.cool_pavements.get_mut(&key) {
            seg.albedo = albedo;
            seg.surface_temp_f = surface_temp_f;
            seg.baseline_temp_f = baseline_temp_f;
            seg.last_update = Some(ts);
        }
    }

    pub fn update_shade_corridor_cooling(
        &mut self,
        id_str: &'static str,
        cooling_bonus_f: f32,
        ts: SystemTime,
    ) {
        let key = CorridorThermalAssetId::ShadeCorridor(id_str);
        if let Some(corr) = self.shade_corridors.get_mut(&key) {
            corr.cooling_bonus_f = cooling_bonus_f;
            corr.last_update = Some(ts);
        }
    }

    pub fn update_dust_cell(
        &mut self,
        id_str: &'static str,
        pm10_ug_m3: f32,
        pm25_ug_m3: f32,
        max_gust_mph: f32,
        ts: SystemTime,
    ) {
        let key = CorridorDustCellId::GridCell(id_str);
        let category = Self::categorize_phx_dust(pm10_ug_m3);
        let snapshot = DustCellSnapshot {
            id: key.clone(),
            timestamp: ts,
            pm10_ug_m3,
            pm25_ug_m3,
            max_gust_mph,
            category,
        };
        self.dust_cells.insert(key, snapshot);
    }

    fn categorize_phx_dust(pm10: f32) -> PhxDustCategory {
        match pm10 {
            v if v < 500.0 => PhxDustCategory::Isolated1,
            v if v < 1_000.0 => PhxDustCategory::Isolated1,
            v if v < 2_500.0 => PhxDustCategory::Minor2,
            v if v < 4_000.0 => PhxDustCategory::Moderate3,
            v if v < 5_000.0 => PhxDustCategory::Major4,
            _ => PhxDustCategory::Extreme5,
        }
    }

    // -------- Indicator exports to L4 optimization + dashboards --------

    pub fn compute_portfolio_indicators(&self) -> CorridorPortfolioIndicators {
        let mut total_storage = 0.0;
        let mut total_awp = 0.0;
        let mut total_gw = 0.0;
        let mut total_colorado = 0.0;

        for src in self.sources.values() {
            let vol = src.storage_af.value.max(0.0);
            total_storage += vol;

            let is_awp = src
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case("awp") || t.eq_ignore_ascii_case("advanced-purified"));
            let is_gw = src
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case("groundwater"));
            let is_colorado = src
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case("colorado-portfolio"));

            if is_awp {
                total_awp += vol;
            }
            if is_gw {
                total_gw += vol;
            }
            if is_colorado {
                total_colorado += vol;
            }
        }

        let total_demand: f64 = self
            .demands
            .values()
            .map(|d| d.daily_demand_af.max(0.0))
            .sum();

        // Approximate reuse fraction: AWP storage vs daily demand (corridor view). [file:5]
        let awp_reuse_fraction = if total_demand > 1e-6 {
            (total_awp / total_demand).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let colorado_exposure_fraction = if total_storage > 1e-6 {
            (total_colorado / total_storage).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // AWG daily yield sum (AF/day).
        let total_awg_yield_af_per_day: f64 = self
            .awg_clusters
            .values()
            .map(|c| c.expected_yield_af.value.max(0.0))
            .sum();

        CorridorPortfolioIndicators {
            total_storage_af: total_storage,
            total_awp_storage_af: total_awp,
            total_groundwater_storage_af: total_gw,
            total_awg_yield_af_per_day,
            colorado_exposure_fraction,
            awp_reuse_fraction,
        }
    }

    pub fn compute_thermal_indicators(&self) -> ThermalIndicators {
        let mut miles_total = 0.0_f32;
        let mut reductions: Vec<f32> = Vec::new();

        for seg in self.cool_pavements.values() {
            miles_total += seg.miles;
            let reduction = (seg.baseline_temp_f - seg.surface_temp_f).max(0.0);
            reductions.push(reduction);
        }

        // Target corridor: 10.5–12°F surface reduction, ~1–2.5°F nighttime offset. [web:15]
        let avg_surface_reduction = if reductions.is_empty() {
            0.0
        } else {
            reductions.iter().sum::<f32>() / reductions.len() as f32
        };

        // Approximate nighttime offset as 0.2 * surface reduction (coarse heuristic).
        let avg_night_offset = avg_surface_reduction * 0.2_f32;

        ThermalIndicators {
            total_cool_pavement_miles: miles_total,
            avg_surface_reduction_f: avg_surface_reduction,
            avg_nighttime_offset_f: avg_night_offset,
        }
    }

    pub fn compute_dust_indicators(&self) -> DustIndicators {
        let mut max_cat = PhxDustCategory::Isolated1;
        let mut high_cells = 0usize;

        for cell in self.dust_cells.values() {
            if cell.category as i32 > max_cat as i32 {
                max_cat = cell.category;
            }
            if matches!(
                cell.category,
                PhxDustCategory::Moderate3
                    | PhxDustCategory::Major4
                    | PhxDustCategory::Extreme5
            ) {
                high_cells += 1;
            }
        }

        DustIndicators {
            max_category_seen: max_cat,
            cells_in_cat3_or_higher: high_cells,
        }
    }

    // -------- Heat- and dust-aware hints for machinery + planners --------

    /// Returns true if the corridor should enter "heat-derated" mode
    /// for machinery and logistics (Step 13), based on thermal assets and heat budgets.
    pub fn should_derate_for_heat(&self, ambient_temp_f: f32) -> bool {
        // Simple heuristic: if ambient >= 118°F and avg pavement reduction < 10°F,
        // treat corridor as heat-stressed. [file:5][web:15]
        let thermal = self.compute_thermal_indicators();
        ambient_temp_f >= 118.0 && thermal.avg_surface_reduction_f < 10.0
    }

    /// Returns the highest PHX-DUST category and whether equipment
    /// should be forced into sealed / protected mode.
    pub fn dust_mode_hint(&self) -> (PhxDustCategory, bool) {
        let dust = self.compute_dust_indicators();
        let sealed_needed = matches!(
            dust.max_category_seen,
            PhxDustCategory::Major4 | PhxDustCategory::Extreme5
        );
        (dust.max_category_seen, sealed_needed)
    }

    /// Quick view for water allocator – pairs corridor indicators with demand zones.
    pub fn export_water_allocation_view(
        &self,
    ) -> (
        Vec<CorridorSourceState>,
        Vec<CorridorDemandState>,
        CorridorPortfolioIndicators,
    ) {
        let sources: Vec<CorridorSourceState> = self.sources.values().cloned().collect();
        let demands: Vec<CorridorDemandState> = self.demands.values().cloned().collect();
        let indicators = self.compute_portfolio_indicators();
        (sources, demands, indicators)
    }
}

// ---------- Minimal smoke tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awg_yield_scales_with_rh_and_solar() {
        let mut core = PhxCorridorWhdCore::new();
        core.register_awg_cluster("SouthPhxRooftopAWG01", 100.0);

        let ts = SystemTime::now();
        core.update_awg_cluster_from_env("SouthPhxRooftopAWG01", 0.10, 800.0, 100.0, ts);
        let low = core
            .awg_clusters
            .get(&CorridorWaterSourceId::AtmosphericCluster(
                "SouthPhxRooftopAWG01",
            ))
            .unwrap()
            .expected_yield_af
            .value;

        core.update_awg_cluster_from_env("SouthPhxRooftopAWG01", 0.30, 1000.0, 100.0, ts);
        let high = core
            .awg_clusters
            .get(&CorridorWaterSourceId::AtmosphericCluster(
                "SouthPhxRooftopAWG01",
            ))
            .unwrap()
            .expected_yield_af
            .value;

        assert!(high > low);
    }

    #[test]
    fn dust_category_classification_works() {
        assert_eq!(PhxCorridorWhdCore::categorize_phx_dust(400.0), PhxDustCategory::Isolated1);
        assert_eq!(PhxCorridorWhdCore::categorize_phx_dust(1500.0), PhxDustCategory::Minor2);
        assert_eq!(PhxCorridorWhdCore::categorize_phx_dust(3000.0), PhxDustCategory::Moderate3);
        assert_eq!(PhxCorridorWhdCore::categorize_phx_dust(4500.0), PhxDustCategory::Major4);
        assert_eq!(PhxCorridorWhdCore::categorize_phx_dust(6000.0), PhxDustCategory::Extreme5);
    }

    #[test]
    fn thermal_indicators_respect_cool_pavement() {
        let mut core = PhxCorridorWhdCore::new();
        core.register_cool_pavement_segment(
            "CP-Test-01",
            5.0,
            0.35,
            150.0, // baseline
            vec!["arterial".into()],
        );
        let ts = SystemTime::now();
        core.update_cool_pavement_segment("CP-Test-01", 0.35, 138.0, 150.0, ts);

        let ind = core.compute_thermal_indicators();
        assert!(ind.avg_surface_reduction_f > 10.0 - 1e-3);
    }
}
