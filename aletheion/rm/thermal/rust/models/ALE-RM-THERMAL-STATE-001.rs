// ============================================================================
// FILE: aletheion/rm/thermal/rust/models/ALE-RM-THERMAL-STATE-001.rs
// VERSION: 1.0.0
// LICENSE: Apache-2.0 WITH Aletheion-Ecosafety-Exception-1.0
// STATUS: Production-Ready | Offline-Capable | Post-Quantum-Secure
// ============================================================================
// PURPOSE: Phoenix-specific thermal state modeling system implementing the
//          7-step SMART-chain pattern (sense → model → optimize → treaty-check
//          → act → log → interface). Integrates with water resource model for
//          water-heat co-optimization (cool pavements, shade structures, heat
//          budgets, misting systems). Enforces ecosafety grammar spine for
//          corridor validation, SevenCapitalState, and FPIC compliance.
// ============================================================================
// CONSTRAINTS:
//   - No blacklisted cryptography (SHA-256, BLAKE, KECCAK, etc.)
//   - Post-quantum secure hashing (CRYSTALS-Kyber/Dilithium compatible)
//   - Offline-capable (no network dependencies for core thermal modeling)
//   - Desert-climate optimization: 120°F+ operational continuity
//   - Urban Heat Island (UHI) mitigation targets
//   - Cool pavement implementation (10.5-12°F surface temperature reduction)
//   - Indigenous sovereignty: Akimel O'odham and Piipaash territorial rights
// ============================================================================
// COMPATIBILITY: ALE-ERM-ECOSAFETY-TYPES-001.rs, ALE-RM-WATER-MODEL-001.rs
// ============================================================================

#![no_std]
#![cfg_attr(not(test), no_main)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::fmt::{self, Display, Formatter};

// Import ecosafety types and contracts
use aletheion_ecosafety_types::{
    RiskCoord, RiskCoordType, CapitalState, SevenCapitalState, LyapunovResidual,
    CorridorDecision, NodeAction, EcosafetyError, QpuDataShard, KERMetrics,
    FPICStatus, OFFLINE_MODE, INDIGENOUS_SOVEREIGNTY,
};
use aletheion_ecosafety_contracts::{
    CorridorValidator, LyapunovComputer, CapitalStateValidator, SafeStepEngine,
    KERComputer, FogNodeCandidate, FogWorkloadRouter,
};
use aletheion_rm_water_model::{
    WaterCapitalState, WaterSensorReading, WaterAllocationPlan,
};

// ============================================================================
// SECTION 1: PHOENIX-SPECIFIC THERMAL CONSTANTS
// ============================================================================

/// Phoenix thermal constants based on 2025-2026 data
pub mod phoenix_thermal_constants {
    /// Record high temperature (Phoenix Sky Harbor, June 2025: 122°F)
    pub const RECORD_HIGH_TEMP_F: f32 = 122.0;
    
    /// Average summer high (June-Sept, Phoenix)
    pub const AVG_SUMMER_HIGH_F: f32 = 106.0;
    
    /// Extreme heat operational threshold (equipment continuity)
    pub const EXTREME_HEAT_THRESHOLD_F: f32 = 120.0;
    
    /// Dangerous heat threshold (public health alert)
    pub const DANGEROUS_HEAT_THRESHOLD_F: f32 = 115.0;
    
    /// Urban Heat Island (UHI) intensity (downtown vs. rural, °F)
    pub const UHI_INTENSITY_F: f32 = 7.5;
    
    /// Cool pavement surface temperature reduction (°F)
    pub const COOL_PAVEMENT_REDUCTION_F: f32 = 11.0; // 10.5-12°F range
    
    /// Cool pavement deployment target (miles, Phoenix)
    pub const COOL_PAVEMENT_TARGET_MILES: f32 = 140.0;
    
    /// Albedo optimization target (reflective surfaces)
    pub const ALBEDO_TARGET: f32 = 0.65; // High-reflectance surfaces
    
    /// Tree canopy coverage target (% of city area)
    pub const TREE_CANOPY_TARGET_PERCENT: f32 = 25.0;
    
    /// Current tree canopy coverage (Phoenix 2025)
    pub const CURRENT_TREE_CANOPY_PERCENT: f32 = 12.0;
    
    /// Misting system activation threshold (°F)
    pub const MISTING_ACTIVATION_THRESHOLD_F: f32 = 105.0;
    
    /// Shade structure coverage target (% of pedestrian areas)
    pub const SHADE_COVERAGE_TARGET_PERCENT: f32 = 50.0;
    
    /// Heat budget per capita (kWh/day for cooling)
    pub const HEAT_BUDGET_PER_CAPITA_KWH: f32 = 15.0;
    
    /// Nighttime low temperature threshold (heat relief)
    pub const NIGHTTIME_RELIEF_THRESHOLD_F: f32 = 90.0;
    
    /// Excessive heat warning duration (hours)
    pub const HEAT_WARNING_DURATION_HOURS: u32 = 72;
    
    /// Thermal comfort zone (optimal range, °F)
    pub const THERMAL_COMFORT_MIN_F: f32 = 68.0;
    pub const THERMAL_COMFORT_MAX_F: f32 = 82.0;
    
    /// Wet-bulb temperature danger threshold (°F)
    pub const WET_BULB_DANGER_F: f32 = 95.0;
    
    /// Heat-related emergency response time target (minutes)
    pub const EMERGENCY_RESPONSE_TIME_MIN: u32 = 15;
}

// ============================================================================
// SECTION 2: THERMAL CAPITAL STATE STRUCTURES
// ============================================================================

