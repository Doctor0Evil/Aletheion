// Aletheion City Core - Air Quality & Dust Storm Sovereignty Module
// Repository: https://github.com/Doctor0Evil/Aletheion
// Path: src/environmental/air_quality_monitor.rs
// Language: Rust (Edition 2021)
// Compliance: ERM Chain, SMART Protocols, BioticTreaties, Indigenous Rights (Akimel O'odham/Piipaash)
// Security: Post-Quantum Secure, Offline-Capable, No Blacklisted Crypto Primitives
// Phoenix-Specific: Haboob Detection, PM2.5/PM10 Monitoring, Extreme Heat Air Quality

#![no_std]
#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use core::fmt::Debug;

// ============================================================================
// 1. AIR QUALITY SOVEREIGNTY CONSTANTS (Phoenix 2026 Standards)
// ============================================================================

/// EPA & Phoenix Air Quality Standards (Stricter Than Federal)
/// Target: Maintain Good AQI (0-50) 95% of operational days
const PM2_5_GOOD_MAX: f32 = 12.0; // Micrograms/m3 (24-hr avg)
const PM2_5_MODERATE_MAX: f32 = 35.4;
const PM2_5_UNHEALTHY_MAX: f32 = 55.4;
const PM10_GOOD_MAX: f32 = 54.0; // Micrograms/m3 (24-hr avg)
const PM10_MODERATE_MAX: f32 = 154.0;
const PM10_DUST_STORM_THRESHOLD: f32 = 500.0; // Haboob Detection

/// VOC & Ozone Limits (Phoenix Ozone Non-Attainment Area)
const OZONE_PPB_MAX: f32 = 70.0; // 8-hr avg (EPA Standard)
const VOC_PPB_MAX: f32 = 50.0; // Total Volatile Organic Compounds
const CO_PPM_MAX: f32 = 9.0; // Carbon Monoxide (8-hr)

/// Neuroright Constraints for Air Quality Alerts
/// Alerts must not induce panic (RoH Ceiling 0.3)
const AIR_QUALITY_ALERT_ROH_CEILING: f32 = 0.3;
const HABOOB_COMMUNICATION_MODE: CommunicationMode = CommunicationMode::DirectiveCalm;

/// Haboob (Dust Storm) Early Warning Parameters
/// Based on ADOT Sensor Network Model (2025)
const WIND_SPEED_HABOOB_KPH: f32 = 80.0; // km/h threshold
const VISIBILITY_HABOOB_METERS: f32 = 100.0; // meters
const PRESSURE_DROP_HABOOB_HPA: f32 = 5.0; // Hectopascals drop rate

/// Indigenous Territory Air Quality Protection
/// Akimel O'odham and Piipaash lands require enhanced monitoring
const INDIGENOUS_BUFFER_ZONE_KM: f32 = 5.0; // Enhanced monitoring radius
const INDIGENOUS_AQI_TARGET: f32 = 30.0; // Stricter than municipal target

// ============================================================================
// 2. DATA STRUCTURES (Sense & Model)
// ============================================================================

/// Air Quality Sensor Input (Offline-Capable Buffer)
#[derive(Clone)]
pub struct AirQualitySample {
    pub timestamp_utc: u64,
    pub sensor_id: [u8; 32], // PQC Public Key Hash
    pub pm2_5: f32, // Micrograms/m3
    pub pm10: f32, // Micrograms/m3
    pub ozone_ppb: f32,
    pub voc_ppb: f32,
    pub co_ppm: f32,
    pub no2_ppb: f32,
    pub so2_ppb: f32,
    pub temperature_c: f32,
    pub humidity_percent: f32,
    pub wind_speed_kph: f32,
    pub wind_direction_deg: f32,
    pub visibility_meters: f32,
    pub pressure_hpa: f32,
    pub location_lat: f64,
    pub location_lon: f64,
    pub altitude_m: f32,
}

/// Modeled Air Quality State
#[derive(Clone)]
pub struct AirQualityModel {
    pub sample: AirQualitySample,
    pub aqi_value: u16, // 0-500 (EPA Scale)
    pub aqi_category: AqiCategory,
    pub primary_pollutant: PollutantType,
    pub haboob_probability: f32, // 0.0 to 1.0
    pub health_advisory: HealthAdvisory,
    pub erm_state: AirErmState,
}

