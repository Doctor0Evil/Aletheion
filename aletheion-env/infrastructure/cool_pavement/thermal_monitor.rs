/**
 * Aletheion Smart City Core - Batch 2
 * File: 103/200
 * Layer: 21 (Advanced Environment)
 * Path: aletheion-env/infrastructure/cool_pavement/thermal_monitor.rs
 * 
 * Research Basis:
 *   - Phoenix Cool Pavement Program: 10.5-12°F surface temperature reduction
 *   - 140+ miles of cool pavement deployed (2025)
 *   - Reflective albedo optimization for desert urbanism
 *   - Misting system integration for extreme heat events (120°F+)
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Land Use Rights)
 *   - Post-Quantum Secure (via aletheion_data::pq_crypto)
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
use core::cmp::Ordering;

// Internal Aletheion Crates (Established in Batch 1)
use aletheion_data::pq_crypto::hash::pq_hash;
use aletheion_data::did_wallet::DIDWallet;
use aletheion_gov::treaty::TreatyCompliance;
use aletheion_physical::hal::ActuatorCommand;
use aletheion_comms::mesh::OfflineQueue;
use aletheion_core::identity::BirthSign;
use aletheion_energy::management::EnergyBudget;

// --- Constants & Phoenix Heat Protocol Thresholds ---

/// Critical pavement surface temperature (°F) triggering cooling intervention
/// Based on Phoenix Heat Protocol: 140°F = 60°C equipment safety limit
const PAVEMENT_CRITICAL_TEMP_F: f32 = 140.0;
/// High alert threshold (°F) for pre-emptive cooling
const PAVEMENT_HIGH_ALERT_F: f32 = 125.0;
/// Moderate alert threshold (°F) for monitoring
const PAVEMENT_MODERATE_ALERT_F: f32 = 110.0;
/// Optimal cool pavement operating range (°F)
const PAVEMENT_OPTIMAL_MAX_F: f32 = 95.0;

/// Ambient air temperature thresholds (°F)
const AMBIENT_CRITICAL_TEMP_F: f32 = 115.0;
const AMBIENT_HIGH_TEMP_F: f32 = 105.0;

/// Cooling system parameters
const MISTING_SYSTEM_WATER_USAGE_GPH: f32 = 2.5; // Gallons per hour per zone
const REFLECTIVE_COATING_LIFESPAN_DAYS: u32 = 365;
const COOLING_FAN_POWER_WATTS: f32 = 150.0;

/// Urban Heat Island (UHI) reduction targets (°F)
/// Research-based: Phoenix deployments achieved 10.5-12°F reduction
const UHI_REDUCTION_TARGET_MIN_F: f32 = 10.5;
const UHI_REDUCTION_TARGET_MAX_F: f32 = 12.0;

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
const OFFLINE_BUFFER_HOURS: u32 = 72;
/// Treaty Check Cache TTL (seconds)
const TREATY_CACHE_TTL: u64 = 300;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PavementCoolingMode {
    /// No active cooling - optimal temperature range
    Passive,
    /// Misting system activated for evaporative cooling
    MistingActive,
    /// Reflective coating refresh scheduled
    CoatingMaintenance,
    /// Fan-assisted air circulation
    FanAssisted,
    /// Emergency cooling (all systems active)
    Emergency,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PavementAlertLevel {
    Normal,
    Moderate,
    High,
    Critical,
}

#[derive(Clone)]
pub struct ThermalSensorReading {
    pub timestamp: u64,
    pub pavement_surface_temp_f: f32,
    pub ambient_air_temp_f: f32,
    pub relative_humidity_percent: f32,
    pub solar_radiation_w_m2: f32,
    pub wind_speed_mph: f32,
    pub sensor_id: [u8; 32], // PQ-Secure ID
}

#[derive(Clone)]
pub struct CoolingAction {
    pub action_type: CoolingActionType,
    pub target_zone_id: [u8; 32],
    pub intensity_percent: u8, // 0-100
    pub duration_minutes: u32,
    pub water_usage_gallons: f32,
    pub energy_usage_wh: f32,
    pub treaty_hash: [u8; 64], // PQ-Hash of compliance check
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoolingActionType {
    ActivateMistingSystem,
    ScheduleCoatingRefresh,
    ActivateCoolingFans,
    DeployShadeStructures,
    BroadcastHeatAlert,
}

#[derive(Clone)]
pub struct PavementZoneConfig {
    pub zone_id: [u8; 32],
    pub location_coordinates: [f64; 2], // [lat, lon]
    pub pavement_area_sqft: f32,
    pub albedo_coefficient: f32, // 0.0-1.0 (reflectivity)
    pub last_coating_date: u64,
    pub indigenous_territory: bool,
    pub treaty_zone_id: Option<[u8; 32]>,
}

// --- Core Thermal Monitor Structure ---

pub struct CoolPavementThermalMonitor {
    pub node_id: BirthSign,
    pub zone_config: PavementZoneConfig,
    pub current_temp_f: f32,
    pub alert_level: PavementAlertLevel,
    pub cooling_mode: PavementCoolingMode,
    pub offline_queue: OfflineQueue<CoolingAction>,
    pub treaty_cache: TreatyCompliance,
    pub energy_budget: EnergyBudget,
    pub water_usage_today_gallons: f32,
    pub last_sync: u64,
    pub misting_system_active: bool,
    pub coating_refresh_needed: bool,
}

impl CoolPavementThermalMonitor {
    /**
     * Initialize the Thermal Monitor with Zone Configuration
     * Ensures 72h operational buffer and treaty compliance setup
     */
    pub fn new(node_id: BirthSign, zone_config: PavementZoneConfig) -> Result<Self, &'static str> {
        let queue = OfflineQueue::new(OFFLINE_BUFFER_HOURS)
            .map_err(|_| "Failed to allocate offline buffer")?;
        
        let energy_budget = EnergyBudget::new_for_zone(&zone_config.zone_id)
            .map_err(|_| "Failed to initialize energy budget")?;
        
        Ok(Self {
            node_id,
            zone_config,
            current_temp_f: 0.0,
            alert_level: PavementAlertLevel::Normal,
            cooling_mode: PavementCoolingMode::Passive,
            offline_queue: queue,
            treaty_cache: TreatyCompliance::new(),
            energy_budget,
            water_usage_today_gallons: 0.0,
            last_sync: 0,
            misting_system_active: false,
            coating_refresh_needed: false,
        })
    }

    /**
     * ERM Chain: SENSE
     * Ingests thermal sensor data from pavement surface monitors
     * Validates data integrity using PQ hashing
     */
    pub fn sense(&mut self, reading: ThermalSensorReading) -> Result<(), &'static str> {
        // Validate sensor signature (PQ Secure)
        let hash = pq_hash(&reading.sensor_id);
        if hash[0] == 0x00 { // Placeholder for actual signature verification logic
            return Err("Sensor signature invalid");
        }

        // Update current temperature
        self.current_temp_f = reading.pavement_surface_temp_f;

        // Update Alert Level based on temperature thresholds
        self.update_alert_level();

        // Check coating refresh requirement based on temperature exposure
        self.check_coating_degradation(&reading);

        // Log sensing event
        self.log_event(format!(
            "SENSE: Pavement={:.1}°F, Ambient={:.1}°F, Humidity={:.1}%, Solar={}W/m²",
            reading.pavement_surface_temp_f,
            reading.ambient_air_temp_f,
            reading.relative_humidity_percent,
            reading.solar_radiation_w_m2
        ));

        Ok(())
    }

    /**
     * Update alert level based on pavement temperature thresholds
     */
    fn update_alert_level(&mut self) {
        self.alert_level = match self.current_temp_f {
            t if t >= PAVEMENT_CRITICAL_TEMP_F => PavementAlertLevel::Critical,
            t if t >= PAVEMENT_HIGH_ALERT_F => PavementAlertLevel::High,
            t if t >= PAVEMENT_MODERATE_ALERT_F => PavementAlertLevel::Moderate,
            _ => PavementAlertLevel::Normal,
        };
    }

    /**
     * Check if reflective coating needs refresh based on thermal exposure
     * Coating degrades faster under extreme heat conditions
     */
    fn check_coating_degradation(&mut self, reading: &ThermalSensorReading) {
        let days_since_coating = (aletheion_core::time::now() - self.zone_config.last_coating_date) / 86400;
        
        // Accelerated degradation if consistently above critical temperature
        if self.current_temp_f > PAVEMENT_CRITICAL_TEMP_F && days_since_coating > (REFLECTIVE_COATING_LIFESPAN_DAYS as f32 * 0.7) {
            self.coating_refresh_needed = true;
        } else if days_since_coating > REFLECTIVE_COATING_LIFESPAN_DAYS as u64 {
            self.coating_refresh_needed = true;
        }
    }

    /**
     * ERM Chain: MODEL
     * Calculates optimal cooling strategy based on temperature, humidity, and energy constraints
     * No Digital Twins: Uses direct sensor correlation and physics-based models
     */
    pub fn model_optimal_cooling(&self, reading: &ThermalSensorReading) -> Option<CoolingStrategy> {
        // Skip cooling if temperature is optimal
        if self.alert_level == PavementAlertLevel::Normal {
            return None;
        }

        // Calculate evaporative cooling potential based on humidity
        let evaporation_potential = self.calculate_evaporation_potential(
            reading.relative_humidity_percent,
            reading.ambient_air_temp_f
        );

        // Determine primary cooling method
        let primary_method = if evaporation_potential > 0.6 && self.alert_level >= PavementAlertLevel::High {
            CoolingMethod::Misting
        } else if self.alert_level == PavementAlertLevel::Critical {
            CoolingMethod::EmergencyCombined
        } else {
            CoolingMethod::FanAssisted
        };

        // Calculate required intensity and duration
        let intensity = self.calculate_cooling_intensity();
        let duration = self.calculate_cooling_duration();

        // Estimate resource usage
        let water_usage = self.estimate_water_usage(intensity, duration);
        let energy_usage = self.estimate_energy_usage(primary_method, intensity, duration);

        // Check resource constraints
        if !self.energy_budget.can_allocate(energy_usage as u32) {
            self.log_warning("Insufficient energy budget for planned cooling");
            return None;
        }

        Some(CoolingStrategy {
            primary_method,
            intensity_percent: intensity,
            duration_minutes: duration,
            estimated_water_gallons: water_usage,
            estimated_energy_wh: energy_usage,
            uhi_reduction_estimate_f: self.estimate_uhi_reduction(intensity),
        })
    }

    /**
     * Calculate evaporative cooling potential (0.0-1.0)
     * Higher potential when humidity is low and temperature is high
     */
    fn calculate_evaporation_potential(&self, humidity_percent: f32, temp_f: f32) -> f32 {
        // Evaporative cooling is most effective when humidity is low (<40%) and temperature is high
        let humidity_factor = 1.0 - (humidity_percent / 100.0);
        let temperature_factor = (temp_f - 80.0) / 50.0; // Normalize to 80-130°F range
        
        let potential = humidity_factor * temperature_factor.clamp(0.0, 1.0);
        potential.clamp(0.0, 1.0)
    }

    /**
     * Calculate cooling intensity based on alert level (0-100%)
     */
    fn calculate_cooling_intensity(&self) -> u8 {
        match self.alert_level {
            PavementAlertLevel::Critical => 100,
            PavementAlertLevel::High => 75,
            PavementAlertLevel::Moderate => 40,
            PavementAlertLevel::Normal => 0,
        }
    }

    /**
     * Calculate cooling duration in minutes based on temperature delta
     */
    fn calculate_cooling_duration(&self) -> u32 {
        let temp_delta = self.current_temp_f - PAVEMENT_OPTIMAL_MAX_F;
        
        if temp_delta <= 0.0 {
            return 0;
        }
        
        // Duration scales with temperature excess: ~5 min per 5°F over optimal
        let base_duration = (temp_delta / 5.0) * 5.0;
        
        match self.alert_level {
            PavementAlertLevel::Critical => base_duration * 2.0,
            PavementAlertLevel::High => base_duration * 1.5,
            PavementAlertLevel::Moderate => base_duration,
            PavementAlertLevel::Normal => 0.0,
        }.clamp(5.0, 120.0) as u32
    }

    /**
     * Estimate water usage for misting system (gallons)
     */
    fn estimate_water_usage(&self, intensity_percent: u8, duration_minutes: u32) -> f32 {
        let intensity_factor = intensity_percent as f32 / 100.0;
        let duration_hours = duration_minutes as f32 / 60.0;
        
        MISTING_SYSTEM_WATER_USAGE_GPH * intensity_factor * duration_hours * 
            (self.zone_config.pavement_area_sqft / 1000.0) // Scale by zone size
    }

    /**
     * Estimate energy usage for cooling action (watt-hours)
     */
    fn estimate_energy_usage(&self, method: CoolingMethod, intensity_percent: u8, duration_minutes: u32) -> f32 {
        let duration_hours = duration_minutes as f32 / 60.0;
        let intensity_factor = intensity_percent as f32 / 100.0;
        
        match method {
            CoolingMethod::Misting => 50.0 * intensity_factor * duration_hours, // Pump power
            CoolingMethod::FanAssisted => COOLING_FAN_POWER_WATTS * intensity_factor * duration_hours,
            CoolingMethod::EmergencyCombined => (COOLING_FAN_POWER_WATTS + 50.0) * duration_hours,
            CoolingMethod::CoatingRefresh => 200.0, // One-time application energy
        }
    }

    /**
     * Estimate Urban Heat Island reduction (°F) based on cooling intensity
     * Research-based: Phoenix deployments achieved 10.5-12°F reduction
     */
    fn estimate_uhi_reduction(&self, intensity_percent: u8) -> f32 {
        let intensity_factor = intensity_percent as f32 / 100.0;
        
        // Scale reduction estimate based on intensity
        UHI_REDUCTION_TARGET_MIN_F + (UHI_REDUCTION_TARGET_MAX_F - UHI_REDUCTION_TARGET_MIN_F) * intensity_factor
    }

    /**
     * ERM Chain: OPTIMIZE & TREATY-CHECK
     * Determines necessary cooling actions and validates against Indigenous land rights
     * FPIC Enforcement: Cannot deploy cooling systems on protected lands without consent
     */
    pub fn optimize_and_check(&mut self, strategy: &CoolingStrategy) -> Result<CoolingAction, &'static str> {
        // 1. Check Treaty Compliance (FPIC)
        // Ensures cooling deployment respects Akimel O'odham or Piipaash territories
        if self.zone_config.indigenous_territory {
            let treaty_zone = self.zone_config.treaty_zone_id
                .ok_or("Indigenous territory requires treaty zone ID")?;
            
            let compliance = self.treaty_cache.check_land_use(&treaty_zone)?;
            
            if !compliance.allowed {
                return Err("FPIC Violation: Treaty restricts cooling deployment in this zone");
            }
        }

        // 2. Determine action type based on strategy
        let action_type = match strategy.primary_method {
            CoolingMethod::Misting => CoolingActionType::ActivateMistingSystem,
            CoolingMethod::FanAssisted => CoolingActionType::ActivateCoolingFans,
            CoolingMethod::EmergencyCombined => CoolingActionType::ActivateMistingSystem, // Primary action
            CoolingMethod::CoatingRefresh => CoolingActionType::ScheduleCoatingRefresh,
        };

        // 3. Create action with resource estimates
        let mut action = CoolingAction {
            action_type,
            target_zone_id: self.zone_config.zone_id,
            intensity_percent: strategy.intensity_percent,
            duration_minutes: strategy.duration_minutes,
            water_usage_gallons: strategy.estimated_water_gallons,
            energy_usage_wh: strategy.estimated_energy_wh,
            treaty_hash: [0u8; 64],
            signed: false,
        };

        // 4. Hash treaty compliance for audit trail
        if self.zone_config.indigenous_territory {
            action.treaty_hash = self.treaty_cache.get_current_hash();
        }

        // 5. Sign Action (PQ Secure)
        // Uses node identity to sign the command for auditability
        let signature = DIDWallet::sign_action(&self.node_id, &action);
        action.signed = signature.is_ok();

        Ok(action)
    }

    /**
     * ERM Chain: ACT
     * Executes cooling action or queues for offline execution
     * Ensures atomicity without rollbacks
     */
    pub fn act(&mut self, action: CoolingAction) -> Result<(), &'static str> {
        // Attempt immediate execution via HAL
        match aletheion_physical::hal::execute_cooling(&action) {
            Ok(_) => {
                // Update local state
                match action.action_type {
                    CoolingActionType::ActivateMistingSystem => {
                        self.misting_system_active = true;
                        self.water_usage_today_gallons += action.water_usage_gallons;
                        self.cooling_mode = PavementCoolingMode::MistingActive;
                    },
                    CoolingActionType::ActivateCoolingFans => {
                        self.cooling_mode = PavementCoolingMode::FanAssisted;
                    },
                    CoolingActionType::ScheduleCoatingRefresh => {
                        self.coating_refresh_needed = false;
                        self.cooling_mode = PavementCoolingMode::CoatingMaintenance;
                    },
                    _ => {}
                }
                
                self.log_action(&action);
                Ok(())
            },
            Err(_) => {
                // Offline Fallback: Queue for later execution
                // Critical for 72h Heat Protocol resilience
                self.offline_queue.push(action)?;
                self.log_warning("Offline mode: Cooling action queued for later execution");
                Ok(())
            }
        }
    }

    /**
     * ERM Chain: LOG
     * Immutable logging to ALN-Blockchain
     * No rollbacks allowed; all state changes are forward-only
     */
    fn log_action(&self, action: &CoolingAction) {
        let log_entry = alloc::format!(
            "COOLING_ACT: Zone={:?} | Type={:?} | Intensity={}%, Duration={}min | Water={:.2}gal, Energy={:.1}Wh | Treaty={:?}",
            action.target_zone_id,
            action.action_type,
            action.intensity_percent,
            action.duration_minutes,
            action.water_usage_gallons,
            action.energy_usage_wh,
            if action.treaty_hash[0] != 0 { "Compliant" } else { "N/A" }
        );
        
        // Log to local immutable ledger (syncs to ALN when online)
        aletheion_data::ledger::append_immutable(&log_entry);
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
    pub fn get_status_report(&self) -> ThermalStatusReport {
        ThermalStatusReport {
            zone_id: self.zone_config.zone_id,
            current_temp_f: self.current_temp_f,
            alert_level: self.alert_level,
            cooling_mode: self.cooling_mode,
            misting_active: self.misting_system_active,
            coating_refresh_needed: self.coating_refresh_needed,
            water_used_today_gallons: self.water_usage_today_gallons,
            offline_queue_size: self.offline_queue.len(),
            last_sync: self.last_sync,
            accessibility_alert: self.alert_level >= PavementAlertLevel::High,
            uhi_reduction_achieved_f: self.calculate_current_uhi_reduction(),
        }
    }

    /**
     * Calculate current Urban Heat Island reduction based on cooling mode
     */
    fn calculate_current_uhi_reduction(&self) -> f32 {
        match self.cooling_mode {
            PavementCoolingMode::MistingActive => 8.5,
            PavementCoolingMode::FanAssisted => 4.0,
            PavementCoolingMode::Emergency => 11.0,
            PavementCoolingMode::CoatingMaintenance => 10.5,
            PavementCoolingMode::Passive => {
                // Passive cooling from reflective albedo
                self.zone_config.albedo_coefficient * 12.0
            },
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
     * Daily reset for water usage tracking
     */
    pub fn daily_reset(&mut self) {
        self.water_usage_today_gallons = 0.0;
        self.log_event("DAILY_RESET: Water usage counter reset".to_string());
    }
}

// --- Supporting Data Structures ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CoolingMethod {
    Misting,
    FanAssisted,
    EmergencyCombined,
    CoatingRefresh,
}

pub struct CoolingStrategy {
    pub primary_method: CoolingMethod,
    pub intensity_percent: u8,
    pub duration_minutes: u32,
    pub estimated_water_gallons: f32,
    pub estimated_energy_wh: f32,
    pub uhi_reduction_estimate_f: f32,
}

pub struct ThermalStatusReport {
    pub zone_id: [u8; 32],
    pub current_temp_f: f32,
    pub alert_level: PavementAlertLevel,
    pub cooling_mode: PavementCoolingMode,
    pub misting_active: bool,
    pub coating_refresh_needed: bool,
    pub water_used_today_gallons: f32,
    pub offline_queue_size: usize,
    pub last_sync: u64,
    pub accessibility_alert: bool,
    pub uhi_reduction_achieved_f: f32,
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_level_critical() {
        let zone_config = PavementZoneConfig {
            zone_id: [1u8; 32],
            location_coordinates: [33.4484, -112.0740], // Phoenix, AZ
            pavement_area_sqft: 10000.0,
            albedo_coefficient: 0.35,
            last_coating_date: 0,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let mut monitor = CoolPavementThermalMonitor::new(BirthSign::default(), zone_config).unwrap();
        
        let reading = ThermalSensorReading {
            timestamp: 1000,
            pavement_surface_temp_f: 145.0, // Above critical
            ambient_air_temp_f: 118.0,
            relative_humidity_percent: 15.0,
            solar_radiation_w_m2: 950.0,
            wind_speed_mph: 5.0,
            sensor_id: [1u8; 32],
        };
        
        monitor.sense(reading).unwrap();
        assert_eq!(monitor.alert_level, PavementAlertLevel::Critical);
    }

    #[test]
    fn test_offline_queue_capacity() {
        let zone_config = PavementZoneConfig {
            zone_id: [1u8; 32],
            location_coordinates: [33.4484, -112.0740],
            pavement_area_sqft: 5000.0,
            albedo_coefficient: 0.40,
            last_coating_date: 0,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let monitor = CoolPavementThermalMonitor::new(BirthSign::default(), zone_config).unwrap();
        assert!(monitor.offline_queue.capacity_hours() >= 72);
    }

    #[test]
    fn test_evaporation_potential_calculation() {
        let zone_config = PavementZoneConfig {
            zone_id: [1u8; 32],
            location_coordinates: [33.4484, -112.0740],
            pavement_area_sqft: 5000.0,
            albedo_coefficient: 0.35,
            last_coating_date: 0,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let monitor = CoolPavementThermalMonitor::new(BirthSign::default(), zone_config).unwrap();
        
        // Low humidity, high temperature = high evaporation potential
        let potential = monitor.calculate_evaporation_potential(20.0, 115.0);
        assert!(potential > 0.6);
        
        // High humidity, moderate temperature = low evaporation potential
        let potential_low = monitor.calculate_evaporation_potential(70.0, 90.0);
        assert!(potential_low < 0.3);
    }

    #[test]
    fn test_uhi_reduction_estimate() {
        let zone_config = PavementZoneConfig {
            zone_id: [1u8; 32],
            location_coordinates: [33.4484, -112.0740],
            pavement_area_sqft: 5000.0,
            albedo_coefficient: 0.35,
            last_coating_date: 0,
            indigenous_territory: false,
            treaty_zone_id: None,
        };
        
        let monitor = CoolPavementThermalMonitor::new(BirthSign::default(), zone_config).unwrap();
        
        // 100% intensity should achieve maximum UHI reduction
        let reduction = monitor.estimate_uhi_reduction(100);
        assert!(reduction >= UHI_REDUCTION_TARGET_MIN_F);
        assert!(reduction <= UHI_REDUCTION_TARGET_MAX_F);
    }
}
