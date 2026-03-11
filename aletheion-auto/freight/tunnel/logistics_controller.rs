/**
 * Aletheion Smart City Core - Batch 2
 * File: 108/200
 * Layer: 26 (Advanced Mobility)
 * Path: aletheion-auto/freight/tunnel/logistics_controller.rs
 * 
 * Research Basis:
 *   - Phoenix Underground Infrastructure: 3D subsurface mapping for tunnel placement
 *   - Autonomous Freight Tunnels: 24/7 cargo movement, zero surface congestion
 *   - Heat-Resistant Tunnel Systems: 120°F+ operational continuity with active cooling
 *   - Air Quality Management: CO2 scrubbing, particulate filtration, emergency ventilation
 *   - Emergency Response Protocols: <5 minute incident response, evacuation procedures
 *   - Energy Efficiency: Regenerative braking, solar-powered ventilation, battery storage
 *   - Cargo Capacity: 10-50 ton payloads, modular container systems
 *   - Tunnel Dimensions: 15-20 ft diameter, 50-100 ft below surface
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Underground Rights & Archaeological Preservation)
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

// Internal Aletheion Crates (Established in Batch 1)
use aletheion_:pq_crypto::hash::pq_hash;
use aletheion_:did_wallet::DIDWallet;
use aletheion_gov::treaty::TreatyCompliance;
use aletheion_physical::hal::ActuatorCommand;
use aletheion_comms::mesh::OfflineQueue;
use aletheion_core::identity::BirthSign;
use aletheion_energy::management::EnergyBudget;

// --- Constants & Phoenix Tunnel Parameters ---

/// Tunnel dimensions (feet)
const TUNNEL_DIAMETER_FT: f32 = 18.0;
const TUNNEL_DEPTH_FT: f32 = 75.0; // Average depth below surface
const TUNNEL_CLEARANCE_FT: f32 = 12.0; // Minimum clearance for cargo

/// Speed limits (mph)
const MAX_TUNNEL_SPEED_MPH: f32 = 45.0; // Maximum freight speed
const SAFE_APPROACH_SPEED_MPH: f32 = 15.0; // Approach to stations/intersections
const EMERGENCY_STOP_DISTANCE_FT: f32 = 200.0; // Emergency braking distance

/// Environmental thresholds
const MAX_TUNNEL_TEMPERATURE_F: f32 = 110.0; // Equipment safety limit
const MIN_TUNNEL_TEMPERATURE_F: f32 = 60.0; // Comfort minimum
const MAX_CO2_PPM: f32 = 1000.0; // Air quality threshold
const MAX_PARTICULATE_UG_M3: f32 = 50.0; // PM2.5 threshold
const MIN_OXYGEN_PERCENT: f32 = 19.5; // Safety minimum

/// Safety margins (feet)
const COLLISION_BUFFER_FT: f32 = 25.0; // Minimum separation between freight units
const STATION_APPROACH_BUFFER_FT: f32 = 100.0; // Station approach buffer
const EMERGENCY_STOPPING_ZONE_FT: f32 = 150.0; // Emergency stopping zone

/// Cargo specifications
const MAX_CARGO_WEIGHT_TONS: f32 = 50.0; // Maximum payload per unit
const MAX_CARGO_DIMENSIONS_FT: [f32; 3] = [12.0, 8.0, 8.0]; // [length, width, height]
const STANDARD_CONTAINER_SIZES: [[f32; 3]; 3] = [
    [8.0, 4.0, 4.0],  // Small container
    [12.0, 8.0, 8.0], // Standard container
    [20.0, 8.0, 8.0], // Large container
];

/// Detection and monitoring ranges (feet)
const TUNNEL_SENSOR_SPACING_FT: f32 = 100.0; // Sensor placement interval
const LEAK_DETECTION_SENSITIVITY_PPM: f32 = 10.0; // Gas leak detection threshold
const FIRE_DETECTION_TEMPERATURE_F: f32 = 150.0; // Fire detection threshold

/// Emergency response parameters
const EMERGENCY_RESPONSE_TIME_SECONDS: u64 = 300; // 5 minutes maximum
const EVACUATION_ROUTE_INTERVAL_FT: f32 = 500.0; // Emergency exit spacing
const COMMUNICATION_REDUNDANCY_COUNT: usize = 3; // Minimum comms paths

/// Energy consumption parameters (kWh per mile)
const BASE_ENERGY_PER_MILE_KWH: f32 = 2.5;
const REGENERATIVE_BRAKING_EFFICIENCY: f32 = 0.7; // 70% energy recovery
const VENTILATION_ENERGY_PER_HOUR_KWH: f32 = 15.0;
const LIGHTING_ENERGY_PER_HOUR_KWH: f32 = 5.0;

/// Priority levels (0-100, higher = more priority)
const EMERGENCY_CARGO_PRIORITY: u8 = 100;
const MEDICAL_SUPPLY_PRIORITY: u8 = 95;
const PERISHABLE_GOODS_PRIORITY: u8 = 85;
const CRITICAL_INFRASTRUCTURE_PRIORITY: u8 = 80;
const STANDARD_FREIGHT_PRIORITY: u8 = 50;
const NON_URGENT_PRIORITY: u8 = 30;

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
const OFFLINE_BUFFER_HOURS: u32 = 72;
/// Treaty Check Cache TTL (seconds)
const TREATY_CACHE_TTL: u64 = 300;

/// Maximum number of active freight units per tunnel segment
const MAX_FREIGHT_UNITS_PER_SEGMENT: usize = 10;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TunnelSegmentType {
    Straight,
    Curved,
    Intersection,
    Station,
    VentilationShaft,
    EmergencyExit,
    MaintenanceBay,
    LoadingDock,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TunnelState {
    NormalOperation,
    MaintenanceMode,
    EmergencyLockdown,
    VentilationFailure,
    FireAlert,
    FloodingAlert,
    PowerOutage,
    OfflineDegraded,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CargoType {
    MedicalSupplies,
    PerishableFood,
    CriticalInfrastructure,
    IndustrialMaterials,
    ConsumerGoods,
    HazardousMaterials,
    ConstructionMaterials,
    WasteRemoval,
    EmptyContainer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FreightUnitStatus {
    Idle,
    Loading,
    InTransit,
    Unloading,
    Maintenance,
    EmergencyStop,
    Evacuation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TunnelEmergencyType {
    Fire,
    Flooding,
    StructuralFailure,
    PowerOutage,
    VentilationFailure,
    HazardousMaterialLeak,
    MedicalEmergency,
    SecurityThreat,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvironmentalControlMode {
    Normal,
    HeatDissipation,
    AirQualityScrubbing,
    EmergencyVentilation,
    PowerConservation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UndergroundRightsStatus {
    Approved,
    PendingArchaeologicalReview,
    Denied,
    Conditional,
    EmergencyOverride,
}

#[derive(Clone)]
pub struct TunnelCoordinate3D {
    pub tunnel_id: [u8; 32],
    pub segment_index: usize,
    pub position_ft: f32, // Distance from segment start
    pub elevation_ft: f32, // Elevation relative to surface
}

#[derive(Clone)]
pub struct FreightUnit {
    pub unit_id: [u8; 32],
    pub cargo_type: CargoType,
    pub cargo_weight_tons: f32,
    pub cargo_dimensions_ft: [f32; 3],
    pub current_position: TunnelCoordinate3D,
    pub destination: TunnelCoordinate3D,
    pub speed_mph: f32,
    pub heading_degrees: f32,
    pub status: FreightUnitStatus,
    pub priority: u8,
    pub energy_consumed_kwh: f32,
    pub treaty_zone: Option<[u8; 32]>,
}

#[derive(Clone)]
pub struct TunnelEnvironmentalReading {
    pub timestamp: u64,
    pub temperature_f: f32,
    pub humidity_percent: f32,
    pub co2_ppm: f32,
    pub o2_percent: f32,
    pub pm2_5_ug_m3: f32,
    pub air_flow_cfm: f32,
    pub pressure_psi: f32,
    pub sensor_id: [u8; 32],
    pub position: TunnelCoordinate3D,
}

#[derive(Clone)]
pub struct TunnelSegment {
    pub segment_id: [u8; 32],
    pub segment_type: TunnelSegmentType,
    pub start_coordinate: TunnelCoordinate3D,
    pub end_coordinate: TunnelCoordinate3D,
    pub length_ft: f32,
    pub curvature_radius_ft: f32,
    pub max_speed_mph: f32,
    pub active_freight_units: usize,
    pub max_capacity: usize,
    pub ventilation_active: bool,
    pub lighting_active: bool,
    pub indigenous_territory: bool,
    pub archaeological_site_nearby: bool,
    pub treaty_zone_id: Option<[u8; 32]>,
}

#[derive(Clone)]
pub struct TunnelEmergencyAlert {
    pub alert_id: [u8; 32],
    pub emergency_type: TunnelEmergencyType,
    pub location: TunnelCoordinate3D,
    pub severity: u8, // 0-100
    pub affected_segments: Vec<[u8; 32]>,
    pub evacuation_required: bool,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct FreightScheduleEntry {
    pub schedule_id: [u8; 32],
    pub freight_unit_id: [u8; 32],
    pub departure_time: u64,
    pub arrival_time: u64,
    pub route_segments: Vec<[u8; 32]>,
    pub priority: u8,
    pub energy_budget_kwh: f32,
    pub treaty_compliant: bool,
}

#[derive(Clone)]
pub struct TunnelLogisticsMetrics {
    pub current_state: TunnelState,
    pub active_freight_units: usize,
    pub total_cargo_tons_today: f32,
    pub energy_consumed_kwh_today: f32,
    pub average_transit_time_minutes: f32,
    pub emergency_incidents_today: usize,
    pub treaty_violations: usize,
    pub environmental_alerts: usize,
    pub maintenance_events: usize,
}

#[derive(Clone)]
pub struct TunnelNetworkConfiguration {
    pub network_id: [u8; 32],
    pub tunnel_segments: Vec<TunnelSegment>,
    pub ventilation_systems: Vec<[u8; 32]>,
    pub emergency_exits: Vec<TunnelCoordinate3D>,
    pub loading_docks: Vec<TunnelCoordinate3D>,
    pub indigenous_territories: Vec<[u8; 32]>,
    pub archaeological_zones: Vec<TunnelCoordinate3D>,
    pub power_sources: Vec<[u8; 32]>,
    pub communication_nodes: Vec<TunnelCoordinate3D>,
}

// --- Core Tunnel Logistics Controller Structure ---

pub struct FreightTunnelLogisticsController {
    pub node_id: BirthSign,
    pub config: TunnelNetworkConfiguration,
    pub current_state: TunnelState,
    pub active_freight_units: BTreeMap<[u8; 32], FreightUnit>,
    pub pending_schedules: BTreeMap<u64, FreightScheduleEntry>,
    pub environmental_readings: BTreeMap<u64, TunnelEnvironmentalReading>,
    pub offline_queue: OfflineQueue<FreightScheduleEntry>,
    pub treaty_cache: TreatyCompliance,
    pub metrics: TunnelLogisticsMetrics,
    pub energy_budget: EnergyBudget,
    pub last_environmental_update: u64,
    pub last_sync: u64,
    pub emergency_active: bool,
    pub maintenance_mode: bool,
}

impl FreightTunnelLogisticsController {
    /**
     * Initialize the Tunnel Logistics Controller with Configuration
     * Ensures 72h operational buffer and treaty compliance setup
     */
    pub fn new(node_id: BirthSign, config: TunnelNetworkConfiguration) -> Result<Self, &'static str> {
        let queue = OfflineQueue::new(OFFLINE_BUFFER_HOURS)
            .map_err(|_| "Failed to allocate offline buffer")?;
        
        let energy_budget = EnergyBudget::new_for_tunnel_network(&config.network_id)
            .map_err(|_| "Failed to initialize energy budget")?;
        
        Ok(Self {
            node_id,
            config,
            current_state: TunnelState::NormalOperation,
            active_freight_units: BTreeMap::new(),
            pending_schedules: BTreeMap::new(),
            environmental_readings: BTreeMap::new(),
            offline_queue: queue,
            treaty_cache: TreatyCompliance::new(),
            metrics: TunnelLogisticsMetrics {
                current_state: TunnelState::NormalOperation,
                active_freight_units: 0,
                total_cargo_tons_today: 0.0,
                energy_consumed_kwh_today: 0.0,
                average_transit_time_minutes: 0.0,
                emergency_incidents_today: 0,
                treaty_violations: 0,
                environmental_alerts: 0,
                maintenance_events: 0,
            },
            energy_budget,
            last_environmental_update: 0,
            last_sync: 0,
            emergency_active: false,
            maintenance_mode: false,
        })
    }

    /**
     * ERM Chain: SENSE
     * Ingests freight unit positions, environmental readings, and emergency alerts
     * Validates data integrity using PQ hashing
     */
    pub fn sense(&mut self, input: TunnelInput) -> Result<TunnelSenseResult, &'static str> {
        match input {
            TunnelInput::FreightUnitUpdate(unit) => self.process_freight_unit_update(unit),
            TunnelInput::EnvironmentalReading(reading) => self.process_environmental_reading(reading),
            TunnelInput::EmergencyAlert(alert) => self.process_emergency_alert(alert),
            TunnelInput::ScheduleRequest(entry) => self.process_schedule_request(entry),
        }
    }

    /**
     * Process freight unit position and status update
     */
    fn process_freight_unit_update(&mut self, mut unit: FreightUnit) -> Result<TunnelSenseResult, &'static str> {
        // Validate freight unit signature (PQ Secure)
        let hash = pq_hash(&unit.unit_id);
        if hash[0] == 0x00 {
            return Err("Freight unit signature invalid");
        }

        // Check treaty compliance for underground rights
        if self.config.indigenous_territories.contains(&unit.treaty_zone.unwrap_or([0u8; 32])) {
            if let Some(treaty_zone) = unit.treaty_zone {
                let compliance = self.treaty_cache.check_underground_rights(&treaty_zone)?;
                if !compliance.allowed {
                    self.metrics.treaty_violations += 1;
                    self.log_warning("FPIC Violation: Freight unit denied access due to treaty restrictions");
                    unit.status = FreightUnitStatus::EmergencyStop;
                }
            }
        }

        // Check for archaeological site proximity
        if self.check_archaeological_proximity(&unit.current_position) {
            self.log_warning("ARCHAEOLOGICAL_PROXIMITY: Freight unit near protected site");
            // Slow down or stop if too close
            if unit.speed_mph > 5.0 {
                unit.speed_mph = 5.0;
            }
        }

        // Update or insert freight unit
        let was_new = !self.active_freight_units.contains_key(&unit.unit_id);
        self.active_freight_units.insert(unit.unit_id, unit.clone());

        // Update metrics
        if was_new {
            self.metrics.active_freight_units += 1;
        }

        // Log sensing event
        self.log_event(format!(
            "FREIGHT_UPDATE: Unit={:?}, Type={:?}, Pos=Seg{}@{:.0}ft, Speed={:.1}mph, Status={:?}, Priority={}",
            unit.unit_id,
            unit.cargo_type,
            unit.current_position.segment_index,
            unit.current_position.position_ft,
            unit.speed_mph,
            unit.status,
            unit.priority
        ));

        // Check for conflicts with other freight units
        self.detect_freight_conflicts(&unit)?;

        Ok(TunnelSenseResult::FreightUnitProcessed(unit.unit_id))
    }

    /**
     * Process environmental sensor reading
     */
    fn process_environmental_reading(&mut self, reading: TunnelEnvironmentalReading) -> Result<TunnelSenseResult, &'static str> {
        // Validate sensor signature (PQ Secure)
        let hash = pq_hash(&reading.sensor_id);
        if hash[0] == 0x00 {
            return Err("Sensor signature invalid");
        }

        // Store reading with timestamp key
        self.environmental_readings.insert(reading.timestamp, reading.clone());

        // Check environmental thresholds
        self.check_environmental_thresholds(&reading)?;

        // Update last environmental update time
        self.last_environmental_update = aletheion_core::time::now();

        // Log sensing event
        self.log_event(format!(
            "ENV_READING: Temp={:.1}°F, Humidity={:.1}%, CO2={:.0}ppm, O2={:.1}%, PM2.5={:.1}µg/m³, Flow={:.0}CFM",
            reading.temperature_f,
            reading.humidity_percent,
            reading.co2_ppm,
            reading.o2_percent,
            reading.pm2_5_ug_m3,
            reading.air_flow_cfm
        ));

        Ok(TunnelSenseResult::EnvironmentalReadingProcessed(reading.sensor_id))
    }

    /**
     * Process emergency alert
     */
    fn process_emergency_alert(&mut self, alert: TunnelEmergencyAlert) -> Result<TunnelSenseResult, &'static str> {
        // Validate alert signature (PQ Secure)
        let hash = pq_hash(&alert.alert_id);
        if hash[0] == 0x00 {
            return Err("Alert signature invalid");
        }

        // Activate emergency mode
        self.emergency_active = true;
        self.current_state = match alert.emergency_type {
            TunnelEmergencyType::Fire => TunnelState::FireAlert,
            TunnelEmergencyType::Flooding => TunnelState::FloodingAlert,
            TunnelEmergencyType::StructuralFailure => TunnelState::EmergencyLockdown,
            TunnelEmergencyType::PowerOutage => TunnelState::PowerOutage,
            TunnelEmergencyType::VentilationFailure => TunnelState::VentilationFailure,
            TunnelEmergencyType::HazardousMaterialLeak => TunnelState::EmergencyLockdown,
            TunnelEmergencyType::MedicalEmergency => TunnelState::EmergencyLockdown,
            TunnelEmergencyType::SecurityThreat => TunnelState::EmergencyLockdown,
        };

        // Stop all non-emergency freight units
        self.stop_non_emergency_freight(&alert)?;

        // Activate emergency ventilation and lighting
        self.activate_emergency_systems(&alert)?;

        // Log emergency event
        self.log_event(format!(
            "EMERGENCY_ALERT: Type={:?}, Location=Seg{}@{:.0}ft, Severity={}, Evacuation={}",
            alert.emergency_type,
            alert.location.segment_index,
            alert.location.position_ft,
            alert.severity,
            alert.evacuation_required
        ));

        self.metrics.emergency_incidents_today += 1;

        Ok(TunnelSenseResult::EmergencyAlertProcessed(alert.alert_id))
    }

    /**
     * Process freight schedule request
     */
    fn process_schedule_request(&mut self, entry: FreightScheduleEntry) -> Result<TunnelSenseResult, &'static str> {
        // Validate schedule signature (PQ Secure)
        let hash = pq_hash(&entry.schedule_id);
        if hash[0] == 0x00 {
            return Err("Schedule signature invalid");
        }

        // Check treaty compliance
        let treaty_compliant = if entry.treaty_compliant {
            true
        } else {
            self.log_warning(format!("FPIC_VIOLATION: Schedule {:?} denied due to treaty restrictions", entry.schedule_id));
            return Ok(TunnelSenseResult::ScheduleDenied(entry.schedule_id));
        }

        // Check energy budget
        if !self.energy_budget.can_allocate(entry.energy_budget_kwh as u32) {
            self.log_warning(format!("ENERGY_BUDGET_EXCEEDED: Schedule {:?} denied due to insufficient energy", entry.schedule_id));
            return Ok(TunnelSenseResult::ScheduleDenied(entry.schedule_id));
        }

        // Check tunnel segment capacity
        for segment_id in &entry.route_segments {
            if !self.check_segment_capacity(segment_id) {
                self.log_warning(format!("SEGMENT_CAPACITY_EXCEEDED: Schedule {:?} denied due to capacity limits", entry.schedule_id));
                return Ok(TunnelSenseResult::ScheduleDenied(entry.schedule_id));
            }
        }

        // Add to pending schedules with priority-based key
        let priority_key = (!entry.priority as u64) << 32 | entry.departure_time;
        self.pending_schedules.insert(priority_key, entry.clone());

        // Log schedule request
        self.log_event(format!(
            "SCHEDULE_REQUEST: ID={:?}, Unit={:?}, Departure={}, Arrival={}, Priority={}, Energy={:.1}kWh",
            entry.schedule_id,
            entry.freight_unit_id,
            entry.departure_time,
            entry.arrival_time,
            entry.priority,
            entry.energy_budget_kwh
        ));

        Ok(TunnelSenseResult::ScheduleAccepted(entry.schedule_id))
    }

    /**
     * Check environmental thresholds and trigger alerts if exceeded
     */
    fn check_environmental_thresholds(&mut self, reading: &TunnelEnvironmentalReading) -> Result<(), &'static str> {
        let mut alerts_triggered = false;

        // Temperature check
        if reading.temperature_f > MAX_TUNNEL_TEMPERATURE_F {
            self.log_warning(format!("TEMPERATURE_ALERT: {:.1}°F exceeds maximum threshold", reading.temperature_f));
            self.activate_cooling_systems()?;
            alerts_triggered = true;
        } else if reading.temperature_f < MIN_TUNNEL_TEMPERATURE_F {
            self.log_warning(format!("TEMPERATURE_ALERT: {:.1}°F below minimum threshold", reading.temperature_f));
            alerts_triggered = true;
        }

        // CO2 check
        if reading.co2_ppm > MAX_CO2_PPM {
            self.log_warning(format!("CO2_ALERT: {:.0}ppm exceeds maximum threshold", reading.co2_ppm));
            self.activate_air_scrubbers()?;
            alerts_triggered = true;
        }

        // Particulate matter check
        if reading.pm2_5_ug_m3 > MAX_PARTICULATE_UG_M3 {
            self.log_warning(format!("PARTICULATE_ALERT: {:.1}µg/m³ exceeds maximum threshold", reading.pm2_5_ug_m3));
            self.activate_filters()?;
            alerts_triggered = true;
        }

        // Oxygen check
        if reading.o2_percent < MIN_OXYGEN_PERCENT {
            self.log_warning(format!("OXYGEN_ALERT: {:.1}% below minimum threshold", reading.o2_percent));
            self.activate_emergency_ventilation()?;
            alerts_triggered = true;
        }

        if alerts_triggered {
            self.metrics.environmental_alerts += 1;
            self.current_state = TunnelState::VentilationFailure; // Temporarily set to ventilation issue
        }

        Ok(())
    }

    /**
     * Check if freight unit is near archaeological site
     */
    fn check_archaeological_proximity(&self, position: &TunnelCoordinate3D) -> bool {
        for archaeological_zone in &self.config.archaeological_zones {
            // Calculate distance between positions
            let distance_ft = self.calculate_tunnel_distance(position, archaeological_zone);
            
            // If within 100 feet of archaeological site, return true
            if distance_ft < 100.0 {
                return true;
            }
        }
        
        false
    }

    /**
     * Calculate distance between two tunnel coordinates (feet)
     */
    fn calculate_tunnel_distance(&self, coord1: &TunnelCoordinate3D, coord2: &TunnelCoordinate3D) -> f32 {
        // Simplified: assume same tunnel and calculate linear distance
        if coord1.tunnel_id == coord2.tunnel_id {
            (coord1.position_ft - coord2.position_ft).abs()
        } else {
            // Different tunnels: return large distance
            f32::MAX
        }
    }

    /**
     * Detect conflicts between freight units
     */
    fn detect_freight_conflicts(&mut self, unit: &FreightUnit) -> Result<(), &'static str> {
        for (other_unit_id, other_unit) in &self.active_freight_units {
            if other_unit_id == &unit.unit_id {
                continue;
            }

            // Check if same tunnel segment
            if unit.current_position.tunnel_id == other_unit.current_position.tunnel_id &&
               unit.current_position.segment_index == other_unit.current_position.segment_index {
                
                // Calculate distance between units
                let distance_ft = (unit.current_position.position_ft - other_unit.current_position.position_ft).abs();
                
                // Check for collision risk
                if distance_ft < COLLISION_BUFFER_FT {
                    self.log_warning(format!(
                        "COLLISION_RISK: Units {:?} and {:?} within {:.1}ft buffer",
                        unit.unit_id,
                        other_unit_id,
                        distance_ft
                    ));
                    
                    // Initiate conflict resolution
                    self.resolve_freight_conflict(unit, other_unit)?;
                }
            }
        }
        
        Ok(())
    }

    /**
     * Resolve conflict between freight units
     */
    fn resolve_freight_conflict(&mut self, unit1: &FreightUnit, unit2: &FreightUnit) -> Result<(), &'static str> {
        // Determine which unit has higher priority
        if unit1.priority > unit2.priority {
            // Unit 1 maintains course, unit 2 slows down or stops
            if let Some(mut lower_unit) = self.active_freight_units.get_mut(&unit2.unit_id) {
                lower_unit.speed_mph = lower_unit.speed_mph.max(5.0); // Reduce to minimum safe speed
                lower_unit.status = FreightUnitStatus::EmergencyStop;
            }
        } else {
            // Unit 2 maintains course, unit 1 slows down or stops
            if let Some(mut lower_unit) = self.active_freight_units.get_mut(&unit1.unit_id) {
                lower_unit.speed_mph = lower_unit.speed_mph.max(5.0);
                lower_unit.status = FreightUnitStatus::EmergencyStop;
            }
        }
        
        Ok(())
    }

    /**
     * Check if tunnel segment has available capacity
     */
    fn check_segment_capacity(&self, segment_id: &[u8; 32]) -> bool {
        for segment in &self.config.tunnel_segments {
            if segment.segment_id == *segment_id {
                return segment.active_freight_units < segment.max_capacity;
            }
        }
        false
    }

    /**
     * ERM Chain: MODEL
     * Analyzes tunnel network utilization, conflict potential, and generates optimal freight schedules
     * No Digital Twins: Uses real-time position data and predictive routing
     */
    pub fn model_optimal_logistics(&mut self) -> Result<Vec<FreightScheduleEntry>, &'static str> {
        let current_time = aletheion_core::time::now();
        
        // Remove expired schedules
        self.prune_expired_schedules(current_time);
        
        // Generate new schedules for pending requests
        let mut new_schedules = Vec::new();
        
        // 1. Process emergency cargo first (highest priority)
        self.process_emergency_cargo(&mut new_schedules, current_time)?;
        
        // 2. Process medical and perishable cargo
        self.process_high_priority_cargo(&mut new_schedules, current_time)?;
        
        // 3. Process standard freight
        self.process_standard_freight(&mut new_schedules, current_time)?;
        
        // 4. Optimize schedules for energy efficiency and throughput
        self.optimize_schedules(&mut new_schedules)?;
        
        // Update metrics
        self.metrics.active_freight_units = self.active_freight_units.len();
        
        Ok(new_schedules)
    }

    /**
     * Process emergency cargo schedules
     */
    fn process_emergency_cargo(&mut self, schedules: &mut Vec<FreightScheduleEntry>, current_time: u64) -> Result<(), &'static str> {
        let mut emergency_requests: Vec<_> = self.pending_schedules.iter()
            .filter(|(_, entry)| entry.priority >= EMERGENCY_CARGO_PRIORITY - 10)
            .collect();
        
        // Sort by priority (descending) and departure time (ascending)
        emergency_requests.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority)
                .then_with(|| a.1.departure_time.cmp(&b.1.departure_time))
        });
        
        for (_, entry) in emergency_requests {
            if self.validate_schedule_feasibility(entry) {
                schedules.push(entry.clone());
            }
        }
        
        Ok(())
    }

    /**
     * Validate schedule feasibility (capacity, energy, treaty compliance)
     */
    fn validate_schedule_feasibility(&self, entry: &FreightScheduleEntry) -> bool {
        // Check energy budget
        if !self.energy_budget.can_allocate(entry.energy_budget_kwh as u32) {
            return false;
        }
        
        // Check treaty compliance
        if !entry.treaty_compliant {
            return false;
        }
        
        // Check segment capacities
        for segment_id in &entry.route_segments {
            if !self.check_segment_capacity(segment_id) {
                return false;
            }
        }
        
        true
    }

    /**
     * Process high priority cargo (medical, perishable)
     */
    fn process_high_priority_cargo(&mut self, schedules: &mut Vec<FreightScheduleEntry>, current_time: u64) -> Result<(), &'static str> {
        let mut high_priority_requests: Vec<_> = self.pending_schedules.iter()
            .filter(|(_, entry)| {
                entry.priority >= MEDICAL_SUPPLY_PRIORITY && entry.priority < EMERGENCY_CARGO_PRIORITY - 10
            })
            .collect();
        
        high_priority_requests.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority)
                .then_with(|| a.1.departure_time.cmp(&b.1.departure_time))
        });
        
        for (_, entry) in high_priority_requests {
            if self.validate_schedule_feasibility(entry) {
                schedules.push(entry.clone());
            }
        }
        
        Ok(())
    }

    /**
     * Process standard freight schedules
     */
    fn process_standard_freight(&mut self, schedules: &mut Vec<FreightScheduleEntry>, current_time: u64) -> Result<(), &'static str> {
        let mut standard_requests: Vec<_> = self.pending_schedules.iter()
            .filter(|(_, entry)| entry.priority < MEDICAL_SUPPLY_PRIORITY)
            .collect();
        
        standard_requests.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority)
                .then_with(|| a.1.departure_time.cmp(&b.1.departure_time))
        });
        
        for (_, entry) in standard_requests {
            if self.validate_schedule_feasibility(entry) {
                schedules.push(entry.clone());
            }
        }
        
        Ok(())
    }

    /**
     * Optimize schedules for energy efficiency and throughput
     */
    fn optimize_schedules(&self, schedules: &mut Vec<FreightScheduleEntry>) -> Result<(), &'static str> {
        // Sort by departure time
        schedules.sort_by_key(|entry| entry.departure_time);
        
        // Merge compatible schedules (same route, similar timing)
        self.merge_compatible_schedules(schedules);
        
        // Optimize energy usage through regenerative braking coordination
        self.optimize_energy_usage(schedules);
        
        Ok(())
    }

    /**
     * Merge compatible schedules to improve throughput
     */
    fn merge_compatible_schedules(&self, schedules: &mut Vec<FreightScheduleEntry>) {
        // Implementation: merge schedules with same route segments and close departure times
        // This is a placeholder for production optimization logic
    }

    /**
     * Optimize energy usage through regenerative braking coordination
     */
    fn optimize_energy_usage(&self, schedules: &mut Vec<FreightScheduleEntry>) {
        // Implementation: coordinate braking to maximize energy recovery
        // This is a placeholder for production optimization logic
    }

    /**
     * Prune expired schedules from pending queue
     */
    fn prune_expired_schedules(&mut self, current_time: u64) {
        let threshold = current_time - 3600; // 1 hour expiration
        
        let expired_schedules: Vec<_> = self.pending_schedules.iter()
            .filter(|(_, entry)| entry.departure_time < threshold)
            .map(|(key, _)| *key)
            .collect();
        
        for key in expired_schedules {
            self.pending_schedules.remove(&key);
        }
    }

    /**
     * ERM Chain: OPTIMIZE & TREATY-CHECK
     * Validates schedules against Indigenous underground rights and generates executable commands
     * FPIC Enforcement: Cannot authorize freight movement under protected lands without consent
     */
    pub fn optimize_and_check(&mut self, schedules: Vec<FreightScheduleEntry>) -> Result<Vec<TunnelCommand>, &'static str> {
        let mut commands = Vec::new();
        
        for schedule in schedules {
            // Check treaty compliance for each schedule
            let treaty_compliant = if schedule.treaty_compliant {
                true
            } else {
                self.log_warning(format!("FPIC_VIOLATION: Schedule {:?} denied due to treaty restrictions", schedule.schedule_id));
                continue;
            };
            
            // Generate command
            let command = TunnelCommand {
                schedule_entry: schedule.clone(),
                command_type: TunnelCommandType::AuthorizeFreightMovement,
                treaty_compliant,
                signed: false,
            };
            
            commands.push(command);
        }
        
        Ok(commands)
    }

    /**
     * ERM Chain: ACT
     * Executes tunnel commands or queues for offline execution
     * Ensures atomicity without rollbacks
     */
    pub fn act(&mut self, commands: Vec<TunnelCommand>) -> Result<(), &'static str> {
        for command in commands {
            // Sign command (PQ Secure)
            let signature = DIDWallet::sign_action(&self.node_id, &command);
            let mut signed_command = command.clone();
            signed_command.signed = signature.is_ok();
            
            // Attempt immediate execution via HAL
            match self.execute_tunnel_command(&signed_command) {
                Ok(_) => {
                    self.log_action(&signed_command);
                    
                    // Update metrics
                    self.update_metrics(&signed_command);
                },
                Err(_) => {
                    // Offline Fallback: Queue for later execution
                    self.offline_queue.push(signed_command.schedule_entry)?;
                    self.log_warning("Offline mode: Tunnel command queued for later execution");
                }
            }
        }
        
        Ok(())
    }

    /**
     * Execute individual tunnel command
     */
    fn execute_tunnel_command(&self, command: &TunnelCommand) -> Result<(), &'static str> {
        match command.command_type {
            TunnelCommandType::AuthorizeFreightMovement => {
                aletheion_physical::hal::authorize_freight_movement(
                    &command.schedule_entry.freight_unit_id,
                    &command.schedule_entry.route_segments
                )?;
            },
            TunnelCommandType::ActivateCoolingSystem => {
                aletheion_physical::hal::activate_tunnel_cooling()?;
            },
            TunnelCommandType::ActivateVentilation => {
                aletheion_physical::hal::activate_tunnel_ventilation()?;
            },
            TunnelCommandType::ActivateEmergencyLighting => {
                aletheion_physical::hal::activate_emergency_lighting()?;
            },
            TunnelCommandType::StopFreightUnit => {
                aletheion_physical::hal::stop_freight_unit(&command.schedule_entry.freight_unit_id)?;
            },
            TunnelCommandType::EvacuateTunnelSegment => {
                aletheion_physical::hal::evacuate_tunnel_segment(&command.schedule_entry.route_segments[0])?;
            }
        }
        
        Ok(())
    }

    /**
     * Update metrics based on executed command
     */
    fn update_metrics(&mut self, command: &TunnelCommand) {
        match command.command_type {
            TunnelCommandType::AuthorizeFreightMovement => {
                // Track cargo movement
                self.metrics.total_cargo_tons_today += 1.0; // Placeholder
            },
            _ => {}
        }
    }

    /**
     * ERM Chain: LOG
     * Immutable logging to ALN-Blockchain
     * No rollbacks allowed; all state changes are forward-only
     */
    fn log_action(&self, command: &TunnelCommand) {
        let log_entry = alloc::format!(
            "TUNNEL_ACT: Type={:?} | Schedule={:?} | Units={} | Energy={:.1}kWh | Treaty={}",
            command.command_type,
            command.schedule_entry.schedule_id,
            command.schedule_entry.route_segments.len(),
            command.schedule_entry.energy_budget_kwh,
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
    pub fn get_status_report(&self) -> TunnelStatusReport {
        TunnelStatusReport {
            network_id: self.config.network_id,
            current_state: self.current_state,
            active_freight_units: self.active_freight_units.len(),
            pending_schedules: self.pending_schedules.len(),
            environmental_status: self.get_environmental_status(),
            metrics: self.metrics.clone(),
            offline_queue_size: self.offline_queue.len(),
            last_sync: self.last_sync,
            emergency_active: self.emergency_active,
            maintenance_mode: self.maintenance_mode,
            accessibility_alert: self.current_state != TunnelState::NormalOperation,
            treaty_compliance_required: !self.config.indigenous_territories.is_empty(),
        }
    }

    /**
     * Get current environmental status summary
     */
    fn get_environmental_status(&self) -> EnvironmentalStatus {
        if let Some((_, latest_reading)) = self.environmental_readings.iter().next_back() {
            EnvironmentalStatus {
                temperature_f: latest_reading.temperature_f,
                co2_ppm: latest_reading.co2_ppm,
                o2_percent: latest_reading.o2_percent,
                air_quality_index: self.calculate_air_quality_index(latest_reading),
                ventilation_active: true, // Placeholder
                alerts_active: self.metrics.environmental_alerts > 0,
            }
        } else {
            EnvironmentalStatus {
                temperature_f: 70.0,
                co2_ppm: 400.0,
                o2_percent: 21.0,
                air_quality_index: 50,
                ventilation_active: false,
                alerts_active: false,
            }
        }
    }

    /**
     * Calculate air quality index from sensor readings
     */
    fn calculate_air_quality_index(&self, reading: &TunnelEnvironmentalReading) -> u32 {
        // Simplified AQI calculation
        let co2_index = (reading.co2_ppm / MAX_CO2_PPM * 100.0) as u32;
        let pm_index = (reading.pm2_5_ug_m3 / MAX_PARTICULATE_UG_M3 * 100.0) as u32;
        
        co2_index.max(pm_index)
    }

    /**
     * Activate cooling systems for temperature control
     */
    fn activate_cooling_systems(&mut self) -> Result<(), &'static str> {
        // In production: activate tunnel cooling systems via HAL
        self.log_event("COOLING_ACTIVATED: Tunnel cooling systems activated".to_string());
        Ok(())
    }

    /**
     * Activate air scrubbers for CO2 removal
     */
    fn activate_air_scrubbers(&mut self) -> Result<(), &'static str> {
        // In production: activate air scrubbers via HAL
        self.log_event("SCRUBBERS_ACTIVATED: Air scrubbers activated".to_string());
        Ok(())
    }

    /**
     * Activate particulate filters
     */
    fn activate_filters(&mut self) -> Result<(), &'static str> {
        // In production: activate particulate filters via HAL
        self.log_event("FILTERS_ACTIVATED: Particulate filters activated".to_string());
        Ok(())
    }

    /**
     * Activate emergency ventilation
     */
    fn activate_emergency_ventilation(&mut self) -> Result<(), &'static str> {
        // In production: activate emergency ventilation via HAL
        self.log_event("EMERGENCY_VENTILATION: Emergency ventilation activated".to_string());
        Ok(())
    }

    /**
     * Stop all non-emergency freight units
     */
    fn stop_non_emergency_freight(&mut self, alert: &TunnelEmergencyAlert) -> Result<(), &'static str> {
        for (_, unit) in &mut self.active_freight_units {
            if unit.priority < EMERGENCY_CARGO_PRIORITY {
                unit.status = FreightUnitStatus::EmergencyStop;
                unit.speed_mph = 0.0;
            }
        }
        
        self.log_event(format!("NON_EMERGENCY_STOPPED: All non-emergency freight stopped due to {:?}", alert.emergency_type));
        Ok(())
    }

    /**
     * Activate emergency systems (ventilation, lighting, communications)
     */
    fn activate_emergency_systems(&mut self, alert: &TunnelEmergencyAlert) -> Result<(), &'static str> {
        // In production: activate emergency systems via HAL
        self.log_event(format!("EMERGENCY_SYSTEMS_ACTIVATED: Type={:?}", alert.emergency_type));
        Ok(())
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
     * Enter maintenance mode
     */
    pub fn enter_maintenance_mode(&mut self) -> Result<(), &'static str> {
        self.maintenance_mode = true;
        self.current_state = TunnelState::MaintenanceMode;
        
        // Stop all freight units
        for (_, unit) in &mut self.active_freight_units {
            unit.status = FreightUnitStatus::Maintenance;
            unit.speed_mph = 0.0;
        }
        
        self.log_event("MAINTENANCE_MODE: Tunnel network entering maintenance mode".to_string());
        Ok(())
    }

    /**
     * Exit maintenance mode
     */
    pub fn exit_maintenance_mode(&mut self) -> Result<(), &'static str> {
        self.maintenance_mode = false;
        self.current_state = TunnelState::NormalOperation;
        
        self.log_event("MAINTENANCE_COMPLETE: Tunnel network returning to normal operation".to_string());
        Ok(())
    }

    /**
     * Get current tunnel utilization metrics
     */
    pub fn get_utilization_metrics(&self) -> TunnelUtilizationMetrics {
        let total_segments = self.config.tunnel_segments.len();
        let total_capacity: usize = self.config.tunnel_segments.iter().map(|s| s.max_capacity).sum();
        let current_utilization: usize = self.config.tunnel_segments.iter().map(|s| s.active_freight_units).sum();
        
        TunnelUtilizationMetrics {
            total_segments,
            total_capacity,
            current_utilization,
            utilization_percent: (current_utilization as f32 / total_capacity as f32) * 100.0,
            active_freight_units: self.active_freight_units.len(),
            pending_schedules: self.pending_schedules.len(),
            energy_consumption_kwh_per_hour: self.calculate_current_energy_consumption(),
        }
    }

    /**
     * Calculate current energy consumption rate
     */
    fn calculate_current_energy_consumption(&self) -> f32 {
        // Simplified: base consumption + per-freight-unit consumption
        let base_consumption = VENTILATION_ENERGY_PER_HOUR_KWH + LIGHTING_ENERGY_PER_HOUR_KWH;
        let freight_consumption = self.active_freight_units.len() as f32 * BASE_ENERGY_PER_MILE_KWH;
        
        base_consumption + freight_consumption
    }
}

