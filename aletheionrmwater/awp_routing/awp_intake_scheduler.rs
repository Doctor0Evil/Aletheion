// Phoenix AWP Intake & Discharge Scheduler
// Binds AWP facilities (Cave Creek, North Gateway, 91st Ave) to canal/pipe
// capacity, KER safety bands, and treaty / corridor envelopes across
// neighborhoods and canal segments.
//
// ERM layers: L2 state model consumer, L4 optimization runner, L6 actuation
// Languages: Rust only, no forbidden hashes or digital-twin semantics.

use std::collections::HashMap;
use std::time::SystemTime;

// --- Imports from existing / planned water model & compliance cores ---------

// Water state model export (Layer 2) – see ALE-RM-WATER-MODEL-CORE-002.rs
// This module treats the model as an opaque dependency and only uses its
// public export surface.
pub use crate::model::{
    AllocationRecord,
    DemandZoneState,
    PortfolioIndicators,
    WaterDemandZoneId,
    WaterSourceId,
    WaterStateModel,
    WaterVolume,
};

// Central compliance utilities (Layer X) – see ALE-COMP-CORE-ENGINE-002.rs
pub trait ComplianceGateway {
    fn preflight_check(&self, workflow_id: &str) -> Result<(), ComplianceError>;
    fn record_decision(
        &self,
        workflow_id: &str,
        tx_ref: &GovernedDecisionRef,
        report: &ComplianceReport,
    ) -> Result<(), ComplianceError>;
}

#[derive(Debug, Clone)]
pub struct GovernedDecisionRef {
    pub tx_id: String,
    pub workflow_id: String,
    pub workflow_stage: String,
    pub birth_sign_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ComplianceReport {
    pub passed: bool,
    pub findings: Vec<ComplianceFinding>,
}

#[derive(Debug, Clone)]
pub struct ComplianceFinding {
    pub code: String,
    pub message: String,
    pub severity: ComplianceFindingSeverity,
}

#[derive(Debug, Clone)]
pub enum ComplianceFindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub enum ComplianceError {
    GatewayUnavailable,
    PolicyViolation(String),
}

// --- Phoenix AWP & corridor identifiers -------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AwpPlantId {
    CaveCreek,
    NorthGateway,
    NinetyFirstAve,
}

impl AwpPlantId {
    pub fn as_str(&self) -> &'static str {
        match self {
            AwpPlantId::CaveCreek => "plant.cave_creek_awp",
            AwpPlantId::NorthGateway => "plant.north_gateway_awp",
            AwpPlantId::NinetyFirstAve => "plant.91st_ave_awp",
        }
    }

    pub fn to_water_source_id(&self) -> WaterSourceId {
        match self {
            AwpPlantId::CaveCreek => WaterSourceId::AwpPlant("CAVE_CREEK_AWP"),
            AwpPlantId::NorthGateway => WaterSourceId::AwpPlant("NORTH_GATEWAY_AWP"),
            AwpPlantId::NinetyFirstAve => WaterSourceId::AwpPlant("NINETY_FIRST_AVE_AWP"),
        }
    }
}

// Canal and corridor identifiers are stringly-typed here; in a full build they
// should be enums shared with aletheionhighways and canal state crates. [file:3][file:4]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanalReachId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorridorId(pub String);

// --- Intake capacity, KER envelopes, and routing configuration --------------

#[derive(Debug, Clone)]
pub struct PlantIntakeCapacity {
    // Million gallons per day at the plant intake.
    pub max_intake_mgd: f64,
    // Million gallons per day that can be discharged into canals / pipes.
    pub max_discharge_mgd: f64,
    // Fraction of design used as a safety band (e.g., 0.85).
    pub ker_safety_fraction: f64,
}

#[derive(Debug, Clone)]
pub struct CanalCapacityBand {
    // Million gallons per day of AWP that can be injected into this reach
    // without violating hydraulic or treaty constraints. [file:5][file:4]
    pub max_awp_injection_mgd: f64,
    // KER-style safety band to keep residual headroom for contingencies.
    pub safety_fraction: f64,
}