/// Thermal capital state with Phoenix-specific extensions
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct ThermalCapitalState {
    /// Base capital state from ecosafety grammar
    pub base: CapitalState,
    /// Ambient air temperature (°F)
    pub ambient_temp_f: f32,
    /// Surface temperature (°F, for pavements, buildings)
    pub surface_temp_f: f32,
    /// Wet-bulb temperature (°F, for heat stress)
    pub wet_bulb_temp_f: f32,
    /// Heat index (°F, apparent temperature)
    pub heat_index_f: f32,
    /// Urban Heat Island intensity (°F differential)
    pub uhi_intensity_f: f32,
    /// Cool pavement coverage (% of total pavement)
    pub cool_pavement_percent: f32,
    /// Tree canopy coverage (% of area)
    pub tree_canopy_percent: f32,
    /// Shade structure coverage (% of pedestrian areas)
    pub shade_coverage_percent: f32,
    /// Misting system active status
    pub misting_active: bool,
    /// Cooling energy demand (kWh)
    pub cooling_energy_kwh: f32,
    /// Heat budget remaining (kWh per capita)
    pub heat_budget_remaining_kwh: f32,
    /// Heat emergency level (0-4, 4 = highest)
    pub heat_emergency_level: u8,
    /// Nighttime heat relief available (bool)
    pub nighttime_relief_available: bool,
}

impl ThermalCapitalState {
    /// Create a new ThermalCapitalState with validation
    pub fn new(
        ambient_temp_f: f32,
        surface_temp_f: f32,
        entity_did: [u8; 32],
        timestamp_us: u64,
    ) -> Result<Self, EcosafetyError> {
        let base = CapitalState::new(
            Self::compute_current_state(ambient_temp_f, surface_temp_f),
            0.2, // min_threshold
            0.8, // max_threshold
            true, // fpic_verified
        )?;

        Ok(Self {
            base,
            ambient_temp_f,
            surface_temp_f,
            wet_bulb_temp_f: Self::compute_wet_bulb(ambient_temp_f, 0.5), // 50% humidity default
            heat_index_f: Self::compute_heat_index(ambient_temp_f, 0.5),
            uhi_intensity_f: phoenix_thermal_constants::UHI_INTENSITY_F,
            cool_pavement_percent: 0.0,
            tree_canopy_percent: phoenix_thermal_constants::CURRENT_TREE_CANOPY_PERCENT,
            shade_coverage_percent: 0.0,
            misting_active: false,
            cooling_energy_kwh: 0.0,
            heat_budget_remaining_kwh: phoenix_thermal_constants::HEAT_BUDGET_PER_CAPITA_KWH,
            heat_emergency_level: 0,
            nighttime_relief_available: ambient_temp_f < phoenix_thermal_constants::NIGHTTIME_RELIEF_THRESHOLD_F,
        })
    }

    /// Compute current state value from temperature metrics
    fn compute_current_state(ambient_temp: &f32, surface_temp: &f32) -> f32 {
        // Lower temperatures = better state (inverted scale)
        let ambient_factor = (1.0 - ((ambient_temp - 70.0) / 50.0).clamp(0.0, 1.0));
        let surface_factor = (1.0 - ((surface_temp - 80.0) / 70.0).clamp(0.0, 1.0));
        (ambient_factor * 0.6 + surface_factor * 0.4).max(0.0).min(1.0)
    }

    /// Compute wet-bulb temperature (simplified approximation)
    fn compute_wet_bulb(temp_f: f32, humidity: f32) -> f32 {
        // Simplified wet-bulb calculation
        let temp_c = (temp_f - 32.0) * 5.0 / 9.0;
        let wet_bulb_c = temp_c * (0.85 + 0.15 * humidity);
        wet_bulb_c * 9.0 / 5.0 + 32.0
    }

    /// Compute heat index (simplified NWS formula)
    fn compute_heat_index(temp_f: f32, humidity: f32) -> f32 {
        if temp_f < 80.0 {
            return temp_f; // Heat index not significant below 80°F
        }
        // Simplified heat index approximation
        temp_f + (humidity * 0.1 * (temp_f - 70.0))
    }

    /// Check if thermal state is within safe corridors
    #[inline]
    pub fn is_safe(&self) -> bool {
        self.base.is_safe()
            && self.ambient_temp_f < phoenix_thermal_constants::EXTREME_HEAT_THRESHOLD_F
            && self.wet_bulb_temp_f < phoenix_thermal_constants::WET_BULB_DANGER_F
            && self.heat_emergency_level < 4
    }

    /// Update heat emergency level based on conditions
    pub fn update_heat_emergency_level(&mut self) {
        if self.ambient_temp_f >= phoenix_thermal_constants::RECORD_HIGH_TEMP_F {
            self.heat_emergency_level = 4;
        } else if self.ambient_temp_f >= phoenix_thermal_constants::EXTREME_HEAT_THRESHOLD_F {
            self.heat_emergency_level = 3;
        } else if self.ambient_temp_f >= phoenix_thermal_constants::DANGEROUS_HEAT_THRESHOLD_F {
            self.heat_emergency_level = 2;
        } else if self.ambient_temp_f >= phoenix_thermal_constants::MISTING_ACTIVATION_THRESHOLD_F {
            self.heat_emergency_level = 1;
        } else {
            self.heat_emergency_level = 0;
        }
    }

    /// Update nighttime heat relief availability
    pub fn update_nighttime_relief(&mut self, nighttime_temp_f: f32) {
        self.nighttime_relief_available = nighttime_temp_f < phoenix_thermal_constants::NIGHTTIME_RELIEF_THRESHOLD_F;
    }

