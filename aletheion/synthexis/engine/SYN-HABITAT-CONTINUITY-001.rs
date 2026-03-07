// Aletheion Synthexis Habitat Continuity Engine v1 (Phoenix / Sonoran)
// Rust-only, exclusion-compliant, no blacklisted tech.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------- Identifier types ----------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TreatyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorridorId(pub String);

// ---------- Basic types ----------

pub type Lux = f32;
pub type DecibelA = f32;
pub type Ppm = f32;
pub type Meters = f32;
pub type Probability = f32;

// ---------- Built form graph ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFormNode {
    pub id: BlockId,
    pub has_riparian_feature: bool,
    pub tree_canopy_fraction: f32, // 0.0..1.0, aligns w/ Phoenix canopy + depaving data[file:1]
    pub native_plant_fraction: f32,
    pub surface_impervious_fraction: f32,
    pub mean_nighttime_lux: Lux,
    pub mean_nighttime_dba: DecibelA,
    pub mean_particulate_ppm: Ppm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFormEdge {
    pub from: BlockId,
    pub to: BlockId,
    pub length_m: Meters,
    pub is_major_road: bool,
    pub is_cool_pavement_segment: bool, // so we can study cool pavement vs habitat continuity[file:1]
    pub night_traffic_intensity01: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFormGraph {
    pub nodes: HashMap<BlockId, BuiltFormNode>,
    pub edges: Vec<BuiltFormEdge>,
}

// ---------- Vegetation layer ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationAttributes {
    pub block_id: BlockId,
    pub native_plant_ids: Vec<String>,
    pub irrigated: bool,
    pub tree_canopy_fraction: f32,
}

// ---------- Hydrology layer ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrologyAttributes {
    pub block_id: BlockId,
    pub has_open_water: bool,
    pub distance_to_open_water_m: Meters,
    pub is_riparian_corridor: bool,
}

// ---------- Traffic layer ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficAttributes {
    pub block_id: BlockId,
    pub night_vehicle_volume_index01: f32,
    pub night_helicopter_overflight_index01: f32,
}

// ---------- SpeciesAgent projections (simplified view) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesToleranceView {
    pub species_id: SpeciesId,
    pub max_lux_corridor: Lux,
    pub max_night_dba: DecibelA,
    pub particulate_sensitivity_ppm: Ppm,
    pub required_native_fraction01: f32,
    pub min_canopy_height_proxy_m: Meters,
    pub riparian_dependency01: f32,
    pub min_open_water_proximity_m: Meters,
    pub conservation_priority01: f32,
}

// ---------- Treaty connectivity target (subset) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyConnectivityTarget {
    pub treaty_id: TreatyId,
    pub species_ids: Vec<SpeciesId>,
    pub min_connectivity_score01: f32,
    pub max_barrier_index: f32,
}

// ---------- Engine inputs ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatContinuityInputs {
    pub built_form: BuiltFormGraph,
    pub vegetation: HashMap<BlockId, VegetationAttributes>,
    pub hydrology: HashMap<BlockId, HydrologyAttributes>,
    pub traffic: HashMap<BlockId, TrafficAttributes>,
    pub species: HashMap<SpeciesId, SpeciesToleranceView>,
    pub treaties: Vec<TreatyConnectivityTarget>,
}

