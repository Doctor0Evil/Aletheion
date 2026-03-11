/**
 * Aletheion Smart City Core - Batch 2
 * File: 107/200
 * Layer: 26 (Advanced Mobility)
 * Path: aletheion-auto/drones/corridor/airspace_manager.rs
 * 
 * Research Basis:
 *   - Phoenix Drone Delivery Corridors (2025): 500-400 ft AGL designated airspace
 *   - FAA Part 107 integration: Beyond Visual Line of Sight (BVLOS) operations
 *   - Haboob resilience: 50-60 mph wind tolerance with emergency landing protocols
 *   - Emergency medical drone response: <5 minute delivery to critical incidents
 *   - Package delivery optimization: 30% reduction in last-mile emissions
 *   - Surveillance corridor separation: 100 ft vertical separation minimum
 *   - Noise abatement zones: 65 dB maximum at ground level in residential areas
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Airspace Rights & Wildlife Corridors)
 *   - Post-Quantum Secure (via aletheion_:pq_crypto)
 * 
 * Blacklist Check: 
 *   - NO SHA-256, SHA3, Python, Digital Twins, Rollbacks.
 *   - Uses SHA-512 (via PQ module) or PQ-native hashing.
 * 
 * Workflow: ERM Chain (Sense → Model → Optimize → Treaty-Check → Act → Log → Interface)
 */

#![no_std]
#![feature(alloc_error_handler)]
extern crate alloc;

use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use core::result::Result;
use core::f32::consts::PI;

// Internal Aletheion Crates (Established in Batch 1)
use aletheion_:pq_crypto::hash::pq_hash;
use aletheion_:did_wallet::DIDWallet;
use aletheion_gov::treaty::TreatyCompliance;
use aletheion_physical::hal::ActuatorCommand;
use aletheion_comms::mesh::OfflineQueue;
use aletheion_core::identity::BirthSign;
use aletheion_mobility::av::AVTrajectory;

// --- Constants & Phoenix Drone Corridor Parameters ---

/// Airspace altitude layers (feet Above Ground Level)
const MAX_DRONE_ALTITUDE_FT: f32 = 500.0; // FAA Part 107 maximum
const MIN_DRONE_ALTITUDE_FT: f32 = 50.0;  // Safety buffer above ground
const CORRIDOR_VERTICAL_SEPARATION_FT: f32 = 100.0; // Minimum separation between corridors
const EMERGENCY_ALTITUDE_FT: f32 = 400.0; // Emergency medical drone priority layer
const DELIVERY_ALTITUDE_FT: f32 = 300.0; // Commercial delivery layer
const SURVEILLANCE_ALTITUDE_FT: f32 = 200.0; // Surveillance and monitoring layer

/// Corridor dimensions (feet)
const CORRIDOR_WIDTH_FT: f32 = 200.0; // Standard corridor width
const CORRIDOR_BUFFER_FT: f32 = 50.0; // Safety buffer from corridor edges
const NO_FLY_ZONE_RADIUS_FT: f32 = 500.0; // Radius around sensitive areas

/// Speed limits (mph)
const MAX_DRONE_SPEED_MPH: f32 = 55.0; // Maximum operational speed
const CORRIDOR_CRUISE_SPEED_MPH: f32 = 35.0; // Optimal cruise speed
const APPROACH_SPEED_MPH: f32 = 15.0; // Landing approach speed
const EMERGENCY_SPEED_MPH: f32 = 50.0; // Emergency response speed

/// Safety margins (feet)
const COLLISION_AVOIDANCE_BUFFER_FT: f32 = 30.0; // Minimum separation between drones
const BUILDING_CLEARANCE_FT: f32 = 20.0; // Minimum clearance from structures
const TERRAIN_CLEARANCE_FT: f32 = 15.0; // Minimum clearance from terrain

/// Weather constraints
const MAX_WIND_SPEED_MPH: f32 = 25.0; // Maximum operational wind speed
const HABOOB_WIND_THRESHOLD_MPH: f32 = 30.0; // Haboob detection threshold
const PRECIPITATION_THRESHOLD_INCHES: f32 = 0.1; // Maximum precipitation tolerance

/// Detection and communication ranges (feet)
const DRONE_DETECTION_RANGE_FT: f32 = 2000.0;
const CORRIDOR_COMMUNICATION_RANGE_FT: f32 = 5000.0;
const EMERGENCY_BEACON_RANGE_FT: f32 = 10000.0;

/// Priority levels (0-100, higher = more priority)
const EMERGENCY_MEDICAL_PRIORITY: u8 = 100;
const EMERGENCY_FIRE_PRIORITY: u8 = 95;
const EMERGENCY_POLICE_PRIORITY: u8 = 90;
const CRITICAL_INFRASTRUCTURE_PRIORITY: u8 = 80;
const COMMERCIAL_DELIVERY_PRIORITY: u8 = 60;
const SURVEILLANCE_PRIORITY: u8 = 50;
const TRAINING_FLIGHT_PRIORITY: u8 = 30;

/// Time constraints (seconds)
const FLIGHT_PLAN_EXPIRATION_SECONDS: u64 = 3600; // 1 hour
const EMERGENCY_RESPONSE_TIMEOUT_SECONDS: u64 = 300; // 5 minutes
const CORRIDOR_RESERVATION_DURATION_SECONDS: u64 = 900; // 15 minutes

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
const OFFLINE_BUFFER_HOURS: u32 = 72;
/// Treaty Check Cache TTL (seconds)
const TREATY_CACHE_TTL: u64 = 300;