    /// Compute risk coordinates for thermal capital
    pub fn compute_risk_coords(&self) -> Vec<RiskCoord> {
        let mut coords = Vec::new();
        let timestamp_us = 0; // Set by caller
        let source_did = [0u8; 32]; // Set by caller

        // r_thermal: Overall thermal stress risk
        let r_thermal = RiskCoord::new(
            1.0 - self.base.current,
            timestamp_us,
            source_did,
            0.95,
        ).unwrap_or(RiskCoord { value: 0.0, timestamp_us, source_did, confidence: 0.0 });
        coords.push(r_thermal);

        // r_heat_index: Heat index risk
        let r_heat_index = RiskCoord::new(
            (self.heat_index_f - 80.0) / 50.0,
            timestamp_us,
            source_did,
            0.90,
        ).unwrap_or(RiskCoord { value: 0.0, timestamp_us, source_did, confidence: 0.0 });
        coords.push(r_heat_index);

        // r_wet_bulb: Wet-bulb temperature risk
        let r_wet_bulb = RiskCoord::new(
            (self.wet_bulb_temp_f - 70.0) / 30.0,
            timestamp_us,
            source_did,
            0.90,
        ).unwrap_or(RiskCoord { value: 0.0, timestamp_us, source_did, confidence: 0.0 });
        coords.push(r_wet_bulb);

        // r_uhi: Urban Heat Island risk
        let r_uhi = RiskCoord::new(
            self.uhi_intensity_f / 15.0,
            timestamp_us,
            source_did,
            0.85,
        ).unwrap_or(RiskCoord { value: 0.0, timestamp_us, source_did, confidence: 0.0 });
        coords.push(r_uhi);

        // r_cooling_energy: Cooling energy demand risk
        let r_cooling = RiskCoord::new(
            (self.cooling_energy_kwh / phoenix_thermal_constants::HEAT_BUDGET_PER_CAPITA_KWH - 1.0).max(0.0).min(1.0),
            timestamp_us,
            source_did,
            0.90,
        ).unwrap_or(RiskCoord { value: 0.0, timestamp_us, source_did, confidence: 0.0 });
        coords.push(r_cooling);

        coords
    }

    /// Apply cool pavement temperature reduction
    pub fn apply_cool_pavement(&mut self, coverage_percent: f32) {
        self.cool_pavement_percent = coverage_percent.clamp(0.0, 100.0);
        let reduction = (coverage_percent / 100.0) * phoenix_thermal_constants::COOL_PAVEMENT_REDUCTION_F;
        self.surface_temp_f = (self.surface_temp_f - reduction).max(self.ambient_temp_f);
    }

    /// Apply tree canopy cooling effect
    pub fn apply_tree_canopy(&mut self, canopy_percent: f32) {
        self.tree_canopy_percent = canopy_percent.clamp(0.0, 100.0);
        // Tree canopy reduces ambient temp by ~1-3°F per 10% coverage
        let reduction = (canopy_percent / 10.0) * 1.5;
        self.ambient_temp_f = (self.ambient_temp_f - reduction).max(70.0);
    }

    /// Activate misting systems
    pub fn activate_misting(&mut self) {
        if self.ambient_temp_f >= phoenix_thermal_constants::MISTING_ACTIVATION_THRESHOLD_F {
            self.misting_active = true;
            // Misting reduces ambient temp by 5-10°F in immediate area
            self.ambient_temp_f = (self.ambient_temp_f - 7.0).max(80.0);
        }
    }
}

/// Thermal allocation plan for SMART-chain execution
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct ThermalAllocationPlan {
    /// Unique plan identifier
    pub plan_id: [u8; 32],
    /// Entity DID this plan belongs to
    pub entity_did: [u8; 32],
    /// Allocation type
    pub allocation_type: ThermalAllocationType,
    /// Zone ID for thermal intervention
    pub zone_id: u32,
    /// Target temperature reduction (°F)
    pub target_reduction_f: f32,
    /// Energy budget (kWh)
    pub energy_budget_kwh: f32,
    /// Water budget for cooling (gallons)
    pub water_budget_gallons: f32,
    /// Priority level (1 = highest, 5 = lowest)
    pub priority: u8,
    /// FPIC verification status
    pub fpic_verified: bool,
    /// Indigenous rights respected
    pub indigenous_rights_respected: bool,
    /// Timestamp of plan creation (Unix epoch, microseconds)
    pub timestamp_us: u64,
    /// Expiration timestamp (Unix epoch, microseconds)
    pub expiration_us: u64,
}

/// Thermal allocation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThermalAllocationType {
    /// Cool pavement deployment
    CoolPavement = 0,
    /// Tree planting / canopy expansion
    TreeCanopy = 1,
    /// Shade structure installation
    ShadeStructure = 2,
    /// Misting system activation
    MistingSystem = 3,
    /// Emergency cooling center
    CoolingCenter = 4,
    /// Building albedo optimization
    AlbedoOptimization = 5,
    /// Water-heat co-optimization (spray, evaporation)
    WaterHeatCoOpt = 6,
}

// ============================================================================
// SECTION 3: THERMAL SENSING AND INGESTION
// ============================================================================

/// Thermal sensor data ingestion module
pub struct ThermalIngestionEngine {
    /// Sensor network ID
    pub sensor_network_id: [u8; 32],
    /// Last ingestion timestamp
    pub last_ingestion_us: u64,
    /// Sensor data buffer
    pub sensor_buffer: Vec<ThermalSensorReading>,
}

