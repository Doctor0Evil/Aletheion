// Managed Aquifer Recharge (MAR) shard types and CEIM-style mass-balance kernel
// for Phoenix-class desert basins (≈30 m × 4 m vault modules).
//
// ERM Layers: L2 State Modeling, L4 Optimization
// Domains: water, mar, cyboquatic
//
// This file is designed to be called by:
// - ALE-RM-GW-STATE-001.rs (aquifer state model)
// - ALE-RM-WATER-ALLOCATION-001.aln runners
// - HeatWaterTree engine bindings for water-thermal co-optimization.
//
// It assumes ecosafety grammar types/contracts exist in:
// - aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-TYPES-001.rs
// - aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-CONTRACTS-001.rs
//
// No blacklisted terminology is used; "state model" and "operational mirror"
// are the only allowed descriptors for system representations.

use std::collections::HashMap;
use std::time::Duration;

// -------- Ecosafety imports (interfaces only; concrete paths wired by build system) --------

use crate::erm::ecosafety::types::{
    RiskCoord,
    RiskVector,
    Corridor,
    CorridorEvalResult,
    CorridorStatus,
    CyboquaticNodeEcosafety,
};
use crate::erm::ecosafety::contracts::{
    require_corridors,
    eval_corridor,
    decide_node_action,
    NodeAction,
    check_lyapunov,
};

// -------- Core MAR geometry and identification --------

/// Identifier for a MAR vault module serving Phoenix-class basins.
/// Example: "MAR_DC_CORE_01".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarVaultId(pub String);

/// High-level classification of MAR vault placement relative to Phoenix.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarZone {
    DowntownCore,
    MetroCenterCorridor,
    CanalAdjacency,
    PeripheralBasin,
    ExperimentalSite,
}

/// Physical geometry for a MAR vault module.
/// Dimensions are in meters; ksat is saturated hydraulic conductivity (m/s).
#[derive(Debug, Clone)]
pub struct MarGeometry {
    pub length_m: f64,
    pub width_m: f64,
    pub depth_m: f64,
    pub ksat_m_per_s: f64,
}

impl MarGeometry {
    /// Default geometry for a 30 m × 4 m × 3 m vault module.
    pub fn default_desert_module() -> Self {
        Self {
            length_m: 30.0,
            width_m: 4.0,
            depth_m: 3.0,
            ksat_m_per_s: 1.0e-5, // conservative value for engineered media
        }
    }

    /// Volume in cubic meters.
    pub fn volume_m3(&self) -> f64 {
        self.length_m * self.width_m * self.depth_m
    }

    /// Footprint area in square meters.
    pub fn area_m2(&self) -> f64 {
        self.length_m * self.width_m
    }
}

// -------- Contaminant and thermal state fields (rx coordinates) --------

/// MAR contaminant / thermal fields, aligned with rx coordinates.
#[derive(Debug, Clone)]
pub struct MarQualityState {
    /// PFAS concentration (ng/L).
    pub pfas_ng_per_l: f64,
    /// Pharmaceutical / endocrine-disruptor proxy (ng/L or µg/L).
    pub pharma_ng_per_l: f64,
    /// Total oxidized nitrogen (mg/L).
    pub nutrients_mg_per_l: f64,
    /// Dissolved oxygen (mg/L).
    pub do_mg_per_l: f64,
    /// Water temperature (°C).
    pub temp_c: f64,
    /// Fouling index (dimensionless 0–1, higher = more fouled).
    pub fouling_index: f64,
    /// Surcharge factor (dimensionless, >1.0 indicates excess hydraulic loading).
    pub surcharge_factor: f64,
}

impl MarQualityState {
    pub fn new() -> Self {
        Self {
            pfas_ng_per_l: 0.0,
            pharma_ng_per_l: 0.0,
            nutrients_mg_per_l: 0.0,
            do_mg_per_l: 8.0,
            temp_c: 20.0,
            fouling_index: 0.0,
            surcharge_factor: 1.0,
        }
    }
}

// -------- MAR shard state --------

/// Static-designed corridors for a MAR vault, expressed as a RiskVector/Corridor.
#[derive(Debug, Clone)]
pub struct MarCorridorProfile {
    pub corridor: Corridor,
}

/// Dynamic MAR vault state shard suitable for inclusion in the water state model.
#[derive(Debug, Clone)]
pub struct MarVaultShard {
    pub id: MarVaultId,
    pub zone: MarZone,
    pub geometry: MarGeometry,

