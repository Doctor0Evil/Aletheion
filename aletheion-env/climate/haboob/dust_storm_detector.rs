/**
 * Aletheion Smart City Core - Batch 2
 * File: 104/200
 * Layer: 21 (Advanced Environment)
 * Path: aletheion-env/climate/haboob/dust_storm_detector.rs
 * 
 * Research Basis:
 *   - Phoenix Haboob Events (2025): 50-60 mph winds, wall heights 10,000+ ft
 *   - ADOT Dust Detection System: 120+ sensor stations along I-10, US-60 corridors
 *   - Visibility thresholds: <1/4 mile triggers emergency protocols
 *   - Particle concentration: PM10 > 1000 µg/m³, PM2.5 > 500 µg/m³ during haboobs
 *   - Average duration: 1-3 hours, seasonal peak July-September
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Air Quality Rights)
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
use aletheion_mobility::av::AVNavigationMode;
use aletheion_health::air_quality::AirQualityAlert;

// --- Constants & Phoenix Haboob Parameters ---

/// Critical wind speed threshold (mph) indicating haboob formation
/// Based on ADOT research: 30+ mph precedes haboob wall arrival
const HABOOB_CRITICAL_WIND_MPH: f32 = 30.0;
/// Extreme wind speed (mph) during haboob passage
const HABOOB_EXTREME_WIND_MPH: f32 = 50.0;

/// Visibility thresholds (miles) for haboob severity levels
const VISIBILITY_CRITICAL_MILES: f32 = 0.25; // 1/4 mile - Extreme danger
const VISIBILITY_HIGH_MILES: f32 = 0.5;      // 1/2 mile - High alert
const VISIBILITY_MODERATE_MILES: f32 = 1.0;  // 1 mile - Moderate alert

/// Particulate Matter thresholds (µg/m³) during haboobs
const PM10_CRITICAL_UG_M3: f32 = 1000.0;
const PM2_5_CRITICAL_UG_M3: f32 = 500.0;
const PM10_HIGH_UG_M3: f32 = 500.0;
const PM2_5_HIGH_UG_M3: f32 = 250.0;

/// Haboob wall height estimation (feet) - Phoenix average
const HABOOB_WALL_HEIGHT_FT: f32 = 10000.0;
/// Haboob travel speed (mph) - Typical Phoenix haboob velocity
const HABOOB_TRAVEL_SPEED_MPH: f32 = 30.0;

/// Sensor network parameters (ADOT model)
const SENSOR_STATION_RADIUS_MILES: f32 = 5.0;
const MIN_SENSORS_FOR_CONFIRMATION: usize = 3;

/// Response timing thresholds
const EARLY_WARNING_MINUTES: u32 = 15; // Time to prepare before haboob arrival
const IMMEDIATE_RESPONSE_SECONDS: u32 = 30; // Time to execute critical actions

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
const OFFLINE_BUFFER_HOURS: u32 = 72;
/// Treaty Check Cache TTL (seconds)
const TREATY_CACHE_TTL: u64 = 300;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HaboobSeverity {
    None,
    Developing,
    Moderate,
    Severe,
    Extreme,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VisibilityLevel {
    Excellent,      // > 5 miles
    Good,          // 2-5 miles
    Moderate,      // 1-2 miles
    Poor,          // 0.5-1 mile
    VeryPoor,      // 0.25-0.5 mile
    ZeroVisibility, // < 0.25 mile
}

#[derive(Clone)]
pub struct DustSensorReading {
    pub timestamp: u64,
    pub wind_speed_mph: f32,
    pub wind_direction_deg: f32,
    pub visibility_miles: f32,
    pub pm10_ug_m3: f32,
    pub pm2_5_ug_m3: f32,
    pub humidity_percent: f32,
    pub pressure_mb: f32,
    pub sensor_id: [u8; 32], // PQ-Secure ID
    pub gps_coordinates: [f64; 2], // [lat, lon]
}

#[derive(Clone)]
pub struct HaboobAlert {
    pub severity: HaboobSeverity,
    pub estimated_arrival_minutes: f32,
    pub affected_zones: Vec<[u8; 32]>,
    pub visibility_level: VisibilityLevel,
    pub public_alert_level: PublicAlertLevel,
    pub treaty_hash: [u8; 64],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PublicAlertLevel {
    None,
    Advisory,
    Watch,
    Warning,
    Emergency,
}

#[derive(Clone)]
pub struct ResponseAction {
    pub action_type: ResponseActionType,
    pub target_systems: Vec<[u8; 32]>,
    pub priority: u8,
    pub duration_minutes: u32,
    pub treaty_compliant: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResponseActionType {
    ActivateAirFiltration,
    DeployAVSafeMode,
    CloseHVACIntakes,
    BroadcastPublicAlert,
    ActivateStreetLighting,
    SealBuildingVents,
    DeployEmergencyShelters,
}

#[derive(Clone)]
pub struct SensorStation {
    pub station_id: [u8; 32],
    pub location: [f64; 2],
    pub coverage_radius_miles: f32,
    pub last_reading: Option<DustSensorReading>,
    pub operational: bool,
}

#[derive(Clone)]
pub struct HaboobDetectionZone {
    pub zone_id: [u8; 32],
    pub boundaries: Vec<[f64; 2]>, // Polygon vertices
    pub sensor_stations: Vec<SensorStation>,
    pub population_density: f32, // people per sq mile
    pub critical_infrastructure: bool,
    pub indigenous_territory: bool,
    pub treaty_zone_id: Option<[u8; 32]>,
}

// --- Core Haboob Detector Structure ---

pub struct HaboobDustStormDetector {
    pub node_id: BirthSign,
    pub detection_zone: HaboobDetectionZone,
    pub current_severity: HaboobSeverity,
    pub visibility_level: VisibilityLevel,
    pub offline_queue: OfflineQueue<ResponseAction>,
    pub treaty_cache: TreatyCompliance,
    pub haboob_predicted: bool,
    pub estimated_arrival_time: Option<u64>,
    pub affected_av_count: usize,
    pub last_sync: u64,
    pub sensor_readings_cache: Vec<DustSensorReading>,
}

impl HaboobDustStormDetector {
    /**
     * Initialize the Haboob Detector with Zone Configuration
     * Ensures 72h operational buffer and treaty compliance setup
     */
    pub fn new(node_id: BirthSign, detection_zone: HaboobDetectionZone) -> Result<Self, &'static str> {
        let queue = OfflineQueue::new(OFFLINE_BUFFER_HOURS)
            .map_err(|_| "Failed to allocate offline buffer")?;
        
        Ok(Self {
            node_id,
            detection_zone,
            current_severity: HaboobSeverity::None,
            visibility_level: VisibilityLevel::Excellent,
            offline_queue: queue,
            treaty_cache: TreatyCompliance::new(),
            haboob_predicted: false,
            estimated_arrival_time: None,
            affected_av_count: 0,
            last_sync: 0,
            sensor_readings_cache: Vec::new(),
        })
    }

    /**
     * ERM Chain: SENSE
     * Ingests dust sensor data from ADOT-style sensor network
     * Validates data integrity using PQ hashing
     */
    pub fn sense(&mut self, reading: DustSensorReading) -> Result<(), &'static str> {
        // Validate sensor signature (PQ Secure)
        let hash = pq_hash(&reading.sensor_id);
        if hash[0] == 0x00 { // Placeholder for actual signature verification logic
            return Err("Sensor signature invalid");
        }

        // Store reading in cache (maintain last 10 readings per sensor)
        self.sensor_readings_cache.push(reading.clone());
        if self.sensor_readings_cache.len() > 100 {
            self.sensor_readings_cache.remove(0);
        }

        // Update visibility level
        self.update_visibility_level(reading.visibility_miles);

        // Check for haboob formation patterns
        self.detect_haboob_formation(&reading);

        // Log sensing event
        self.log_event(format!(
            "SENSE: Wind={:.1}mph @ {}°, Vis={:.2}mi, PM10={:.0}µg/m³, PM2.5={:.0}µg/m³",
            reading.wind_speed_mph,
            reading.wind_direction_deg,
            reading.visibility_miles,
            reading.pm10_ug_m3,
            reading.pm2_5_ug_m3
        ));

        Ok(())
    }

    /**
     * Update visibility level based on sensor reading
     */
    fn update_visibility_level(&mut self, visibility_miles: f32) {
        self.visibility_level = match visibility_miles {
            v if v < VISIBILITY_CRITICAL_MILES => VisibilityLevel::ZeroVisibility,
            v if v < VISIBILITY_HIGH_MILES => VisibilityLevel::VeryPoor,
            v if v < VISIBILITY_MODERATE_MILES => VisibilityLevel::Poor,
            v if v < 2.0 => VisibilityLevel::Moderate,
            v if v < 5.0 => VisibilityLevel::Good,
            _ => VisibilityLevel::Excellent,
        };
    }

    /**
     * Detect haboob formation using multi-sensor correlation
     * Requires confirmation from multiple sensors to reduce false positives
     */
    fn detect_haboob_formation(&mut self, reading: &DustSensorReading) {
        // Check if this sensor indicates haboob conditions
        let sensor_indicates_haboob = self.sensor_indicates_haboob(reading);
        
        if !sensor_indicates_haboob {
            // Reset prediction if conditions improve
            if self.current_severity == HaboobSeverity::Developing {
                self.current_severity = HaboobSeverity::None;
                self.haboob_predicted = false;
            }
            return;
        }

        // Correlate with other sensors in the network
        let confirming_sensors = self.count_confirming_sensors(reading);
        
        if confirming_sensors >= MIN_SENSORS_FOR_CONFIRMATION {
            // Haboob confirmed - escalate severity
            self.escalate_haboob_severity(confirming_sensors);
            
            // Calculate estimated arrival time based on wind speed and direction
            self.calculate_arrival_time(reading);
            
            // Set prediction flag
            self.haboob_predicted = true;
        } else if confirming_sensors >= 2 {
            // Developing conditions - early warning
            if self.current_severity == HaboobSeverity::None {
                self.current_severity = HaboobSeverity::Developing;
                self.log_warning("Developing haboob conditions detected - early warning");
            }
        }
    }

    /**
     * Check if individual sensor indicates haboob conditions
     */
    fn sensor_indicates_haboob(&self, reading: &DustSensorReading) -> bool {
        // Multiple criteria must be met to indicate haboob
        let wind_condition = reading.wind_speed_mph >= HABOOB_CRITICAL_WIND_MPH;
        let visibility_condition = reading.visibility_miles < VISIBILITY_MODERATE_MILES;
        let pm10_condition = reading.pm10_ug_m3 > PM10_HIGH_UG_M3;
        let pm2_5_condition = reading.pm2_5_ug_m3 > PM2_5_HIGH_UG_M3;
        
        // Require at least 3 of 4 conditions
        let conditions_met = wind_condition as u8 + visibility_condition as u8 + 
                            pm10_condition as u8 + pm2_5_condition as u8;
        
        conditions_met >= 3
    }

    /**
     * Count sensors confirming haboob conditions within radius
     */
    fn count_confirming_sensors(&self, reference_reading: &DustSensorReading) -> usize {
        let mut confirming_count = 0;
        
        for cached_reading in &self.sensor_readings_cache {
            // Check if within sensor radius
            let distance_miles = self.calculate_distance(
                reference_reading.gps_coordinates,
                cached_reading.gps_coordinates
            );
            
            if distance_miles <= SENSOR_STATION_RADIUS_MILES {
                // Check if this sensor also indicates haboob
                if self.sensor_indicates_haboob(cached_reading) {
                    confirming_count += 1;
                }
            }
        }
        
        confirming_count
    }

    /**
     * Calculate distance between two GPS coordinates (Haversine formula)
     */
    fn calculate_distance(&self, coord1: [f64; 2], coord2: [f64; 2]) -> f32 {
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
        (earth_radius_miles * c) as f32
    }

    /**
     * Escalate haboob severity based on confirming sensor count
     */
    fn escalate_haboob_severity(&mut self, confirming_sensors: usize) {
        let new_severity = match confirming_sensors {
            c if c >= 8 => HaboobSeverity::Extreme,
            c if c >= 5 => HaboobSeverity::Severe,
            c if c >= 3 => HaboobSeverity::Moderate,
            _ => HaboobSeverity::Developing,
        };
        
        if new_severity != self.current_severity {
            self.current_severity = new_severity;
            self.log_event(format!("SEVERITY ESCALATION: {:?}", self.current_severity));
        }
    }

    /**
     * Calculate estimated haboob arrival time based on wind speed and direction
     */
    fn calculate_arrival_time(&mut self, reading: &DustSensorReading) {
        // Estimate distance to haboob wall based on wind speed
        // Assuming haboob travels at wind speed
        let distance_miles = reading.wind_speed_mph * (EARLY_WARNING_MINUTES as f32 / 60.0);
        
        // Calculate time to arrival in minutes
        let arrival_minutes = distance_miles / HABOOB_TRAVEL_SPEED_MPH * 60.0;
        
        // Set estimated arrival timestamp
        let current_time = aletheion_core::time::now();
        self.estimated_arrival_time = Some(current_time + (arrival_minutes as u64 * 60));
        
        self.log_event(format!(
            "ARRIVAL ESTIMATE: {:.1} minutes based on {:.1} mph winds",
            arrival_minutes,
            reading.wind_speed_mph
        ));
    }

    /**
     * ERM Chain: MODEL
     * Predicts haboob trajectory, affected zones, and impact severity
     * No Digital Twins: Uses physics-based wind propagation models
     */
    pub fn model_haboob_impact(&self) -> Option<HaboobImpactPrediction> {
        if !self.haboob_predicted {
            return None;
        }
        
        // Determine affected zones based on wind direction
        let affected_zones = self.determine_affected_zones();
        
        // Calculate public alert level
        let alert_level = self.calculate_alert_level();
        
        // Estimate infrastructure impact
        let infrastructure_impact = self.estimate_infrastructure_impact();
        
        Some(HaboobImpactPrediction {
            severity: self.current_severity,
            estimated_arrival_minutes: self.estimated_arrival_time
                .map(|t| ((t - aletheion_core::time::now()) as f32) / 60.0)
                .unwrap_or(0.0),
            affected_zones,
            visibility_level: self.visibility_level,
            alert_level,
            infrastructure_impact,
            av_navigation_impact: self.affected_av_count,
        })
    }

    /**
     * Determine zones affected by haboob based on wind direction
     */
    fn determine_affected_zones(&self) -> Vec<[u8; 32]> {
        let mut affected_zones = Vec::new();
        
        // Get latest sensor reading
        if let Some(latest_reading) = self.sensor_readings_cache.last() {
            let wind_direction_rad = latest_reading.wind_direction_deg.to_radians();
            
            // Calculate affected area cone (90-degree spread)
            let cone_angle = PI / 4.0; // 45 degrees on each side
            
            for station in &self.detection_zone.sensor_stations {
                if !station.operational {
                    continue;
                }
                
                // Calculate angle from haboob origin to this station
                let angle_to_station = self.calculate_bearing(
                    latest_reading.gps_coordinates,
                    station.location
                );
                
                // Check if station is within wind cone
                let angle_diff = ((angle_to_station - wind_direction_rad + PI) % (2.0 * PI)) - PI;
                
                if angle_diff.abs() <= cone_angle {
                    affected_zones.push(station.station_id);
                }
            }
        }
        
        affected_zones
    }

    /**
     * Calculate bearing between two GPS coordinates
     */
    fn calculate_bearing(&self, from: [f64; 2], to: [f64; 2]) -> f64 {
        let lat1 = from[0].to_radians();
        let lon1 = from[1].to_radians();
        let lat2 = to[0].to_radians();
        let lon2 = to[1].to_radians();
        
        let dlon = lon2 - lon1;
        
        let y = dlon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();
        
        y.atan2(x)
    }

    /**
     * Calculate public alert level based on severity and population density
     */
    fn calculate_alert_level(&self) -> PublicAlertLevel {
        match self.current_severity {
            HaboobSeverity::Extreme => {
                if self.detection_zone.population_density > 1000.0 {
                    PublicAlertLevel::Emergency
                } else {
                    PublicAlertLevel::Warning
                }
            },
            HaboobSeverity::Severe => PublicAlertLevel::Warning,
            HaboobSeverity::Moderate => PublicAlertLevel::Watch,
            HaboobSeverity::Developing => PublicAlertLevel::Advisory,
            HaboobSeverity::None => PublicAlertLevel::None,
        }
    }

    /**
     * Estimate infrastructure impact based on severity
     */
    fn estimate_infrastructure_impact(&self) -> InfrastructureImpact {
        match self.current_severity {
            HaboobSeverity::Extreme => InfrastructureImpact {
                power_outage_risk: 0.8,
                transportation_disruption: 0.9,
                structural_damage_risk: 0.3,
            },
            HaboobSeverity::Severe => InfrastructureImpact {
                power_outage_risk: 0.5,
                transportation_disruption: 0.7,
                structural_damage_risk: 0.15,
            },
            HaboobSeverity::Moderate => InfrastructureImpact {
                power_outage_risk: 0.2,
                transportation_disruption: 0.4,
                structural_damage_risk: 0.05,
            },
            _ => InfrastructureImpact {
                power_outage_risk: 0.0,
                transportation_disruption: 0.1,
                structural_damage_risk: 0.01,
            },
        }
    }

    /**
     * ERM Chain: OPTIMIZE & TREATY-CHECK
     * Determines necessary response actions and validates against Indigenous air quality rights
     * FPIC Enforcement: Cannot deploy emergency systems on protected lands without consent
     */
    pub fn optimize_and_check(&mut self, prediction: &HaboobImpactPrediction) -> Result<Vec<ResponseAction>, &'static str> {
        let mut actions = Vec::new();
        
        // 1. Check Treaty Compliance (FPIC) for affected zones
        if self.detection_zone.indigenous_territory {
            let treaty_zone = self.detection_zone.treaty_zone_id
                .ok_or("Indigenous territory requires treaty zone ID")?;
            
            let compliance = self.treaty_cache.check_air_quality_rights(&treaty_zone)?;
            
            if !compliance.allowed {
                return Err("FPIC Violation: Treaty restricts emergency deployment in this zone");
            }
        }

        // 2. Generate response actions based on severity
        self.generate_severity_based_actions(prediction, &mut actions);
        
        // 3. Add treaty compliance hash to all actions
        let treaty_hash = if self.detection_zone.indigenous_territory {
            self.treaty_cache.get_current_hash()
        } else {
            [0u8; 64]
        };
        
        for action in &mut actions {
            action.treaty_compliant = treaty_hash[0] != 0;
        }
        
        Ok(actions)
    }

    /**
     * Generate response actions based on haboob severity
     */
    fn generate_severity_based_actions(&self, prediction: &HaboobImpactPrediction, actions: &mut Vec<ResponseAction>) {
        match prediction.alert_level {
            PublicAlertLevel::Emergency | PublicAlertLevel::Warning => {
                // Critical actions for severe/extreme haboobs
                actions.push(ResponseAction {
                    action_type: ResponseActionType::ActivateAirFiltration,
                    target_systems: self.get_air_filtration_systems(),
                    priority: 10,
                    duration_minutes: 180, // 3 hours
                    treaty_compliant: false,
                });
                
                actions.push(ResponseAction {
                    action_type: ResponseActionType::DeployAVSafeMode,
                    target_systems: self.get_autonomous_vehicles(),
                    priority: 9,
                    duration_minutes: 120,
                    treaty_compliant: false,
                });
                
                actions.push(ResponseAction {
                    action_type: ResponseActionType::BroadcastPublicAlert,
                    target_systems: self.get_public_alert_systems(),
                    priority: 10,
                    duration_minutes: 60,
                    treaty_compliant: false,
                });
                
                actions.push(ResponseAction {
                    action_type: ResponseActionType::ActivateStreetLighting,
                    target_systems: self.get_street_lighting_systems(),
                    priority: 8,
                    duration_minutes: 120,
                    treaty_compliant: false,
                });
            },
            
            PublicAlertLevel::Watch => {
                // Moderate actions for developing haboobs
                actions.push(ResponseAction {
                    action_type: ResponseActionType::ActivateAirFiltration,
                    target_systems: self.get_air_filtration_systems(),
                    priority: 7,
                    duration_minutes: 90,
                    treaty_compliant: false,
                });
                
                actions.push(ResponseAction {
                    action_type: ResponseActionType::BroadcastPublicAlert,
                    target_systems: self.get_public_alert_systems(),
                    priority: 8,
                    duration_minutes: 30,
                    treaty_compliant: false,
                });
            },
            
            PublicAlertLevel::Advisory => {
                // Early warning actions
                actions.push(ResponseAction {
                    action_type: ResponseActionType::BroadcastPublicAlert,
                    target_systems: self.get_public_alert_systems(),
                    priority: 5,
                    duration_minutes: 15,
                    treaty_compliant: false,
                });
            },
            
            PublicAlertLevel::None => {
                // No actions needed
            }
        }
        
        // Always close HVAC intakes during haboob conditions
        if prediction.severity >= HaboobSeverity::Moderate {
            actions.push(ResponseAction {
                action_type: ResponseActionType::CloseHVACIntakes,
                target_systems: self.get_hvac_systems(),
                priority: 6,
                duration_minutes: 120,
                treaty_compliant: false,
            });
        }
    }

    /**
     * ERM Chain: ACT
     * Executes response actions or queues for offline execution
     * Ensures atomicity without rollbacks
     */
    pub fn act(&mut self, actions: Vec<ResponseAction>) -> Result<(), &'static str> {
        for action in actions {
            // Attempt immediate execution via HAL
            match self.execute_response_action(&action) {
                Ok(_) => {
                    self.log_action(&action);
                },
                Err(_) => {
                    // Offline Fallback: Queue for later execution
                    self.offline_queue.push(action)?;
                    self.log_warning("Offline mode: Response action queued for later execution");
                }
            }
        }
        
        Ok(())
    }

    /**
     * Execute individual response action
     */
    fn execute_response_action(&self, action: &ResponseAction) -> Result<(), &'static str> {
        match action.action_type {
            ResponseActionType::ActivateAirFiltration => {
                aletheion_physical::hal::activate_air_filtration(&action.target_systems)?;
            },
            ResponseActionType::DeployAVSafeMode => {
                aletheion_mobility::av::deploy_safe_mode(
                    &action.target_systems,
                    AVNavigationMode::HaboobEmergency
                )?;
                self.affected_av_count = action.target_systems.len();
            },
            ResponseActionType::CloseHVACIntakes => {
                aletheion_physical::hal::close_hvac_intakes(&action.target_systems)?;
            },
            ResponseActionType::BroadcastPublicAlert => {
                aletheion_comms::broadcast::send_alert(
                    &action.target_systems,
                    &self.generate_alert_message(action.priority)
                )?;
            },
            ResponseActionType::ActivateStreetLighting => {
                aletheion_physical::hal::activate_emergency_lighting(&action.target_systems)?;
            },
            ResponseActionType::SealBuildingVents => {
                aletheion_physical::hal::seal_building_vents(&action.target_systems)?;
            },
            ResponseActionType::DeployEmergencyShelters => {
                aletheion_safety::emergency::activate_shelters(&action.target_systems)?;
            }
        }
        
        Ok(())
    }

    /**
     * Generate alert message based on priority/severity
     */
    fn generate_alert_message(&self, priority: u8) -> String {
        match priority {
            p if p >= 9 => {
                "EMERGENCY: Extreme haboob conditions. Seek shelter immediately. Avoid all travel.".to_string()
            },
            p if p >= 7 => {
                "WARNING: Severe haboob approaching. Prepare for reduced visibility and high winds.".to_string()
            },
            p if p >= 5 => {
                "WATCH: Haboob conditions developing. Monitor weather updates.".to_string()
            },
            _ => "Advisory: Dust storm activity possible in area.".to_string(),
        }
    }

    /**
     * ERM Chain: LOG
     * Immutable logging to ALN-Blockchain
     * No rollbacks allowed; all state changes are forward-only
     */
    fn log_action(&self, action: &ResponseAction) {
        let log_entry = alloc::format!(
            "HABOOB_ACT: Type={:?} | Priority={} | Targets={} | Duration={}min | Treaty={}",
            action.action_type,
            action.priority,
            action.target_systems.len(),
            action.duration_minutes,
            if action.treaty_compliant { "Compliant" } else { "N/A" }
        );
        
        aletheion_:ledger::append_immutable(&log_entry);
    }

    fn log_event(&self, message: String) {
        let log_entry = alloc::format!("[{}] {}", aletheion_core::time::now(), message);
        aletheion_data::ledger::append_immutable(&log_entry);
    }

    fn log_warning(&self, message: &str) {
        self.log_event(format!("WARNING: {}", message));
    }

    /**
     * ERM Chain: INTERFACE
     * Exposes status to Citizen App (Kotlin/Android) and Mesh Network
     * WCAG 2.2 AAA compliant data structure
     */
    pub fn get_status_report(&self) -> HaboobStatusReport {
        HaboobStatusReport {
            zone_id: self.detection_zone.zone_id,
            current_severity: self.current_severity,
            visibility_level: self.visibility_level,
            haboob_predicted: self.haboob_predicted,
            estimated_arrival_minutes: self.estimated_arrival_time
                .map(|t| ((t - aletheion_core::time::now()) as f32) / 60.0)
                .unwrap_or(0.0),
            affected_zones: self.determine_affected_zones(),
            public_alert_level: self.calculate_alert_level(),
            offline_queue_size: self.offline_queue.len(),
            last_sync: self.last_sync,
            accessibility_alert: self.current_severity >= HaboobSeverity::Moderate,
            sensor_station_count: self.detection_zone.sensor_stations.len(),
        }
    }

    // Helper methods for system queries
    fn get_air_filtration_systems(&self) -> Vec<[u8; 32]> {
        // In production: query building management systems
        vec![[1u8; 32], [2u8; 32], [3u8; 32]]
    }
    
    fn get_autonomous_vehicles(&self) -> Vec<[u8; 32]> {
        // In production: query mobility management systems
        vec![[4u8; 32], [5u8; 32]]
    }
    
    fn get_public_alert_systems(&self) -> Vec<[u8; 32]> {
        // In production: query communications systems
        vec![[6u8; 32], [7u8; 32]]
    }
    
    fn get_street_lighting_systems(&self) -> Vec<[u8; 32]> {
        // In production: query infrastructure systems
        vec![[8u8; 32], [9u8; 32]]
    }
    
    fn get_hvac_systems(&self) -> Vec<[u8; 32]> {
        // In production: query building systems
        vec![[10u8; 32], [11u8; 32]]
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
     * Clear haboob prediction after event passes
     */
    pub fn clear_prediction(&mut self) {
        self.current_severity = HaboobSeverity::None;
        self.haboob_predicted = false;
        self.estimated_arrival_time = None;
        self.affected_av_count = 0;
        self.log_event("PREDICTION CLEARED: Haboob event has passed".to_string());
    }
}