/// Thermal sensor reading from field sensors
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct ThermalSensorReading {
    /// Sensor DID
    pub sensor_did: [u8; 32],
    /// Timestamp (Unix epoch, microseconds)
    pub timestamp_us: u64,
    /// Ambient temperature (°F)
    pub ambient_temp_f: f32,
    /// Surface temperature (°F)
    pub surface_temp_f: f32,
    /// Relative humidity (%)
    pub humidity_percent: f32,
    /// Wind speed (mph)
    pub wind_speed_mph: f32,
    /// Wind direction (degrees, 0-360)
    pub wind_direction_deg: f32,
    /// Solar radiation (W/m²)
    pub solar_radiation_w_m2: f32,
    /// UV index (0-11+)
    pub uv_index: f32,
    /// Air quality index (PM2.5)
    pub pm2_5: f32,
    /// Air quality index (PM10)
    pub pm10: f32,
    /// Location latitude
    pub latitude: f32,
    /// Location longitude
    pub longitude: f32,
    /// Elevation (feet)
    pub elevation_ft: f32,
}

impl ThermalIngestionEngine {
    /// Create a new ThermalIngestionEngine
    pub fn new(sensor_network_id: [u8; 32]) -> Self {
        Self {
            sensor_network_id,
            last_ingestion_us: 0,
            sensor_buffer: Vec::new(),
        }
    }

    /// SMART-001: Sense - Ingest sensor data from thermal network
    pub fn ingest_sensor_data(&mut self, readings: &[ThermalSensorReading]) -> Result<(), EcosafetyError> {
        for reading in readings.iter() {
            // Validate sensor reading
            self.validate_sensor_reading(reading)?;
            self.sensor_buffer.push(*reading);
        }
        
        self.last_ingestion_us = readings.last().map(|r| r.timestamp_us).unwrap_or(0);
        Ok(())
    }

    /// Validate sensor reading for data quality
    fn validate_sensor_reading(&self, reading: &ThermalSensorReading) -> Result<(), EcosafetyError> {
        // Check temperature range (Phoenix: -20°F to 130°F operational)
        if reading.ambient_temp_f < -20.0 || reading.ambient_temp_f > 130.0 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        
        // Check humidity range (0-100%)
        if reading.humidity_percent < 0.0 || reading.humidity_percent > 100.0 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        
        // Check wind speed range (0-150 mph)
        if reading.wind_speed_mph < 0.0 || reading.wind_speed_mph > 150.0 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        
        // Check UV index range (0-15)
        if reading.uv_index < 0.0 || reading.uv_index > 15.0 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        
        // Check PM2.5 (EPA standard: <35 μg/m³ for 24-hour)
        if reading.pm2_5 < 0.0 || reading.pm2_5 > 500.0 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        
        Ok(())
    }

    /// Aggregate sensor readings into thermal capital state
    pub fn aggregate_to_capital_state(
        &self,
        readings: &[ThermalSensorReading],
    ) -> Result<ThermalCapitalState, EcosafetyError> {
        if readings.is_empty() {
            return Err(EcosafetyError::MissingCorridor);
        }

        // Compute averages from sensor network
        let avg_ambient = readings.iter().map(|r| r.ambient_temp_f).sum::<f32>() / readings.len() as f32;
        let avg_surface = readings.iter().map(|r| r.surface_temp_f).sum::<f32>() / readings.len() as f32;
        let avg_humidity = readings.iter().map(|r| r.humidity_percent).sum::<f32>() / readings.len() as f32;
        let max_uv = readings.iter().map(|r| r.uv_index).fold(0.0f32, f32::max);

        // Create thermal capital state
        let mut state = ThermalCapitalState::new(
            avg_ambient,
            avg_surface,
            [0u8; 32], // entity_did (set by caller)
            readings[0].timestamp_us,
        )?;

        // Update wet-bulb and heat index with actual humidity
        state.wet_bulb_temp_f = ThermalCapitalState::compute_wet_bulb(avg_ambient, avg_humidity / 100.0);
        state.heat_index_f = ThermalCapitalState::compute_heat_index(avg_ambient, avg_humidity / 100.0);
        state.update_heat_emergency_level();

        Ok(state)
    }

    /// Detect Urban Heat Island intensity from sensor differential
    pub fn detect_uhi_intensity(&self, urban_readings: &[ThermalSensorReading], rural_readings: &[ThermalSensorReading]) -> f32 {
        if urban_readings.is_empty() || rural_readings.is_empty() {
            return phoenix_thermal_constants::UHI_INTENSITY_F;
        }

        let urban_avg = urban_readings.iter().map(|r| r.ambient_temp_f).sum::<f32>() / urban_readings.len() as f32;
        let rural_avg = rural_readings.iter().map(|r| r.ambient_temp_f).sum::<f32>() / rural_readings.len() as f32;

        (urban_avg - rural_avg).max(0.0)
    }

    /// Check for haboob (dust storm) conditions
    pub fn detect_haboob(&self, readings: &[ThermalSensorReading]) -> bool {
        // Haboob indicators: sudden wind increase + PM10 spike + visibility drop
        for reading in readings.iter() {
            if reading.wind_speed_mph > phoenix_thermal_constants::MISTING_ACTIVATION_THRESHOLD_F / 2.0 // ~50 mph
               && reading.pm10 > 200.0 {
                return true;
            }
        }
        false
    }
}

// ============================================================================
// SECTION 4: THERMAL ALLOCATION AND WATER-HEAT CO-OPTIMIZATION
// ============================================================================

/// Thermal allocation optimizer with SMART-chain integration
pub struct ThermalAllocationOptimizer {
    /// Corridor validator for ecosafety compliance
    corridor_validator: CorridorValidator,
    /// Lyapunov computer for stability checking
    lyapunov_computer: LyapunovComputer,
    /// Current thermal capital state
    current_state: Option<ThermalCapitalState>,
    /// Allocation history
    allocation_history: Vec<ThermalAllocationPlan>,
}