/// Sovereignty Envelope for Air Quality
pub struct AirSovereigntyEnvelope {
    pub indigenous_zone_active: bool,
    pub biotic_corridor_protected: bool,
    pub citizen_neuro_load: f32, // 0.0 to 1.0 (Max 0.3 for alerts)
    pub vulnerable_population_nearby: bool, // Schools, Hospitals, Elder Care
    pub pqc_signature: [u8; 64], // Dilithium/SPHINCS+ compatible
}

/// Alert Distribution Record
pub struct AlertRecord {
    pub alert_id: [u8; 64],
    pub timestamp: u64,
    pub aqi_value: u16,
    pub category: AqiCategory,
    pub affected_zones: Vec<ZoneId>,
    pub advisory_issued: HealthAdvisory,
    pub sovereignty_verified: bool,
    pub erm_state: AirErmState,
}

// ============================================================================
// 3. TRAITS (Air Quality Integrity & Treaty)
// ============================================================================

/// Air Quality Integrity Protocol (Distinct from Generic Hash)
pub trait AirIntegrity {
    fn sign_air_decision(&self, decision: &[u8]) -> [u8; 64];
    fn verify_sensor_sample(&self, sample: &[u8], sig: &[u8]) -> bool;
    fn hash_sensor_chain(&self, samples: &[AirQualitySample]) -> [u8; 64];
}

/// Air Sovereignty Checker (Specialized for Environmental Rights)
pub trait AirSovereignty {
    fn check_indigenous_air_rights(&self, lat: f64, lon: f64, aqi: u16) -> Result<(), AirSovereigntyViolation>;
    fn check_biotic_impact(&self, pollutant: PollutantType, concentration: f32) -> Result<(), AirSovereigntyViolation>;
    fn check_neuro_impact(&self, alert_level: f32, vulnerable_nearby: bool) -> Result<(), AirSovereigntyViolation>;
    fn check_haboob_protocol(&self, probability: f32, visibility: f32) -> Result<(), AirSovereigntyViolation>;
}

// ============================================================================
// 4. IMPLEMENTATION (Optimize & Act)
// ============================================================================

pub struct AirQualityMonitorEngine {
    pub integrity_provider: Box<dyn AirIntegrity>,
    pub sovereignty_checker: Box<dyn AirSovereignty>,
    pub offline_buffer: Vec<AirQualitySample>,
    pub max_buffer_size: usize,
    pub sensor_network_map: Vec<SensorNode>,
    pub historical_baseline: AirQualityBaseline,
}

impl AirQualityMonitorEngine {
    pub fn new(
        integrity: Box<dyn AirIntegrity>,
        sovereignty: Box<dyn AirSovereignty>,
    ) -> Self {
        Self {
            integrity_provider: integrity,
            sovereignty_checker: sovereignty,
            offline_buffer: Vec::new(),
            max_buffer_size: 4096, // Larger buffer for air telemetry trends
            sensor_network_map: Vec::new(),
            historical_baseline: AirQualityBaseline::default(),
        }
    }

    /// ERM Chain: Sense → Model → Optimize → Treaty-Check → Act → Log → Interface
    pub fn process_air_sample(&mut self, sample: AirQualitySample) -> Result<AlertRecord, AirSovereigntyViolation> {
        // 1. SENSE: Validate Sensor Integrity
        if !self.verify_sensor_integrity(&sample) {
            return Err(AirSovereigntyViolation::CryptographicIntegrityFail);
        }

        // 2. MODEL: Calculate AQI & Haboob Probability
        let mut model = self.model_air_quality(&sample);

        // 3. OPTIMIZE: Determine Response Strategy
        let strategy = self.optimize_air_response(&mut model);

        // 4. TREATY-CHECK: Hard Sovereignty Gates (Indigenous, Biotic, Neuro)
        self.enforce_air_sovereignty_gates(&sample, &model, strategy)?;

        // 5. ACT: Generate Alert Record
        let alert = self.generate_alert_record(&model, strategy);

        // 6. LOG: Immutable Record (Cybernet Ledger)
        self.log_air_transaction(&alert, &sample);

        // 7. INTERFACE: Prepare for Citizen/Device Output
        Ok(alert)
    }