// ---------- Engine outputs ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorSegmentScore {
    pub corridor_id: CorridorId,
    pub block_id: BlockId,
    pub connectivity_score: f32, // 0.0..1.0
    pub barrier_index: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorProposal {
    pub corridor_id: CorridorId,
    pub species_id: SpeciesId,
    pub treaty_id: TreatyId,
    pub path_blocks: Vec<BlockId>,
    pub total_connectivity_score: f32,
    pub total_detour_cost: f32,
    pub is_treaty_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatContinuityResult {
    pub proposals: Vec<CorridorProposal>,
    pub segment_scores: Vec<CorridorSegmentScore>,
}

// ---------- Core scoring helpers ----------

fn score_block_for_species(
    node: &BuiltFormNode,
    veg: Option<&VegetationAttributes>,
    hydro: Option<&HydrologyAttributes>,
    traffic: Option<&TrafficAttributes>,
    species: &SpeciesToleranceView,
) -> (f32, f32) {
    // Connectivity score: higher is better
    // Barrier index: higher is worse
    let mut connectivity = 1.0_f32;
    let mut barrier = 0.0_f32;

    // Light: penalize over-tolerance lux
    if node.mean_nighttime_lux > species.max_lux_corridor {
        let over = (node.mean_nighttime_lux - species.max_lux_corridor)
            / (species.max_lux_corridor.max(1.0));
        connectivity -= 0.4 * over.min(1.0);
        barrier += 0.4 * over.min(1.0);
    }

    // Noise
    if node.mean_nighttime_dba > species.max_night_dba {
        let over = (node.mean_nighttime_dba - species.max_night_dba)
            / (species.max_night_dba.max(1.0));
        connectivity -= 0.3 * over.min(1.0);
        barrier += 0.3 * over.min(1.0);
    }

    // Particulates
    if node.mean_particulate_ppm > species.particulate_sensitivity_ppm {
        let over = (node.mean_particulate_ppm - species.particulate_sensitivity_ppm)
            / (species.particulate_sensitivity_ppm.max(1.0));
        connectivity -= 0.1 * over.min(1.0);
        barrier += 0.1 * over.min(1.0);
    }

    // Vegetation / native plants
    if let Some(v) = veg {
        if v.native_plant_ids.is_empty() {
            connectivity -= 0.1;
            barrier += 0.1;
        }
        if v.tree_canopy_fraction < species.required_native_fraction01 {
            let deficit = (species.required_native_fraction01 - v.tree_canopy_fraction)
                / species.required_native_fraction01.max(0.01);
            connectivity -= 0.15 * deficit.min(1.0);
            barrier += 0.15 * deficit.min(1.0);
        }
    }

    // Hydrology and riparian dependency
    if let Some(h) = hydro {
        if species.riparian_dependency01 > 0.3 && !h.is_riparian_corridor {
            connectivity -= 0.2 * species.riparian_dependency01;
            barrier += 0.2 * species.riparian_dependency01;
        }
        if h.distance_to_open_water_m > species.min_open_water_proximity_m {
            let over = (h.distance_to_open_water_m - species.min_open_water_proximity_m)
                / species.min_open_water_proximity_m.max(1.0);
            connectivity -= 0.1 * over.min(1.0);
            barrier += 0.1 * over.min(1.0);
        }
    }

    // Traffic
    if let Some(t) = traffic {
        connectivity -= 0.1 * t.night_vehicle_volume_index01;
        barrier += 0.1 * t.night_vehicle_volume_index01;
        connectivity -= 0.1 * t.night_helicopter_overflight_index01;
        barrier += 0.1 * t.night_helicopter_overflight_index01;
    }

    // Clamp
    let connectivity_clamped = connectivity.clamp(0.0, 1.0);
    let barrier_clamped = barrier.clamp(0.0, 1.0);

    (connectivity_clamped, barrier_clamped)
}

// ---------- Engine main API ----------

#[derive(Debug)]
pub struct HabitatContinuityEngine;

impl HabitatContinuityEngine {
    pub fn evaluate(inputs: &HabitatContinuityInputs) -> HabitatContinuityResult {
        let mut segment_scores: Vec<CorridorSegmentScore> = Vec::new();
        let mut proposals: Vec<CorridorProposal> = Vec::new();

        // For v1, we take a simple approach:
        // - For each treaty and species, score all blocks.
        // - Build a "corridor" as the set of blocks with connectivity above threshold.
        // - Detour cost is placeholder (0.0); a later version will integrate real routing.

        for treaty in &inputs.treaties {
            for species_id in &treaty.species_ids {
                let Some(spec) = inputs.species.get(species_id) else {
                    continue;
                };

                let corridor_id = CorridorId(format!(
                    "corridor-{}-{}",
                    treaty.treaty_id_string(),
                    species_id.0
                ));

                let mut path_blocks: Vec<BlockId> = Vec::new();
                let mut total_conn = 0.0_f32;
                let mut count = 0_usize;

                for (block_id, node) in &inputs.built_form.nodes {
                    let veg = inputs.vegetation.get(block_id);
                    let hydro = inputs.hydrology.get(block_id);
                    let traffic = inputs.traffic.get(block_id);

                    let (conn, barrier) =
                        score_block_for_species(node, veg, hydro, traffic, spec);

                    let segment = CorridorSegmentScore {
                        corridor_id: corridor_id.clone(),
                        block_id: block_id.clone(),
                        connectivity_score: conn,
                        barrier_index: barrier,
                    };
                    segment_scores.push(segment);

                    if conn >= treaty.min_connectivity_score01 {
                        path_blocks.push(block_id.clone());
                        total_conn += conn;
                        count += 1;
                    }
                }

                let avg_conn = if count == 0 {
                    0.0
                } else {
                    total_conn / (count as f32)
                };

                let is_treaty_compliant = avg_conn >= treaty.min_connectivity_score01;

                let proposal = CorridorProposal {
                    corridor_id: corridor_id.clone(),
                    species_id: species_id.clone(),
                    treaty_id: treaty.treaty_id.clone(),
                    path_blocks,
                    total_connectivity_score: avg_conn,
                    total_detour_cost: 0.0, // placeholder; to be replaced with real routing
                    is_treaty_compliant,
                };

                proposals.push(proposal);
            }
        }

        HabitatContinuityResult {
            proposals,
            segment_scores,
        }
    }
}

// ---------- Helper methods ----------

impl TreatyConnectivityTarget {
    pub fn treaty_id_string(&self) -> &str {
        &self.treaty_id.0
    }
}