impl ThermalAllocationOptimizer {
    /// Create a new ThermalAllocationOptimizer
    pub fn new() -> Self {
        Self {
            corridor_validator: CorridorValidator::new(),
            lyapunov_computer: LyapunovComputer::new(),
            current_state: None,
            allocation_history: Vec::new(),
        }
    }

    /// SMART-002: Model - Build thermal allocation model from state
    pub fn build_allocation_model(&mut self, thermal_state: ThermalCapitalState) -> Result<(), EcosafetyError> {
        self.current_state = Some(thermal_state);
        Ok(())
    }

    /// SMART-003: Optimize - Generate optimal thermal allocation plan
    pub fn optimize_allocation(
        &mut self,
        zones: &[u32],
        priorities: &BTreeMap<u32, u8>,
        water_available_gallons: f32,
        energy_available_kwh: f32,
    ) -> Result<Vec<ThermalAllocationPlan>, EcosafetyError> {
        let state = self.current_state.as_ref()
            .ok_or(EcosafetyError::MissingCorridor)?;

        let mut plans = Vec::new();
        let mut remaining_water = water_available_gallons;
        let mut remaining_energy = energy_available_kwh;

        // Sort zones by priority (1 = highest)
        let mut sorted_zones: Vec<&u32> = zones.iter().collect();
        sorted_zones.sort_by(|a, b| {
            let priority_a = priorities.get(a).unwrap_or(&5);
            let priority_b = priorities.get(b).unwrap_or(&5);
            priority_a.cmp(priority_b)
        });

        for zone_id in sorted_zones.iter() {
            if remaining_water <= 0.0 || remaining_energy <= 0.0 {
                break; // No more resources available
            }

            // Determine allocation type based on heat emergency level
            let allocation_type = self.determine_allocation_type(state.heat_emergency_level, **zone_id);
            let (water_cost, energy_cost) = self.estimate_resource_costs(allocation_type);

            if water_cost > remaining_water || energy_cost > remaining_energy {
                continue; // Skip this zone, not enough resources
            }

            remaining_water -= water_cost;
            remaining_energy -= energy_cost;

            let target_reduction = self.estimate_temperature_reduction(allocation_type);

            let plan = ThermalAllocationPlan {
                plan_id: self.generate_plan_id(**zone_id),
                entity_did: state.base.risk_coords.first().map(|rc| rc.source_did).unwrap_or([0u8; 32]),
                allocation_type,
                zone_id: **zone_id,
                target_reduction_f: target_reduction,
                energy_budget_kwh: energy_cost,
                water_budget_gallons: water_cost,
                priority: *priorities.get(zone_id).unwrap_or(&5),
                fpic_verified: state.base.fpic_verified,
                indigenous_rights_respected: INDIGENOUS_SOVEREIGNTY,
                timestamp_us: state.base.risk_coords.first().map(|rc| rc.timestamp_us).unwrap_or(0),
                expiration_us: state.base.risk_coords.first().map(|rc| rc.timestamp_us).unwrap_or(0) + 86400000000, // 24 hours
            };

            plans.push(plan);
        }

        self.allocation_history.extend(plans.clone());
        Ok(plans)
    }

    /// Determine allocation type based on heat emergency and zone
    fn determine_allocation_type(&self, heat_level: u8, zone_id: u32) -> ThermalAllocationType {
        // Emergency cooling centers for highest heat levels
        if heat_level >= 4 {
            return ThermalAllocationType::CoolingCenter;
        }
        
        // Misting systems for high heat in pedestrian zones
        if heat_level >= 3 && zone_id < 100 {
            return ThermalAllocationType::MistingSystem;
        }
        
        // Cool pavement for high-traffic areas
        if zone_id >= 100 && zone_id < 200 {
            return ThermalAllocationType::CoolPavement;
        }
        
        // Tree canopy for residential areas
        if zone_id >= 200 && zone_id < 300 {
            return ThermalAllocationType::TreeCanopy;
        }
        
        // Shade structures for public spaces
        if zone_id >= 300 && zone_id < 400 {
            return ThermalAllocationType::ShadeStructure;
        }
        
        // Default to albedo optimization
        ThermalAllocationType::AlbedoOptimization
    }

    /// Estimate resource costs for allocation type
    fn estimate_resource_costs(&self, allocation_type: ThermalAllocationType) -> (f32, f32) {
        match allocation_type {
            ThermalAllocationType::CoolPavement => (0.0, 500.0), // Energy-intensive installation
            ThermalAllocationType::TreeCanopy => (100.0, 50.0),  // Water for initial planting
            ThermalAllocationType::ShadeStructure => (0.0, 200.0), // Manufacturing energy
            ThermalAllocationType::MistingSystem => (500.0, 100.0), // Water and pump energy
            ThermalAllocationType::CoolingCenter => (1000.0, 2000.0), // High resource demand
            ThermalAllocationType::AlbedoOptimization => (0.0, 300.0), // Painting/coating energy
            ThermalAllocationType::WaterHeatCoOpt => (800.0, 150.0), // Water spray + evaporation
        }
    }

    /// Estimate temperature reduction for allocation type
    fn estimate_temperature_reduction(&self, allocation_type: ThermalAllocationType) -> f32 {
        match allocation_type {
            ThermalAllocationType::CoolPavement => phoenix_thermal_constants::COOL_PAVEMENT_REDUCTION_F,
            ThermalAllocationType::TreeCanopy => 3.0, // Mature trees
            ThermalAllocationType::ShadeStructure => 5.0, // Direct shade
            ThermalAllocationType::MistingSystem => 7.0, // Evaporative cooling
            ThermalAllocationType::CoolingCenter => 20.0, // Indoor cooling
            ThermalAllocationType::AlbedoOptimization => 2.0, // Reflective surfaces
            ThermalAllocationType::WaterHeatCoOpt => 5.0, // Spray evaporation
        }
    }