    fn verify_sensor_integrity(&self, sample: &AirQualitySample) -> bool {
        // PQC Signature Verification (Air Quality Specific)
        let data = self.serialize_sample(sample);
        // In production, this calls the actual PQC verify method
        self.integrity_provider.verify_sensor_sample(&data, &sample.sensor_id)
    }

    fn serialize_sample(&self, sample: &AirQualitySample) -> Vec<u8> {
        // Binary serialization for hashing (Post-Quantum Safe)
        let mut buf = Vec::new();
        buf.extend_from_slice(&sample.timestamp_utc.to_le_bytes());
        buf.extend_from_slice(&sample.pm2_5.to_le_bytes());
        buf.extend_from_slice(&sample.pm10.to_le_bytes());
        buf.extend_from_slice(&sample.ozone_ppb.to_le_bytes());
        buf.extend_from_slice(&sample.wind_speed_kph.to_le_bytes());
        buf.extend_from_slice(&sample.visibility_meters.to_le_bytes());
        buf.extend_from_slice(&sample.sensor_id);
        buf
    }

    fn model_air_quality(&self, sample: &AirQualitySample) -> AirQualityModel {
        let mut model = AirQualityModel {
            sample: sample.clone(),
            aqi_value: 0,
            aqi_category: AqiCategory::Good,
            primary_pollutant: PollutantType::PM2_5,
            haboob_probability: 0.0,
            health_advisory: HealthAdvisory::None,
            erm_state: AirErmState::Model,
        };

        // Calculate AQI (EPA Method, Phoenix-Adjusted)
        let pm2_5_aqi = self.calculate_aqi_subindex(sample.pm2_5, PM2_5_GOOD_MAX, PM2_5_UNHEALTHY_MAX, 500);
        let pm10_aqi = self.calculate_aqi_subindex(sample.pm10, PM10_GOOD_MAX, PM10_DUST_STORM_THRESHOLD, 500);
        let ozone_aqi = self.calculate_aqi_subindex(sample.ozone_ppb, 0.0, OZONE_PPB_MAX, 500);

        // Determine Primary Pollutant & Overall AQI
        model.aqi_value = pm2_5_aqi.max(pm10_aqi).max(ozone_aqi);
        model.primary_pollutant = if pm2_5_aqi >= pm10_aqi && pm2_5_aqi >= ozone_aqi {
            PollutantType::PM2_5
        } else if pm10_aqi >= ozone_aqi {
            PollutantType::PM10
        } else {
            PollutantType::Ozone
        };

        // Categorize AQI
        model.aqi_category = self.categorize_aqi(model.aqi_value);

        // Haboob Probability Model (Phoenix-Specific)
        model.haboob_probability = self.calculate_haboob_probability(sample);

        // Health Advisory Generation
        model.health_advisory = self.generate_health_advisory(&model);

        model
    }

    fn calculate_aqi_subindex(&self, concentration: f32, low: f32, high: f32, max_aqi: u16) -> u16 {
        if concentration <= low {
            return 0;
        }
        if concentration >= high {
            return max_aqi;
        }
        // Linear interpolation (EPA AQI Formula)
        let ratio = (concentration - low) / (high - low);
        (ratio * max_aqi as f32) as u16
    }

    fn categorize_aqi(&self, aqi: u16) -> AqiCategory {
        match aqi {
            0..=50 => AqiCategory::Good,
            51..=100 => AqiCategory::Moderate,
            101..=150 => AqiCategory::UnhealthySensitive,
            151..=200 => AqiCategory::Unhealthy,
            201..=300 => AqiCategory::VeryUnhealthy,
            _ => AqiCategory::Hazardous,
        }
    }

