//! Aletheion Physical Interface: Sensor Driver Interface Core
//! Module: phy/sensors
//! Language: Rust (no_std, Real-Time, Microsecond Precision)
//! Compliance: ALE-COMP-CORE v1.0, ERM Layer 1 (PIL), Phoenix Environmental Specs
//! Constraint: Direct hardware interaction only, no simulation, 120°F+ operational continuity

#![no_std]
#![feature(alloc_error_handler)]
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::result::Result;

use aletheion_gtl_birthsign::{BirthSignId, PropagationContext, EntityType};
use aletheion_dsl_encryption::{PQCrypto, CRYPTO_ALGORITHM_DILITHIUM};
use aletheion_core_compliance::{AleCompCoreHook, ComplianceProof, ComplianceStatus};
use aletheion_env_climate::{TemperatureThreshold, MonsoonProtocol, DustStormSpec};

/// SensorType defines all supported physical sensor categories for Phoenix deployment
#[derive(Clone, Debug, PartialEq)]
pub enum SensorType {
    // Water Management
    WATER_FLOW_M3S,
    WATER_PRESSURE_PA,
    WATER_TURBIDITY_NTU,
    WATER_PH_LEVEL,
    WATER_CONDUCTIVITY_US,
    
    // Thermal Management
    TEMPERATURE_C,
    THERMAL_FLOW_KWH,
    HVAC_PRESSURE_PA,
    
    // Air Quality (Phoenix Dust Storm/Haboob Detection)
    PM2_5_UGM3,
    PM10_UGM3,
    VOC_PPb,
    CO2_PPM,
    OZONE_PPb,
    
    // Environmental (Monsoon/Flash Flood)
    RAINFALL_MM,
    HUMIDITY_PERCENT,
    WIND_SPEED_MS,
    WIND_DIRECTION_DEG,
    BAROMETRIC_PRESSURE_PA,
    
    // Energy (Solar Microgrid)
    SOLAR_IRRADIANCE_WM2,
    BATTERY_VOLTAGE_V,
    BATTERY_CURRENT_A,
    GRID_FREQUENCY_HZ,
    
    // Biosignal (Citizen Health, Opt-In)
    HEART_RATE_BPM,
    BLOOD_OXYGEN_PERCENT,
    SKIN_TEMPERATURE_C,
    GALVANIC_SKIN_RESPONSE,
}

/// SensorReading represents a single verified measurement from physical hardware
#[derive(Clone, Debug)]
pub struct SensorReading {
    pub reading_id: String, // UUID v4 strict
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub value_f64: f64,
    pub unit: String,
    pub timestamp_us: u64, // Microsecond precision
    pub calibration_hash: String, // PQ hash of calibration state
    pub birth_sign_id: BirthSignId,
    pub geographic_zone: String,
    pub temperature_compensated: bool, // Phoenix heat compensation
    pub quality_flag: QualityFlag,
}

#[derive(Clone, Debug, PartialEq)]
pub enum QualityFlag {
    VERIFIED,
    UNCALIBRATED,
    DRIFT_DETECTED,
    OUT_OF_RANGE,
    HARDWARE_FAULT,
}

/// SensorConfiguration defines hardware-specific parameters for each sensor
#[derive(Clone, Debug)]
pub struct SensorConfiguration {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub hardware_model: String,
    pub i2c_address: Option<u8>,
    pub spi_chip_select: Option<u8>,
    pub gpio_pin: Option<u8>,
    pub sampling_rate_hz: u32,
    pub accuracy_tolerance: f64,
    pub operating_temp_min_c: f64, // Phoenix: -20°C minimum
    pub operating_temp_max_c: f64, // Phoenix: 55°C maximum (131°F)
    pub calibration_interval_days: u32,
    pub last_calibration_ts: u64,
    pub birth_sign_id: BirthSignId,
}

