// Role: Sovereign test harness for HabitatContinuityEngine, using a Phoenix
//       bat / pollinator corridor fixture (riparian dark corridors, cool pavement,
//       and tree canopy programs).

#![cfg(test)]

use super::*; // assumes HabitatContinuityEngine and related types live in parent module
use crate::state::synthexis::schema::species_agent::{
    BioticTreaty, ChemicalConstraints, EnforcementMode, HabitatRequirements, LightTolerance,
    NoiseTolerance, PriorityWeights, RangeF64, SeasonalSensitivity, SpeciesAgent,
    SpatialScope, TemporalScope, TreatyMetricsConfig, VegetationRequirements,
    VegetationStructureConstraints, WaterRequirements,
};

/// Minimal stand-in types for built environment layers used by HabitatContinuityEngine.
#[derive(Clone, Debug)]
pub struct BuiltFormNode {
    pub id: String,
    pub kind: String, // "park", "canal", "street_segment", etc.
}

#[derive(Clone, Debug)]
pub struct BuiltFormEdge {
    pub from: String,
    pub to: String,
    pub impervious_fraction: f64,
    pub canopy_cover_fraction: f64,
    pub alan_lux: f64,
    pub night_noise_db_a: f64,
    pub pesticide_use: bool,
}

#[derive(Clone, Debug)]
pub struct BuiltFormGraph {
    pub nodes: Vec<BuiltFormNode>,
    pub edges: Vec<BuiltFormEdge>,
}

/// Output corridor proposal from HabitatContinuityEngine (interface only).
#[derive(Clone, Debug)]
pub struct CorridorProposal {
    pub species_id: String,
    pub path_node_ids: Vec<String>,
    pub connectivity_score: f64,
    pub required_interventions: Vec<String>,
}

/// Expected engine interface.
pub trait HabitatContinuityEngine {
    fn compute_corridors(
        built_form: &BuiltFormGraph,
        treaties: &[BioticTreaty],
        species: &[SpeciesAgent],
    ) -> Vec<CorridorProposal>;
}

/// Phoenix fixture: Myotis velifer bat species agent, tuned to dark riparian corridors.
fn fixture_myotis_velifer() -> SpeciesAgent {
    SpeciesAgent {
        species_id: "myotis_velifer".to_string(),
        taxon: crate::state::synthexis::schema::species_agent::Taxon {
            kingdom: "Animalia".into(),
            phylum: "Chordata".into(),
            class_name: "Mammalia".into(),
            order: "Chiroptera".into(),
            family: "Vespertilionidae".into(),
            genus: "Myotis".into(),
            species: "velifer".into(),
            common_name: "Cave Myotis Bat".into(),
        },
        habitat_requirements: HabitatRequirements {
            canopy_height_range_m: RangeF64 { min: 5.0, max: 30.0 },
            cavity_nesting: true,
            ground_nesting: false,
            riparian_dependency: 0.8,
        },
        water_requirements: WaterRequirements {
            min_open_water_proximity_m: 200.0,
            acceptable_water_sources: vec![
                "riparian".into(),
                "canal".into(),
                "irrigation".into(),
            ],
        },
        vegetation_requirements: VegetationRequirements {
            required_native_plants: vec![
                "sonoran_cottonwood".into(),
                "willow".into(),
                "mesquite".into(),
            ],
            min_native_fraction: 0.5,
        },
        light_tolerance: LightTolerance {
            light_averse: true,
            max_corridor_alan_lux: 1.0,
            max_forage_alan_lux: 5.0,
            spectral_sensitivity_bands_nm: vec![(400, 500)],
        },
        noise_tolerance: NoiseTolerance {
            max_night_db_a: 45.0,
            impulsive_noise_penalty: 1.0,
            continuous_noise_penalty: 0.5,
        },
        chemical_tolerance: ChemicalTolerance {
            pesticide_classes_prohibited: vec!["neonicotinoid".into()],
            dust_sensitivity: 0.7,
        },
        seasonal_sensitivities: SeasonalSensitivity {
            active_seasons: vec!["spring".into(), "summer".into(), "monsoon".into()],
            breeding_windows: vec![("2026-05-01T00:00Z".into(), "2026-08-31T23:59Z".into())],
            migration_windows: vec![],
            critical_dark_corridor_periods: vec![(
                "2026-05-01T18:00Z".into(),
                "2026-08-31T06:00Z".into(),
            )],
        },
        priority_weights: PriorityWeights {
            conservation_priority: 0.8,
            cultural_priority: 0.4,
            ecosystem_service_weight: 0.6,
        },
    }
}

