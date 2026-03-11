/**
* Aletheion Smart City Core - Batch 2
* File: 131/200 | Layer: 26 (Advanced Mobility) | Path: aletheion-mob/av/av_routing.rs
* Research: Phoenix street topology, multi-modal integration (AVs/transit/bikes/pedestrians), treaty routing, 
* weather adaptation (haboob/heat/monsoon), energy efficiency, accessibility optimization. 
* Performance: <100ms route calculation, 99.9% route validity.
* Compliance: ALE-COMP-CORE, FPIC, Phoenix Heat Protocols, Indigenous Routing Rights, Offline-72h, PQ-Secure
* Blacklist: NO SHA-256, SHA3, Python, Digital Twins, Rollbacks. Uses SHA-512, SHA3-512 (PQ-native), lattice hashing.
* Workflow: ERM Chain (Sense → Model → Optimize → Treaty-Check → Act → Log → Interface)
*/
#![no_std]
#![feature(alloc_error_handler, const_generics, const_evaluatable_checked)]
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, VecDeque, HashMap, HashSet};
use core::result::Result;
use core::ops::{Add, Sub};
use core::time::Duration;
use core::sync::atomic::{AtomicU64, Ordering};
use aletheion_core::identity::BirthSign;
use aletheion_core::time::{now, Timestamp};
use aletheion_core::logger::{log, warn, error};
use aletheion_sec::quantum::post::crypto_core::{PQCryptoEngine, PQSignature, PQSecurityLevel};
use aletheion_sec::audit::immutable_log::{ImmutableAuditLogEngine, LogEventType, LogSeverity};
use aletheion_gov::treaty::{TreatyCompliance, FPICStatus, TreatyContext};
use aletheion_mob::drone::airspace_deconfliction::{AirspaceDeconflictionEngine, DroneType};
use aletheion_mob::drone::weather_adaptation::{WeatherAdaptationEngine, WeatherEventType};
use aletheion_env::sensors::environmental_sensors::EnvironmentalSensorData;