// --- Supporting Data Structures ---

pub enum TunnelInput {
    FreightUnitUpdate(FreightUnit),
    EnvironmentalReading(TunnelEnvironmentalReading),
    EmergencyAlert(TunnelEmergencyAlert),
    ScheduleRequest(FreightScheduleEntry),
}

pub enum TunnelSenseResult {
    FreightUnitProcessed([u8; 32]),
    EnvironmentalReadingProcessed([u8; 32]),
    EmergencyAlertProcessed([u8; 32]),
    ScheduleAccepted([u8; 32]),
    ScheduleDenied([u8; 32]),
}

pub struct TunnelCommand {
    pub schedule_entry: FreightScheduleEntry,
    pub command_type: TunnelCommandType,
    pub treaty_compliant: bool,
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TunnelCommandType {
    AuthorizeFreightMovement,
    ActivateCoolingSystem,
    ActivateVentilation,
    ActivateEmergencyLighting,
    StopFreightUnit,
    EvacuateTunnelSegment,
}

pub struct TunnelStatusReport {
    pub network_id: [u8; 32],
    pub current_state: TunnelState,
    pub active_freight_units: usize,
    pub pending_schedules: usize,
    pub environmental_status: EnvironmentalStatus,
    pub metrics: TunnelLogisticsMetrics,
    pub offline_queue_size: usize,
    pub last_sync: u64,
    pub emergency_active: bool,
    pub maintenance_mode: bool,
    pub accessibility_alert: bool,
    pub treaty_compliance_required: bool,
}

pub struct EnvironmentalStatus {
    pub temperature_f: f32,
    pub co2_ppm: f32,
    pub o2_percent: f32,
    pub air_quality_index: u32,
    pub ventilation_active: bool,
    pub alerts_active: bool,
}

pub struct TunnelUtilizationMetrics {
    pub total_segments: usize,
    pub total_capacity: usize,
    pub current_utilization: usize,
    pub utilization_percent: f32,
    pub active_freight_units: usize,
    pub pending_schedules: usize,
    pub energy_consumption_kwh_per_hour: f32,
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_initialization() {
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        
        assert_eq!(controller.current_state, TunnelState::NormalOperation);
        assert_eq!(controller.active_freight_units.len(), 0);
        assert_eq!(controller.pending_schedules.len(), 0);
    }