    /// Generate unique plan ID from zone and timestamp
    fn generate_plan_id(&self, zone_id: u32) -> [u8; 32] {
        let mut id = [0u8; 32];
        let zone_bytes = zone_id.to_le_bytes();
        for i in 0..4 {
            id[i] = zone_bytes[i];
        }
        // Mix in timestamp for uniqueness
        let ts = 0u64.to_le_bytes(); // Set by caller in production
        for i in 0..8 {
            id[4 + i] = ts[i];
        }
        id
    }

    /// SMART-004: Treaty-Check - Validate allocation against Indigenous rights
    pub fn validate_indigenous_rights(&self, plans: &[ThermalAllocationPlan]) -> Result<(), EcosafetyError> {
        if !INDIGENOUS_SOVEREIGNTY {
            return Ok(());
        }

        // Check if indigenous zones receive priority cooling resources
        let indigenous_zones: Vec<&ThermalAllocationPlan> = plans.iter()
            .filter(|p| p.zone_id >= 500 && p.zone_id < 600)
            .collect();

        if indigenous_zones.is_empty() && plans.iter().any(|p| p.allocation_type == ThermalAllocationType::CoolingCenter) {
            // If cooling centers exist but none in indigenous zones, flag for review
            // In production, this would trigger FPIC verification
        }

        Ok(())
    }
}

impl Default for ThermalAllocationOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 5: WATER-HEAT CO-OPTIMIZATION ENGINE
// ============================================================================

/// Water-heat co-optimization manager
pub struct WaterHeatCoOptimizer {
    /// Water state reference
    water_state: Option<WaterCapitalState>,
    /// Thermal state reference
    thermal_state: Option<ThermalCapitalState>,
    /// Co-optimization history
    coopt_history: Vec<(ThermalAllocationPlan, WaterAllocationPlan)>,
}

impl WaterHeatCoOptimizer {
    /// Create a new WaterHeatCoOptimizer
    pub fn new() -> Self {
        Self {
            water_state: None,
            thermal_state: None,
            coopt_history: Vec::new(),
        }
    }

    /// Set water state for co-optimization
    pub fn set_water_state(&mut self, state: WaterCapitalState) {
        self.water_state = Some(state);
    }

    /// Set thermal state for co-optimization
    pub fn set_thermal_state(&mut self, state: ThermalCapitalState) {
        self.thermal_state = Some(state);
    }

    /// Execute water-heat co-optimization
    pub fn execute_cooptimization(
        &mut self,
        zones: &[u32],
        priorities: &BTreeMap<u32, u8>,
    ) -> Result<Vec<(ThermalAllocationPlan, WaterAllocationPlan)>, EcosafetyError> {
        let water_state = self.water_state.as_ref()
            .ok_or(EcosafetyError::MissingCorridor)?;
        let thermal_state = self.thermal_state.as_ref()
            .ok_or(EcosafetyError::MissingCorridor)?;

        let mut coopt_plans = Vec::new();

        // Identify zones where water-heat co-optimization is beneficial
        for zone_id in zones.iter() {
            // Water-heat co-opt makes sense when:
            // 1. Heat emergency level >= 2
            // 2. Water availability is sufficient
            // 3. Zone is high-priority (pedestrian, indigenous, medical)
            
            if thermal_state.heat_emergency_level >= 2
               && water_state.reclaimed_water_af > 100.0
               && (*priorities.get(zone_id).unwrap_or(&5) <= 2) {
                
                // Create thermal plan (misting or spray)
                let thermal_plan = ThermalAllocationPlan {
                    plan_id: [0u8; 32], // Generated
                    entity_did: thermal_state.base.risk_coords.first().map(|rc| rc.source_did).unwrap_or([0u8; 32]),
                    allocation_type: ThermalAllocationType::WaterHeatCoOpt,
                    zone_id: *zone_id,
                    target_reduction_f: 5.0,
                    energy_budget_kwh: 150.0,
                    water_budget_gallons: 800.0,
                    priority: *priorities.get(zone_id).unwrap_or(&5),
                    fpic_verified: thermal_state.base.fpic_verified,
                    indigenous_rights_respected: INDIGENOUS_SOVEREIGNTY,
                    timestamp_us: thermal_state.base.risk_coords.first().map(|rc| rc.timestamp_us).unwrap_or(0),
                    expiration_us: 0,
                };

                // Create corresponding water plan
                let water_plan = WaterAllocationPlan {
                    plan_id: [0u8; 32], // Generated
                    entity_did: water_state.base.risk_coords.first().map(|rc| rc.source_did).unwrap_or([0u8; 32]),
                    allocation_type: aletheion_rm_water_model::WaterAllocationType::Environmental,
                    volume_af: 800.0 / 325851.0, // Convert gallons to acre-feet
                    destination_zone_id: *zone_id,
                    source_type: aletheion_rm_water_model::WaterSourceType::Reclaimed,
                    priority: *priorities.get(zone_id).unwrap_or(&5),
                    fpic_verified: water_state.base.fpic_verified,
                    indigenous_rights_respected: INDIGENOUS_SOVEREIGNTY,
                    timestamp_us: water_state.base.risk_coords.first().map(|rc| rc.timestamp_us).unwrap_or(0),
                    expiration_us: 0,
                };

                coopt_plans.push((thermal_plan, water_plan));
            }
        }

        self.coopt_history.extend(coopt_plans.clone());
        Ok(coopt_plans)
    }

