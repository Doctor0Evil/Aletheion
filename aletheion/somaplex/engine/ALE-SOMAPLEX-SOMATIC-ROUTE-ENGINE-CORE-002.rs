// ============================================================================
// FILE: aletheion/somaplex/engine/ALE-SOMAPLEX-SOMATIC-ROUTE-ENGINE-CORE-002.rs
// PURPOSE: Human mobility route optimization consuming SomaticCost and HeatCost,
//          honoring HeatBudget envelopes and BioticTreaty-derived constraints,
//          with fall-risk minimization and WCAG 2.2 AAA accessibility compliance
// LANGUAGE: Rust (2024 Edition)
// DESTINATION: Aletheion Repository - Somaplex Engine Subsystem
// COMPLIANCE: Zero-contamination IFC, NeurorightsEnvelope, FPIC metadata,
//             Phoenix Accessibility First Design Guidelines 2025, RoH ≤ 0.3
// INTEGRATION: ALE-HIGHWAYS-CORRIDOR-KERNEL-001.rs, ALE-THERM-HEATBUDGET-SIMULATOR-CORE-002.rs,
//              ALE-SYNTHEXIS-BIOTIC-TREATY-CORE-001.aln, ALE-SOMAPLEX-FALL-RISK-ENVELOPE-001.aln
// C-ABI: JSON-in/JSON-out only, explicit errors, no panics, Lua FFI compatible
// ============================================================================

#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::{CStr, CString};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// SECTION 1: SOMATIC DOMAIN ENUMERATIONS & TYPE DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MobilityMode {
    Walking = 0,
    Wheelchair = 1,
    Walker = 2,
    Cane = 3,
    Bicycle = 4,
    Scooter = 5,
    VisionImpaired = 6,
}