    // Operational mirror fields:
    pub inflow_m3_per_h: f64,
    pub outflow_m3_per_h: f64,
    pub recharge_m3_per_h: f64,

    pub quality: MarQualityState,

    // Ecosafety integration:
    pub ecosafety: CyboquaticNodeEcosafety,
    pub corridor_profile: MarCorridorProfile,
}

impl MarVaultShard {
    /// Construct a new MAR shard with default geometry and a given ecosafety profile.
    pub fn new(id: &str, zone: MarZone, ecosafety: CyboquaticNodeEcosafety, corridor: Corridor) -> Self {
        Self {
            id: MarVaultId(id.to_string()),
            zone,
            geometry: MarGeometry::default_desert_module(),
            inflow_m3_per_h: 0.0,
            outflow_m3_per_h: 0.0,
            recharge_m3_per_h: 0.0,
            quality: MarQualityState::new(),
            ecosafety,
            corridor_profile: MarCorridorProfile { corridor },
        }
    }
}

// -------- CEIM-style mass-balance kernel --------

/// External boundary condition for the MAR shard (linked to aquifer or canal models).
#[derive(Debug, Clone)]
pub struct MarBoundary {
    pub upstream_inflow_m3_per_h: f64,
    pub upstream_quality: MarQualityState,
    pub downstream_head_m: f64,
    pub aquifer_head_m: f64,
}

/// Mass-balance step result for a MAR shard over a time step.
#[derive(Debug, Clone)]
pub struct MarStepResult {
    pub updated_shard: MarVaultShard,
    pub recharge_m3: f64,
    pub inflow_m3: f64,
    pub outflow_m3: f64,
    pub ecosafety_eval: CorridorEvalResult,
    pub node_action: NodeAction,
}

/// CEIM-style mass-balance update for a single time step.
///
/// This function:
/// - Updates inflow/outflow/recharge based on simple hydraulic approximations.
/// - Updates quality fields using conservative mixing and simple first-order sinks.
/// - Projects the updated state into a RiskVector and runs ecosafety evaluation.
/// - Returns a NodeAction (Normal/Derate/Stop) that callers MUST respect.
///
/// dt: time step duration.
/// boundary: hydrological boundary conditions (from canals/aquifer).
/// shard: current MAR vault shard state.
pub fn mar_step_ceim(
    dt: Duration,
    boundary: &MarBoundary,
    shard: &MarVaultShard,
) -> MarStepResult {
    // 1) Ensure node declares ecosafety corridors.
    let _ = require_corridors(&shard.ecosafety)
        .expect("MAR shard missing ecosafety corridors; CI should have caught this");

    let hours = dt.as_secs_f64() / 3600.0;
    let mut updated = shard.clone();

    // 2) Hydraulic approximation (very simple, placeholder CEIM kernel).
    // Inflow is bounded by upstream and design; recharge limited by Ksat and area.
    let max_recharge_m3_per_h = shard.geometry.ksat_m_per_s
        * shard.geometry.area_m2()
        * 3600.0; // convert m^3/s to m^3/h

    let inflow_m3_per_h = boundary.upstream_inflow_m3_per_h;
    let recharge_m3_per_h = inflow_m3_per_h.min(max_recharge_m3_per_h);
    let outflow_m3_per_h = (inflow_m3_per_h - recharge_m3_per_h).max(0.0);

    updated.inflow_m3_per_h = inflow_m3_per_h;
    updated.recharge_m3_per_h = recharge_m3_per_h;
    updated.outflow_m3_per_h = outflow_m3_per_h;

    let inflow_m3 = inflow_m3_per_h * hours;
    let recharge_m3 = recharge_m3_per_h * hours;
    let outflow_m3 = outflow_m3_per_h * hours;

    // 3) Simple conservative mixing for quality fields (no complex chemistry here).
    // Mass_in + Mass_existing - Mass_out - Mass_recharge
    // For v1, assume vault behaves as a continuously stirred tank reactor.

    let vol_vault_m3 = shard.geometry.volume_m3().max(1.0);
    let vol_new_m3 = (vol_vault_m3 + inflow_m3 - outflow_m3 - recharge_m3).max(1.0);

    let mix = |existing: f64, inflow: f64| -> f64 {
        let mass_existing = existing * vol_vault_m3;
        let mass_in = inflow * inflow_m3;
        let mass_total = mass_existing + mass_in;
        mass_total / vol_new_m3
    };

    let q_in = &boundary.upstream_quality;
    let q_old = &shard.quality;

    let mut q_new = MarQualityState::new();
    q_new.pfas_ng_per_l = mix(q_old.pfas_ng_per_l, q_in.pfas_ng_per_l);
    q_new.pharma_ng_per_l = mix(q_old.pharma_ng_per_l, q_in.pharma_ng_per_l);
    q_new.nutrients_mg_per_l = mix(q_old.nutrients_mg_per_l, q_in.nutrients_mg_per_l);
    q_new.do_mg_per_l = mix(q_old.do_mg_per_l, q_in.do_mg_per_l);
    q_new.temp_c = mix(q_old.temp_c, q_in.temp_c);

    // Fouling index and surcharge are driven by loading; simple heuristic v1.
    let load_factor = (inflow_m3_per_h / max_recharge_m3_per_h).min(2.0);
    q_new.fouling_index = (q_old.fouling_index + 0.05 * load_factor * hours).min(1.0);
    q_new.surcharge_factor = (1.0 + (load_factor - 1.0).max(0.0)).max(1.0);

    updated.quality = q_new;

    // 4) Map MAR quality state into a RiskVector (rx).
    let risk_vec = build_mar_risk_vector(&updated);

    // 5) Evaluate ecosafety corridor (single profile for now).
    let ecosafety_eval = eval_corridor(&updated.corridor_profile.corridor, &risk_vec);

    // 6) Decide node action (Normal/Derate/Stop).
    let node_action = decide_node_action(&ecosafety_eval);

    // 7) Optionally, compute and check Lyapunov residual if available.
    if let Some(vt) = &ecosafety_eval.vt {
        let _stable = check_lyapunov(vt);
        // Callers may log or enforce additional policy based on _stable.
    }

    MarStepResult {
        updated_shard: updated,
        recharge_m3,
        inflow_m3,
        outflow_m3,
        ecosafety_eval,
        node_action,
    }
}