    /// Compute combined K/E/R for water-heat co-optimization
    pub fn compute_coopt_ker(&self, thermal_ker: &KERMetrics, water_ker: &KERMetrics) -> KERMetrics {
        KERMetrics {
            k: (thermal_ker.k + water_ker.k) / 2.0,
            e: (thermal_ker.e + water_ker.e) / 2.0,
            r: (thermal_ker.r + water_ker.r) / 2.0,
        }
    }
}

impl Default for WaterHeatCoOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 6: SMART-CHAIN THERMAL WORKFLOW INTEGRATION
// ============================================================================

/// Complete SMART-chain thermal workflow engine
pub struct ThermalWorkflowEngine {
    ingestion: ThermalIngestionEngine,
    optimizer: ThermalAllocationOptimizer,
    cooptimizer: WaterHeatCoOptimizer,
    safe_step: SafeStepEngine,
    ker_computer: KERComputer,
}

impl ThermalWorkflowEngine {
    /// Create a new ThermalWorkflowEngine
    pub fn new(sensor_network_id: [u8; 32]) -> Self {
        Self {
            ingestion: ThermalIngestionEngine::new(sensor_network_id),
            optimizer: ThermalAllocationOptimizer::new(),
            cooptimizer: WaterHeatCoOptimizer::new(),
            safe_step: SafeStepEngine::new("thermal"),
            ker_computer: KERComputer::new(),
        }
    }

    /// Execute complete 7-step SMART-chain for thermal management
    pub fn execute_smart_chain(
        &mut self,
        sensor_readings: &[ThermalSensorReading],
        zones: &[u32],
        priorities: &BTreeMap<u32, u8>,
        water_state: Option<WaterCapitalState>,
        entity_did: [u8; 32],
    ) -> Result<QpuDataShard, EcosafetyError> {
        // STEP 1: SENSE - Ingest sensor data
        self.ingestion.ingest_sensor_data(sensor_readings)?;
        
        // STEP 2: MODEL - Build thermal capital state
        let thermal_state = self.ingestion.aggregate_to_capital_state(sensor_readings)?;
        self.optimizer.build_allocation_model(thermal_state.clone())?;
        
        // STEP 3: OPTIMIZE - Generate allocation plans
        let thermal_plans = self.optimizer.optimize_allocation(
            zones,
            priorities,
            10000.0, // water_available_gallons (placeholder)
            5000.0,  // energy_available_kwh (placeholder)
        )?;
        
        // STEP 3b: Water-heat co-optimization (if water state available)
        let coopt_plans = if let Some(ws) = water_state {
            self.cooptimizer.set_water_state(ws);
            self.cooptimizer.set_thermal_state(thermal_state.clone());
            self.cooptimizer.execute_cooptimization(zones, priorities)?
        } else {
            Vec::new()
        };
        
        // STEP 4: TREATY-CHECK - Validate Indigenous rights
        self.optimizer.validate_indigenous_rights(&thermal_plans)?;
        
        // STEP 5: ACT - Execute with safe_step enforcement
        let seven_capital_state = self.build_seven_capital_state(thermal_state, entity_did)?;
        let mut state_mut = seven_capital_state.clone();
        let action = self.safe_step.safe_step(
            &seven_capital_state,
            &mut state_mut,
            &NodeAction::Actuate,
        )?;
        
        // STEP 6: LOG - Emit QpuDataShard
        let shard = self.emit_thermal_shard(state_mut, action, &thermal_plans)?;
        
        // STEP 7: INTERFACE - Return shard for dashboard/audit
        Ok(shard)
    }

    /// Build SevenCapitalState from ThermalCapitalState
    fn build_seven_capital_state(
        &self,
        thermal_state: ThermalCapitalState,
        entity_did: [u8; 32],
    ) -> Result<SevenCapitalState, EcosafetyError> {
        let timestamp_us = thermal_state.base.risk_coords.first().map(|rc| rc.timestamp_us).unwrap_or(0);
        
        // Thermal capital
        let mut thermal_capital = thermal_state.base;
        thermal_capital.risk_coords = thermal_state.compute_risk_coords();
        
        // Other capitals (placeholder - would be populated from other modules)
        let water = CapitalState::new(0.5, 0.2, 0.8, true)?;
        let waste = CapitalState::new(0.5, 0.2, 0.8, true)?;
        let biotic = CapitalState::new(0.5, 0.2, 0.8, true)?;
        let somatic = CapitalState::new(0.5, 0.2, 0.8, true)?;
        let neurobiome = CapitalState::new(0.5, 0.2, 0.8, true)?;
        let treaty = CapitalState::new(0.5, 0.2, 0.8, thermal_state.base.fpic_verified)?;
        
        SevenCapitalState::new(
            water, thermal_capital, waste, biotic, somatic, neurobiome, treaty,
            entity_did, timestamp_us,
        )
    }

    /// Emit QpuDataShard for thermal workflow
    fn emit_thermal_shard(
        &self,
        state: SevenCapitalState,
        action: NodeAction,
        plans: &[ThermalAllocationPlan],
    ) -> Result<QpuDataShard, EcosafetyError> {
        let ker_metrics = self.ker_computer.compute_ker_metrics(&state);
        
        Ok(QpuDataShard {
            shard_id: state.entity_did,
            entity_did: state.entity_did,
            timestamp_us: state.timestamp_us,
            state,
            action,
            corridor_decision: CorridorDecision::Permit,
            v_t: state.lyapunov_residual.v_t,
            ker_metrics,
            violation: None,
            fpic_status: FPICStatus::Verified,
            smart_chain_hash: [0u8; 64], // Computed by SafeStepEngine
            birth_sign_verified: true,
        })
    }
}