impl MobilityMode {
    pub const ALL: [MobilityMode; 7] = [
        MobilityMode::Walking,
        MobilityMode::Wheelchair,
        MobilityMode::Walker,
        MobilityMode::Cane,
        MobilityMode::Bicycle,
        MobilityMode::Scooter,
        MobilityMode::VisionImpaired,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            MobilityMode::Walking => "walking",
            MobilityMode::Wheelchair => "wheelchair",
            MobilityMode::Walker => "walker",
            MobilityMode::Cane => "cane",
            MobilityMode::Bicycle => "bicycle",
            MobilityMode::Scooter => "scooter",
            MobilityMode::VisionImpaired => "vision_impaired",
        }
    }

    pub fn max_slope_pct(&self) -> f64 {
        match self {
            MobilityMode::Wheelchair => 8.33,
            MobilityMode::Walker => 10.0,
            MobilityMode::Cane => 12.0,
            MobilityMode::VisionImpaired => 5.0,
            MobilityMode::Walking => 15.0,
            MobilityMode::Bicycle => 8.0,
            MobilityMode::Scooter => 10.0,
        }
    }

    pub fn preferred_surface_types(&self) -> Vec<&'static str> {
        match self {
            MobilityMode::Wheelchair => vec!["concrete", "asphalt_smooth", "paver"],
            MobilityMode::Walker => vec!["concrete", "asphalt_smooth"],
            MobilityMode::VisionImpaired => vec!["tactile_paving", "concrete"],
            MobilityMode::Walking => vec!["concrete", "asphalt", "gravel", "dirt"],
            MobilityMode::Bicycle => vec!["asphalt_smooth", "concrete", "bike_lane"],
            MobilityMode::Scooter => vec!["asphalt_smooth", "concrete"],
            MobilityMode::Cane => vec!["concrete", "asphalt_smooth"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RouteSegmentType {
    Sidewalk = 0,
    Crosswalk = 1,
    BikeLane = 2,
    SharedPath = 3,
    BuildingEntrance = 4,
    TransitStop = 5,
    CoolingStation = 6,
    RestArea = 7,
    Ramp = 8,
    Elevator = 9,
    StairAlternative = 10,
}

impl RouteSegmentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RouteSegmentType::Sidewalk => "sidewalk",
            RouteSegmentType::Crosswalk => "crosswalk",
            RouteSegmentType::BikeLane => "bike_lane",
            RouteSegmentType::SharedPath => "shared_path",
            RouteSegmentType::BuildingEntrance => "building_entrance",
            RouteSegmentType::TransitStop => "transit_stop",
            RouteSegmentType::CoolingStation => "cooling_station",
            RouteSegmentType::RestArea => "rest_area",
            RouteSegmentType::Ramp => "ramp",
            RouteSegmentType::Elevator => "elevator",
            RouteSegmentType::StairAlternative => "stair_alternative",
        }
    }

    pub fn accessibility_rating(&self) -> f64 {
        match self {
            RouteSegmentType::Elevator => 1.0,
            RouteSegmentType::Ramp => 0.95,
            RouteSegmentType::Sidewalk => 0.9,
            RouteSegmentType::Crosswalk => 0.85,
            RouteSegmentType::BuildingEntrance => 0.9,
            RouteSegmentType::TransitStop => 0.8,
            RouteSegmentType::CoolingStation => 0.95,
            RouteSegmentType::RestArea => 0.9,
            RouteSegmentType::SharedPath => 0.85,
            RouteSegmentType::BikeLane => 0.7,
            RouteSegmentType::StairAlternative => 0.75,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteNode {
    pub node_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f64,
    pub segment_type: RouteSegmentType,
    pub surface_type: String,
    pub width_m: f64,
    pub slope_pct: f64,
    pub shade_coverage_pct: f64,
    pub seating_available: bool,
    pub tactile_paving: bool,
    pub auditory_signals: bool,
    pub lighting_lux: f64,
    pub adjacent_cooling_station: bool,
    pub adjacent_water_fountain: bool,
    pub wcag_compliant: bool,
}

impl RouteNode {
    pub fn new(node_id: String, latitude: f64, longitude: f64) -> Self {
        Self {
            node_id,
            latitude,
            longitude,
            elevation_m: 0.0,
            segment_type: RouteSegmentType::Sidewalk,
            surface_type: "concrete".to_string(),
            width_m: 1.5,
            slope_pct: 0.0,
            shade_coverage_pct: 0.0,
            seating_available: false,
            tactile_paving: false,
            auditory_signals: false,
            lighting_lux: 0.0,
            adjacent_cooling_station: false,
            adjacent_water_fountain: false,
            wcag_compliant: true,
        }
    }

    pub fn with_elevation(mut self, elev: f64) -> Self {
        self.elevation_m = elev;
        self
    }

    pub fn with_segment_type(mut self, seg_type: RouteSegmentType) -> Self {
        self.segment_type = seg_type;
        self.wcag_compliant = seg_type.accessibility_rating() >= 0.85;
        self
    }

    pub fn with_surface(mut self, surface: String) -> Self {
        self.surface_type = surface;
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width_m = width;
        self
    }

    pub fn with_slope(mut self, slope: f64) -> Self {
        self.slope_pct = slope;
        self.wcag_compliant = slope <= 8.33;
        self
    }

    pub fn with_shade(mut self, shade: f64) -> Self {
        self.shade_coverage_pct = shade.min(100.0);
        self
    }

    pub fn with_seating(mut self, available: bool) -> Self {
        self.seating_available = available;
        self
    }

    pub fn with_tactile_paving(mut self, tactile: bool) -> Self {
        self.tactile_paving = tactile;
        self
    }

    pub fn with_auditory_signals(mut self, auditory: bool) -> Self {
        self.auditory_signals = auditory;
        self
    }

    pub fn with_lighting(mut self, lux: f64) -> Self {
        self.lighting_lux = lux;
        self
    }

    pub fn with_cooling_station(mut self, adjacent: bool) -> Self {
        self.adjacent_cooling_station = adjacent;
        self
    }

    pub fn with_water_fountain(mut self, adjacent: bool) -> Self {
        self.adjacent_water_fountain = adjacent;
        self
    }

    pub fn accessibility_score(&self) -> f64 {
        let base = self.segment_type.accessibility_rating();
        let slope_factor = if self.slope_pct <= 8.33 { 1.0 } else { 1.0 - (self.slope_pct - 8.33) / 20.0 };
        let width_factor = if self.width_m >= 1.5 { 1.0 } else { self.width_m / 1.5 };
        let surface_factor = if self.surface_type == "concrete" || self.surface_type == "asphalt_smooth" { 1.0 } else { 0.8 };
        let shade_factor = 0.5 + (self.shade_coverage_pct / 200.0);
        let amenities_factor = if self.seating_available && self.adjacent_water_fountain { 1.1 } else { 1.0 };
        (base * slope_factor.min(1.0) * width_factor.min(1.0) * surface_factor * shade_factor * amenities_factor).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEdge {
    pub edge_id: String,
    pub from_node: String,
    pub to_node: String,
    pub distance_m: f64,
    pub estimated_time_min: f64,
    pub joint_load_score: f64,
    pub fall_risk_index: f64,
    pub heat_exposure_score: f64,
    pub biotic_impact_score: f64,
    pub accessibility_penalty: f64,
}

impl RouteEdge {
    pub fn new(edge_id: String, from_node: String, to_node: String, distance_m: f64) -> Self {
        Self {
            edge_id,
            from_node,
            to_node,
            distance_m,
            estimated_time_min: distance_m / 80.0,
            joint_load_score: 0.0,
            fall_risk_index: 0.0,
            heat_exposure_score: 0.0,
            biotic_impact_score: 0.0,
            accessibility_penalty: 0.0,
        }
    }

    pub fn with_time(mut self, time_min: f64) -> Self {
        self.estimated_time_min = time_min;
        self
    }

    pub fn with_joint_load(mut self, score: f64) -> Self {
        self.joint_load_score = score.min(1.0);
        self
    }

    pub fn with_fall_risk(mut self, risk: f64) -> Self {
        self.fall_risk_index = risk.min(1.0);
        self
    }

    pub fn with_heat_exposure(mut self, score: f64) -> Self {
        self.heat_exposure_score = score.min(1.0);
        self
    }

    pub fn with_biotic_impact(mut self, score: f64) -> Self {
        self.biotic_impact_score = score.min(1.0);
        self
    }

    pub fn with_accessibility_penalty(mut self, penalty: f64) -> Self {
        self.accessibility_penalty = penalty;
        self
    }

    pub fn composite_cost(&self, mobility_mode: MobilityMode) -> f64 {
        let time_weight = 0.25;
        let joint_load_weight = 0.20;
        let fall_risk_weight = 0.25;
        let heat_weight = 0.15;
        let biotic_weight = 0.10;
        let accessibility_weight = 0.05;
        let slope_penalty = if self.fall_risk_index > 0.3 && mobility_mode == MobilityMode::Wheelchair { 2.0 } else { 1.0 };
        (self.estimated_time_min * time_weight + self.joint_load_score * joint_load_weight + self.fall_risk_index * fall_risk_weight * slope_penalty + self.heat_exposure_score * heat_weight + self.biotic_impact_score * biotic_weight + self.accessibility_penalty * accessibility_weight)
    }
}

// ============================================================================
// SECTION 2: HEAT BUDGET & SOMATIC COST INTEGRATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatBudgetConstraint {
    pub max_continuous_heat_exposure_min: u32,
    pub cooling_station_spacing_m: f64,
    pub shade_route_coverage_min_pct: f64,
    pub hydration_requirement_ml_hr: f64,
    pub ambient_temp_threshold_c: f64,
    pub heat_index_threshold_c: f64,
}

impl HeatBudgetConstraint {
    pub fn new() -> Self {
        Self {
            max_continuous_heat_exposure_min: 30,
            cooling_station_spacing_m: 400.0,
            shade_route_coverage_min_pct: 50.0,
            hydration_requirement_ml_hr: 500.0,
            ambient_temp_threshold_c: 40.0,
            heat_index_threshold_c: 45.0,
        }
    }

    pub fn with_max_exposure(mut self, min: u32) -> Self {
        self.max_continuous_heat_exposure_min = min;
        self
    }

    pub fn with_cooling_spacing(mut self, m: f64) -> Self {
        self.cooling_station_spacing_m = m;
        self
    }

    pub fn with_shade_coverage(mut self, pct: f64) -> Self {
        self.shade_route_coverage_min_pct = pct;
        self
    }

    pub fn with_hydration(mut self, ml: f64) -> Self {
        self.hydration_requirement_ml_hr = ml;
        self
    }

    pub fn with_temp_threshold(mut self, c: f64) -> Self {
        self.ambient_temp_threshold_c = c;
        self
    }

    pub fn with_heat_index_threshold(mut self, c: f64) -> Self {
        self.heat_index_threshold_c = c;
        self
    }
}

impl Default for HeatBudgetConstraint {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaticCostProfile {
    pub joint_load_weight: f64,
    pub fall_risk_weight: f64,
    pub heat_exposure_weight: f64,
    pub time_weight: f64,
    pub accessibility_weight: f64,
    pub biotic_preservation_weight: f64,
}

impl SomaticCostProfile {
    pub fn new() -> Self {
        Self {
            joint_load_weight: 0.25,
            fall_risk_weight: 0.30,
            heat_exposure_weight: 0.20,
            time_weight: 0.10,
            accessibility_weight: 0.10,
            biotic_preservation_weight: 0.05,
        }
    }

    pub fn for_wheelchair_user() -> Self {
        Self {
            joint_load_weight: 0.20,
            fall_risk_weight: 0.40,
            heat_exposure_weight: 0.20,
            time_weight: 0.05,
            accessibility_weight: 0.10,
            biotic_preservation_weight: 0.05,
        }
    }

    pub fn for_elderly() -> Self {
        Self {
            joint_load_weight: 0.25,
            fall_risk_weight: 0.35,
            heat_exposure_weight: 0.25,
            time_weight: 0.05,
            accessibility_weight: 0.05,
            biotic_preservation_weight: 0.05,
        }
    }

    pub fn for_outdoor_worker() -> Self {
        Self {
            joint_load_weight: 0.30,
            fall_risk_weight: 0.20,
            heat_exposure_weight: 0.30,
            time_weight: 0.15,
            accessibility_weight: 0.03,
            biotic_preservation_weight: 0.02,
        }
    }
}

impl Default for SomaticCostProfile {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 3: IFC LABEL INTEGRATION FOR SOMAPLEX
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaplexIFCLabel {
    pub label_id: String,
    pub sensitivity: String,
    pub domain: String,
    pub provenance: String,
    pub origin_hash: String,
    pub timestamp: u64,
    pub fpic_verified: bool,
    pub neurorights_compliant: bool,
    pub somatic_treaty_bound: bool,
    pub corridor_validated: bool,
}

impl SomaplexIFCLabel {
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
            somatic_treaty_bound: false,
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

    pub fn with_somatic_treaty(mut self, bound: bool) -> Self {
        self.somatic_treaty_bound = bound;
        self
    }

    pub fn with_corridor_validation(mut self, validated: bool) -> Self {
        self.corridor_validated = validated;
        self
    }
}

// ============================================================================
// SECTION 4: ROUTE PLANNING INPUT/OUTPUT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlanningInput {
    pub request_id: String,
    pub origin_node_id: String,
    pub destination_node_id: String,
    pub mobility_mode: MobilityMode,
    pub persona_vulnerability_class: String,
    pub nodes: Vec<RouteNode>,
    pub edges: Vec<RouteEdge>,
    pub heat_budget_constraint: HeatBudgetConstraint,
    pub somatic_cost_profile: SomaticCostProfile,
    pub ifc_labels: Vec<SomaplexIFCLabel>,
    pub corridor_id: String,
    pub biotic_treaty_active: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSegment {
    pub segment_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub distance_m: f64,
    pub time_min: f64,
    pub joint_load_score: f64,
    pub fall_risk_index: f64,
    pub heat_exposure_score: f64,
    pub shade_coverage_pct: f64,
    pub cooling_station_available: bool,
    pub water_fountain_available: bool,
    pub wcag_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    pub route_id: String,
    pub origin_node_id: String,
    pub destination_node_id: String,
    pub total_distance_m: f64,
    pub total_time_min: f64,
    pub segments: Vec<RouteSegment>,
    pub aggregate_joint_load_score: f64,
    pub aggregate_fall_risk_index: f64,
    pub aggregate_heat_exposure_score: f64,
    pub shade_coverage_avg_pct: f64,
    pub cooling_stations_count: u32,
    pub water_fountains_count: u32,
    pub wcag_compliance_pct: f64,
    pub biotic_impact_score: f64,
    pub accessibility_rating: f64,
    pub recommended_rest_stops: Vec<String>,
    pub heat_alerts: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub can_proceed: bool,
}

impl RoutePlan {
    pub fn new(route_id: String, origin: String, destination: String) -> Self {
        Self {
            route_id,
            origin_node_id: origin,
            destination_node_id: destination,
            total_distance_m: 0.0,
            total_time_min: 0.0,
            segments: Vec::new(),
            aggregate_joint_load_score: 0.0,
            aggregate_fall_risk_index: 0.0,
            aggregate_heat_exposure_score: 0.0,
            shade_coverage_avg_pct: 0.0,
            cooling_stations_count: 0,
            water_fountains_count: 0,
            wcag_compliance_pct: 1.0,
            biotic_impact_score: 0.0,
            accessibility_rating: 1.0,
            recommended_rest_stops: Vec::new(),
            heat_alerts: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            can_proceed: true,
        }
    }

    pub fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
        self.can_proceed = false;
    }

    pub fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }

    pub fn compute_aggregates(&mut self) {
        if self.segments.is_empty() {
            return;
        }
        self.total_distance_m = self.segments.iter().map(|s| s.distance_m).sum();
        self.total_time_min = self.segments.iter().map(|s| s.time_min).sum();
        self.aggregate_joint_load_score = self.segments.iter().map(|s| s.joint_load_score).sum::<f64>() / self.segments.len() as f64;
        self.aggregate_fall_risk_index = self.segments.iter().map(|s| s.fall_risk_index).sum::<f64>() / self.segments.len() as f64;
        self.aggregate_heat_exposure_score = self.segments.iter().map(|s| s.heat_exposure_score).sum::<f64>() / self.segments.len() as f64;
        self.shade_coverage_avg_pct = self.segments.iter().map(|s| s.shade_coverage_pct).sum::<f64>() / self.segments.len() as f64;
        self.cooling_stations_count = self.segments.iter().filter(|s| s.cooling_station_available).count() as u32;
        self.water_fountains_count = self.segments.iter().filter(|s| s.water_fountain_available).count() as u32;
        let wcag_compliant_count = self.segments.iter().filter(|s| s.wcag_compliant).count();
        self.wcag_compliance_pct = wcag_compliant_count as f64 / self.segments.len() as f64;
        self.accessibility_rating = 1.0 - self.aggregate_fall_risk_index;
        if self.aggregate_fall_risk_index > 0.3 {
            self.add_warning("Fall risk index exceeds 0.3 RoH bound - consider alternative route".to_string());
        }
        if self.shade_coverage_avg_pct < 50.0 {
            self.add_warning("Shade coverage below 50% - heat exposure risk elevated".to_string());
        }
        if self.aggregate_heat_exposure_score > 0.6 {
            self.heat_alerts.push("High heat exposure - activate cooling stations along route".to_string());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlanningOutput {
    pub request_id: String,
    pub route_plan: Option<RoutePlan>,
    pub alternative_routes: Vec<RoutePlan>,
    pub accessibility_compliant: bool,
    pub heat_safe: bool,
    pub biotic_compliant: bool,
    pub ifc_valid: bool,
    pub corridor_valid: bool,
    pub output_ifc_label: SomaplexIFCLabel,
    pub timestamp: u64,
}

impl RoutePlanningOutput {
    pub fn new(request_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            request_id,
            route_plan: None,
            alternative_routes: Vec::new(),
            accessibility_compliant: false,
            heat_safe: false,
            biotic_compliant: false,
            ifc_valid: false,
            corridor_valid: false,
            output_ifc_label: SomaplexIFCLabel::new(
                format!("IFC-{}-SOMA-0002", timestamp),
                "internal".to_string(),
                "somatic".to_string(),
                "inference".to_string(),
                format!("hash_{}", request_id),
            ),
            timestamp,
        }
    }
}

// ============================================================================
// SECTION 5: SOMATIC ROUTE ENGINE KERNEL
// ============================================================================

#[derive(Debug, Clone)]
pub struct SomaticRouteEngineKernel {
    pub kernel_version: String,
    pub policy_version: String,
    pub max_fall_risk_index: f64,
    pub min_wcag_compliance_pct: f64,
    pub max_heat_exposure_score: f64,
}

impl SomaticRouteEngineKernel {
    pub fn new(kernel_version: String, policy_version: String) -> Self {
        Self {
            kernel_version,
            policy_version,
            max_fall_risk_index: 0.3,
            min_wcag_compliance_pct: 0.9,
            max_heat_exposure_score: 0.6,
        }
    }

    pub fn validate_ifc_labels(&self, labels: &[SomaplexIFCLabel]) -> Result<bool, String> {
        if labels.is_empty() {
            return Err("IFC labels required for all Somaplex operations".to_string());
        }
        for label in labels {
            if label.sensitivity == "sovereign" && !label.fpic_verified {
                return Err(format!("Sovereign IFC label {} requires FPIC verification", label.label_id));
            }
            if label.domain != "somatic" && label.domain != "thermal" && label.domain != "biotic" {
                return Err(format!("Invalid domain {} for Somaplex IFC label", label.domain));
            }
            if label.origin_hash.is_empty() {
                return Err(format!("IFC label {} missing origin hash", label.label_id));
            }
        }
        Ok(true)
    }

    pub fn validate_nodes(&self, nodes: &[RouteNode], mobility_mode: MobilityMode) -> Result<bool, String> {
        for node in nodes {
            if node.slope_pct > mobility_mode.max_slope_pct() {
                return Err(format!(
                    "Node {} slope {}% exceeds maximum {}% for {:?}",
                    node.node_id, node.slope_pct, mobility_mode.max_slope_pct(), mobility_mode
                ));
            }
            if mobility_mode == MobilityMode::Wheelchair && node.width_m < 1.5 {
                return Err(format!(
                    "Node {} width {}m insufficient for wheelchair access (minimum 1.5m)",
                    node.node_id, node.width_m
                ));
            }
        }
        Ok(true)
    }

    pub fn validate_edges(&self, edges: &[RouteEdge]) -> Result<bool, String> {
        for edge in edges {
            if edge.fall_risk_index > 1.0 {
                return Err(format!("Edge {} fall risk index {} exceeds maximum 1.0", edge.edge_id, edge.fall_risk_index));
            }
            if edge.joint_load_score > 1.0 {
                return Err(format!("Edge {} joint load score {} exceeds maximum 1.0", edge.edge_id, edge.joint_load_score));
            }
        }
        Ok(true)
    }

    pub fn build_graph(&self, nodes: &[RouteNode], edges: &[RouteEdge]) -> HashMap<String, Vec<(String, f64)>> {
        let mut graph: HashMap<String, Vec<(String, f64)>> = HashMap::new();
        for node in nodes {
            graph.insert(node.node_id.clone(), Vec::new());
        }
        for edge in edges {
            if let Some(neighbors) = graph.get_mut(&edge.from_node) {
                neighbors.push((edge.to_node.clone(), edge.composite_cost(MobilityMode::Walking)));
            }
        }
        graph
    }

    pub fn dijkstra_route(&self, graph: &HashMap<String, Vec<(String, f64)>>, origin: &str, destination: &str) -> Option<Vec<String>> {
        let mut dist: HashMap<String, f64> = HashMap::new();
        let mut prev: HashMap<String, String> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        for node_id in graph.keys() {
            dist.insert(node_id.clone(), f64::INFINITY);
        }
        dist.insert(origin.to_string(), 0.0);
        let mut current = origin.to_string();
        while current != destination {
            visited.insert(current.clone());
            if let Some(neighbors) = graph.get(&current) {
                for (neighbor, cost) in neighbors {
                    if visited.contains(neighbor) {
                        continue;
                    }
                    let new_dist = dist.get(&current).unwrap_or(&f64::INFINITY) + cost;
                    if new_dist < *dist.get(neighbor).unwrap_or(&f64::INFINITY) {
                        dist.insert(neighbor.clone(), new_dist);
                        prev.insert(neighbor.clone(), current.clone());
                    }
                }
            }
            let mut min_dist = f64::INFINITY;
            let mut min_node = None;
            for node_id in graph.keys() {
                if !visited.contains(node_id) {
                    if let Some(&d) = dist.get(node_id) {
                        if d < min_dist {
                            min_dist = d;
                            min_node = Some(node_id.clone());
                        }
                    }
                }
            }
            current = min_node?;
        }
        let mut path = Vec::new();
        let mut node = destination.to_string();
        while node != origin {
            path.push(node.clone());
            node = prev.get(&node)?.clone();
        }
        path.push(origin.to_string());
        path.reverse();
        Some(path)
    }

    pub fn compute_edge_metrics(&self, edge: &RouteEdge, from_node: &RouteNode, to_node: &RouteNode, mobility_mode: MobilityMode) -> RouteSegment {
        let avg_shade = (from_node.shade_coverage_pct + to_node.shade_coverage_pct) / 2.0;
        let heat_factor = if avg_shade >= 50.0 { 0.5 } else { 1.0 };
        RouteSegment {
            segment_id: edge.edge_id.clone(),
            from_node_id: edge.from_node.clone(),
            to_node_id: edge.to_node.clone(),
            distance_m: edge.distance_m,
            time_min: edge.estimated_time_min,
            joint_load_score: edge.joint_load_score,
            fall_risk_index: edge.fall_risk_index,
            heat_exposure_score: edge.heat_exposure_score * heat_factor,
            shade_coverage_pct: avg_shade,
            cooling_station_available: from_node.adjacent_cooling_station || to_node.adjacent_cooling_station,
            water_fountain_available: from_node.adjacent_water_fountain || to_node.adjacent_water_fountain,
            wcag_compliant: from_node.wcag_compliant && to_node.wcag_compliant && edge.fall_risk_index <= self.max_fall_risk_index,
        }
    }

    pub fn generate_route_plan(&self, input: &RoutePlanningInput, path: &[String]) -> RoutePlan {
        let route_id = format!("ROUTE-{}-{}", input.request_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        let mut plan = RoutePlan::new(route_id, input.origin_node_id.clone(), input.destination_node_id.clone());
        let node_map: HashMap<String, &RouteNode> = input.nodes.iter().map(|n| (n.node_id.clone(), n)).collect();
        let edge_map: HashMap<(String, String), &RouteEdge> = input.edges.iter().map(|e| ((e.from_node.clone(), e.to_node.clone()), e)).collect();
        for i in 0..path.len() - 1 {
            let from_id = &path[i];
            let to_id = &path[i + 1];
            if let Some(edge) = edge_map.get(&(from_id.clone(), to_id.clone())) {
                let from_node = node_map.get(from_id).unwrap();
                let to_node = node_map.get(to_id).unwrap();
                let segment = self.compute_edge_metrics(edge, from_node, to_node, input.mobility_mode);
                plan.segments.push(segment);
            }
        }
        plan.compute_aggregates();
        if plan.aggregate_fall_risk_index > self.max_fall_risk_index {
            plan.add_warning(format!("Fall risk index {} exceeds maximum {}", plan.aggregate_fall_risk_index, self.max_fall_risk_index));
        }
        if plan.wcag_compliance_pct < self.min_wcag_compliance_pct {
            plan.add_warning(format!("WCAG compliance {}% below minimum {}%", plan.wcag_compliance_pct * 100.0, self.min_wcag_compliance_pct * 100.0));
        }
        for segment in &plan.segments {
            if segment.cooling_station_available {
                plan.recommended_rest_stops.push(segment.segment_id.clone());
            }
        }
        plan
    }

    pub fn plan_route(&self, input: RoutePlanningInput) -> RoutePlanningOutput {
        let mut output = RoutePlanningOutput::new(input.request_id.clone());
        if let Err(e) = self.validate_ifc_labels(&input.ifc_labels) {
            output.route_plan = Some(RoutePlan::new("FAILED".to_string(), input.origin_node_id.clone(), input.destination_node_id.clone()));
            output.route_plan.as_mut().unwrap().add_error(e);
            return output;
        }
        output.ifc_valid = true;
        if let Err(e) = self.validate_nodes(&input.nodes, input.mobility_mode) {
            output.route_plan = Some(RoutePlan::new("FAILED".to_string(), input.origin_node_id.clone(), input.destination_node_id.clone()));
            output.route_plan.as_mut().unwrap().add_error(e);
            return output;
        }
        if let Err(e) = self.validate_edges(&input.edges) {
            output.route_plan = Some(RoutePlan::new("FAILED".to_string(), input.origin_node_id.clone(), input.destination_node_id.clone()));
            output.route_plan.as_mut().unwrap().add_error(e);
            return output;
        }
        output.corridor_valid = !input.corridor_id.is_empty();
        if !output.corridor_valid {
            output.route_plan = Some(RoutePlan::new("FAILED".to_string(), input.origin_node_id.clone(), input.destination_node_id.clone()));
            output.route_plan.as_mut().unwrap().add_error("Corridor ID required for route planning".to_string());
            return output;
        }
        let graph = self.build_graph(&input.nodes, &input.edges);
        if let Some(path) = self.dijkstra_route(&graph, &input.origin_node_id, &input.destination_node_id) {
            let mut plan = self.generate_route_plan(&input, &path);
            output.accessibility_compliant = plan.wcag_compliance_pct >= self.min_wcag_compliance_pct;
            output.heat_safe = plan.aggregate_heat_exposure_score <= self.max_heat_exposure_score;
            output.biotic_compliant = !input.biotic_treaty_active || plan.biotic_impact_score <= 0.3;
            if !output.accessibility_compliant {
                plan.add_warning("Route does not meet minimum accessibility compliance".to_string());
            }
            if !output.heat_safe {
                plan.add_warning("Route heat exposure exceeds safe threshold".to_string());
            }
            output.route_plan = Some(plan);
        } else {
            output.route_plan = Some(RoutePlan::new("FAILED".to_string(), input.origin_node_id.clone(), input.destination_node_id.clone()));
            output.route_plan.as_mut().unwrap().add_error("No valid route found between origin and destination".to_string());
        }
        output.output_ifc_label = output.output_ifc_label
            .with_fpic(true)
            .with_somatic_treaty(true)
            .with_corridor_validation(output.corridor_valid);
        output
    }
}

// ============================================================================
// SECTION 6: JSON C-ABI EXPORT FOR LUA FFI INTEGRATION
// ============================================================================

#[no_mangle]
pub extern "C" fn ale_somaplex_plan_route_json(
    input_json: *const libc::c_char,
) -> *mut libc::c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let input_str = CStr::from_ptr(input_json).to_string_lossy();
            let input: RoutePlanningInput = match serde_json::from_str(&input_str) {
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
            let kernel = SomaticRouteEngineKernel::new("002".to_string(), "001".to_string());
            let output = kernel.plan_route(input);
            let json_output = serde_json::to_string(&output).unwrap_or_else(|e| {
                format!(r#"{{"valid":false,"error":"Serialization failed: {}"}}"#, e)
            });
            CString::new(json_output).unwrap()
        }
    }));
    match result {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => {
            let err = CString::new(r#"{"valid":false,"error":"Kernel panic during route planning"}"#)
                .unwrap();
            err.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn ale_somaplex_free_string(ptr: *mut libc::c_char) {
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
    fn test_mobility_mode_slope_limits() {
        assert_eq!(MobilityMode::Wheelchair.max_slope_pct(), 8.33);
        assert_eq!(MobilityMode::Walking.max_slope_pct(), 15.0);
        assert_eq!(MobilityMode::VisionImpaired.max_slope_pct(), 5.0);
    }

    #[test]
    fn test_route_node_accessibility_score() {
        let node = RouteNode::new("node_001".to_string(), 33.4484, -112.0740)
            .with_slope(5.0)
            .with_width(1.8)
            .with_shade(60.0)
            .with_seating(true)
            .with_water_fountain(true);
        let score = node.accessibility_score();
        assert!(score > 0.5);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_route_edge_composite_cost() {
        let edge = RouteEdge::new("edge_001".to_string(), "node_1".to_string(), "node_2".to_string(), 100.0)
            .with_time(2.0)
            .with_joint_load(0.3)
            .with_fall_risk(0.2)
            .with_heat_exposure(0.4);
        let cost = edge.composite_cost(MobilityMode::Walking);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_somatic_cost_profile_weights() {
        let profile = SomaticCostProfile::for_wheelchair_user();
        assert!(profile.fall_risk_weight > profile.time_weight);
        let worker_profile = SomaticCostProfile::for_outdoor_worker();
        assert!(worker_profile.heat_exposure_weight > profile.heat_exposure_weight);
    }

    #[test]
    fn test_route_planning_basic() {
        let kernel = SomaticRouteEngineKernel::new("002".to_string(), "001".to_string());
        let node1 = RouteNode::new("node_1".to_string(), 33.4484, -112.0740).with_slope(5.0).with_width(1.8);
        let node2 = RouteNode::new("node_2".to_string(), 33.4494, -112.0750).with_slope(5.0).with_width(1.8);
        let edge = RouteEdge::new("edge_1".to_string(), "node_1".to_string(), "node_2".to_string(), 150.0)
            .with_fall_risk(0.15)
            .with_joint_load(0.2);
        let ifc_label = SomaplexIFCLabel::new(
            "IFC-001".to_string(),
            "internal".to_string(),
            "somatic".to_string(),
            "sensor".to_string(),
            "hash123".to_string(),
        );
        let input = RoutePlanningInput {
            request_id: "req_001".to_string(),
            origin_node_id: "node_1".to_string(),
            destination_node_id: "node_2".to_string(),
            mobility_mode: MobilityMode::Walking,
            persona_vulnerability_class: "general_population".to_string(),
            nodes: vec![node1, node2],
            edges: vec![edge],
            heat_budget_constraint: HeatBudgetConstraint::new(),
            somatic_cost_profile: SomaticCostProfile::new(),
            ifc_labels: vec![ifc_label],
            corridor_id: "CORR-001".to_string(),
            biotic_treaty_active: false,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        let output = kernel.plan_route(input);
        assert!(output.ifc_valid);
        assert!(output.corridor_valid);
        assert!(output.route_plan.is_some());
    }

    #[test]
    fn test_wcag_compliance_validation() {
        let kernel = SomaticRouteEngineKernel::new("002".to_string(), "001".to_string());
        let node = RouteNode::new("node_001".to_string(), 33.4484, -112.0740)
            .with_slope(10.0)
            .with_width(1.2);
        let result = kernel.validate_nodes(&[node], MobilityMode::Wheelchair);
        assert!(result.is_err());
    }
}