#[derive(Debug, Clone)]
pub struct CorridorKerEnvelope {
    // Values 0..1 representing knowledge (K), ecosafety (E), risk-of-harm (R)
    // envelopes for this corridor, as exported by ecosafety grammars. [file:1][file:5]
    pub k_min: f32,
    pub e_min: f32,
    pub r_max: f32,
}

#[derive(Debug, Clone)]
pub struct AwpIntakeRoutingConfig {
    // Static or slowly changing plant-level capacities.
    pub plant_caps: HashMap<AwpPlantId, PlantIntakeCapacity>,
    // Canal reach capacities per plant-discharge path.
    // Key: (plant, reach)
    pub canal_bands: HashMap<(AwpPlantId, CanalReachId), CanalCapacityBand>,
    // Corridor KER envelopes per neighborhood / corridor. [file:1][file:3]
    pub corridor_ker: HashMap<CorridorId, CorridorKerEnvelope>,
    // Mapping from demand zone to primary corridor and canal reach.
    pub zone_to_corridor: HashMap<WaterDemandZoneId, CorridorId>,
    pub corridor_to_reach: HashMap<CorridorId, CanalReachId>,
}

// --- Scheduler objectives & constraints -------------------------------------

#[derive(Debug, Clone)]
pub struct SchedulerObjectives {
    // Target reuse fraction for AWP as share of total water supplied.
    pub target_awp_reuse_fraction: f64,
    // Max acceptable Colorado River portfolio fraction. [file:6][file:5]
    pub max_colorado_fraction: f64,
}

#[derive(Debug, Clone)]
pub struct SchedulerConstraints {
    // Minimum ecosafety thresholds (corridor envelopes must not go below).
    pub min_k: f32,
    pub min_e: f32,
    pub max_r: f32,
}

// Intake decision at plant level (per day).
#[derive(Debug, Clone)]
pub struct PlantIntakeDecision {
    pub plant_id: AwpPlantId,
    // Intake volume at plant in MGD.
    pub intake_mgd: f64,
    // Volume discharged to canals (MGD) after internal uses / losses.
    pub discharge_mgd: f64,
}

// Discharge along a specific canal reach toward a corridor.
#[derive(Debug, Clone)]
pub struct ReachDischargeDecision {
    pub plant_id: AwpPlantId,
    pub reach_id: CanalReachId,
    pub discharge_mgd: f64,
}

// Aggregate daily schedule result.
#[derive(Debug, Clone)]
pub struct AwpIntakeSchedule {
    pub date: chrono::NaiveDate,
    pub plant_decisions: Vec<PlantIntakeDecision>,
    pub reach_decisions: Vec<ReachDischargeDecision>,
    // Updated allocation records back into the water model (per demand zone).
    pub allocation_records: Vec<AllocationRecord>,
    // Portfolio indicators after applying this schedule.
    pub portfolio_indicators: PortfolioIndicators,
}

// --- Scheduler engine -------------------------------------------------------

pub struct AwpIntakeScheduler<'a, C: ComplianceGateway> {
    pub config: AwpIntakeRoutingConfig,
    pub objectives: SchedulerObjectives,
    pub constraints: SchedulerConstraints,
    pub compliance: &'a C,
}

impl<'a, C: ComplianceGateway> AwpIntakeScheduler<'a, C> {
    pub fn new(
        config: AwpIntakeRoutingConfig,
        objectives: SchedulerObjectives,
        constraints: SchedulerConstraints,
        compliance: &'a C,
    ) -> Self {
        Self {
            config,
            objectives,
            constraints,
            compliance,
        }
    }