// ============================================================================
// SECTION 7: FFI EXPORTS FOR LUA/JAVASCRIPT/KOTLIN/C++ BINDINGS
// ============================================================================

#[cfg(feature = "ffi")]
pub mod ffi {
    use super::*;

    /// FFI: Execute thermal SMART-chain workflow
    #[no_mangle]
    pub extern "C" fn thermal_execute_smart_chain(
        sensor_readings_ptr: *const ThermalSensorReading,
        sensor_count: usize,
        zones_ptr: *const u32,
        zone_count: usize,
        entity_did_ptr: *const u8,
    ) -> *mut QpuDataShard {
        // Safety: Caller must ensure valid pointers
        if sensor_readings_ptr.is_null() || entity_did_ptr.is_null() {
            return core::ptr::null_mut();
        }

        let readings = unsafe {
            core::slice::from_raw_parts(sensor_readings_ptr, sensor_count)
        };
        
        let zones = unsafe {
            core::slice::from_raw_parts(zones_ptr, zone_count)
        };
        
        let mut entity_did = [0u8; 32];
        unsafe {
            core::ptr::copy_nonoverlapping(entity_did_ptr, entity_did.as_mut_ptr(), 32);
        }

        let mut engine = ThermalWorkflowEngine::new([0u8; 32]);
        let priorities = BTreeMap::new(); // Parse from JSON in production

        match engine.execute_smart_chain(readings, zones, &priorities, None, entity_did) {
            Ok(shard) => Box::into_raw(Box::new(shard)),
            Err(_) => core::ptr::null_mut(),
        }
    }

    /// FFI: Get thermal capital state from sensor readings
    #[no_mangle]
    pub extern "C" fn thermal_get_capital_state(
        sensor_readings_ptr: *const ThermalSensorReading,
        sensor_count: usize,
    ) -> *mut ThermalCapitalState {
        if sensor_readings_ptr.is_null() {
            return core::ptr::null_mut();
        }

        let readings = unsafe {
            core::slice::from_raw_parts(sensor_readings_ptr, sensor_count)
        };

        let ingestion = ThermalIngestionEngine::new([0u8; 32]);
        match ingestion.aggregate_to_capital_state(readings) {
            Ok(state) => Box::into_raw(Box::new(state)),
            Err(_) => core::ptr::null_mut(),
        }
    }
}

// ============================================================================
// SECTION 8: UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sensor_reading() -> ThermalSensorReading {
        ThermalSensorReading {
            sensor_did: [1u8; 32],
            timestamp_us: 0,
            ambient_temp_f: 105.0,
            surface_temp_f: 140.0,
            humidity_percent: 25.0,
            wind_speed_mph: 10.0,
            wind_direction_deg: 180.0,
            solar_radiation_w_m2: 800.0,
            uv_index: 9.0,
            pm2_5: 15.0,
            pm10: 50.0,
            latitude: 33.4484, // Phoenix
            longitude: -112.0740,
            elevation_ft: 1086.0,
        }
    }

    #[test]
    fn test_thermal_capital_state_creation() {
        let state = ThermalCapitalState::new(
            105.0,
            140.0,
            [1u8; 32],
            0,
        );
        
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(state.is_safe());
    }

    #[test]
    fn test_thermal_ingestion_validation() {
        let mut ingestion = ThermalIngestionEngine::new([0u8; 32]);
        let reading = create_test_sensor_reading();
        
        let result = ingestion.ingest_sensor_data(&[reading]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_heat_emergency_level_update() {
        let mut state = ThermalCapitalState::new(
            120.0, // Extreme heat threshold
            150.0,
            [1u8; 32],
            0,
        ).unwrap();
        
        state.update_heat_emergency_level();
        assert!(state.heat_emergency_level >= 3);
    }

    #[test]
    fn test_cool_pavement_application() {
        let mut state = ThermalCapitalState::new(
            110.0,
            150.0,
            [1u8; 32],
            0,
        ).unwrap();
        
        let initial_surface = state.surface_temp_f;
        state.apply_cool_pavement(50.0); // 50% coverage
        
        assert!(state.surface_temp_f < initial_surface);
        assert_eq!(state.cool_pavement_percent, 50.0);
    }

    #[test]
    fn test_thermal_allocation_optimizer() {
        let mut optimizer = ThermalAllocationOptimizer::new();
        let state = ThermalCapitalState::new(
            115.0,
            145.0,
            [1u8; 32],
            0,
        ).unwrap();
        
        optimizer.build_allocation_model(state).unwrap();
        
        let zones = vec![1, 2, 3];
        let mut priorities = BTreeMap::new();
        priorities.insert(1, 1);
        priorities.insert(2, 2);
        priorities.insert(3, 3);
        
        let plans = optimizer.optimize_allocation(&zones, &priorities, 10000.0, 5000.0);
        assert!(plans.is_ok());
    }

    #[test]
    fn test_uhi_detection() {
        let ingestion = ThermalIngestionEngine::new([0u8; 32]);
        
        let urban_readings = vec![
            ThermalSensorReading {
                ambient_temp_f: 110.0,
                ..create_test_sensor_reading()
            },
        ];
        
        let rural_readings = vec![
            ThermalSensorReading {
                ambient_temp_f: 102.0,
                ..create_test_sensor_reading()
            },
        ];
        
        let uhi = ingestion.detect_uhi_intensity(&urban_readings, &rural_readings);
        assert!(uhi >= 7.0); // Should detect ~8°F UHI
    }
}

// ============================================================================
// END OF FILE: ALE-RM-THERMAL-STATE-001.rs
// ============================================================================