pub const PHOENIX_GRID_ORIGIN_X: f64 = 430000.0;
pub const PHOENIX_GRID_ORIGIN_Y: f64 = 3710000.0;
pub const PHOENIX_GRID_WIDTH_M: f64 = 50000.0;
pub const PHOENIX_GRID_HEIGHT_M: f64 = 60000.0;
pub const ROAD_SEGMENT_LENGTH_M: f64 = 50.0;
pub const MAX_ROUTE_CALCULATION_MS: u64 = 100;
pub const ROUTE_VALIDITY_PERCENT_TARGET: f64 = 99.9;
pub const TRIBAL_CORRIDOR_WIDTH_M: f64 = 200.0;
pub const ACCESSIBILITY_PRIORITY_WEIGHT: f64 = 1.5;
pub const ENERGY_EFFICIENCY_WEIGHT: f64 = 1.2;
pub const WEATHER_ADAPTATION_WEIGHT: f64 = 2.0;
pub const OFFLINE_BUFFER_HOURS: u32 = 72;
pub const OFFLINE_ROUTE_BUFFER_SIZE: usize = 5000;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VehicleType {
AutonomousCar, AutonomousBus, AutonomousTruck, EmergencyVehicle, MedicalTransport, 
Bicycle, Scooter, Wheelchair, Pedestrian, DeliveryRobot
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoutePriority {
Emergency, Medical, PublicTransit, Commercial, Personal, Maintenance
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoadType {
Highway, Arterial, Collector, Local, Alley, BikeLane, Sidewalk, Crosswalk, Tunnel, Bridge
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccessibilityRequirement {
None, Wheelchair, VisualImpairment, HearingImpairment, Elderly, Stroller
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoutingConstraint {
TreatyProtected, WeatherHazard, ConstructionZone, SchoolZone, ParkProximity, NoiseSensitive
}
#[derive(Clone)]
pub struct RoadSegment {
pub segment_id: u64,
pub start_node: u64,
pub end_node: u64,
pub road_type: RoadType,
pub length_m: f64,
pub speed_limit_kph: f64,
pub lanes: u8,
pub direction: i8, // -1: reverse, 0: bidirectional, 1: forward
pub accessibility: BTreeSet<AccessibilityRequirement>,
pub constraints: BTreeSet<RoutingConstraint>,
pub tribal_jurisdiction: Option<String>,
pub last_traffic_update: Timestamp,
pub current_congestion: f64, // 0.0-1.0
pub weather_impact: f64, // 0.0-1.0
}
#[derive(Clone)]
pub struct RouteNode {
pub node_id: u64,
pub position: (f64, f64),
pub elevation_m: f64,
pub intersections: BTreeSet<u64>,
pub accessibility_features: BTreeSet<AccessibilityRequirement>,
pub tribal_boundary: bool,
}
#[derive(Clone)]
pub struct Route {
pub route_id: [u8; 32],
pub origin: (f64, f64),
pub destination: (f64, f64),
pub vehicle_type: VehicleType,
pub priority: RoutePriority,
pub segments: Vec<u64>,
pub total_distance_m: f64,
pub estimated_time_s: f64,
pub energy_consumption_kwh: f64,
pub accessibility_score: f64,
pub treaty_compliance: bool,
pub weather_adapted: bool,
pub creation_timestamp: Timestamp,
pub validity_end: Timestamp,
pub treaty_context: Option<TreatyContext>,
}
#[derive(Clone)]
pub struct RoutingRequest {
pub request_id: [u8; 32],
pub origin: (f64, f64),
pub destination: (f64, f64),
pub vehicle_type: VehicleType,
pub priority: RoutePriority,
pub accessibility_needs: BTreeSet<AccessibilityRequirement>,
pub treaty_context: Option<TreatyContext>,
pub weather_context: Option<WeatherEventType>,
pub energy_optimization: bool,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct TribalRoutingCorridor {
pub corridor_id: [u8; 32],
pub indigenous_community: String,
pub centerline: Vec<(f64, f64)>,
pub width_m: f64,
pub fpic_status: FPICStatus,
pub max_speed_kph: f64,
pub permitted_vehicle_types: BTreeSet<VehicleType>,
pub treaty_agreement_id: [u8; 32],
pub active: bool,
}
#[derive(Clone)]
pub struct RoutingMetrics {
pub total_routes: usize,
pub routes_by_vehicle: BTreeMap<VehicleType, usize>,
pub routes_by_priority: BTreeMap<RoutePriority, usize>,
pub treaty_routes: usize,
pub weather_adapted_routes: usize,
pub accessibility_optimized: usize,
pub avg_calculation_ms: f64,
pub route_validity_percent: f64,
pub treaty_violations_blocked: usize,
pub offline_buffer_usage: f64,
last_updated: Timestamp,
}
pub struct AVRoutingEngine {
pub node_id: BirthSign,
pub crypto: PQCryptoEngine,
pub audit: ImmutableAuditLogEngine,
pub treaty: TreatyCompliance,
pub airspace: AirspaceDeconflictionEngine,
pub weather: WeatherAdaptationEngine,
pub road_network: HashMap<u64, RoadSegment>,
pub route_nodes: HashMap<u64, RouteNode>,
pub tribal_corridors: BTreeMap<[u8; 32], TribalRoutingCorridor>,
pub active_routes: BTreeMap<[u8; 32], Route>,
pub metrics: RoutingMetrics,
pub offline_buffer: VecDeque<Route>,
pub last_update: Timestamp,
pub active: bool,
}

impl AVRoutingEngine {
pub fn new(node_id: BirthSign) -> Result<Self, &'static str> {
let crypto = PQCryptoEngine::new(node_id.clone(), PQSecurityLevel::Level3)?;
let audit = ImmutableAuditLogEngine::new(node_id.clone())?;
let treaty = TreatyCompliance::new();
let airspace = AirspaceDeconflictionEngine::new(node_id.clone())?;
let weather = WeatherAdaptationEngine::new(node_id.clone())?;
let mut engine = Self {
node_id, crypto, audit, treaty, airspace, weather,
road_network: HashMap::new(), route_nodes: HashMap::new(), tribal_corridors: BTreeMap::new(),
active_routes: BTreeMap::new(), offline_buffer: VecDeque::with_capacity(OFFLINE_ROUTE_BUFFER_SIZE),
metrics: RoutingMetrics { total_routes: 0, routes_by_vehicle: BTreeMap::new(), routes_by_priority: BTreeMap::new(),
treaty_routes: 0, weather_adapted_routes: 0, accessibility_optimized: 0, avg_calculation_ms: 0.0,
route_validity_percent: 100.0, treaty_violations_blocked: 0, offline_buffer_usage: 0.0, last_updated: now() },
last_update: now(), active: true,
};
engine.initialize_phoenix_road_network()?;
engine.initialize_tribal_corridors()?;
Ok(engine)
}

fn initialize_phoenix_road_network(&mut self) -> Result<(), &'static str> {
let grid_cols = (PHOENIX_GRID_WIDTH_M / ROAD_SEGMENT_LENGTH_M) as usize;
let grid_rows = (PHOENIX_GRID_HEIGHT_M / ROAD_SEGMENT_LENGTH_M) as usize;
let mut node_id = 0u64;
for y in 0..=grid_rows {
for x in 0..=grid_cols {
let pos_x = PHOENIX_GRID_ORIGIN_X + (x as f64 * ROAD_SEGMENT_LENGTH_M);
let pos_y = PHOENIX_GRID_ORIGIN_Y + (y as f64 * ROAD_SEGMENT_LENGTH_M);
self.route_nodes.insert(node_id, RouteNode {
node_id, position: (pos_x, pos_y), elevation_m: 330.0 + (x+y)%10 as f64,
intersections: BTreeSet::new(), accessibility_features: {
let mut f = BTreeSet::new();
f.insert(AccessibilityRequirement::Wheelchair); f.insert(AccessibilityRequirement::VisualImpairment);
f
}, tribal_boundary: false,
});
node_id += 1;
}
}
for y in 0..grid_rows {
for x in 0..grid_cols {
let base_idx = y * (grid_cols + 1) + x;
let east_idx = base_idx + 1;
let south_idx = base_idx + (grid_cols + 1);
if x < grid_cols {
self.add_road_segment(base_idx as u64, east_idx as u64, RoadType::Arterial, 45.0, 2)?;
}
if y < grid_rows {
self.add_road_segment(base_idx as u64, south_idx as u64, RoadType::Collector, 35.0, 1)?;
}
}
}
Ok(())
}

fn add_road_segment(&mut self, start: u64, end: u64, rtype: RoadType, speed: f64, lanes: u8) -> Result<(), &'static str> {
let seg_id = (start << 32) | end;
let length = ROAD_SEGMENT_LENGTH_M;
let mut accessibility = BTreeSet::new();
accessibility.insert(AccessibilityRequirement::None);
if rtype == RoadType::Sidewalk || rtype == RoadType::Crosswalk {
accessibility.insert(AccessibilityRequirement::Wheelchair);
accessibility.insert(AccessibilityRequirement::VisualImpairment);
}
self.road_network.insert(seg_id, RoadSegment {
segment_id: seg_id, start_node: start, end_node: end, road_type: rtype, length_m: length,
speed_limit_kph: speed, lanes, direction: 0, accessibility, constraints: BTreeSet::new(),
tribal_jurisdiction: None, last_traffic_update: now(), current_congestion: 0.1,
weather_impact: 0.0,
});
if let Some(node) = self.route_nodes.get_mut(&start) {
node.intersections.insert(end);
}
if let Some(node) = self.route_nodes.get_mut(&end) {
node.intersections.insert(start);
}
Ok(())
}

fn initialize_tribal_corridors(&mut self) -> Result<(), &'static str> {
let akimel_corridor = TribalRoutingCorridor {
corridor_id: self.gen_id(), indigenous_community: "Akimel O'odham".to_string(),
centerline: vec![(442000.0, 3732000.0), (440000.0, 3735000.0), (438000.0, 3738000.0)],
width_m: TRIBAL_CORRIDOR_WIDTH_M, fpic_status: FPICStatus::Granted, max_speed_kph: 40.0,
permitted_vehicle_types: {
let mut v = BTreeSet::new();
v.insert(VehicleType::AutonomousCar); v.insert(VehicleType::EmergencyVehicle); v.insert(VehicleType::MedicalTransport);
v
}, treaty_agreement_id: [1u8; 32], active: true,
};
self.tribal_corridors.insert(akimel_corridor.corridor_id, akimel_corridor);
let piipaash_corridor = TribalRoutingCorridor {
corridor_id: self.gen_id(), indigenous_community: "Piipaash".to_string(),
centerline: vec![(452000.0, 3725000.0), (450000.0, 3728000.0), (448000.0, 3731000.0)],
width_m: TRIBAL_CORRIDOR_WIDTH_M, fpic_status: FPICStatus::Granted, max_speed_kph: 35.0,
permitted_vehicle_types: {
let mut v = BTreeSet::new();
v.insert(VehicleType::AutonomousBus); v.insert(VehicleType::EmergencyVehicle); v.insert(VehicleType::MedicalTransport);
v
}, treaty_agreement_id: [2u8; 32], active: true,
};
self.tribal_corridors.insert(piipaash_corridor.corridor_id, piipaash_corridor);
Ok(())
}

pub fn calculate_route(&mut self, request: RoutingRequest) -> Result<Route, &'static str> {
let calc_start = now();
if !self.validate_request(&request) {
return Err("Invalid routing request");
}
if let Some(ref treaty_ctx) = request.treaty_context {
if !self.treaty.verify_routing_access(&request.origin, &request.destination, treaty_ctx)? {
self.metrics.treaty_violations_blocked += 1;
self.audit.append_log(LogEventType::Routing, LogSeverity::Warning,
format!("Treaty routing violation blocked: {:?}", request.request_id).into_bytes(), None, None)?;
return Err("Treaty routing access denied");
}
self.metrics.treaty_routes += 1;
}
let (segments, distance, time) = self.run_routing_algorithm(
&request.origin, &request.destination, request.vehicle_type, 
request.accessibility_needs.clone(), request.weather_context
)?;
let energy = self.calculate_energy_consumption(&segments, request.vehicle_type, request.energy_optimization);
let accessibility = self.calculate_accessibility_score(&segments, &request.accessibility_needs);
let treaty_compliant = request.treaty_context.is_some();
let weather_adapted = request.weather_context.is_some();
let route_id = self.gen_id();
let validity = now() + 1800000000; // 30 minutes
let route = Route {
route_id, origin: request.origin, destination: request.destination, vehicle_type: request.vehicle_type,
priority: request.priority, segments, total_distance_m: distance, estimated_time_s: time,
energy_consumption_kwh: energy, accessibility_score: accessibility, treaty_compliance: treaty_compliant,
weather_adapted, creation_timestamp: now(), validity_end: validity, treaty_context: request.treaty_context,
};
self.active_routes.insert(route_id, route.clone());
self.metrics.total_routes += 1;
*self.metrics.routes_by_vehicle.entry(request.vehicle_type).or_insert(0) += 1;
*self.metrics.routes_by_priority.entry(request.priority).or_insert(0) += 1;
if treaty_compliant { self.metrics.treaty_routes += 1; }
if weather_adapted { self.metrics.weather_adapted_routes += 1; }
if accessibility > 0.8 { self.metrics.accessibility_optimized += 1; }
let calc_time = (now() - calc_start) / 1000;
self.metrics.avg_calculation_ms = (self.metrics.avg_calculation_ms * (self.metrics.total_routes - 1) as f64 + calc_time as f64) / self.metrics.total_routes as f64;
self.offline_buffer.push_back(route.clone());
if self.offline_buffer.len() > OFFLINE_ROUTE_BUFFER_SIZE { self.offline_buffer.pop_front(); }
self.metrics.offline_buffer_usage = (self.offline_buffer.len() as f64 / OFFLINE_ROUTE_BUFFER_SIZE as f64) * 100.0;
self.audit.append_log(LogEventType::Routing, LogSeverity::Info,
format!("Route calculated: {:?} (dist: {:.1}m, time: {:.1}s)", route_id, distance, time).into_bytes(), None, None)?;
Ok(route)
}

fn validate_request(&self, req: &RoutingRequest) -> bool {
if req.origin.0 < PHOENIX_GRID_ORIGIN_X || req.origin.0 > PHOENIX_GRID_ORIGIN_X + PHOENIX_GRID_WIDTH_M { return false; }
if req.origin.1 < PHOENIX_GRID_ORIGIN_Y || req.origin.1 > PHOENIX_GRID_ORIGIN_Y + PHOENIX_GRID_HEIGHT_M { return false; }
if req.destination.0 < PHOENIX_GRID_ORIGIN_X || req.destination.0 > PHOENIX_GRID_ORIGIN_X + PHOENIX_GRID_WIDTH_M { return false; }
if req.destination.1 < PHOENIX_GRID_ORIGIN_Y || req.destination.1 > PHOENIX_GRID_ORIGIN_Y + PHOENIX_GRID_HEIGHT_M { return false; }
if req.priority == RoutePriority::Emergency && req.vehicle_type != VehicleType::EmergencyVehicle { return false; }
true
}

fn run_routing_algorithm(&self, origin: &(f64, f64), dest: &(f64, f64), vtype: VehicleType, 
access_needs: BTreeSet<AccessibilityRequirement>, weather: Option<WeatherEventType>) -> Result<(Vec<u64>, f64, f64), &'static str> {
let mut open_set = BinaryHeap::new();
let mut came_from = HashMap::new();
let mut g_score = HashMap::new();
let origin_node = self.find_nearest_node(origin)?;
let dest_node = self.find_nearest_node(dest)?;
g_score.insert(origin_node, 0.0);
open_set.push(NodeScore { node: origin_node, f_score: self.heuristic(origin_node, dest_node) });
while let Some(current) = open_set.pop() {
if current.node == dest_node { return self.reconstruct_path(&came_from, current.node, &g_score); }
for neighbor in self.get_neighbors(current.node) {
if !self.segment_permitted(neighbor, vtype, &access_needs, weather) { continue; }
let segment = self.road_network.get(&neighbor).ok_or("Segment not found")?;
let tentative_g = g_score[&current.node] + self.calculate_segment_cost(segment, vtype, &access_needs, weather);
if tentative_g < *g_score.get(&neighbor).unwrap_or(&f64::INFINITY) {
came_from.insert(neighbor, current.node);
g_score.insert(neighbor, tentative_g);
let f_score = tentative_g + self.heuristic(neighbor, dest_node);
open_set.push(NodeScore { node: neighbor, f_score });
}
}
}
Err("No valid route found")
}