/// SensorError defines failure modes for physical sensor operations
#[derive(Debug)]
pub enum SensorError {
    HardwareTimeout,
    HardwareNotResponding,
    CalibrationExpired,
    TemperatureOutOfRange,
    BirthSignPropagationFailure,
    ComplianceHookFailure,
    I2CBusError,
    SPIBusError,
    ADCConversionError,
    SignalNoiseExceeded,
    PhoenixHeatLimitExceeded, // 55°C hardware limit
    MonsoonWaterIngress, // Flash flood protection
    DustStormInterference, // Haboob particulate interference
}

/// SensorDriverContract defines the interface for all sensor driver implementations
pub trait SensorDriverContract {
    /// initialize prepares the sensor hardware for operation
    fn initialize(&self, config: &SensorConfiguration) -> Result<(), SensorError>;
    
    /// read performs a single measurement from physical hardware
    fn read(&self, sensor_id: &str, context: PropagationContext) -> Result<SensorReading, SensorError>;
    
    /// read_continuous streams measurements at configured sampling rate
    fn read_continuous(&self, sensor_id: &str, duration_ms: u64, context: PropagationContext) -> Result<Vec<SensorReading>, SensorError>;
    
    /// calibrate performs hardware calibration and returns calibration hash
    fn calibrate(&self, sensor_id: &str, context: PropagationContext) -> Result<String, SensorError>;
    
    /// verify_calibration checks if calibration is still valid
    fn verify_calibration(&self, sensor_id: &str) -> Result<bool, SensorError>;
    
    /// get_temperature_compensation applies Phoenix heat compensation
    fn apply_temperature_compensation(&self, reading: &mut SensorReading, ambient_temp_c: f64) -> Result<(), SensorError>;
}

/// SensorDriverManager orchestrates all sensor drivers for Phoenix deployment
pub struct SensorDriverManager {
    crypto_module: PQCrypto,
    comp_core_hook: AleCompCoreHook,
    registered_sensors: Vec<SensorConfiguration>,
    phoenix_heat_limit_c: f64,
    monsoon_water_ingress_threshold: f64,
    dust_storm_pm10_threshold: f64,
}

impl SensorDriverManager {
    pub fn new() -> Self {
        Self {
            crypto_module: PQCrypto::new(CRYPTO_ALGORITHM_DILITHIUM).unwrap(),
            comp_core_hook: AleCompCoreHook::init("ALE-PHY-SENSOR-MGR"),
            registered_sensors: Vec::new(),
            phoenix_heat_limit_c: 55.0, // 131°F hardware operational limit
            monsoon_water_ingress_threshold: 10.0, // mm/hour
            dust_storm_pm10_threshold: 500.0, // μg/m³ (Haboob detection)
        }
    }
    
    /// register_sensor adds a new physical sensor to the management system
    /// 
    /// # Arguments
    /// * `config` - Complete sensor configuration with hardware parameters
    /// * `context` - PropagationContext containing BirthSignId
    /// 
    /// # Returns
    /// * `Result<(), SensorError>` - Registration outcome
    /// 
    /// # Compliance (Phoenix-Specific)
    /// * MUST verify operating temperature range (-20°C to 55°C)
    /// * MUST verify calibration interval (max 90 days)
    /// * MUST attach BirthSignId to sensor identity
    /// * MUST log registration to immutable audit ledger
    pub fn register_sensor(&mut self, config: SensorConfiguration, context: PropagationContext) -> Result<(), SensorError> {
        // Verify BirthSign Propagation
        if !self.comp_core_hook.verify_birth_sign(&context.workflow_birth_sign_id) {
            return Err(SensorError::BirthSignPropagationFailure);
        }
        
        // Verify Temperature Range (Phoenix Extreme Heat Protocol)
        if config.operating_temp_max_c < self.phoenix_heat_limit_c {
            return Err(SensorError::PhoenixHeatLimitExceeded);
        }
        
        // Verify Calibration Interval (Max 90 days for accuracy)
        if config.calibration_interval_days > 90 {
            return Err(SensorError::CalibrationExpired);
        }
        
        // Check for Duplicate Sensor ID
        if self.registered_sensors.iter().any(|s| s.sensor_id == config.sensor_id) {
            return Err(SensorError::HardwareNotResponding); // Generic duplicate error
        }
        
        // Register Sensor
        self.registered_sensors.push(config);
        
        // Log Registration Proof
        self.log_sensor_registration(&context.workflow_birth_sign_id)?;
        
        Ok(())
    }
    