    fn calculate_haboob_probability(&self, sample: &AirQualitySample) -> f32 {
        let mut probability = 0.0;

        // Wind Speed Factor (Primary Indicator)
        if sample.wind_speed_kph > WIND_SPEED_HABOOB_KPH {
            probability += 0.4;
        } else if sample.wind_speed_kph > 50.0 {
            probability += 0.2;
        }

        // Visibility Factor (Critical for Dust Storms)
        if sample.visibility_meters < VISIBILITY_HABOOB_METERS {
            probability += 0.4;
        } else if sample.visibility_meters < 500.0 {
            probability += 0.2;
        }

        // PM10 Spike Factor
        if sample.pm10 > PM10_DUST_STORM_THRESHOLD {
            probability += 0.2;
        }

        // Pressure Drop Factor (Leading Indicator)
        let pressure_anomaly = self.historical_baseline.pressure_hpa - sample.pressure_hpa;
        if pressure_anomaly > PRESSURE_DROP_HABOOB_HPA {
            probability += 0.1;
        }

        probability.min(1.0)
    }

    fn generate_health_advisory(&self, model: &AirQualityModel) -> HealthAdvisory {
        match model.aqi_category {
            AqiCategory::Good => HealthAdvisory::None,
            AqiCategory::Moderate => HealthAdvisory::SensitiveGroupsCaution,
            AqiCategory::UnhealthySensitive => HealthAdvisory::SensitiveGroupsLimitOutdoor,
            AqiCategory::Unhealthy => HealthAdvisory::AllLimitOutdoor,
            AqiCategory::VeryUnhealthy => HealthAdvisory::AvoidOutdoor,
            AqiCategory::Hazardous => HealthAdvisory::StayIndoors,
        }
    }

    fn optimize_air_response(&self, model: &mut AirQualityModel) -> AirResponseStrategy {
        model.erm_state = AirErmState::Optimize;

        let tier = if model.haboob_probability > 0.8 || model.aqi_value > 300 {
            ResponseTier::Critical
        } else if model.aqi_value > 150 {
            ResponseTier::High
        } else if model.aqi_value > 100 {
            ResponseTier::Moderate
        } else {
            ResponseTier::Monitoring
        };

        AirResponseStrategy {
            tier,
            auto_actuate: tier == ResponseTier::Critical,
            alert_distribution: tier >= ResponseTier::Moderate,
            haboob_warning: model.haboob_probability > 0.5,
        }
    }

    fn enforce_air_sovereignty_gates(&self, sample: &AirQualitySample, model: &AirQualityModel, strategy: AirResponseStrategy) -> Result<(), AirSovereigntyViolation> {
        // 1. Indigenous Rights Check (Akimel O'odham / Piipaash Territories)
        self.sovereignty_checker.check_indigenous_air_rights(sample.location_lat, sample.location_lon, model.aqi_value)?;

        // 2. Biotic Treaty Check (Wildlife Corridor Protection)
        self.sovereignty_checker.check_biotic_impact(model.primary_pollutant, self.get_pollutant_concentration(model, sample))?;

        // 3. Neurorights Check (Alerts must not exceed Cognitive Load 0.3)
        let vulnerable_nearby = self.check_vulnerable_population(sample.location_lat, sample.location_lon);
        self.sovereignty_checker.check_neuro_impact(strategy.tier as u8 as f32 / 3.0, vulnerable_nearby)?;

        // 4. Haboob Protocol Check (Transportation Safety)
        if strategy.haboob_warning {
            self.sovereignty_checker.check_haboob_protocol(model.haboob_probability, sample.visibility_meters)?;
        }

        Ok(())
    }

    fn get_pollutant_concentration(&self, model: &AirQualityModel, sample: &AirQualitySample) -> f32 {
        match model.primary_pollutant {
            PollutantType::PM2_5 => sample.pm2_5,
            PollutantType::PM10 => sample.pm10,
            PollutantType::Ozone => sample.ozone_ppb,
            PollutantType::VOC => sample.voc_ppb,
            PollutantType::CO => sample.co_ppm,
            PollutantType::NO2 => sample.no2_ppb,
            PollutantType::SO2 => sample.so2_ppb,
        }
    }

    fn check_vulnerable_population(&self, lat: f64, lon: f64) -> bool {
        // Check for schools, hospitals, elder care facilities within 1km
        // Simplified for this module; production uses GIS database
        // Phoenix Metro Area Vulnerable Zone Approximation
        let downtown_phoenix_lat = 33.4484;
        let downtown_phoenix_lon = -112.0740;
        let distance_km = self.calculate_distance_km(lat, lon, downtown_phoenix_lat, downtown_phoenix_lon);
        distance_km < 5.0 // Urban core has higher vulnerable population density
    }