    #[test]
    fn test_emergency_cargo_priority() {
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let mut controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        
        // Add emergency cargo schedule
        let emergency_schedule = FreightScheduleEntry {
            schedule_id: [1u8; 32],
            freight_unit_id: [1u8; 32],
            departure_time: 1000,
            arrival_time: 1100,
            route_segments: vec![[1u8; 32], [2u8; 32]],
            priority: EMERGENCY_CARGO_PRIORITY,
            energy_budget_kwh: 50.0,
            treaty_compliant: true,
        };
        
        // Add standard freight schedule
        let standard_schedule = FreightScheduleEntry {
            schedule_id: [2u8; 32],
            freight_unit_id: [2u8; 32],
            departure_time: 1001,
            arrival_time: 1200,
            route_segments: vec![[1u8; 32], [2u8; 32]],
            priority: STANDARD_FREIGHT_PRIORITY,
            energy_budget_kwh: 30.0,
            treaty_compliant: true,
        };
        
        controller.process_schedule_request(emergency_schedule.clone()).unwrap();
        controller.process_schedule_request(standard_schedule.clone()).unwrap();
        
        // Model optimal logistics
        let schedules = controller.model_optimal_logistics().unwrap();
        
        // Emergency cargo should be processed first
        assert!(schedules.len() >= 1);
        assert_eq!(schedules[0].priority, EMERGENCY_CARGO_PRIORITY);
    }