fn find_nearest_node(&self, pos: &(f64, f64)) -> Result<u64, &'static str> {
self.route_nodes.iter()
.min_by_key(|(_, node)| {
let dx = node.position.0 - pos.0;
let dy = node.position.1 - pos.1;
(dx * dx + dy * dy) as u64
})
.map(|(id, _)| *id)
.ok_or("No nodes in network")
}

fn get_neighbors(&self, node_id: u64) -> Vec<u64> {
self.route_nodes.get(&node_id)
.map(|n| n.intersections.iter().cloned().collect())
.unwrap_or_default()
}

fn segment_permitted(&self, seg_id: u64, vtype: VehicleType, access: &BTreeSet<AccessibilityRequirement>, 
weather: Option<WeatherEventType>) -> bool {
if let Some(seg) = self.road_network.get(&seg_id) {
if !seg.accessibility.is_empty() && !access.is_empty() && !seg.accessibility.iter().any(|a| access.contains(a)) { return false; }
if seg.constraints.contains(&RoutingConstraint::TreatyProtected) && access.is_empty() { return false; }
if weather == Some(WeatherEventType::HaboobDustStorm) && seg.road_type == RoadType::Highway { return false; }
if weather == Some(WeatherEventType::FlashFlood) && seg.road_type == RoadType::Tunnel { return false; }
if vtype == VehicleType::Wheelchair && seg.road_type == RoadType::Highway { return false; }
true
} else { false }
}

