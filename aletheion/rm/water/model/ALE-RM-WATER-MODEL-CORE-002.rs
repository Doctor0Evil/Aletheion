// Core water state model for Aletheion Phoenix.
// Consumes normalized snapshots from ingestion engines and maintains
// an operational mirror of Phoenix's water inventory and flows.
//
// ERM Layers: L2 State Modeling (fed by L1 ingestion, serving L4 optimization).
// Language: Rust (no blacklisted terminology).

use std::collections::HashMap;
use std::time::SystemTime;

// --------- Shared IDs (kept minimal; ingestion-specific structs live elsewhere) ---------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaterSourceId {
    AwpPlant(&'static str),
    SurfaceReservoir(&'static str),
    GroundwaterBlock(&'static str),
    ImportedPortfolio(&'static str), // e.g., Colorado River contracts
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaterDemandZoneId {
    Neighborhood(&'static str),
    CoolingCorridor(&'static str),
    IndustrialCluster(&'static str),
    AgriculturalDistrict(&'static str),
    EcologicalReserve(&'static str),
}

#[derive(Debug, Clone)]
pub struct WaterVolume {
    /// Volume in acre-feet.
    pub acre_feet: f64,
}

impl WaterVolume {
    pub fn zero() -> Self {
        Self { acre_feet: 0.0 }
    }
    pub fn from_mgd(mgd: f64, days: f64) -> Self {
        // 1 MGD ≈ 3.068883 acre-feet/day
        Self {
            acre_feet: mgd * 3.068883 * days,
        }
    }
}

// --------- Portfolio and state structs ---------

#[derive(Debug, Clone)]
pub struct SourceState {
    pub id: WaterSourceId,
    pub last_update: Option<SystemTime>,
    pub current_storage: WaterVolume,
    pub max_storage: Option<WaterVolume>,
    pub inflow_rate_afy: f64,
    pub outflow_rate_afy: f64,
    pub quality_score: f32, // 0–1
    /// Portfolio tags, e.g. ["ColoradoRiver", "LocalGroundwater"]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DemandZoneState {
    pub id: WaterDemandZoneId,
    pub last_update: Option<SystemTime>,
    pub estimated_daily_demand_af: f64,
    pub vulnerability_score: f32, // 0–1 (for heat and socio-economic vulnerability)
    pub equity_weight: f32,       // ≥ 0; higher => prioritized in allocation
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub source: WaterSourceId,
    pub zone: WaterDemandZoneId,
    pub volume: WaterVolume,
    pub timestamp: SystemTime,
    pub scenario_tag: String,
}

// High-level portfolio indicators used by optimization and dashboards.
#[derive(Debug, Clone)]
pub struct PortfolioIndicators {
    pub total_storage_af: f64,
    pub total_awp_contribution_af: f64,
    pub total_groundwater_storage_af: f64,
    pub colorado_river_exposure_fraction: f64, // 0–1
    pub awp_reuse_fraction: f64,               // 0–1 relative to total demand
}

// --------- Water state model core ---------

pub struct WaterStateModel {
    sources: HashMap<WaterSourceId, SourceState>,
    demand_zones: HashMap<WaterDemandZoneId, DemandZoneState>,
    latest_allocations: Vec<AllocationRecord>,
}

impl WaterStateModel {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            demand_zones: HashMap::new(),
            latest_allocations: Vec::new(),
        }
    }

    // ------ Source & demand initialization helpers ------

    pub fn upsert_source(&mut self, state: SourceState) {
        self.sources.insert(state.id.clone(), state);
    }

    pub fn upsert_demand_zone(&mut self, state: DemandZoneState) {
        self.demand_zones.insert(state.id.clone(), state);
    }

    // ------ Update APIs called by ingestion layer ------

    /// Apply an updated storage/inflow snapshot for a given water source.
    pub fn update_source_snapshot(
        &mut self,
        id: WaterSourceId,
        storage_volume_af: f64,
        inflow_rate_afy: f64,
        outflow_rate_afy: f64,
        quality_score: f32,
        timestamp: SystemTime,
    ) {
        let entry = self.sources.entry(id.clone()).or_insert(SourceState {
            id,
            last_update: None,
            current_storage: WaterVolume::zero(),
            max_storage: None,
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 1.0,
            tags: Vec::new(),
        });
        entry.current_storage.acre_feet = storage_volume_af;
        entry.inflow_rate_afy = inflow_rate_afy;
        entry.outflow_rate_afy = outflow_rate_afy;
        entry.quality_score = quality_score;
        entry.last_update = Some(timestamp);
    }

    /// Update demand zone characteristics (usually slower cadence).
    pub fn update_demand_zone_snapshot(
        &mut self,
        id: WaterDemandZoneId,
        estimated_daily_demand_af: f64,
        vulnerability_score: f32,
        equity_weight: f32,
        timestamp: SystemTime,
    ) {
        let entry = self
            .demand_zones
            .entry(id.clone())
            .or_insert(DemandZoneState {
                id,
                last_update: None,
                estimated_daily_demand_af: 0.0,
                vulnerability_score: 0.0,
                equity_weight: 1.0,
                tags: Vec::new(),
            });
        entry.estimated_daily_demand_af = estimated_daily_demand_af;
        entry.vulnerability_score = vulnerability_score;
        entry.equity_weight = equity_weight;
        entry.last_update = Some(timestamp);
    }

    // ------ Allocation tracking (fed by optimization layer) ------

    pub fn record_allocation(&mut self, allocation: AllocationRecord) {
        self.latest_allocations.push(allocation);
        // In a full implementation, we'd also update per-source storage and
        // per-zone satisfaction coverage based on these records.
    }

    // ------ Derived indicators for optimization & dashboards ------

    pub fn compute_portfolio_indicators(&self) -> PortfolioIndicators {
        let mut total_storage_af = 0.0;
        let mut total_awp_storage_af = 0.0;
        let mut total_groundwater_storage_af = 0.0;
        let mut colorado_river_storage_af = 0.0;

        for s in self.sources.values() {
            let volume = s.current_storage.acre_feet;
            total_storage_af += volume;

            let is_awp = s
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case("awp") || t.eq_ignore_ascii_case("advanced_purified_water"));
            let is_groundwater = s
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case("groundwater"));
            let is_colorado = s
                .tags
                .iter()
                .any(|t| t.eq_ignore_ascii_case("colorado_river"));

            if is_awp {
                total_awp_storage_af += volume;
            }
            if is_groundwater {
                total_groundwater_storage_af += volume;
            }
            if is_colorado {
                colorado_river_storage_af += volume;
            }
        }

        let total_demand_af: f64 = self
            .demand_zones
            .values()
            .map(|z| z.estimated_daily_demand_af)
            .sum();

        // Simple reuse fraction proxy: awp storage / (demand + epsilon)
        let awp_reuse_fraction = if total_demand_af > 1e-6 {
            (total_awp_storage_af / total_demand_af).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let colorado_river_exposure_fraction = if total_storage_af > 1e-6 {
            (colorado_river_storage_af / total_storage_af).clamp(0.0, 1.0)
        } else {
            0.0
        };

        PortfolioIndicators {
            total_storage_af,
            total_awp_contribution_af: total_awp_storage_af,
            total_groundwater_storage_af,
            colorado_river_exposure_fraction,
            awp_reuse_fraction,
        }
    }

    /// Export a view suitable for optimization engines:
    /// ordered lists of sources and demand zones with their attributes.
    pub fn export_for_optimization(
        &self,
    ) -> (Vec<SourceState>, Vec<DemandZoneState>, PortfolioIndicators) {
        let sources: Vec<SourceState> = self.sources.values().cloned().collect();
        let zones: Vec<DemandZoneState> = self.demand_zones.values().cloned().collect();
        let indicators = self.compute_portfolio_indicators();
        (sources, zones, indicators)
    }
}

// --------- Minimal tests ---------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_indicators_compute_sensible_values() {
        let mut model = WaterStateModel::new();
        model.upsert_source(SourceState {
            id: WaterSourceId::AwpPlant("CaveCreekAWP"),
            last_update: None,
            current_storage: WaterVolume { acre_feet: 10_000.0 },
            max_storage: None,
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 0.99,
            tags: vec!["awp".into()],
        });
        model.upsert_source(SourceState {
            id: WaterSourceId::ImportedPortfolio("ColoradoPortfolio"),
            last_update: None,
            current_storage: WaterVolume { acre_feet: 20_000.0 },
            max_storage: None,
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 0.96,
            tags: vec!["colorado_river".into()],
        });
        model.upsert_demand_zone(DemandZoneState {
            id: WaterDemandZoneId::Neighborhood("HeatVulnerableZone1"),
            last_update: None,
            estimated_daily_demand_af: 500.0,
            vulnerability_score: 0.9,
            equity_weight: 2.0,
            tags: vec!["heat_vulnerable".into()],
        });

        let indicators = model.compute_portfolio_indicators();
        assert!(indicators.total_storage_af > 0.0);
        assert!(indicators.colorado_river_exposure_fraction >= 0.0);
        assert!(indicators.colorado_river_exposure_fraction <= 1.0);
        assert!(indicators.awp_reuse_fraction >= 0.0);
        assert!(indicators.awp_reuse_fraction <= 1.0);
    }
}