// -------- RiskVector construction helpers --------

/// Map MAR quality fields into a RiskVector aligned with ecosafety grammar.
/// For v1, safe bands are expected to be encoded in the Corridor; this function
/// only fills current values, names must match Corridor coords.
pub fn build_mar_risk_vector(shard: &MarVaultShard) -> RiskVector {
    let q = &shard.quality;

    let coords = vec![
        RiskCoord {
            name: "PFAS".to_string(),
            value: q.pfas_ng_per_l,
            min_safe: 0.0,
            max_safe: 10.0, // example band; exact values defined by ecosafety policy
        },
        RiskCoord {
            name: "Pharma".to_string(),
            value: q.pharma_ng_per_l,
            min_safe: 0.0,
            max_safe: 100.0,
        },
        RiskCoord {
            name: "Nutrients".to_string(),
            value: q.nutrients_mg_per_l,
            min_safe: 0.0,
            max_safe: 5.0,
        },
        RiskCoord {
            name: "DO".to_string(),
            value: q.do_mg_per_l,
            min_safe: 4.0,
            max_safe: 12.0,
        },
        RiskCoord {
            name: "TempC".to_string(),
            value: q.temp_c,
            min_safe: 10.0,
            max_safe: 28.0,
        },
        RiskCoord {
            name: "FoulingIndex".to_string(),
            value: q.fouling_index,
            min_safe: 0.0,
            max_safe: 0.7,
        },
        RiskCoord {
            name: "SurchargeFactor".to_string(),
            value: q.surcharge_factor,
            min_safe: 1.0,
            max_safe: 1.2,
        },
    ];

    RiskVector {
        id: format!("MAR_{}", shard.id.0),
        coords,
    }
}

// -------- MAR shard collection for Phoenix regions --------

/// Collection of MAR shards keyed by identifier; suitable for embedding into
/// the water state model's operational mirror for Phoenix.
#[derive(Debug, Default)]
pub struct MarShardRegistry {
    pub shards: HashMap<MarVaultId, MarVaultShard>,
}

impl MarShardRegistry {
    pub fn new() -> Self {
        Self {
            shards: HashMap::new(),
        }
    }

    pub fn insert(&mut self, shard: MarVaultShard) {
        self.shards.insert(shard.id.clone(), shard);
    }