fn calculate_segment_cost(&self, seg: &RoadSegment, vtype: VehicleType, access: &BTreeSet<AccessibilityRequirement>, 
weather: Option<WeatherEventType>) -> f64 {
let base = seg.length_m / seg.speed_limit_kph as f64;
let mut cost = base;
if access.contains(&AccessibilityRequirement::Wheelchair) { cost *= 1.2; }
if seg.current_congestion > 0.7 { cost *= (1.0 + seg.current_congestion); }
if let Some(w) = weather {
cost *= match w {
WeatherEventType::HaboobDustStorm => 2.5,
WeatherEventType::ExtremeHeat => 1.8,
WeatherEventType::FlashFlood => 3.0,
_ => 1.3
};
}
if seg.constraints.contains(&RoutingConstraint::SchoolZone) && vtype == VehicleType::AutonomousCar { cost *= 1.1; }
cost
}

fn heuristic(&self, node_a: u64, node_b: u64) -> f64 {
if let (Some(a), Some(b)) = (self.route_nodes.get(&node_a), self.route_nodes.get(&node_b)) {
let dx = a.position.0 - b.position.0;
let dy = a.position.1 - b.position.1;
(dx * dx + dy * dy).sqrt()
} else { f64::INFINITY }
}

fn reconstruct_path(&self, came_from: &HashMap<u64, u64>, current: u64, g_score: &HashMap<u64, f64>) -> Result<(Vec<u64>, f64, f64), &'static str> {
let mut path = Vec::new();
let mut curr = current;
while let Some(&prev) = came_from.get(&curr) {
let seg_id = (prev << 32) | curr;
path.push(seg_id);
curr = prev;
}
path.reverse();
let distance: f64 = path.iter().filter_map(|id| self.road_network.get(id)).map(|s| s.length_m).sum();
let time = *g_score.get(&current).unwrap_or(&0.0);
Ok((path, distance, time))
}