/// Phoenix fixture: BioticTreaty enforcing dark riparian corridors along a canal / river.
fn fixture_riparian_bat_treaty() -> BioticTreaty {
    BioticTreaty {
        treaty_id: "phx_riparian_dark_corridor_bats_v1".into(),
        species_ids: vec!["myotis_velifer".into()],
        spatial_scope: SpatialScope {
            polygon_ids: vec!["phx_salt_river_riparian".into()],
            corridor_segment_ids: vec!["canal_segment_night_bat_corridor".into()],
        },
        temporal_scope: TemporalScope {
            start: "2026-05-01T18:00Z".into(),
            end: "2026-08-31T06:00Z".into(),
            recurrence_rules: vec!["YEARLY_MAY_AUG_NIGHTS".into()],
        },
        lighting: Some(crate::state::synthexis::schema::species_agent::LightingConstraints {
            max_lux: 1.0,
            allowed_spectra: vec!["amber".into(), "low_blue".into()],
            shielding_required: true,
            dark_corridor_segments: vec!["canal_segment_night_bat_corridor".into()],
        }),
        noise: Some(crate::state::synthexis::schema::species_agent::NoiseConstraints {
            max_db_by_time_band: vec![("night".into(), 45.0)],
            forbidden_impulsive_sources: vec!["leaf_blower".into()],
        }),
        chemicals: Some(crate::state::synthexis::schema::species_agent::ChemicalConstraints {
            banned_pesticide_classes: vec!["neonicotinoid".into()],
            spray_blackout_windows: vec![(
                "2026-05-01T18:00Z".into(),
                "2026-08-31T06:00Z".into(),
            )],
            no_spray_buffers_m: 50.0,
        }),
        vegetation_structure: Some(
            crate::state::synthexis::schema::species_agent::VegetationStructureConstraints {
                required_native_plant_fraction: 0.6,
                required_host_plants: vec![
                    "sonoran_cottonwood".into(),
                    "willow".into(),
                    "mesquite".into(),
                ],
                snag_roost_requirements: vec!["min_snags_per_hectare:2".into()],
            },
        ),
        enforcement_mode: EnforcementMode::HardLimits,
        metrics_to_track: TreatyMetricsConfig {
            track_corridor_connectivity_score: true,
            track_alan_violation_count: true,
            track_pesticide_violation_count: true,
            track_species_activity_index: true,
        },
    }
}

/// Phoenix built-form fixture: a dark riparian path plus a conflicting lit arterial.
fn fixture_built_form_phx_riparian() -> BuiltFormGraph {
    let nodes = vec![
        BuiltFormNode {
            id: "park_riparian_node".into(),
            kind: "park".into(),
        },
        BuiltFormNode {
            id: "canal_segment_night_bat_corridor".into(),
            kind: "canal".into(),
        },
        BuiltFormNode {
            id: "downtown_arterial".into(),
            kind: "street_segment".into(),
        },
    ];

    let edges = vec![
        // Dark, vegetated canal corridor between park and riparian segment.
        BuiltFormEdge {
            from: "park_riparian_node".into(),
            to: "canal_segment_night_bat_corridor".into(),
            impervious_fraction: 0.2,
            canopy_cover_fraction: 0.7,
            alan_lux: 0.5,
            night_noise_db_a: 40.0,
            pesticide_use: false,
        },
        // Bright, noisy arterial that should be disfavoured by the engine.
        BuiltFormEdge {
            from: "park_riparian_node".into(),
            to: "downtown_arterial".into(),
            impervious_fraction: 0.9,
            canopy_cover_fraction: 0.1,
            alan_lux: 20.0,
            night_noise_db_a: 65.0,
            pesticide_use: true,
        },
    ];

    BuiltFormGraph { nodes, edges }
}

/// Sovereign test: HabitatContinuityEngine must select dark riparian canal corridor
/// over bright arterial for Myotis velifer under the Phoenix bat treaty.
#[test]
fn habitat_continuity_prefers_dark_riparian_corridor_for_bats() {
    // Arrange
    let species = vec![fixture_myotis_velifer()];
    let treaties = vec![fixture_riparian_bat_treaty()];
    let built_form = fixture_built_form_phx_riparian();

    // Act
    let corridors = <crate::opt::synthexis::HabitatContinuityEngineImpl as HabitatContinuityEngine>::compute_corridors(
        &built_form,
        &treaties,
        &species,
    );

    // Assert
    // There should be at least one corridor for Myotis velifer.
    let bat_corridors: Vec<_> = corridors
        .iter()
        .filter(|c| c.species_id == "myotis_velifer")
        .collect();
    assert!(!bat_corridors.is_empty(), "no bat corridors returned");

    // The best corridor must include the dark canal segment and avoid the lit arterial.
    let best = bat_corridors
        .iter()
        .max_by(|a, b| a.connectivity_score.partial_cmp(&b.connectivity_score).unwrap())
        .expect("no best corridor found");

    assert!(
        best.path_node_ids.contains(&"canal_segment_night_bat_corridor".into()),
        "best corridor does not include riparian canal segment"
    );
    assert!(
        !best.path_node_ids.contains(&"downtown_arterial".into()),
        "best corridor incorrectly includes bright arterial"
    );
    assert!(
        best.connectivity_score > 0.0,
        "connectivity_score should be positive for valid corridor"
    );
}
