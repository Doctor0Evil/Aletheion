#![forbid(unsafe_code)]

use std::collections::HashMap;

use aletheion::synthexis::engine::SYN_HABITAT_CONTINUITY_001::{
    BuiltFormGraph, BuiltFormNode, HabitatContinuityEngine, HabitatContinuityInputs,
    HydrologyAttributes, SpeciesId, SpeciesToleranceView, TreatyConnectivityTarget,
    TreatyId, VegetationAttributes, TrafficAttributes, BlockId,
};

fn block(id: &str) -> BlockId {
    BlockId(id.to_string())
}

#[test]
fn basic_corridor_exists_for_simple_graph() {
    let mut nodes = HashMap::new();
    nodes.insert(
        block("A"),
        BuiltFormNode {
            id: block("A"),
            has_riparian_feature: true,
            tree_canopy_fraction: 0.6,
            native_plant_fraction: 0.7,
            surface_impervious_fraction: 0.3,
            mean_nighttime_lux: 2.0,
            mean_nighttime_dba: 35.0,
            mean_particulate_ppm: 30.0,
        },
    );

    let built_form = BuiltFormGraph { nodes, edges: vec![] };

    let mut vegetation = HashMap::new();
    vegetation.insert(
        block("A"),
        VegetationAttributes {
            block_id: block("A"),
            native_plant_ids: vec!["native_plant_1".into()],
            irrigated: true,
            tree_canopy_fraction: 0.6,
        },
    );

    let mut hydrology = HashMap::new();
    hydrology.insert(
        block("A"),
        HydrologyAttributes {
            block_id: block("A"),
            has_open_water: true,
            distance_to_open_water_m: 20.0,
            is_riparian_corridor: true,
        },
    );

    let mut traffic = HashMap::new();
    traffic.insert(
        block("A"),
        TrafficAttributes {
            block_id: block("A"),
            night_vehicle_volume_index01: 0.1,
            night_helicopter_overflight_index01: 0.0,
        },
    );

    let species_id = SpeciesId("test_bat".into());
    let mut species_map = HashMap::new();
    species_map.insert(
        species_id.clone(),
        SpeciesToleranceView {
            species_id: species_id.clone(),
            max_lux_corridor: 5.0,
            max_night_dba: 45.0,
            particulate_sensitivity_ppm: 50.0,
            required_native_fraction01: 0.4,
            min_canopy_height_proxy_m: 4.0,
            riparian_dependency01: 0.8,
            min_open_water_proximity_m: 50.0,
            conservation_priority01: 0.9,
        },
    );

    let treaty = TreatyConnectivityTarget {
        treaty_id: TreatyId("synthexis-test-treaty-001".into()),
        species_ids: vec![species_id.clone()],
        min_connectivity_score01: 0.5,
        max_barrier_index: 0.8,
    };

    let inputs = HabitatContinuityInputs {
        built_form,
        vegetation,
        hydrology,
        traffic,
        species: species_map,
        treaties: vec![treaty],
    };

    let result = HabitatContinuityEngine::evaluate(&inputs);

    assert!(!result.proposals.is_empty(), "Expected at least one corridor proposal");
    let proposal = &result.proposals[0];
    assert!(
        !proposal.path_blocks.is_empty(),
        "Expected corridor to include at least one block"
    );
    assert!(
        proposal.total_connectivity_score >= 0.5,
        "Expected connectivity score to meet treaty minimum"
    );
}