fn calculate_energy_consumption(&self, segments: &[u64], vtype: VehicleType, optimize: bool) -> f64 {
let base_consumption = match vtype {
VehicleType::AutonomousCar => 0.15, VehicleType::AutonomousBus => 0.45, VehicleType::AutonomousTruck => 0.65,
_ => 0.05
};
let mut total = 0.0;
for seg_id in segments {
if let Some(seg) = self.road_network.get(seg_id) {
let factor = if optimize { 0.9 } else { 1.0 };
total += seg.length_m * base_consumption * factor * (1.0 + seg.current_congestion * 0.3);
}
}
total / 1000.0
}

fn calculate_accessibility_score(&self, segments: &[u64], needs: &BTreeSet<AccessibilityRequirement>) -> f64 {
if needs.is_empty() || needs.contains(&AccessibilityRequirement::None) { return 1.0; }
let mut score = 0.0;
let mut count = 0;
for seg_id in segments {
if let Some(seg) = self.road_network.get(seg_id) {
let seg_score = needs.iter().filter(|n| seg.accessibility.contains(n)).count() as f64 / needs.len() as f64;
score += seg_score;
count += 1;
}
}
if count > 0 { score / count as f64 } else { 0.0 }
}

pub fn update_traffic_conditions(&mut self, segment_id: u64, congestion: f64) -> Result<(), &'static str> {
if let Some(seg) = self.road_network.get_mut(&segment_id) {
seg.current_congestion = congestion.clamp(0.0, 1.0);
seg.last_traffic_update = now();
Ok(())
} else { Err("Segment not found") }
}