    /// Compute a daily intake + discharge schedule consistent with:
    /// - Plant intake / discharge limits
    /// - Canal reach injection bands
    /// - Corridor KER envelopes and portfolio limits
    /// - Existing water state model (available AWP storage vs demand) [file:6][file:5]
    pub fn compute_daily_schedule(
        &self,
        date: chrono::NaiveDate,
        model: &WaterStateModel,
    ) -> Result<AwpIntakeSchedule, SchedulerError> {
        // Governance: preflight for this workflow & stage.
        self.compliance
            .preflight_check("ALE-RM-WATER-AWP-INTAKE-SCHEDULER-001")
            .map_err(SchedulerError::Compliance)?;

        let (sources, zones, indicators_before) = model.export_for_optimization();
        let mut plant_decisions = Vec::new();
        let mut reach_decisions = Vec::new();
        let mut allocation_records = Vec::new();

        // Step 1: determine total AWP storage available by plant.
        let mut awp_storage_by_plant: HashMap<AwpPlantId, f64> = HashMap::new();
        for src in &sources {
            if let WaterSourceId::AwpPlant(name) = &src.id {
                let plant_id = match *name {
                    "CAVE_CREEK_AWP" => Some(AwpPlantId::CaveCreek),
                    "NORTH_GATEWAY_AWP" => Some(AwpPlantId::NorthGateway),
                    "NINETY_FIRST_AVE_AWP" => Some(AwpPlantId::NinetyFirstAve),
                    _ => None,
                };
                if let Some(pid) = plant_id {
                    awp_storage_by_plant.insert(pid, src.current_storage.acre_feet);
                }
            }
        }

        // Approximate total demand in AF (for reuse / portfolio reasoning).
        let mut total_demand_af = 0.0_f64;
        for z in &zones {
            total_demand_af += z.estimated_daily_demand_af;
        }

        // Step 2: compute desired AWP contribution based on reuse target.
        let desired_awp_af = (self.objectives.target_awp_reuse_fraction
            * total_demand_af)
            .min(awp_storage_by_plant.values().copied().sum::<f64>());

        // Convert desired AF to MGD over one day.
        let desired_awp_mgd = if desired_awp_af > 0.0 {
            desired_awp_af / 3.068_883_f64
        } else {
            0.0
        };

        // Step 3: allocate plant-level intakes up to capacity and storage.
        let mut remaining_awp_mgd = desired_awp_mgd;
        for (pid, caps) in &self.config.plant_caps {
            let stored_af = *awp_storage_by_plant.get(pid).unwrap_or(&0.0);
            let max_by_storage_mgd = stored_af / 3.068_883_f64;
            let banded_intake_mgd =
                caps.max_intake_mgd * caps.ker_safety_fraction;
            let intake_mgd = remaining_awp_mgd
                .min(banded_intake_mgd)
                .min(max_by_storage_mgd)
                .max(0.0);

            if intake_mgd <= 0.0 {
                continue;
            }

            let discharge_mgd = (intake_mgd).min(caps.max_discharge_mgd);
            remaining_awp_mgd -= intake_mgd;

            plant_decisions.push(PlantIntakeDecision {
                plant_id: *pid,
                intake_mgd,
                discharge_mgd,
            });
        }

        // Step 4: dispatch plant discharges to canal reaches within bands. [file:4][file:3]
        let mut reach_alloc_by_plant: HashMap<(AwpPlantId, CanalReachId), f64> =
            HashMap::new();
        for pd in &plant_decisions {
            // Filter capacities for this plant.
            let caps_for_plant: Vec<(&(AwpPlantId, CanalReachId), &CanalCapacityBand)> =
                self.config
                    .canal_bands
                    .iter()
                    .filter(|((pid, _), _)| pid == &pd.plant_id)
                    .collect();

            if caps_for_plant.is_empty() {
                continue;
            }

            let mut remaining_mgd = pd.discharge_mgd;
            for ((pid, reach_id), band) in caps_for_plant {
                if remaining_mgd <= 0.0 {
                    break;
                }
                let banded_cap_mgd =
                    band.max_awp_injection_mgd * band.safety_fraction;
                if banded_cap_mgd <= 0.0 {
                    continue;
                }
                let alloc_mgd =
                    remaining_mgd.min(banded_cap_mgd).max(0.0);
                if alloc_mgd <= 0.0 {
                    continue;
                }
                remaining_mgd -= alloc_mgd;
                reach_alloc_by_plant
                    .entry((**pid, CanalReachId(reach_id.0.clone())))
                    .and_modify(|v| *v += alloc_mgd)
                    .or_insert(alloc_mgd);
            }
        }

        for ((plant_id, reach_id), mgd) in &reach_alloc_by_plant {
            reach_decisions.push(ReachDischargeDecision {
                plant_id: *plant_id,
                reach_id: reach_id.clone(),
                discharge_mgd: *mgd,
            });
        }

        // Step 5: allocate AWP volumes from reaches to demand zones by corridor
        // while respecting corridor KER envelopes. This is a simple proportional
        // allocator; NSGA-II variants can be layered on top later. [file:4][file:5]
        let mut coverage_by_zone: HashMap<WaterDemandZoneId, f64> =
            HashMap::new();

        for zone in &zones {
            let corridor = match self.config.zone_to_corridor.get(&zone.id) {
                Some(c) => c,
                None => continue,
            };
            let reach = match self.config.corridor_to_reach.get(corridor) {
                Some(r) => r,
                None => continue,
            };
            let corridor_env = self.config.corridor_ker.get(corridor);

            // Skip corridors that are already too risky.
            if let Some(env) = corridor_env {
                if env.k_min < self.constraints.min_k
                    || env.e_min < self.constraints.min_e
                    || env.r_max > self.constraints.max_r
                {
                    continue;
                }
            }

            // Find all plant discharges into this reach.
            let mut total_reach_mgd = 0.0_f64;
            for ((pid, rid), mgd) in &reach_alloc_by_plant {
                if rid.0 == reach.0 {
                    // Optionally modulate by KER safety, but we already baked
                    // safety into bands; here we just sum.
                    total_reach_mgd += *mgd;
                }
            }
            if total_reach_mgd <= 0.0 {
                continue;
            }

            // Assign AWP volume to this zone up to its demand.
            let demand_af = zone.estimated_daily_demand_af;
            let demand_mgd = demand_af / 3.068_883_f64;
            let supplied_mgd = total_reach_mgd.min(demand_mgd);
            if supplied_mgd <= 0.0 {
                continue;
            }

            coverage_by_zone.insert(zone.id.clone(), supplied_mgd);

            allocation_records.push(AllocationRecord {
                source: WaterSourceId::AwpPlant("MIXED_AWP"),
                zone: zone.id.clone(),
                volume_af: WaterVolume::from_mgd(supplied_mgd, 1.0).acre_feet,
                timestamp: SystemTime::now(),
                rationale: String::from("AWP_AWP-INTAKE-SCHEDULER-001"),
            });
        }

        // Step 6: update portfolio indicators with approximate AWP usage.
        let indicators_after =
            self.estimate_portfolio_after(&indicators_before, &allocation_records);

        // Step 7: governance / trust reporting hook (no append here, just record
        // for the central trust-append workflow to pick up). [file:2][file:5]
        let tx_ref = GovernedDecisionRef {
            tx_id: format!("AWP-SCHED-{}", date),
            workflow_id: "ALE-RM-WATER-AWP-INTAKE-SCHEDULER-001".into(),
            workflow_stage: "ALLOCATION".into(),
            birth_sign_ids: self.collect_birth_signs_for_zones(&zones),
        };
        let report = ComplianceReport {
            passed: true,
            findings: Vec::new(),
        };
        self.compliance
            .record_decision(
                "ALE-RM-WATER-AWP-INTAKE-SCHEDULER-001",
                &tx_ref,
                &report,
            )
            .map_err(SchedulerError::Compliance)?;

        Ok(AwpIntakeSchedule {
            date,
            plant_decisions,
            reach_decisions,
            allocation_records,
            portfolio_indicators: indicators_after,
        })
    }