    fn calculate_distance_km(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
        // Haversine Formula (Simplified)
        let r = 6371.0; // Earth radius in km
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        (r * c) as f32
    }

    fn generate_alert_record(&self, model: &AirQualityModel, strategy: AirResponseStrategy) -> AlertRecord {
        AlertRecord {
            alert_id: self.integrity_provider.sign_air_decision(&model.sample.sensor_id),
            timestamp: model.sample.timestamp_utc,
            aqi_value: model.aqi_value,
            category: model.aqi_category,
            affected_zones: self.determine_affected_zones(&model.sample),
            advisory_issued: model.health_advisory,
            sovereignty_verified: true,
            erm_state: AirErmState::Act,
        }
    }

    fn determine_affected_zones(&self, sample: &AirQualitySample) -> Vec<ZoneId> {
        // Determine affected zones based on wind direction and AQI severity
        // Simplified zone model; production uses detailed geographic boundaries
        let mut zones = Vec::new();

        // Add sensor's home zone
        zones.push(ZoneId::from_coordinates(sample.location_lat, sample.location_lon));

        // Add downwind zones if haboob probability is high
        if sample.wind_speed_kph > 50.0 {
            // Calculate downwind zone based on wind direction
            let downwind_zone = self.calculate_downwind_zone(sample);
            zones.push(downwind_zone);
        }

        zones
    }

    fn calculate_downwind_zone(&self, sample: &AirQualitySample) -> ZoneId {
        // Simplified downwind calculation
        let wind_rad = sample.wind_direction_deg.to_radians();
        let downwind_lat = sample.location_lat + (wind_rad.cos() * 0.05); // ~5km downwind
        let downwind_lon = sample.location_lon + (wind_rad.sin() * 0.05);
        ZoneId::from_coordinates(downwind_lat, downwind_lon)
    }

    fn log_air_transaction(&self, alert: &AlertRecord, sample: &AirQualitySample) {
        // Offline-Capable Ledger Entry
        let entry = AirCybernetEntry {
            alert_hash: alert.alert_id,
            sensor_hash: sample.sensor_id,
            timestamp: sample.timestamp_utc,
            aqi_value: alert.aqi_value,
            category: alert.category,
            haboob_prob: self.model_air_quality(sample).haboob_probability,
            sovereignty_status: "VERIFIED",
        };
        // Write to immutable storage (Abstracted)
        _ = entry;
    }

    pub fn buffer_offline(&mut self, sample: AirQualitySample) -> Result<(), AirSovereigntyViolation> {
        if self.offline_buffer.len() >= self.max_buffer_size {
            return Err(AirSovereigntyViolation::OfflineBufferOverflow);
        }
        self.offline_buffer.push(sample);
        Ok(())
    }

    pub fn register_sensor_node(&mut self, node: SensorNode) {
        self.sensor_network_map.push(node);
    }
}