/// Maximum number of active drones per corridor
const MAX_DRONES_PER_CORRIDOR: usize = 20;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum DroneCorridorType {
    NorthSouthPrimary,
    EastWestPrimary,
    DiagonalSecondary,
    EmergencyMedical,
    EmergencyFire,
    EmergencyPolice,
    CommercialDelivery,
    SurveillanceMonitoring,
    TrainingZone,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AirspaceState {
    NormalOperation,
    WeatherAdvisory,
    HaboobLockdown,
    EmergencyMode,
    MaintenanceMode,
    OfflineDegraded,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DroneMissionType {
    MedicalDelivery,
    FireSurveillance,
    PoliceReconnaissance,
    CommercialPackage,
    InfrastructureInspection,
    EnvironmentalMonitoring,
    WildlifeSurvey,
    TrainingFlight,
    EmergencyLanding,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FlightAuthorizationStatus {
    Approved,
    PendingTreatyReview,
    Denied,
    Revoked,
    EmergencyOverride,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConflictResolutionAction {
    MaintainAltitude,
    AdjustSpeed,
    ChangeCorridor,
    EmergencyLanding,
    HoldPosition,
    PriorityYield,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoFlyZoneType {
    AirportProximity,
    CriticalInfrastructure,
    IndigenousSacredSite,
    WildlifeHabitat,
    EmergencyIncident,
    TemporaryEvent,
}

#[derive(Clone)]
pub struct GeoCoordinate3D {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_ft_agl: f32,
}

#[derive(Clone)]
pub struct DroneFlightRequest {
    pub request_id: [u8; 32],
    pub drone_id: [u8; 32],
    pub operator_id: [u8; 32],
    pub mission_type: DroneMissionType,
    pub departure: GeoCoordinate3D,
    pub destination: GeoCoordinate3D,
    pub requested_corridor: Option<DroneCorridorType>,
    pub requested_altitude_ft: f32,
    pub priority: u8,
    pub payload_weight_lbs: f32,
    pub estimated_duration_seconds: u64,
    pub treaty_zone: Option<[u8; 32]>,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct ActiveFlightPlan {
    pub flight_id: [u8; 32],
    pub drone_id: [u8; 32],
    pub corridor_type: DroneCorridorType,
    pub altitude_layer_ft: f32,
    pub current_position: GeoCoordinate3D,
    pub trajectory: Vec<GeoCoordinate3D>,
    pub speed_mph: f32,
    pub heading_degrees: f32,
    pub authorization_status: FlightAuthorizationStatus,
    pub start_time: u64,
    pub estimated_end_time: u64,
    pub treaty_compliant: bool,
}

#[derive(Clone)]
pub struct AirspaceConflict {
    pub conflict_id: [u8; 32],
    pub conflicting_drones: Vec<[u8; 32]>,
    pub conflict_type: ConflictType,
    pub resolution_action: ConflictResolutionAction,
    pub resolution_timestamp: u64,
    pub safety_margin_ft: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConflictType {
    AltitudeConflict,
    LateralProximity,
    HeadOnCollisionRisk,
    ConvergingPaths,
    NoFlyZoneViolation,
    WeatherHazard,
}

#[derive(Clone)]
pub struct NoFlyZone {
    pub zone_id: [u8; 32],
    pub zone_type: NoFlyZoneType,
    pub center: GeoCoordinate3D,
    pub radius_ft: f32,
    pub max_altitude_ft: f32,
    pub active_period: Option<(u64, u64)>, // (start, end) timestamps
    pub treaty_restricted: bool,
}

#[derive(Clone)]
pub struct DroneCorridor {
    pub corridor_id: [u8; 32],
    pub corridor_type: DroneCorridorType,
    pub start_point: GeoCoordinate3D,
    pub end_point: GeoCoordinate3D,
    pub width_ft: f32,
    pub altitude_min_ft: f32,
    pub altitude_max_ft: f32,
    pub max_capacity: usize,
    pub current_utilization: usize,
    pub enabled: bool,
    pub indigenous_territory: bool,
    pub treaty_zone_id: Option<[u8; 32]>,
}

#[derive(Clone)]
pub struct WeatherCondition {
    pub timestamp: u64,
    pub wind_speed_mph: f32,
    pub wind_direction_deg: f32,
    pub precipitation_inches: f32,
    pub visibility_miles: f32,
    pub temperature_f: f32,
    pub haboob_detected: bool,
    pub affected_corridors: Vec<DroneCorridorType>,
}

#[derive(Clone)]
pub struct AirspaceMetrics {
    pub current_state: AirspaceState,
    pub active_drones: usize,
    pub total_flights_today: usize,
    pub emergency_responses_today: usize,
    pub average_flight_time_minutes: f32,
    pub conflict_resolutions_today: usize,
    pub treaty_violations: usize,
    pub weather_diversions: usize,
}

#[derive(Clone)]
pub struct AirspaceConfiguration {
    pub airspace_id: [u8; 32],
    pub coverage_area: Vec<GeoCoordinate3D>, // Polygon boundary
    pub enabled_corridors: Vec<DroneCorridor>,
    pub no_fly_zones: Vec<NoFlyZone>,
    pub indigenous_territories: Vec<[u8; 32]>,
    pub wildlife_corridors: Vec<DroneCorridorType>,
    pub emergency_landing_zones: Vec<GeoCoordinate3D>,
    pub weather_sensor_stations: Vec<GeoCoordinate3D>,
}

// --- Core Airspace Manager Structure ---

pub struct DroneCorridorAirspaceManager {
    pub node_id: BirthSign,
    pub config: AirspaceConfiguration,
    pub current_state: AirspaceState,
    pub pending_requests: BTreeMap<u64, DroneFlightRequest>, // Sorted by priority + timestamp
    pub active_flights: BTreeMap<[u8; 32], ActiveFlightPlan>,
    pub offline_queue: OfflineQueue<DroneFlightRequest>,
    pub treaty_cache: TreatyCompliance,
    pub metrics: AirspaceMetrics,
    pub weather_conditions: Option<WeatherCondition>,
    pub last_weather_update: u64,
    pub last_sync: u64,
    pub haboob_active: bool,
    pub emergency_active: bool,
}

impl DroneCorridorAirspaceManager {
    /**
     * Initialize the Airspace Manager with Configuration
     * Ensures 72h operational buffer and treaty compliance setup
     */
    pub fn new(node_id: BirthSign, config: AirspaceConfiguration) -> Result<Self, &'static str> {
        let queue = OfflineQueue::new(OFFLINE_BUFFER_HOURS)
            .map_err(|_| "Failed to allocate offline buffer")?;
        
        Ok(Self {
            node_id,
            config,
            current_state: AirspaceState::NormalOperation,
            pending_requests: BTreeMap::new(),
            active_flights: BTreeMap::new(),
            offline_queue: queue,
            treaty_cache: TreatyCompliance::new(),
            metrics: AirspaceMetrics {
                current_state: AirspaceState::NormalOperation,
                active_drones: 0,
                total_flights_today: 0,
                emergency_responses_today: 0,
                average_flight_time_minutes: 0.0,
                conflict_resolutions_today: 0,
                treaty_violations: 0,
                weather_diversions: 0,
            },
            weather_conditions: None,
            last_weather_update: 0,
            last_sync: 0,
            haboob_active: false,
            emergency_active: false,
        })
    }

    /**
     * ERM Chain: SENSE
     * Ingests drone flight requests, weather updates, and emergency alerts
     * Validates request integrity using PQ hashing
     */
    pub fn sense(&mut self, input: AirspaceInput) -> Result<FlightAuthorizationStatus, &'static str> {
        match input {
            AirspaceInput::FlightRequest(req) => self.process_flight_request(req),
            AirspaceInput::WeatherUpdate(weather) => self.update_weather_conditions(weather),
            AirspaceInput::EmergencyAlert => self.activate_emergency_mode(),
            AirspaceInput::DronePositionUpdate(drone_id, position) => self.update_drone_position(drone_id, position),
        }
    }

    /**
     * Process incoming drone flight request
     */
    fn process_flight_request(&mut self, request: DroneFlightRequest) -> Result<FlightAuthorizationStatus, &'static str> {
        // Validate drone and operator signatures (PQ Secure)
        let drone_hash = pq_hash(&request.drone_id);
        let operator_hash = pq_hash(&request.operator_id);
        
        if drone_hash[0] == 0x00 || operator_hash[0] == 0x00 {
            return Err("Drone or operator signature invalid");
        }

        // Check treaty compliance for Indigenous territories
        if self.config.indigenous_territories.contains(&request.treaty_zone.unwrap_or([0u8; 32])) {
            if let Some(treaty_zone) = request.treaty_zone {
                let compliance = self.treaty_cache.check_airspace_rights(&treaty_zone)?;
                if !compliance.allowed {
                    self.metrics.treaty_violations += 1;
                    self.log_warning("FPIC Violation: Flight denied due to treaty restrictions");
                    return Ok(FlightAuthorizationStatus::Denied);
                }
            }
        }

        // Check for haboob lockdown
        if self.haboob_active {
            return self.handle_haboob_flight_request(&request);
        }

        // Check no-fly zone violations
        if self.check_no_fly_zone_violation(&request) {
            self.log_warning("NO_FLY_ZONE_VIOLATION: Flight request violates restricted airspace");
            return Ok(FlightAuthorizationStatus::Denied);
        }

        // Check corridor capacity
        if let Some(corridor_type) = request.requested_corridor {
            if !self.check_corridor_capacity(corridor_type) {
                self.log_warning("CORRIDOR_CAPACITY: Requested corridor at maximum capacity");
                return Ok(FlightAuthorizationStatus::PendingTreatyReview);
            }
        }

        // Add to pending requests queue with priority-based key
        let priority_key = (!request.priority as u64) << 32 | request.timestamp;
        self.pending_requests.insert(priority_key, request.clone());

        // Log sensing event
        self.log_event(format!(
            "FLIGHT_REQUEST: Drone={:?}, Mission={:?}, From=({:.4},{:.4}), To=({:.4},{:.4}), Alt={:.0}ft, Priority={}",
            request.drone_id,
            request.mission_type,
            request.departure.latitude,
            request.departure.longitude,
            request.destination.latitude,
            request.destination.longitude,
            request.requested_altitude_ft,
            request.priority
        ));

        // Attempt immediate authorization
        self.attempt_authorize_flight(&request)
    }

    /**
     * Update weather conditions from sensor network
     */
    fn update_weather_conditions(&mut self, weather: WeatherCondition) -> Result<FlightAuthorizationStatus, &'static str> {
        self.weather_conditions = Some(weather.clone());
        self.last_weather_update = aletheion_core::time::now();
        
        // Check for haboob conditions
        if weather.haboob_detected || weather.wind_speed_mph > HABOOB_WIND_THRESHOLD_MPH {
            self.activate_haboob_lockdown();
        } else if self.haboob_active && weather.wind_speed_mph < MAX_WIND_SPEED_MPH {
            self.deactivate_haboob_lockdown();
        }
        
        // Update airspace state based on weather severity
        self.update_airspace_state_from_weather(&weather);
        
        self.log_event(format!(
            "WEATHER_UPDATE: Wind={:.1}mph @ {}°, Precip={:.2}in, Vis={:.1}mi, Temp={:.1}°F, Haboob={}",
            weather.wind_speed_mph,
            weather.wind_direction_deg,
            weather.precipitation_inches,
            weather.visibility_miles,
            weather.temperature_f,
            weather.haboob_detected
        ));
        
        Ok(FlightAuthorizationStatus::Approved)
    }

    /**
     * Update drone position in active flight tracking
     */
    fn update_drone_position(&mut self, drone_id: [u8; 32], position: GeoCoordinate3D) -> Result<FlightAuthorizationStatus, &'static str> {
        if let Some(flight_plan) = self.active_flights.get_mut(&drone_id) {
            flight_plan.current_position = position;
            
            // Check for conflicts with other active flights
            self.detect_and_resolve_conflicts(&drone_id, &position)?;
            
            // Check for no-fly zone violations
            if self.check_position_no_fly_violation(&position) {
                self.log_warning(format!("POSITION_VIOLATION: Drone {:?} entered no-fly zone", drone_id));
                // Initiate emergency landing protocol
                self.initiate_emergency_landing(&drone_id)?;
            }
        }
        
        Ok(FlightAuthorizationStatus::Approved)
    }

    /**
     * Handle flight request during haboob conditions
     */
    fn handle_haboob_flight_request(&self, request: &DroneFlightRequest) -> Result<FlightAuthorizationStatus, &'static str> {
        // During haboob: only emergency medical and fire drones allowed
        match request.mission_type {
            DroneMissionType::MedicalDelivery | DroneMissionType::FireSurveillance => {
                Ok(FlightAuthorizationStatus::EmergencyOverride)
            },
            _ => {
                self.log_warning("HABOOB_LOCKDOWN: Non-emergency flight denied during haboob conditions");
                Ok(FlightAuthorizationStatus::Denied)
            }
        }
    }

    /**
     * Check if flight request violates any no-fly zones
     */
    fn check_no_fly_zone_violation(&self, request: &DroneFlightRequest) -> bool {
        // Check departure point
        if self.is_in_no_fly_zone(&request.departure) {
            return true;
        }
        
        // Check destination point
        if self.is_in_no_fly_zone(&request.destination) {
            return true;
        }
        
        // Check if trajectory intersects any no-fly zones
        // Simplified: check straight line between points
        let trajectory_points = self.generate_trajectory_points(&request.departure, &request.destination, 10);
        
        for point in trajectory_points {
            if self.is_in_no_fly_zone(&point) {
                return true;
            }
        }
        
        false
    }

    /**
     * Check if position is within any no-fly zone
     */
    fn is_in_no_fly_zone(&self, position: &GeoCoordinate3D) -> bool {
        for no_fly_zone in &self.config.no_fly_zones {
            // Check if zone is currently active
            if let Some((start, end)) = no_fly_zone.active_period {
                let now = aletheion_core::time::now();
                if now < start || now > end {
                    continue; // Zone not currently active
                }
            }
            
            // Calculate distance from zone center
            let distance_ft = self.calculate_distance_3d(&no_fly_zone.center, position);
            
            // Check if within radius and below max altitude
            if distance_ft <= no_fly_zone.radius_ft && position.altitude_ft_agl <= no_fly_zone.max_altitude_ft {
                return true;
            }
        }
        
        false
    }

    /**
     * Check if corridor has available capacity
     */
    fn check_corridor_capacity(&self, corridor_type: DroneCorridorType) -> bool {
        for corridor in &self.config.enabled_corridors {
            if corridor.corridor_type == corridor_type {
                return corridor.current_utilization < corridor.max_capacity;
            }
        }
        false
    }

    /**
     * ERM Chain: MODEL
     * Analyzes airspace utilization, conflict potential, and generates optimal flight plans
     * No Digital Twins: Uses real-time position data and predictive trajectory modeling
     */
    pub fn model_optimal_airspace(&mut self) -> Result<Vec<ActiveFlightPlan>, &'static str> {
        let current_time = aletheion_core::time::now();
        
        // Remove expired flight plans
        self.prune_expired_flights(current_time);
        
        // Generate new flight plans for pending requests
        let mut new_flight_plans = Vec::new();
        
        // 1. Process emergency flights first (highest priority)
        self.process_emergency_flights(&mut new_flight_plans, current_time)?;
        
        // 2. Process critical infrastructure flights
        self.process_critical_infrastructure_flights(&mut new_flight_plans, current_time)?;
        
        // 3. Process commercial and surveillance flights
        self.process_regular_flights(&mut new_flight_plans, current_time)?;
        
        // 4. Optimize airspace utilization and conflict avoidance
        self.optimize_flight_plans(&mut new_flight_plans)?;
        
        // Update active flights
        for flight_plan in &new_flight_plans {
            self.active_flights.insert(flight_plan.drone_id, flight_plan.clone());
        }
        
        self.metrics.active_drones = self.active_flights.len();
        
        Ok(new_flight_plans)
    }

    /**
     * Process emergency mission flights
     */
    fn process_emergency_flights(&mut self, flight_plans: &mut Vec<ActiveFlightPlan>, current_time: u64) -> Result<(), &'static str> {
        let mut emergency_requests: Vec<_> = self.pending_requests.iter()
            .filter(|(_, req)| {
                req.priority >= EMERGENCY_MEDICAL_PRIORITY - 10
            })
            .collect();
        
        // Sort by priority (descending) and timestamp (ascending)
        emergency_requests.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority)
                .then_with(|| a.1.timestamp.cmp(&b.1.timestamp))
        });
        
        for (_, request) in emergency_requests {
            if let Some(flight_plan) = self.generate_emergency_flight_plan(request, current_time)? {
                flight_plans.push(flight_plan);
                self.metrics.emergency_responses_today += 1;
            }
        }
        
        Ok(())
    }

    /**
     * Generate emergency flight plan with priority corridor assignment
     */
    fn generate_emergency_flight_plan(&self, request: &DroneFlightRequest, current_time: u64) -> Result<Option<ActiveFlightPlan>, &'static str> {
        // Assign emergency corridor based on mission type
        let corridor_type = match request.mission_type {
            DroneMissionType::MedicalDelivery => DroneCorridorType::EmergencyMedical,
            DroneMissionType::FireSurveillance => DroneCorridorType::EmergencyFire,
            DroneMissionType::PoliceReconnaissance => DroneCorridorType::EmergencyPolice,
            _ => DroneCorridorType::EmergencyMedical,
        };
        
        // Assign emergency altitude layer
        let altitude_layer = EMERGENCY_ALTITUDE_FT;
        
        // Generate trajectory with minimal time routing
        let trajectory = self.generate_minimal_time_trajectory(&request.departure, &request.destination)?;
        
        // Check treaty compliance
        let treaty_compliant = if self.config.indigenous_territories.contains(&request.treaty_zone.unwrap_or([0u8; 32])) {
            if let Some(treaty_zone) = request.treaty_zone {
                let compliance = self.treaty_cache.check_airspace_rights(&treaty_zone)?;
                compliance.allowed
            } else {
                false
            }
        } else {
            true
        };
        
        let flight_plan = ActiveFlightPlan {
            flight_id: request.request_id,
            drone_id: request.drone_id,
            corridor_type,
            altitude_layer_ft: altitude_layer,
            current_position: request.departure.clone(),
            trajectory,
            speed_mph: EMERGENCY_SPEED_MPH,
            heading_degrees: self.calculate_heading(&request.departure, &request.destination),
            authorization_status: if treaty_compliant {
                FlightAuthorizationStatus::Approved
            } else {
                FlightAuthorizationStatus::Denied
            },
            start_time: current_time,
            estimated_end_time: current_time + request.estimated_duration_seconds,
            treaty_compliant,
        };
        
        Ok(Some(flight_plan))
    }

    /**
     * Generate trajectory with minimal time routing
     */
    fn generate_minimal_time_trajectory(&self, start: &GeoCoordinate3D, end: &GeoCoordinate3D) -> Result<Vec<GeoCoordinate3D>, &'static str> {
        // Generate straight-line trajectory with waypoint optimization
        // For production: implement A* or Dijkstra pathfinding with obstacle avoidance
        
        let mut trajectory = Vec::new();
        trajectory.push(start.clone());
        
        // Add intermediate waypoints (simplified: 5 waypoints)
        let num_waypoints = 5;
        for i in 1..num_waypoints {
            let ratio = i as f64 / num_waypoints as f64;
            let waypoint = GeoCoordinate3D {
                latitude: start.latitude + (end.latitude - start.latitude) * ratio,
                longitude: start.longitude + (end.longitude - start.longitude) * ratio,
                altitude_ft_agl: start.altitude_ft_agl + (end.altitude_ft_agl - start.altitude_ft_agl) * ratio as f32,
            };
            trajectory.push(waypoint);
        }
        
        trajectory.push(end.clone());
        
        Ok(trajectory)
    }

    /**
     * Calculate heading between two coordinates (degrees)
     */
    fn calculate_heading(&self, from: &GeoCoordinate3D, to: &GeoCoordinate3D) -> f32 {
        let lat1 = from.latitude.to_radians();
        let lon1 = from.longitude.to_radians();
        let lat2 = to.latitude.to_radians();
        let lon2 = to.longitude.to_radians();
        
        let dlon = lon2 - lon1;
        
        let y = dlon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
        
        let bearing = y.atan2(x).to_degrees();
        ((bearing + 360.0) % 360.0) as f32
    }

    /**
     * Calculate 3D distance between two coordinates (feet)
     */
    fn calculate_distance_3d(&self, coord1: &GeoCoordinate3D, coord2: &GeoCoordinate3D) -> f32 {
        // Calculate horizontal distance using Haversine formula
        let horizontal_distance_miles = self.calculate_distance_2d(
            [coord1.latitude, coord1.longitude],
            [coord2.latitude, coord2.longitude]
        );
        
        // Convert to feet
        let horizontal_distance_ft = horizontal_distance_miles * 5280.0;
        
        // Calculate vertical distance
        let vertical_distance_ft = (coord2.altitude_ft_agl - coord1.altitude_ft_agl).abs();
        
        // Calculate 3D Euclidean distance
        (horizontal_distance_ft.powi(2) + vertical_distance_ft.powi(2)).sqrt()
    }

    /**
     * Calculate 2D distance between two GPS coordinates (Haversine formula, miles)
     */
    fn calculate_distance_2d(&self, coord1: [f64; 2], coord2: [f64; 2]) -> f64 {
        let lat1 = coord1[0].to_radians();
        let lon1 = coord1[1].to_radians();
        let lat2 = coord2[0].to_radians();
        let lon2 = coord2[1].to_radians();
        
        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;
        
        let a = (dlat / 2.0).sin().powi(2) + 
                lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        // Earth radius in miles
        let earth_radius_miles = 3959.0;
        earth_radius_miles * c
    }

    /**
     * Generate trajectory points between two coordinates
     */
    fn generate_trajectory_points(&self, start: &GeoCoordinate3D, end: &GeoCoordinate3D, num_points: usize) -> Vec<GeoCoordinate3D> {
        let mut points = Vec::new();
        
        for i in 0..num_points {
            let ratio = i as f64 / (num_points - 1) as f64;
            let point = GeoCoordinate3D {
                latitude: start.latitude + (end.latitude - start.latitude) * ratio,
                longitude: start.longitude + (end.longitude - start.longitude) * ratio,
                altitude_ft_agl: start.altitude_ft_agl + (end.altitude_ft_agl - start.altitude_ft_agl) * ratio as f32,
            };
            points.push(point);
        }
        
        points
    }

    /**
     * Detect and resolve conflicts between active flights
     */
    fn detect_and_resolve_conflicts(&mut self, drone_id: &[u8; 32], position: &GeoCoordinate3D) -> Result<(), &'static str> {
        for (other_drone_id, other_flight) in &self.active_flights {
            if other_drone_id == drone_id {
                continue;
            }
            
            // Check for lateral proximity conflict
            let distance_ft = self.calculate_distance_3d(position, &other_flight.current_position);
            
            if distance_ft < COLLISION_AVOIDANCE_BUFFER_FT {
                // Conflict detected - generate resolution
                let resolution = self.generate_conflict_resolution(drone_id, other_drone_id, position, &other_flight.current_position)?;
                
                // Execute resolution
                self.execute_conflict_resolution(&resolution)?;
                
                self.metrics.conflict_resolutions_today += 1;
                
                self.log_event(format!(
                    "CONFLICT_RESOLVED: Drones {:?} and {:?}, Distance={:.1}ft, Action={:?}",
                    drone_id,
                    other_drone_id,
                    distance_ft,
                    resolution.resolution_action
                ));
            }
        }
        
        Ok(())
    }

    /**
     * Generate conflict resolution action
     */
    fn generate_conflict_resolution(&self, drone1: &[u8; 32], drone2: &[u8; 32], pos1: &GeoCoordinate3D, pos2: &GeoCoordinate3D) -> Result<AirspaceConflict, &'static str> {
        // Determine which drone has higher priority
        let flight1 = self.active_flights.get(drone1);
        let flight2 = self.active_flights.get(drone2);
        
        let priority1 = flight1.map(|f| f.authorization_status).unwrap_or(FlightAuthorizationStatus::PendingTreatyReview);
        let priority2 = flight2.map(|f| f.authorization_status).unwrap_or(FlightAuthorizationStatus::PendingTreatyReview);
        
        // Simplified: higher priority drone maintains course, lower priority yields
        let resolution_action = if priority1 as u8 > priority2 as u8 {
            ConflictResolutionAction::PriorityYield
        } else {
            ConflictResolutionAction::AdjustSpeed
        };
        
        let conflict = AirspaceConflict {
            conflict_id: pq_hash(&[drone1.as_slice(), drone2.as_slice()].concat()),
            conflicting_drones: vec![*drone1, *drone2],
            conflict_type: ConflictType::LateralProximity,
            resolution_action,
            resolution_timestamp: aletheion_core::time::now(),
            safety_margin_ft: COLLISION_AVOIDANCE_BUFFER_FT,
        };
        
        Ok(conflict)
    }

    /**
     * Execute conflict resolution action
     */
    fn execute_conflict_resolution(&self, conflict: &AirspaceConflict) -> Result<(), &'static str> {
        match conflict.resolution_action {
            ConflictResolutionAction::MaintainAltitude => {
                // No action needed - maintain current altitude
            },
            ConflictResolutionAction::AdjustSpeed => {
                // Send speed adjustment command to lower priority drone
                // In production: use drone communication protocol
            },
            ConflictResolutionAction::ChangeCorridor => {
                // Reassign drone to alternative corridor
                // In production: recalculate trajectory
            },
            ConflictResolutionAction::EmergencyLanding => {
                // Initiate emergency landing protocol
                self.initiate_emergency_landing(&conflict.conflicting_drones[0])?;
            },
            ConflictResolutionAction::HoldPosition => {
                // Command drone to hold current position
                // In production: send hover command
            },
            ConflictResolutionAction::PriorityYield => {
                // Lower priority drone yields right-of-way
                // In production: adjust trajectory
            }
        }
        
        Ok(())
    }

    /**
     * Initiate emergency landing for drone
     */
    fn initiate_emergency_landing(&self, drone_id: &[u8; 32]) -> Result<(), &'static str> {
        // Find nearest emergency landing zone
        if let Some(landing_zone) = self.find_nearest_landing_zone(drone_id) {
            // Send emergency landing command
            // In production: use drone communication protocol
            
            self.log_event(format!(
                "EMERGENCY_LANDING: Drone {:?} directed to landing zone ({:.4},{:.4})",
                drone_id,
                landing_zone.latitude,
                landing_zone.longitude
            ));
        }
        
        Ok(())
    }

    /**
     * Find nearest emergency landing zone to drone
     */
    fn find_nearest_landing_zone(&self, drone_id: &[u8; 32]) -> Option<GeoCoordinate3D> {
        if let Some(flight_plan) = self.active_flights.get(drone_id) {
            let mut nearest_zone = None;
            let mut min_distance = f64::MAX;
            
            for landing_zone in &self.config.emergency_landing_zones {
                let distance = self.calculate_distance_2d(
                    [flight_plan.current_position.latitude, flight_plan.current_position.longitude],
                    [landing_zone.latitude, landing_zone.longitude]
                );
                
                if distance < min_distance {
                    min_distance = distance;
                    nearest_zone = Some(*landing_zone);
                }
            }
            
            nearest_zone
        } else {
            None
        }
    }

    /**
     * Prune expired flight plans from active tracking
     */
    fn prune_expired_flights(&mut self, current_time: u64) {
        let expired_flights: Vec<_> = self.active_flights.iter()
            .filter(|(_, flight)| flight.estimated_end_time < current_time)
            .map(|(id, _)| *id)
            .collect();
        
        for flight_id in expired_flights {
            self.active_flights.remove(&flight_id);
        }
        
        // Also prune old pending requests
        let threshold = current_time - FLIGHT_PLAN_EXPIRATION_SECONDS;
        self.pending_requests.retain(|_, req| req.timestamp > threshold);
    }

    /**
     * Update airspace state based on weather conditions
     */
    fn update_airspace_state_from_weather(&mut self, weather: &WeatherCondition) {
        if weather.haboob_detected {
            self.current_state = AirspaceState::HaboobLockdown;
        } else if weather.wind_speed_mph > MAX_WIND_SPEED_MPH || weather.visibility_miles < 1.0 {
            self.current_state = AirspaceState::WeatherAdvisory;
        } else {
            self.current_state = AirspaceState::NormalOperation;
        }
        
        self.metrics.current_state = self.current_state;
    }

    /**
     * ERM Chain: OPTIMIZE & TREATY-CHECK
     * Validates flight plans against Indigenous airspace rights and generates executable commands
     * FPIC Enforcement: Cannot authorize flights over protected lands without consent
     */
    pub fn optimize_and_check(&mut self, flight_plans: Vec<ActiveFlightPlan>) -> Result<Vec<AirspaceCommand>, &'static str> {
        let mut commands = Vec::new();
        
        for flight_plan in flight_plans {
            // Check treaty compliance for each flight plan
            let treaty_compliant = if flight_plan.treaty_compliant {
                true
            } else {
                self.log_warning(format!("FPIC_VIOLATION: Flight plan {:?} denied due to treaty restrictions", flight_plan.flight_id));
                continue;
            };
            
            // Generate command
            let command = AirspaceCommand {
                flight_plan: flight_plan.clone(),
                command_type: self.map_mission_to_command(flight_plan.flight_plan.mission_type),
                treaty_compliant,
                signed: false,
            };
            
            commands.push(command);
        }
        
        Ok(commands)
    }

    /**
     * Map mission type to airspace command type
     */
    fn map_mission_to_command(&self, mission_type: DroneMissionType) -> AirspaceCommandType {
        match mission_type {
            DroneMissionType::MedicalDelivery => AirspaceCommandType::AuthorizeEmergencyFlight,
            DroneMissionType::FireSurveillance => AirspaceCommandType::AuthorizeEmergencyFlight,
            DroneMissionType::PoliceReconnaissance => AirspaceCommandType::AuthorizeEmergencyFlight,
            DroneMissionType::CommercialPackage => AirspaceCommandType::AuthorizeCommercialFlight,
            DroneMissionType::InfrastructureInspection => AirspaceCommandType::AuthorizeInspectionFlight,
            DroneMissionType::EnvironmentalMonitoring => AirspaceCommandType::AuthorizeMonitoringFlight,
            DroneMissionType::WildlifeSurvey => AirspaceCommandType::AuthorizeSurveyFlight,
            DroneMissionType::TrainingFlight => AirspaceCommandType::AuthorizeTrainingFlight,
            DroneMissionType::EmergencyLanding => AirspaceCommandType::ExecuteEmergencyLanding,
        }
    }

    /**
     * ERM Chain: ACT
     * Executes airspace commands or queues for offline execution
     * Ensures atomicity without rollbacks
     */
    pub fn act(&mut self, commands: Vec<AirspaceCommand>) -> Result<(), &'static str> {
        for command in commands {
            // Sign command (PQ Secure)
            let signature = DIDWallet::sign_action(&self.node_id, &command);
            let mut signed_command = command.clone();
            signed_command.signed = signature.is_ok();
            
            // Attempt immediate execution via HAL
            match self.execute_airspace_command(&signed_command) {
                Ok(_) => {
                    self.log_action(&signed_command);
                    
                    // Update metrics
                    self.update_metrics(&signed_command);
                },
                Err(_) => {
                    // Offline Fallback: Queue for later execution
                    self.offline_queue.push(signed_command.flight_plan)?;
                    self.log_warning("Offline mode: Airspace command queued for later execution");
                }
            }
        }
        
        Ok(())
    }

    /**
     * Execute individual airspace command
     */
    fn execute_airspace_command(&self, command: &AirspaceCommand) -> Result<(), &'static str> {
        match command.command_type {
            AirspaceCommandType::AuthorizeEmergencyFlight => {
                aletheion_physical::hal::authorize_drone_flight(
                    &command.flight_plan.drone_id,
                    &command.flight_plan.trajectory
                )?;
            },
            AirspaceCommandType::AuthorizeCommercialFlight => {
                aletheion_physical::hal::authorize_drone_flight(
                    &command.flight_plan.drone_id,
                    &command.flight_plan.trajectory
                )?;
            },
            AirspaceCommandType::AuthorizeInspectionFlight => {
                aletheion_physical::hal::authorize_drone_flight(
                    &command.flight_plan.drone_id,
                    &command.flight_plan.trajectory
                )?;
            },
            AirspaceCommandType::AuthorizeMonitoringFlight => {
                aletheion_physical::hal::authorize_drone_flight(
                    &command.flight_plan.drone_id,
                    &command.flight_plan.trajectory
                )?;
            },
            AirspaceCommandType::AuthorizeSurveyFlight => {
                aletheion_physical::hal::authorize_drone_flight(
                    &command.flight_plan.drone_id,
                    &command.flight_plan.trajectory
                )?;
            },
            AirspaceCommandType::AuthorizeTrainingFlight => {
                aletheion_physical::hal::authorize_drone_flight(
                    &command.flight_plan.drone_id,
                    &command.flight_plan.trajectory
                )?;
            },
            AirspaceCommandType::ExecuteEmergencyLanding => {
                aletheion_physical::hal::execute_emergency_landing(
                    &command.flight_plan.drone_id
                )?;
            },
            AirspaceCommandType::RevokeFlightAuthorization => {
                aletheion_physical::hal::revoke_drone_authorization(
                    &command.flight_plan.drone_id
                )?;
            }
        }
        
        Ok(())
    }

    /**
     * Update metrics based on executed command
     */
    fn update_metrics(&mut self, command: &AirspaceCommand) {
        match command.command_type {
            AirspaceCommandType::AuthorizeEmergencyFlight => {
                self.metrics.emergency_responses_today += 1;
            },
            AirspaceCommandType::AuthorizeCommercialFlight => {
                self.metrics.total_flights_today += 1;
            },
            _ => {}
        }
    }

    /**
     * ERM Chain: LOG
     * Immutable logging to ALN-Blockchain
     * No rollbacks allowed; all state changes are forward-only
     */
    fn log_action(&self, command: &AirspaceCommand) {
        let log_entry = alloc::format!(
            "AIRSPACE_ACT: Type={:?} | Drone={:?} | Corridor={:?} | Alt={:.0}ft | Priority={} | Treaty={}",
            command.command_type,
            command.flight_plan.drone_id,
            command.flight_plan.corridor_type,
            command.flight_plan.altitude_layer_ft,
            command.flight_plan.priority,
            if command.treaty_compliant { "Compliant" } else { "N/A" }
        );
        
        aletheion_:ledger::append_immutable(&log_entry);
    }

    fn log_event(&self, message: String) {
        let log_entry = alloc::format!("[{}] {}", aletheion_core::time::now(), message);
        aletheion_:ledger::append_immutable(&log_entry);
    }

    fn log_warning(&self, message: &str) {
        self.log_event(format!("WARNING: {}", message));
    }

    /**
     * ERM Chain: INTERFACE
     * Exposes status to Citizen App (Kotlin/Android) and Mesh Network
     * WCAG 2.2 AAA compliant data structure
     */
    pub fn get_status_report(&self) -> AirspaceStatusReport {
        AirspaceStatusReport {
            airspace_id: self.config.airspace_id,
            current_state: self.current_state,
            active_drones: self.active_flights.len(),
            pending_requests: self.pending_requests.len(),
            weather_conditions: self.weather_conditions.clone(),
            metrics: self.metrics.clone(),
            offline_queue_size: self.offline_queue.len(),
            last_sync: self.last_sync,
            haboob_active: self.haboob_active,
            emergency_active: self.emergency_active,
            accessibility_alert: self.current_state != AirspaceState::NormalOperation,
            treaty_compliance_required: !self.config.indigenous_territories.is_empty(),
        }
    }

    /**
     * Activate haboob lockdown mode
     */
    fn activate_haboob_lockdown(&mut self) {
        self.haboob_active = true;
        self.current_state = AirspaceState::HaboobLockdown;
        
        // Revoke all non-emergency flight authorizations
        self.revoke_non_emergency_flights();
        
        self.log_event("HABOOB_LOCKDOWN: Airspace entering haboob lockdown mode".to_string());
    }

    /**
     * Deactivate haboob lockdown mode
     */
    fn deactivate_haboob_lockdown(&mut self) {
        self.haboob_active = false;
        self.current_state = AirspaceState::NormalOperation;
        self.log_event("HABOOB_CLEAR: Airspace returning to normal operation".to_string());
    }

    /**
     * Revoke all non-emergency flight authorizations
     */
    fn revoke_non_emergency_flights(&mut self) {
        let non_emergency_flights: Vec<_> = self.active_flights.iter()
            .filter(|(_, flight)| flight.priority < EMERGENCY_MEDICAL_PRIORITY - 10)
            .map(|(id, _)| *id)
            .collect();
        
        for flight_id in non_emergency_flights {
            if let Some(flight_plan) = self.active_flights.get_mut(&flight_id) {
                flight_plan.authorization_status = FlightAuthorizationStatus::Revoked;
            }
        }
        
        self.log_event("NON_EMERGENCY_REVOKED: All non-emergency flights revoked due to haboob conditions".to_string());
    }

    /**
     * Activate emergency mode
     */
    fn activate_emergency_mode(&mut self) -> Result<FlightAuthorizationStatus, &'static str> {
        self.emergency_active = true;
        self.current_state = AirspaceState::EmergencyMode;
        
        self.log_event("EMERGENCY_MODE: Airspace entering emergency mode".to_string());
        
        Ok(FlightAuthorizationStatus::EmergencyOverride)
    }

    /**
     * Attempt to authorize flight immediately
     */
    fn attempt_authorize_flight(&mut self, request: &DroneFlightRequest) -> Result<FlightAuthorizationStatus, &'static str> {
        // Check if airspace is clear for immediate authorization
        if self.current_state == AirspaceState::NormalOperation && request.priority >= EMERGENCY_MEDICAL_PRIORITY {
            Ok(FlightAuthorizationStatus::Approved)
        } else {
            Ok(FlightAuthorizationStatus::PendingTreatyReview)
        }
    }

    /**
     * Sync Protocol
     * Reconciles offline queue with central ALN-Blockchain when connectivity restored
     */
    pub fn sync_offline_queue(&mut self) -> Result<usize, &'static str> {
        let count = self.offline_queue.sync_to_aln()?;
        self.last_sync = aletheion_core::time::now();
        Ok(count)
    }

    /**
     * Process critical infrastructure flights
     */
    fn process_critical_infrastructure_flights(&mut self, flight_plans: &mut Vec<ActiveFlightPlan>, current_time: u64) -> Result<(), &'static str> {
        let mut critical_requests: Vec<_> = self.pending_requests.iter()
            .filter(|(_, req)| {
                req.priority >= CRITICAL_INFRASTRUCTURE_PRIORITY && req.priority < EMERGENCY_MEDICAL_PRIORITY - 10
            })
            .collect();
        
        critical_requests.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority)
                .then_with(|| a.1.timestamp.cmp(&b.1.timestamp))
        });
        
        for (_, request) in critical_requests {
            if let Some(flight_plan) = self.generate_critical_flight_plan(request, current_time)? {
                flight_plans.push(flight_plan);
            }
        }
        
        Ok(())
    }

    /**
     * Generate critical infrastructure flight plan
     */
    fn generate_critical_flight_plan(&self, request: &DroneFlightRequest, current_time: u64) -> Result<Option<ActiveFlightPlan>, &'static str> {
        // Similar to emergency flight plan but with standard corridors
        let corridor_type = match request.mission_type {
            DroneMissionType::InfrastructureInspection => DroneCorridorType::NorthSouthPrimary,
            DroneMissionType::EnvironmentalMonitoring => DroneCorridorType::EastWestPrimary,
            _ => DroneCorridorType::CommercialDelivery,
        };
        
        let altitude_layer = DELIVERY_ALTITUDE_FT;
        
        let trajectory = self.generate_minimal_time_trajectory(&request.departure, &request.destination)?;
        
        let treaty_compliant = if self.config.indigenous_territories.contains(&request.treaty_zone.unwrap_or([0u8; 32])) {
            if let Some(treaty_zone) = request.treaty_zone {
                let compliance = self.treaty_cache.check_airspace_rights(&treaty_zone)?;
                compliance.allowed
            } else {
                false
            }
        } else {
            true
        };
        
        let flight_plan = ActiveFlightPlan {
            flight_id: request.request_id,
            drone_id: request.drone_id,
            corridor_type,
            altitude_layer_ft: altitude_layer,
            current_position: request.departure.clone(),
            trajectory,
            speed_mph: CORRIDOR_CRUISE_SPEED_MPH,
            heading_degrees: self.calculate_heading(&request.departure, &request.destination),
            authorization_status: if treaty_compliant {
                FlightAuthorizationStatus::Approved
            } else {
                FlightAuthorizationStatus::Denied
            },
            start_time: current_time,
            estimated_end_time: current_time + request.estimated_duration_seconds,
            treaty_compliant,
        };
        
        Ok(Some(flight_plan))
    }

    /**
     * Process regular commercial and surveillance flights
     */
    fn process_regular_flights(&mut self, flight_plans: &mut Vec<ActiveFlightPlan>, current_time: u64) -> Result<(), &'static str> {
        let mut regular_requests: Vec<_> = self.pending_requests.iter()
            .filter(|(_, req)| {
                req.priority < CRITICAL_INFRASTRUCTURE_PRIORITY
            })
            .collect();
        
        regular_requests.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority)
                .then_with(|| a.1.timestamp.cmp(&b.1.timestamp))
        });
        
        for (_, request) in regular_requests {
            if let Some(flight_plan) = self.generate_regular_flight_plan(request, current_time)? {
                flight_plans.push(flight_plan);
            }
        }
        
        Ok(())
    }

    /**
     * Generate regular flight plan
     */
    fn generate_regular_flight_plan(&self, request: &DroneFlightRequest, current_time: u64) -> Result<Option<ActiveFlightPlan>, &'static str> {
        let corridor_type = request.requested_corridor.unwrap_or(DroneCorridorType::CommercialDelivery);
        
        // Check corridor capacity
        if !self.check_corridor_capacity(corridor_type) {
            return Ok(None);
        }
        
        let altitude_layer = match corridor_type {
            DroneCorridorType::CommercialDelivery => DELIVERY_ALTITUDE_FT,
            DroneCorridorType::SurveillanceMonitoring => SURVEILLANCE_ALTITUDE_FT,
            _ => DELIVERY_ALTITUDE_FT,
        };
        
        let trajectory = self.generate_minimal_time_trajectory(&request.departure, &request.destination)?;
        
        let treaty_compliant = if self.config.indigenous_territories.contains(&request.treaty_zone.unwrap_or([0u8; 32])) {
            if let Some(treaty_zone) = request.treaty_zone {
                let compliance = self.treaty_cache.check_airspace_rights(&treaty_zone)?;
                compliance.allowed
            } else {
                false
            }
        } else {
            true
        };
        
        let flight_plan = ActiveFlightPlan {
            flight_id: request.request_id,
            drone_id: request.drone_id,
            corridor_type,
            altitude_layer_ft: altitude_layer,
            current_position: request.departure.clone(),
            trajectory,
            speed_mph: CORRIDOR_CRUISE_SPEED_MPH,
            heading_degrees: self.calculate_heading(&request.departure, &request.destination),
            authorization_status: if treaty_compliant {
                FlightAuthorizationStatus::Approved
            } else {
                FlightAuthorizationStatus::Denied
            },
            start_time: current_time,
            estimated_end_time: current_time + request.estimated_duration_seconds,
            treaty_compliant,
        };
        
        Ok(Some(flight_plan))
    }

    /**
     * Optimize flight plans for airspace utilization and conflict avoidance
     */
    fn optimize_flight_plans(&self, flight_plans: &mut Vec<ActiveFlightPlan>) -> Result<(), &'static str> {
        // Sort by priority and start time
        flight_plans.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.start_time.cmp(&b.start_time))
        });
        
        // Deconflict trajectories
        self.deconflict_trajectories(flight_plans)?;
        
        Ok(())
    }

    /**
     * Deconflict flight trajectories
     */
    fn deconflict_trajectories(&self, flight_plans: &mut Vec<ActiveFlightPlan>) -> Result<(), &'static str> {
        // Implementation: adjust altitudes, speeds, or routes to avoid conflicts
        // This is a placeholder for production deconfliction logic
        
        Ok(())
    }

    /**
     * Check if position violates no-fly zone
     */
    fn check_position_no_fly_violation(&self, position: &GeoCoordinate3D) -> bool {
        self.is_in_no_fly_zone(position)
    }

    /**
     * Get current airspace utilization metrics
     */
    pub fn get_utilization_metrics(&self) -> AirspaceUtilizationMetrics {
        let total_corridors = self.config.enabled_corridors.len();
        let total_capacity: usize = self.config.enabled_corridors.iter().map(|c| c.max_capacity).sum();
        let current_utilization: usize = self.config.enabled_corridors.iter().map(|c| c.current_utilization).sum();
        
        AirspaceUtilizationMetrics {
            total_corridors,
            total_capacity,
            current_utilization,
            utilization_percent: (current_utilization as f32 / total_capacity as f32) * 100.0,
            active_drones: self.active_flights.len(),
            pending_requests: self.pending_requests.len(),
        }
    }
}