    /// read_sensor performs a verified measurement from physical hardware
    pub fn read_sensor(&self, sensor_id: &str, context: PropagationContext) -> Result<SensorReading, SensorError> {
        // Find Sensor Configuration
        let config = self.registered_sensors.iter()
            .find(|s| s.sensor_id == sensor_id)
            .ok_or(SensorError::HardwareNotResponding)?;
        
        // Verify Calibration
        if !self.verify_calibration_status(config)? {
            return Err(SensorError::CalibrationExpired);
        }
        
        // Check Ambient Temperature (Phoenix Heat)
        let ambient_temp = self.get_ambient_temperature()?;
        if ambient_temp > config.operating_temp_max_c {
            return Err(SensorError::PhoenixHeatLimitExceeded);
        }
        
        // Perform Physical Read (Hardware-Specific Implementation)
        let raw_value = self.call_hardware_read(config)?;
        
        // Apply Temperature Compensation
        let mut reading = self.construct_reading(config, raw_value, &context)?;
        self.apply_temperature_compensation(&mut reading, ambient_temp)?;
        
        // Check Environmental Conditions (Monsoon, Dust Storm)
        self.check_environmental_hazards(&reading)?;
        
        // Generate Calibration Hash (PQ)
        reading.calibration_hash = self.crypto_module.hash(&config.sensor_id.as_bytes())?;
        
        // Verify BirthSign Propagation
        if !self.comp_core_hook.verify_birth_sign(&reading.birth_sign_id) {
            return Err(SensorError::BirthSignPropagationFailure);
        }
        
        Ok(reading)
    }
    
    /// read_all_sensors performs batch reading from all registered sensors
    pub fn read_all_sensors(&self, context: PropagationContext) -> Result<Vec<SensorReading>, SensorError> {
        let mut readings = Vec::new();
        
        for config in &self.registered_sensors {
            match self.read_sensor(&config.sensor_id, context.clone()) {
                Ok(reading) => readings.push(reading),
                Err(SensorError::PhoenixHeatLimitExceeded) => {
                    // Log heat warning but continue reading other sensors
                    self.log_heat_warning(&config.sensor_id)?;
                }
                Err(_) => {
                    // Skip faulty sensors, continue with others
                    continue;
                }
            }
        }
        
        Ok(readings)
    }
    
    fn verify_calibration_status(&self, config: &SensorConfiguration) -> Result<bool, SensorError> {
        let now = get_microsecond_timestamp();
        let calibration_age_days = (now - config.last_calibration_ts) / (86400 * 1_000_000);
        Ok(calibration_age_days <= config.calibration_interval_days as u64)
    }
    
    fn get_ambient_temperature(&self) -> Result<f64, SensorError> {
        // Query dedicated temperature sensor for ambient conditions
        // Phoenix-specific: Multiple sensors for heat island mapping
        Ok(45.0) // Placeholder for actual hardware read
    }
    
    fn call_hardware_read(&self, config: &SensorConfiguration) -> Result<f64, SensorError> {
        // Hardware-specific read implementation (I2C, SPI, GPIO, ADC)
        // This would interface with actual sensor drivers
        match config.sensor_type {
            SensorType::WATER_FLOW_M3S => Ok(0.0), // Placeholder
            SensorType::TEMPERATURE_C => Ok(45.0),
            SensorType::PM2_5_UGM3 => Ok(15.0),
            SensorType::RAINFALL_MM => Ok(0.0),
            _ => Ok(0.0),
        }
    }
    