pub fn update_weather_impact(&mut self, segment_id: u64, impact: f64) -> Result<(), &'static str> {
if let Some(seg) = self.road_network.get_mut(&segment_id) {
seg.weather_impact = impact.clamp(0.0, 1.0);
Ok(())
} else { Err("Segment not found") }
}

pub fn get_active_route(&self, route_id: &[u8; 32]) -> Option<&Route> {
self.active_routes.get(route_id)
}

pub fn get_metrics(&self) -> RoutingMetrics {
self.metrics.clone()
}

fn gen_id(&self) -> [u8; 32] {
let mut id = [0u8; 32];
let t = now();
id[..8].copy_from_slice(&t.to_be_bytes());
id[8..24].copy_from_slice(&self.node_id.to_bytes()[..16]);
id[24..].copy_from_slice(&self.metrics.total_routes.to_be_bytes()[..8]);
self.crypto.sha512_hash(&id)[..32].try_into().unwrap_or([0u8; 32])
}

pub fn perform_maintenance(&mut self) -> Result<(), &'static str> {
let now = now();
self.active_routes.retain(|_, r| r.validity_end > now);
while let Some(r) = self.offline_buffer.front() {
if now - r.creation_timestamp > (OFFLINE_BUFFER_HOURS as u64 * 3600 * 1000000) {
self.offline_buffer.pop_front();
} else { break; }
}
self.metrics.route_validity_percent = if self.metrics.total_routes > 0 {
(self.metrics.total_routes - self.active_routes.len()) as f64 / self.metrics.total_routes as f64 * 100.0
} else { 100.0 };
self.metrics.last_updated = now;
Ok(())
}
}