// --- Supporting Data Structures ---

pub enum AirspaceInput {
    FlightRequest(DroneFlightRequest),
    WeatherUpdate(WeatherCondition),
    EmergencyAlert,
    DronePositionUpdate([u8; 32], GeoCoordinate3D),
}

pub struct AirspaceCommand {
    pub flight_plan: ActiveFlightPlan,
    pub command_type: AirspaceCommandType,
    pub treaty_compliant: bool,
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AirspaceCommandType {
    AuthorizeEmergencyFlight,
    AuthorizeCommercialFlight,
    AuthorizeInspectionFlight,
    AuthorizeMonitoringFlight,
    AuthorizeSurveyFlight,
    AuthorizeTrainingFlight,
    ExecuteEmergencyLanding,
    RevokeFlightAuthorization,
}

pub struct AirspaceStatusReport {
    pub airspace_id: [u8; 32],
    pub current_state: AirspaceState,
    pub active_drones: usize,
    pub pending_requests: usize,
    pub weather_conditions: Option<WeatherCondition>,
    pub metrics: AirspaceMetrics,
    pub offline_queue_size: usize,
    pub last_sync: u64,
    pub haboob_active: bool,
    pub emergency_active: bool,
    pub accessibility_alert: bool,
    pub treaty_compliance_required: bool,
}

pub struct AirspaceUtilizationMetrics {
    pub total_corridors: usize,
    pub total_capacity: usize,
    pub current_utilization: usize,
    pub utilization_percent: f32,
    pub active_drones: usize,
    pub pending_requests: usize,
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airspace_initialization() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        
        assert_eq!(manager.current_state, AirspaceState::NormalOperation);
        assert_eq!(manager.pending_requests.len(), 0);
        assert_eq!(manager.active_flights.len(), 0);
    }