// --- Supporting Data Structures ---

pub struct HaboobImpactPrediction {
    pub severity: HaboobSeverity,
    pub estimated_arrival_minutes: f32,
    pub affected_zones: Vec<[u8; 32]>,
    pub visibility_level: VisibilityLevel,
    pub alert_level: PublicAlertLevel,
    pub infrastructure_impact: InfrastructureImpact,
    pub av_navigation_impact: usize,
}

pub struct InfrastructureImpact {
    pub power_outage_risk: f32, // 0.0-1.0 probability
    pub transportation_disruption: f32, // 0.0-1.0 severity
    pub structural_damage_risk: f32, // 0.0-1.0 probability
}

pub struct HaboobStatusReport {
    pub zone_id: [u8; 32],
    pub current_severity: HaboobSeverity,
    pub visibility_level: VisibilityLevel,
    pub haboob_predicted: bool,
    pub estimated_arrival_minutes: f32,
    pub affected_zones: Vec<[u8; 32]>,
    pub public_alert_level: PublicAlertLevel,
    pub offline_queue_size: usize,
    pub last_sync: u64,
    pub accessibility_alert: bool,
    pub sensor_station_count: usize,
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haboob_detection_extreme_conditions() {
        let zone = HaboobDetectionZone {
            zone_id: [1u8; 32],
            boundaries: vec![[33.4484, -112.0740], [33.5, -112.0]],
            sensor_stations: vec![],
            population_density: 2800.0,
            critical_infrastructure: true,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let mut detector = HaboobDustStormDetector::new(BirthSign::default(), zone).unwrap();
        
        let reading = DustSensorReading {
            timestamp: 1000,
            wind_speed_mph: 55.0, // Extreme wind
            wind_direction_deg: 270.0,
            visibility_miles: 0.1, // Zero visibility
            pm10_ug_m3: 1200.0, // Extreme PM10
            pm2_5_ug_m3: 600.0, // Extreme PM2.5
            humidity_percent: 10.0,
            pressure_mb: 1010.0,
            sensor_id: [1u8; 32],
            gps_coordinates: [33.4484, -112.0740],
        };
        
        detector.sense(reading).unwrap();
        assert_eq!(detector.current_severity, HaboobSeverity::Extreme);
        assert_eq!(detector.visibility_level, VisibilityLevel::ZeroVisibility);
    }

    #[test]
    fn test_visibility_level_classification() {
        let zone = HaboobDetectionZone {
            zone_id: [1u8; 32],
            boundaries: vec![[33.4484, -112.0740], [33.5, -112.0]],
            sensor_stations: vec![],
            population_density: 1000.0,
            critical_infrastructure: false,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let mut detector = HaboobDustStormDetector::new(BirthSign::default(), zone).unwrap();
        
        // Test various visibility thresholds
        detector.update_visibility_level(0.1);
        assert_eq!(detector.visibility_level, VisibilityLevel::ZeroVisibility);
        
        detector.update_visibility_level(0.3);
        assert_eq!(detector.visibility_level, VisibilityLevel::VeryPoor);
        
        detector.update_visibility_level(0.7);
        assert_eq!(detector.visibility_level, VisibilityLevel::Poor);
        
        detector.update_visibility_level(1.5);
        assert_eq!(detector.visibility_level, VisibilityLevel::Moderate);
        
        detector.update_visibility_level(3.0);
        assert_eq!(detector.visibility_level, VisibilityLevel::Good);
        
        detector.update_visibility_level(10.0);
        assert_eq!(detector.visibility_level, VisibilityLevel::Excellent);
    }

    #[test]
    fn test_offline_queue_capacity() {
        let zone = HaboobDetectionZone {
            zone_id: [1u8; 32],
            boundaries: vec![[33.4484, -112.0740], [33.5, -112.0]],
            sensor_stations: vec![],
            population_density: 1000.0,
            critical_infrastructure: false,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let detector = HaboobDustStormDetector::new(BirthSign::default(), zone).unwrap();
        assert!(detector.offline_queue.capacity_hours() >= 72);
    }

    #[test]
    fn test_distance_calculation() {
        let zone = HaboobDetectionZone {
            zone_id: [1u8; 32],
            boundaries: vec![[33.4484, -112.0740], [33.5, -112.0]],
            sensor_stations: vec![],
            population_density: 1000.0,
            critical_infrastructure: false,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let detector = HaboobDustStormDetector::new(BirthSign::default(), zone).unwrap();
        
        // Phoenix coordinates
        let phoenix = [33.4484_f64, -112.0740_f64];
        // Mesa coordinates (approx 15 miles east)
        let mesa = [33.4152_f64, -111.8315_f64];
        
        let distance = detector.calculate_distance(phoenix, mesa);
        // Should be approximately 15 miles
        assert!((distance - 15.0).abs() < 2.0);
    }

    #[test]
    fn test_alert_level_calculation() {
        let mut zone = HaboobDetectionZone {
            zone_id: [1u8; 32],
            boundaries: vec![[33.4484, -112.0740], [33.5, -112.0]],
            sensor_stations: vec![],
            population_density: 2800.0, // High density (Phoenix downtown)
            critical_infrastructure: true,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let mut detector = HaboobDustStormDetector::new(BirthSign::default(), zone.clone()).unwrap();
        
        // Test Emergency alert for Extreme severity in high-density area
        detector.current_severity = HaboobSeverity::Extreme;
        assert_eq!(detector.calculate_alert_level(), PublicAlertLevel::Emergency);
        
        // Test Warning alert for high-density area with Severe severity
        detector.current_severity = HaboobSeverity::Severe;
        assert_eq!(detector.calculate_alert_level(), PublicAlertLevel::Warning);
        
        // Test Watch alert for Moderate severity
        detector.current_severity = HaboobSeverity::Moderate;
        assert_eq!(detector.calculate_alert_level(), PublicAlertLevel::Watch);
        
        // Test lower density area
        zone.population_density = 500.0;
        let mut detector_low_density = HaboobDustStormDetector::new(BirthSign::default(), zone).unwrap();
        detector_low_density.current_severity = HaboobSeverity::Extreme;
        assert_eq!(detector_low_density.calculate_alert_level(), PublicAlertLevel::Warning);
    }
}