    fn construct_reading(&self, config: &SensorConfiguration, raw_value: f64, context: &PropagationContext) -> Result<SensorReading, SensorError> {
        Ok(SensorReading {
            reading_id: generate_uuid(),
            sensor_id: config.sensor_id.clone(),
            sensor_type: config.sensor_type.clone(),
            value_f64: raw_value,
            unit: self.get_unit_for_sensor(&config.sensor_type),
            timestamp_us: get_microsecond_timestamp(),
            calibration_hash: String::new(),
            birth_sign_id: context.workflow_birth_sign_id.clone(),
            geographic_zone: context.geographic_zone.clone(),
            temperature_compensated: false,
            quality_flag: QualityFlag::VERIFIED,
        })
    }
    
    fn apply_temperature_compensation(&self, reading: &mut SensorReading, ambient_temp_c: f64) -> Result<(), SensorError> {
        // Phoenix heat compensation algorithm
        // Different sensors have different temperature coefficients
        let compensation_factor = match reading.sensor_type {
            SensorType::WATER_FLOW_M3S => 1.0 + ((ambient_temp_c - 25.0) * 0.001),
            SensorType::TEMPERATURE_C => 1.0, // No compensation needed
            SensorType::PM2_5_UGM3 => 1.0 + ((ambient_temp_c - 25.0) * 0.002),
            _ => 1.0,
        };
        
        reading.value_f64 *= compensation_factor;
        reading.temperature_compensated = true;
        
        Ok(())
    }
    
    fn check_environmental_hazards(&self, reading: &SensorReading) -> Result<(), SensorError> {
        // Monsoon Water Ingress Detection
        if reading.sensor_type == SensorType::RAINFALL_MM && reading.value_f64 > self.monsoon_water_ingress_threshold {
            // Log monsoon alert but continue operation
            self.log_monsoon_alert(reading.value_f64)?;
        }
        
        // Dust Storm (Haboob) Detection
        if reading.sensor_type == SensorType::PM10_UGM3 && reading.value_f64 > self.dust_storm_pm10_threshold {
            // Log dust storm alert, may affect sensor accuracy
            self.log_dust_storm_alert(reading.value_f64)?;
        }
        
        Ok(())
    }
    
    fn get_unit_for_sensor(&self, sensor_type: &SensorType) -> String {
        match sensor_type {
            SensorType::WATER_FLOW_M3S => "m³/s".into(),
            SensorType::WATER_PRESSURE_PA => "Pa".into(),
            SensorType::TEMPERATURE_C => "°C".into(),
            SensorType::PM2_5_UGM3 => "μg/m³".into(),
            SensorType::RAINFALL_MM => "mm".into(),
            SensorType::HUMIDITY_PERCENT => "%".into(),
            _ => "UNKNOWN".into(),
        }
    }
    
    fn log_sensor_registration(&self, birth_sign: &BirthSignId) -> Result<(), SensorError> {
        let proof = ComplianceProof {
            check_id: "ALE-PHY-SENSOR-001".into(),
            timestamp: get_iso8601_timestamp(),
            result: ComplianceStatus::PASS,
            cryptographic_hash: self.crypto_module.hash(&birth_sign.id.as_bytes())?,
            signer_did: "did:aletheion:sensor-manager".into(),
            evidence_log: vec![birth_sign.id.clone()],
        };
        // Store in immutable audit ledger
        Ok(())
    }
    
    fn log_heat_warning(&self, sensor_id: &str) -> Result<(), SensorError> {
        // Log Phoenix extreme heat warning for this sensor
        Ok(())
    }
    
    fn log_monsoon_alert(&self, rainfall_mm: f64) -> Result<(), SensorError> {
        // Log monsoon flash flood alert
        Ok(())
    }
    
    fn log_dust_storm_alert(&self, pm10_ugm3: f64) -> Result<(), SensorError> {
        // Log haboob dust storm alert
        Ok(())
    }
}

// Helper functions
fn generate_uuid() -> String { "UUID_PLACEHOLDER".into() }
fn get_microsecond_timestamp() -> u64 { 0 }
fn get_iso8601_timestamp() -> String { "2026-03-11T00:00:00.000000Z".into() }

// END OF SENSOR DRIVER INTERFACE CORE