    pub fn get(&self, id: &MarVaultId) -> Option<&MarVaultShard> {
        self.shards.get(id)
    }

    pub fn get_mut(&mut self, id: &MarVaultId) -> Option<&mut MarVaultShard> {
        self.shards.get_mut(id)
    }

    /// Step all MAR shards with the same boundary for a given region (v1).
    /// In production this would accept per-vault boundaries.
    pub fn step_all(
        &mut self,
        dt: Duration,
        boundary: &MarBoundary,
    ) -> HashMap<MarVaultId, MarStepResult> {
        let mut results = HashMap::new();
        let ids: Vec<MarVaultId> = self.shards.keys().cloned().collect();
        for id in ids {
            if let Some(shard) = self.shards.get(&id).cloned() {
                let step = mar_step_ceim(dt, boundary, &shard);
                if let Some(s_mut) = self.shards.get_mut(&id) {
                    *s_mut = step.updated_shard.clone();
                }
                results.insert(id, step);
            }
        }
        results
    }
}

// -------- Minimal tests (can be expanded in dedicated test modules) --------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erm::ecosafety::types::{LyapunovResidual};

    fn dummy_corridor() -> Corridor {
        let rv = RiskVector {
            id: "MAR_TEST".to_string(),
            coords: vec![
                RiskCoord { name: "PFAS".to_string(), value: 0.0, min_safe: 0.0, max_safe: 10.0 },
                RiskCoord { name: "Pharma".to_string(), value: 0.0, min_safe: 0.0, max_safe: 100.0 },
                RiskCoord { name: "Nutrients".to_string(), value: 0.0, min_safe: 0.0, max_safe: 5.0 },
                RiskCoord { name: "DO".to_string(), value: 8.0, min_safe: 4.0, max_safe: 12.0 },
                RiskCoord { name: "TempC".to_string(), value: 20.0, min_safe: 10.0, max_safe: 28.0 },
                RiskCoord { name: "FoulingIndex".to_string(), value: 0.0, min_safe: 0.0, max_safe: 0.7 },
                RiskCoord { name: "SurchargeFactor".to_string(), value: 1.0, min_safe: 1.0, max_safe: 1.2 },
            ],
        };
        Corridor {
            corridor_id: "MAR_DESERT_BASELINE_V1".to_string(),
            domain: "mar".to_string(),
            risk_vector: rv,
            lyapunov_template: Some(LyapunovResidual {
                system_id: "MAR_DESERT_BASELINE_V1".to_string(),
                t: 0.0,
                value: 0.0,
                d_value_dt: -0.1,
                stable: true,
            }),
        }
    }

    fn dummy_ecosafety(node_id: &str) -> CyboquaticNodeEcosafety {
        CyboquaticNodeEcosafety {
            node_id: node_id.to_string(),
            corridors: vec![dummy_corridor()],
        }
    }

    #[test]
    fn mar_step_runs_and_produces_results() {
        let ecosafety = dummy_ecosafety("MAR_DC_CORE_01");
        let corridor = dummy_corridor();
        let shard = MarVaultShard::new(
            "MAR_DC_CORE_01",
            MarZone::DowntownCore,
            ecosafety,
            corridor,
        );
        let boundary = MarBoundary {
            upstream_inflow_m3_per_h: 50.0,
            upstream_quality: MarQualityState::new(),
            downstream_head_m: 0.0,
            aquifer_head_m: 0.0,
        };
        let dt = Duration::from_secs(3600);
        let result = mar_step_ceim(dt, &boundary, &shard);
        assert!(result.inflow_m3 > 0.0);
        assert!(result.recharge_m3 >= 0.0);
    }

    #[test]
    fn registry_step_all_updates_shards() {
        let ecosafety = dummy_ecosafety("MAR_DC_CORE_02");
        let corridor = dummy_corridor();
        let shard = MarVaultShard::new(
            "MAR_DC_CORE_02",
            MarZone::DowntownCore,
            ecosafety,
            corridor,
        );

        let mut reg = MarShardRegistry::new();
        reg.insert(shard);

        let boundary = MarBoundary {
            upstream_inflow_m3_per_h: 40.0,
            upstream_quality: MarQualityState::new(),
            downstream_head_m: 0.0,
            aquifer_head_m: 0.0,
        };
        let dt = Duration::from_secs(1800);
        let results = reg.step_all(dt, &boundary);
        assert_eq!(results.len(), 1);
    }
}