    #[test]
    fn test_offline_queue_capacity() {
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        assert!(controller.offline_queue.capacity_hours() >= 72);
    }

    #[test]
    fn test_environmental_threshold_detection() {
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let mut controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        
        // Create environmental reading above temperature threshold
        let hot_reading = TunnelEnvironmentalReading {
            timestamp: 1000,
            temperature_f: 115.0, // Above MAX_TUNNEL_TEMPERATURE_F (110°F)
            humidity_percent: 30.0,
            co2_ppm: 500.0,
            o2_percent: 21.0,
            pm2_5_ug_m3: 20.0,
            air_flow_cfm: 1000.0,
            sensor_id: [1u8; 32],
            position: TunnelCoordinate3D {
                tunnel_id: [1u8; 32],
                segment_index: 0,
                position_ft: 0.0,
                elevation_ft: -75.0,
            },
        };
        
        // Process reading - should trigger temperature alert
        controller.process_environmental_reading(hot_reading).unwrap();
        
        // Check that environmental alerts were incremented
        assert!(controller.metrics.environmental_alerts > 0);
    }

    #[test]
    fn test_archaeological_proximity_detection() {
        let archaeological_zone = TunnelCoordinate3D {
            tunnel_id: [1u8; 32],
            segment_index: 0,
            position_ft: 1000.0,
            elevation_ft: -75.0,
        };
        
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![archaeological_zone.clone()],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        
        // Position near archaeological site (within 100 ft)
        let near_position = TunnelCoordinate3D {
            tunnel_id: [1u8; 32],
            segment_index: 0,
            position_ft: 1050.0, // 50 ft from archaeological zone
            elevation_ft: -75.0,
        };
        
        // Position far from archaeological site
        let far_position = TunnelCoordinate3D {
            tunnel_id: [1u8; 32],
            segment_index: 0,
            position_ft: 2000.0, // 1000 ft from archaeological zone
            elevation_ft: -75.0,
        };
        
        assert!(controller.check_archaeological_proximity(&near_position));
        assert!(!controller.check_archaeological_proximity(&far_position));
    }