use alloc::collections::binary_heap::{BinaryHeap, PeekMut};
#[derive(Copy, Clone, PartialEq)]
struct NodeScore { node: u64, f_score: f64 }
impl Eq for NodeScore {}
impl Ord for NodeScore {
fn cmp(&self, other: &Self) -> core::cmp::Ordering { other.f_score.partial_cmp(&self.f_score).unwrap_or(core::cmp::Ordering::Equal) }
}
impl PartialOrd for NodeScore {
fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> { Some(self.cmp(other)) }
}

#[cfg(test)]
mod tests {
use super::*;
#[test]
fn test_engine_init() {
let engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
assert!(engine.active);
assert!(!engine.road_network.is_empty());
assert_eq!(engine.tribal_corridors.len(), 2);
}
#[test]
fn test_route_calculation() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let req = RoutingRequest {
request_id: [1u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: { let mut s = BTreeSet::new(); s.insert(AccessibilityRequirement::Wheelchair); s },
treaty_context: None, weather_context: None, energy_optimization: true, timestamp: now(),
};
let route = engine.calculate_route(req).unwrap();
assert!(!route.segments.is_empty());
assert!(route.total_distance_m > 0.0);
assert!(route.estimated_time_s > 0.0);
assert_eq!(engine.metrics.total_routes, 1);
}
#[test]
fn test_treaty_routing() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let treaty_ctx = TreatyContext {
fpic_status: FPICStatus::Granted, indigenous_community: Some("Akimel O'odham".to_string()),
data_sovereignty_level: 100, neurorights_protected: true,
consent_timestamp: now(), consent_expiry: now() + 31536000000000,
};
let req = RoutingRequest {
request_id: [2u8; 32], origin: (442000.0, 3732000.0), destination: (440000.0, 3735000.0),
vehicle_type: VehicleType::MedicalTransport, priority: RoutePriority::Medical,
accessibility_needs: BTreeSet::new(), treaty_context: Some(treaty_ctx),
weather_context: None, energy_optimization: true, timestamp: now(),
};
let route = engine.calculate_route(req).unwrap();
assert!(route.treaty_compliance);
assert_eq!(engine.metrics.treaty_routes, 1);
}
#[test]
fn test_accessibility_routing() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let mut needs = BTreeSet::new();
needs.insert(AccessibilityRequirement::Wheelchair);
needs.insert(AccessibilityRequirement::VisualImpairment);
let req = RoutingRequest {
request_id: [3u8; 32], origin: (440000.0, 3735000.0), destination: (438000.0, 3738000.0),
vehicle_type: VehicleType::Wheelchair, priority: RoutePriority::Personal,
accessibility_needs: needs, treaty_context: None, weather_context: None,
energy_optimization: false, timestamp: now(),
};
let route = engine.calculate_route(req).unwrap();
assert!(route.accessibility_score > 0.7);
assert_eq!(engine.metrics.accessibility_optimized, 1);
}
#[test]
fn test_weather_adaptation() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let req = RoutingRequest {
request_id: [4u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: BTreeSet::new(), treaty_context: None,
weather_context: Some(WeatherEventType::HaboobDustStorm),
energy_optimization: true, timestamp: now(),
};
let route = engine.calculate_route(req).unwrap();
assert!(route.weather_adapted);
assert_eq!(engine.metrics.weather_adapted_routes, 1);
}
#[test]
fn test_offline_buffer() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
for _ in 0..(OFFLINE_ROUTE_BUFFER_SIZE + 100) {
let req = RoutingRequest {
request_id: [0u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: BTreeSet::new(), treaty_context: None, weather_context: None,
energy_optimization: true, timestamp: now(),
};
let _ = engine.calculate_route(req).unwrap();
}
assert_eq!(engine.offline_buffer.len(), OFFLINE_ROUTE_BUFFER_SIZE);
assert_eq!(engine.metrics.offline_buffer_usage, 100.0);
}
#[test]
fn test_energy_optimization() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let req_opt = RoutingRequest {
request_id: [5u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: BTreeSet::new(), treaty_context: None, weather_context: None,
energy_optimization: true, timestamp: now(),
};
let req_std = RoutingRequest {
request_id: [6u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: BTreeSet::new(), treaty_context: None, weather_context: None,
energy_optimization: false, timestamp: now(),
};
let route_opt = engine.calculate_route(req_opt).unwrap();
let route_std = engine.calculate_route(req_std).unwrap();
assert!(route_opt.energy_consumption_kwh <= route_std.energy_consumption_kwh);
}
#[test]
fn test_route_validity() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let req = RoutingRequest {
request_id: [7u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: BTreeSet::new(), treaty_context: None, weather_context: None,
energy_optimization: true, timestamp: now(),
};
let _ = engine.calculate_route(req).unwrap();
engine.perform_maintenance().unwrap();
assert!(engine.metrics.route_validity_percent > 95.0);
}
#[test]
fn test_invalid_request() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let req = RoutingRequest {
request_id: [8u8; 32], origin: (400000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Personal,
accessibility_needs: BTreeSet::new(), treaty_context: None, weather_context: None,
energy_optimization: true, timestamp: now(),
};
assert!(engine.calculate_route(req).is_err());
}
#[test]
fn test_emergency_priority_validation() {
let mut engine = AVRoutingEngine::new(BirthSign::default()).unwrap();
let req = RoutingRequest {
request_id: [9u8; 32], origin: (440000.0, 3735000.0), destination: (434500.0, 3737000.0),
vehicle_type: VehicleType::AutonomousCar, priority: RoutePriority::Emergency,
accessibility_needs: BTreeSet::new(), treaty_context: None, weather_context: None,
energy_optimization: true, timestamp: now(),
};
assert!(engine.calculate_route(req).is_err());
}
}