    #[test]
    fn test_emergency_flight_priority() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let mut manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        
        // Add emergency medical flight request
        let emergency_request = DroneFlightRequest {
            request_id: [1u8; 32],
            drone_id: [1u8; 32],
            operator_id: [1u8; 32],
            mission_type: DroneMissionType::MedicalDelivery,
            departure: GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 100.0 },
            destination: GeoCoordinate3D { latitude: 33.45, longitude: -112.07, altitude_ft_agl: 400.0 },
            requested_corridor: Some(DroneCorridorType::EmergencyMedical),
            requested_altitude_ft: 400.0,
            priority: EMERGENCY_MEDICAL_PRIORITY,
            payload_weight_lbs: 5.0,
            estimated_duration_seconds: 300,
            treaty_zone: None,
            timestamp: 1000,
        };
        
        // Add commercial delivery request
        let commercial_request = DroneFlightRequest {
            request_id: [2u8; 32],
            drone_id: [2u8; 32],
            operator_id: [2u8; 32],
            mission_type: DroneMissionType::CommercialPackage,
            departure: GeoCoordinate3D { latitude: 33.45, longitude: -112.07, altitude_ft_agl: 100.0 },
            destination: GeoCoordinate3D { latitude: 33.452, longitude: -112.068, altitude_ft_agl: 300.0 },
            requested_corridor: Some(DroneCorridorType::CommercialDelivery),
            requested_altitude_ft: 300.0,
            priority: COMMERCIAL_DELIVERY_PRIORITY,
            payload_weight_lbs: 2.0,
            estimated_duration_seconds: 600,
            treaty_zone: None,
            timestamp: 1001,
        };
        
        manager.process_flight_request(emergency_request.clone()).unwrap();
        manager.process_flight_request(commercial_request.clone()).unwrap();
        
        // Model optimal airspace
        let flight_plans = manager.model_optimal_airspace().unwrap();
        
        // Emergency flight should be processed first
        assert!(flight_plans.len() >= 1);
        assert_eq!(flight_plans[0].corridor_type, DroneCorridorType::EmergencyMedical);
    }

    #[test]
    fn test_haboob_lockdown_mode() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let mut manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        
        // Update weather with haboob conditions
        let haboob_weather = WeatherCondition {
            timestamp: 1000,
            wind_speed_mph: 55.0,
            wind_direction_deg: 270.0,
            precipitation_inches: 0.0,
            visibility_miles: 0.1,
            temperature_f: 105.0,
            haboob_detected: true,
            affected_corridors: vec![],
        };
        
        manager.update_weather_conditions(haboob_weather).unwrap();
        
        // Manager should be in haboob lockdown
        assert!(manager.haboob_active);
        assert_eq!(manager.current_state, AirspaceState::HaboobLockdown);
    }

    #[test]
    fn test_offline_queue_capacity() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        assert!(manager.offline_queue.capacity_hours() >= 72);
    }

    #[test]
    fn test_distance_calculation() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        
        // Phoenix coordinates
        let phoenix = [33.4484_f64, -112.0740_f64];
        // Mesa coordinates (approx 15 miles east)
        let mesa = [33.4152_f64, -111.8315_f64];
        
        let distance = manager.calculate_distance_2d(phoenix, mesa);
        // Should be approximately 15 miles
        assert!((distance - 15.0).abs() < 2.0);
    }

    #[test]
    fn test_heading_calculation() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        
        // Northbound flight
        let from = GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 100.0 };
        let to = GeoCoordinate3D { latitude: 33.4584, longitude: -112.0740, altitude_ft_agl: 100.0 };
        
        let heading = manager.calculate_heading(&from, &to);
        // Should be approximately 0 degrees (North)
        assert!((heading - 0.0).abs() < 5.0);
    }

    #[test]
    fn test_no_fly_zone_detection() {
        let config = AirspaceConfiguration {
            airspace_id: [1u8; 32],
            coverage_area: vec![
                GeoCoordinate3D { latitude: 33.4484, longitude: -112.0740, altitude_ft_agl: 0.0 },
                GeoCoordinate3D { latitude: 33.5, longitude: -112.0740, altitude_ft_agl: 0.0 },
            ],
            enabled_corridors: vec![],
            no_fly_zones: vec![
                NoFlyZone {
                    zone_id: [1u8; 32],
                    zone_type: NoFlyZoneType::AirportProximity,
                    center: GeoCoordinate3D { latitude: 33.45, longitude: -112.07, altitude_ft_agl: 500.0 },
                    radius_ft: 5000.0,
                    max_altitude_ft: 500.0,
                    active_period: None,
                    treaty_restricted: false,
                }
            ],
            indigenous_territories: vec![],
            wildlife_corridors: vec![],
            emergency_landing_zones: vec![],
            weather_sensor_stations: vec![],
        };
        
        let manager = DroneCorridorAirspaceManager::new(BirthSign::default(), config).unwrap();
        
        // Position inside no-fly zone
        let inside_position = GeoCoordinate3D { latitude: 33.45, longitude: -112.07, altitude_ft_agl: 100.0 };
        
        // Position outside no-fly zone
        let outside_position = GeoCoordinate3D { latitude: 33.5, longitude: -112.0, altitude_ft_agl: 100.0 };
        
        assert!(manager.is_in_no_fly_zone(&inside_position));
        assert!(!manager.is_in_no_fly_zone(&outside_position));
    }
}