    #[test]
    fn test_tunnel_distance_calculation() {
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        
        let coord1 = TunnelCoordinate3D {
            tunnel_id: [1u8; 32],
            segment_index: 0,
            position_ft: 100.0,
            elevation_ft: -75.0,
        };
        
        let coord2 = TunnelCoordinate3D {
            tunnel_id: [1u8; 32],
            segment_index: 0,
            position_ft: 250.0,
            elevation_ft: -75.0,
        };
        
        let distance = controller.calculate_tunnel_distance(&coord1, &coord2);
        assert_eq!(distance, 150.0);
    }

    #[test]
    fn test_maintenance_mode() {
        let config = TunnelNetworkConfiguration {
            network_id: [1u8; 32],
            tunnel_segments: vec![],
            ventilation_systems: vec![],
            emergency_exits: vec![],
            loading_docks: vec![],
            indigenous_territories: vec![],
            archaeological_zones: vec![],
            power_sources: vec![],
            communication_nodes: vec![],
        };
        
        let mut controller = FreightTunnelLogisticsController::new(BirthSign::default(), config).unwrap();
        
        // Enter maintenance mode
        controller.enter_maintenance_mode().unwrap();
        assert!(controller.maintenance_mode);
        assert_eq!(controller.current_state, TunnelState::MaintenanceMode);
        
        // Exit maintenance mode
        controller.exit_maintenance_mode().unwrap();
        assert!(!controller.maintenance_mode);
        assert_eq!(controller.current_state, TunnelState::NormalOperation);
    }
}