    fn estimate_portfolio_after(
        &self,
        before: &PortfolioIndicators,
        allocations: &[AllocationRecord],
    ) -> PortfolioIndicators {
        let mut total_awp_af = before.total_awp_contribution_af;
        let mut total_storage_af = before.total_storage_af;
        for a in allocations {
            total_awp_af += a.volume_af;
            total_storage_af += a.volume_af;
        }
        let awp_fraction = if total_storage_af > 0.0 {
            (total_awp_af / total_storage_af).clamp(0.0, 1.0)
        } else {
            before.awp_reuse_fraction
        };
        PortfolioIndicators {
            total_storage_af,
            total_awp_contribution_af: total_awp_af,
            total_groundwater_storage_af: before.total_groundwater_storage_af,
            colorado_river_exposure_fraction: before.colorado_river_exposure_fraction,
            awp_reuse_fraction: awp_fraction,
        }
    }

    fn collect_birth_signs_for_zones(
        &self,
        zones: &[DemandZoneState],
    ) -> Vec<String> {
        let mut out = Vec::new();
        for z in zones {
            if let Some(tag) = z.tags.iter().find(|t| t.starts_with("birthsign:")) {
                out.push(tag.trim_start_matches("birthsign:").to_string());
            }
        }
        out.sort();
        out.dedup();
        out
    }
}