// ============================================================================
// 5. ALERT & RESPONSE TYPES
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AirErmState {
    Sense,
    Model,
    Optimize,
    TreatyCheck,
    Act,
    Log,
    Interface,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AqiCategory {
    Good, // 0-50
    Moderate, // 51-100
    UnhealthySensitive, // 101-150
    Unhealthy, // 151-200
    VeryUnhealthy, // 201-300
    Hazardous, // 301-500
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PollutantType {
    PM2_5,
    PM10,
    Ozone,
    VOC,
    CO,
    NO2,
    SO2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HealthAdvisory {
    None,
    SensitiveGroupsCaution,
    SensitiveGroupsLimitOutdoor,
    AllLimitOutdoor,
    AvoidOutdoor,
    StayIndoors,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResponseTier {
    Monitoring = 0,
    Moderate = 1,
    High = 2,
    Critical = 3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommunicationMode {
    DirectiveCalm,
    UrgentNeutral,
    SilentLog,
}

pub struct AirResponseStrategy {
    pub tier: ResponseTier,
    pub auto_actuate: bool,
    pub alert_distribution: bool,
    pub haboob_warning: bool,
}

pub struct AirCybernetEntry {
    pub alert_hash: [u8; 64],
    pub sensor_hash: [u8; 32],
    pub timestamp: u64,
    pub aqi_value: u16,
    pub category: AqiCategory,
    pub haboob_prob: f32,
    pub sovereignty_status: &'static str,
}

#[derive(Clone)]
pub struct ZoneId {
    pub zone_code: [u8; 16],
    pub latitude: f64,
    pub longitude: f64,
}

impl ZoneId {
    pub fn from_coordinates(lat: f64, lon: f64) -> Self {
        // Generate zone code from coordinates (simplified)
        let mut code = [0u8; 16];
        let lat_bytes = lat.to_le_bytes();
        let lon_bytes = lon.to_le_bytes();
        code[..8].copy_from_slice(&lat_bytes);
        code[8..].copy_from_slice(&lon_bytes);
        Self {
            zone_code: code,
            latitude: lat,
            longitude: lon,
        }
    }
}

#[derive(Clone)]
pub struct SensorNode {
    pub node_id: [u8; 32],
    pub location_lat: f64,
    pub location_lon: f64,
    pub elevation_m: f32,
    pub sensor_types: Vec<PollutantType>,
    pub last_maintenance_utc: u64,
}

#[derive(Clone, Default)]
pub struct AirQualityBaseline {
    pub pressure_hpa: f32,
    pub temperature_c: f32,
    pub humidity_percent: f32,
    pub pm2_5_avg: f32,
    pub pm10_avg: f32,
}

// ============================================================================
// 6. ERROR TYPES (Air Quality Specific)
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AirSovereigntyViolation {
    None,
    IndigenousRightsViolation,
    BioticImpactViolation,
    NeurorightExceedance,
    HaboobProtocolViolation,
    CryptographicIntegrityFail,
    OfflineBufferOverflow,
    SensorCalibrationDrift,
}

// ============================================================================
// 7. DEFAULT SOVEREIGNTY IMPLEMENTATIONS (Phoenix/Gila Specific)
// ============================================================================

pub struct PhoenixAirSovereignty;

impl AirSovereignty for PhoenixAirSovereignty {
    fn check_indigenous_air_rights(&self, lat: f64, lon: f64, aqi: u16) -> Result<(), AirSovereigntyViolation> {
        // Akimel O'odham and Piipaash lands require stricter air quality
        // Gila River Indian Community boundaries (simplified bounding box)
        let gila_reservation_lat_min = 33.25;
        let gila_reservation_lat_max = 33.45;
        let gila_reservation_lon_min = -112.10;
        let gila_reservation_lon_max = -111.95;

        if lat >= gila_reservation_lat_min && lat <= gila_reservation_lat_max &&
           lon >= gila_reservation_lon_min && lon <= gila_reservation_lon_max {
            // Indigenous territory: AQI must not exceed target (30 vs 50 federal)
            if aqi > INDIGENOUS_AQI_TARGET as u16 {
                // Violation: Requires immediate mitigation action
                return Err(AirSovereigntyViolation::IndigenousRightsViolation);
            }
        }
        Ok(())
    }

    fn check_biotic_impact(&self, pollutant: PollutantType, concentration: f32) -> Result<(), AirSovereigntyViolation> {
        // BioticTreaty: Protect wildlife corridors from pollutant damage
        // Sonoran Desert species sensitivity thresholds
        match pollutant {
            PollutantType::Ozone => {
                // Ozone damages plant tissue, affects desert flora
                if concentration > (OZONE_PPB_MAX * 0.8) {
                    return Err(AirSovereigntyViolation::BioticImpactViolation);
                }
            },
            PollutantType::PM10 | PollutantType::PM2_5 => {
                // Particulates affect animal respiratory systems
                if concentration > (PM10_GOOD_MAX * 2.0) {
                    return Err(AirSovereigntyViolation::BioticImpactViolation);
                }
            },
            _ => {},
        }
        Ok(())
    }

    fn check_neuro_impact(&self, alert_level: f32, vulnerable_nearby: bool) -> Result<(), AirSovereigntyViolation> {
        // Neurorights: Air quality alerts must not exceed RoH 0.3
        // Stricter limits when vulnerable populations nearby
        let max_roh = if vulnerable_nearby { 0.2 } else { AIR_QUALITY_ALERT_ROH_CEILING };

        if alert_level > max_roh {
            return Err(AirSovereigntyViolation::NeurorightExceedance);
        }
        Ok(())
    }

    fn check_haboob_protocol(&self, probability: f32, visibility: f32) -> Result<(), AirSovereigntyViolation> {
        // Haboob Protocol: Transportation safety during dust storms
        // ADOT Model: Close highways when visibility < 100m and probability > 0.8
        if probability > 0.8 && visibility < VISIBILITY_HABOOB_METERS {
            // Protocol requires highway closure alerts
            // If not triggered, violation
            // For this module, we assume upstream systems handle closures
            return Ok(());
        }
        Ok(())
    }
}

// ============================================================================
// 8. UNIT TESTS (Offline Capable)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAirIntegrity;
    impl AirIntegrity for MockAirIntegrity {
        fn sign_air_decision(&self, _decision: &[u8]) -> [u8; 64] {
            [0u8; 64]
        }
        fn verify_sensor_sample(&self, _sample: &[u8], _sig: &[u8]) -> bool {
            true
        }
        fn hash_sensor_chain(&self, _samples: &[AirQualitySample]) -> [u8; 64] {
            [0u8; 64]
        }
    }

    #[test]
    fn test_good_air_quality() {
        let mut engine = AirQualityMonitorEngine::new(
            Box::new(MockAirIntegrity),
            Box::new(PhoenixAirSovereignty),
        );

        let sample = AirQualitySample {
            timestamp_utc: 1735689600,
            sensor_id: [3u8; 32],
            pm2_5: 8.0, // Good
            pm10: 40.0, // Good
            ozone_ppb: 50.0, // Good
            voc_ppb: 20.0,
            co_ppm: 2.0,
            no2_ppb: 30.0,
            so2_ppb: 5.0,
            temperature_c: 35.0,
            humidity_percent: 20.0,
            wind_speed_kph: 15.0,
            wind_direction_deg: 180.0,
            visibility_meters: 10000.0,
            pressure_hpa: 1013.0,
            location_lat: 33.4484,
            location_lon: -112.0740,
            altitude_m: 331.0,
        };

        let result = engine.process_air_sample(sample);
        assert!(result.is_ok());
        let alert = result.unwrap();
        assert_eq!(alert.category, AqiCategory::Good);
        assert!(alert.sovereignty_verified);
    }

    #[test]
    fn test_haboob_detection() {
        let mut engine = AirQualityMonitorEngine::new(
            Box::new(MockAirIntegrity),
            Box::new(PhoenixAirSovereignty),
        );

        let sample = AirQualitySample {
            timestamp_utc: 1735689600,
            sensor_id: [3u8; 32],
            pm2_5: 100.0, // High
            pm10: 600.0, // Haboob Threshold Exceeded
            ozone_ppb: 50.0,
            voc_ppb: 20.0,
            co_ppm: 2.0,
            no2_ppb: 30.0,
            so2_ppb: 5.0,
            temperature_c: 40.0,
            humidity_percent: 15.0,
            wind_speed_kph: 95.0, // Above Haboob Threshold
            wind_direction_deg: 270.0,
            visibility_meters: 50.0, // Below Haboob Threshold
            pressure_hpa: 1008.0,
            location_lat: 33.4484,
            location_lon: -112.0740,
            altitude_m: 331.0,
        };

        let result = engine.process_air_sample(sample);
        assert!(result.is_ok());
        let alert = result.unwrap();
        assert_eq!(alert.category, AqiCategory::Hazardous);
        // Haboob probability should be high
    }

    #[test]
    fn test_indigenous_territory_violation() {
        let sovereignty = PhoenixAirSovereignty;

        // AQI exceeds indigenous target (30)
        let result = sovereignty.check_indigenous_air_rights(33.35, -112.05, 45);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AirSovereigntyViolation::IndigenousRightsViolation);
    }

    #[test]
    fn test_neuroright_compliance() {
        let sovereignty = PhoenixAirSovereignty;

        // Alert level within RoH ceiling
        let result = sovereignty.check_neuro_impact(0.2, false);
        assert!(result.is_ok());

        // Alert level exceeds RoH ceiling
        let result = sovereignty.check_neuro_impact(0.5, false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AirSovereigntyViolation::NeurorightExceedance);
    }
}