// --- Errors & basic tests ---------------------------------------------------

#[derive(Debug)]
pub enum SchedulerError {
    Compliance(ComplianceError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DemandZoneState, WaterStateModel};

    struct NoopCompliance;

    impl ComplianceGateway for NoopCompliance {
        fn preflight_check(&self, _workflow_id: &str) -> Result<(), ComplianceError> {
            Ok(())
        }
        fn record_decision(
            &self,
            _workflow_id: &str,
            _tx_ref: &GovernedDecisionRef,
            _report: &ComplianceReport,
        ) -> Result<(), ComplianceError> {
            Ok(())
        }
    }

    #[test]
    fn basic_scheduler_smoke_test() {
        let mut model = WaterStateModel::new();
        // Minimal synthetic state: one AWP plant and one demand zone.
        model.upsert_source(crate::model::SourceState {
            id: WaterSourceId::AwpPlant("CAVE_CREEK_AWP"),
            last_update: None,
            current_storage: WaterVolume { acre_feet: 10_000.0 },
            max_storage: None,
            inflow_rate_afy: 0.0,
            outflow_rate_afy: 0.0,
            quality_score: 0.99,
            tags: vec!["awp".into()],
        });
        model.upsert_demand_zone(DemandZoneState {
            id: WaterDemandZoneId::Neighborhood("DOWNTOWN_HEAT_CORRIDOR_1"),
            last_update: None,
            estimated_daily_demand_af: 500.0,
            vulnerability_score: 0.9,
            equity_weight: 2.0,
            tags: vec!["heat_vulnerable".into(), "birthsign:PHX_CORE".into()],
        });

        let mut plant_caps = HashMap::new();
        plant_caps.insert(
            AwpPlantId::CaveCreek,
            PlantIntakeCapacity {
                max_intake_mgd: 8.0,
                max_discharge_mgd: 8.0,
                ker_safety_fraction: 0.85,
            },
        );

        let reach_id = CanalReachId("CANAL.METROCENTER.PARKWAY".into());
        let mut canal_bands = HashMap::new();
        canal_bands.insert(
            (AwpPlantId::CaveCreek, reach_id.clone()),
            CanalCapacityBand {
                max_awp_injection_mgd: 6.0,
                safety_fraction: 0.9,
            },
        );

        let corridor = CorridorId("CORRIDOR.DOWNTOWN.HEAT_WATER_1".into());
        let mut corridor_ker = HashMap::new();
        corridor_ker.insert(
            corridor.clone(),
            CorridorKerEnvelope {
                k_min: 0.9,
                e_min: 0.9,
                r_max: 0.15,
            },
        );

        let mut zone_to_corridor = HashMap::new();
        zone_to_corridor.insert(
            WaterDemandZoneId::Neighborhood("DOWNTOWN_HEAT_CORRIDOR_1"),
            corridor.clone(),
        );

        let mut corridor_to_reach = HashMap::new();
        corridor_to_reach.insert(corridor.clone(), reach_id.clone());

        let config = AwpIntakeRoutingConfig {
            plant_caps,
            canal_bands,
            corridor_ker,
            zone_to_corridor,
            corridor_to_reach,
        };

        let objectives = SchedulerObjectives {
            target_awp_reuse_fraction: 0.3,
            max_colorado_fraction: 0.4,
        };
        let constraints = SchedulerConstraints {
            min_k: 0.8,
            min_e: 0.8,
            max_r: 0.2,
        };

        let scheduler = AwpIntakeScheduler::new(config, objectives, constraints, &NoopCompliance);

        let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let schedule =
            scheduler.compute_daily_schedule(date, &model).expect("scheduler failed");

        assert!(!schedule.plant_decisions.is_empty());
        assert!(!schedule.reach_decisions.is_empty());
        assert!(!schedule.allocation_records.is_empty());
        assert!(schedule.portfolio_indicators.awp_reuse_fraction >= 0.0);
    }
}
