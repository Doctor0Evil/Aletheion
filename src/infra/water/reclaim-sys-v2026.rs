//! # Water Reclamation System Module (reclaim_sys_v2026)
//!
//! Implements analysis and modeling for Phoenix water reclamation as per
//! research action plan `ENV-CLIMATE-20260308` Step 01.
//!
//! ## Objectives
//! - Analyze Pure-Water-Phoenix 2025-26 expansions
//! - Achieve 99% reclamation efficiency (target-99pct-efficiency)
//! - Meet 50 gallons per capita daily goal (50gal-per-capita)
//! - Integrate USGS aquifer data and O'odham traditional knowledge
//!
//! ## Sources
//! - phoenix.gov/waterservices/reports
//! - USGS-AZ-aquifer-2025
//! - SUNDTCOM-CAVE-CREEK-REHAB
//!
//! ## Output
//! - Water reclamation efficiency model
//!
//! ## Placement
//! Industrial reclaim plants south Phoenix, remote from residential to prevent unrest.
//!
//! ## Fairness
//! Incorporate O'odham traditional aquifer knowledge.
//!
//! ## Device Requirements
//! - biosignal-collector-node
//! - region-nodes
//! - BCI types
//!
//! ## Machinery
//! autonomous-reclamation-plant-kit

// External dependencies (if any) would be listed here.
// For simulation we use std only, but in practice serde, etc.

use std::collections::HashMap;
use std::time::{Duration, Instant};

// =============================================================================
// Constants from the plan
// =============================================================================

/// Target reclamation efficiency (99%)
pub const TARGET_EFFICIENCY: f64 = 0.99;

/// Per capita water goal (gallons per day)
pub const PER_CAPITA_GOAL_GAL: f64 = 50.0;

/// Conversion factor: gallons to cubic meters (for aquifer data)
pub const GAL_TO_CUBIC_M: f64 = 0.00378541;

// =============================================================================
// Core data structures
// =============================================================================

/// Represents a water reclamation plant.
#[derive(Debug, Clone)]
pub struct WaterReclamationPlant {
    /// Unique identifier
    pub id: String,
    /// Location (latitude, longitude)
    pub latitude: f64,
    pub longitude: f64,
    /// Design capacity (gallons per day)
    pub capacity_gpd: f64,
    /// Current operational efficiency (0.0 - 1.0)
    pub efficiency: f64,
    /// Whether the plant is operational
    pub operational: bool,
    /// Last maintenance timestamp
    pub last_maintenance: Option<Instant>,
    /// Associated biosignal collector node ID
    pub biosignal_node_id: Option<String>,
    /// Associated region node IDs
    pub region_node_ids: Vec<String>,
    /// Associated BCI types (e.g., EEG, ECG) – placeholder
    pub bci_types: Vec<String>,
}

impl WaterReclamationPlant {
    /// Creates a new plant with default values.
    pub fn new(id: impl Into<String>, lat: f64, lon: f64, capacity_gpd: f64) -> Self {
        Self {
            id: id.into(),
            latitude: lat,
            longitude: lon,
            capacity_gpd,
            efficiency: 0.0,
            operational: true,
            last_maintenance: None,
            biosignal_node_id: None,
            region_node_ids: Vec::new(),
            bci_types: Vec::new(),
        }
    }

    /// Computes actual daily output in gallons.
    pub fn daily_output(&self) -> f64 {
        self.capacity_gpd * self.efficiency
    }
}

/// USGS Aquifer data for a specific region.
#[derive(Debug, Clone)]
pub struct AquiferData {
    /// Aquifer name
    pub name: String,
    /// Current storage volume (cubic meters)
    pub storage_m3: f64,
    /// Sustainable recharge rate (cubic meters per day)
    pub recharge_rate_m3_per_day: f64,
    /// Extraction rate (cubic meters per day)
    pub extraction_rate_m3_per_day: f64,
    /// Historical data points (timestamp, storage)
    pub history: Vec<(Instant, f64)>,
}

impl AquiferData {
    /// Converts storage to gallons.
    pub fn storage_gal(&self) -> f64 {
        self.storage_m3 / GAL_TO_CUBIC_M
    }

    /// Estimates years until depletion at current extraction (if negative).
    pub fn years_until_depletion(&self) -> Option<f64> {
        let net = self.extraction_rate_m3_per_day - self.recharge_rate_m3_per_day;
        if net <= 0.0 {
            None // stable or increasing
        } else {
            Some(self.storage_m3 / net / 365.0)
        }
    }
}

/// O'odham traditional ecological knowledge related to water.
/// This is a placeholder for a more sophisticated representation.
#[derive(Debug, Clone)]
pub struct OodhamKnowledge {
    /// Seasonal indicators for aquifer recharge
    pub seasonal_recharge_months: Vec<u32>,
    /// Traditional well locations (lat, lon)
    pub traditional_wells: Vec<(f64, f64)>,
    /// Oral history notes (simplified)
    pub oral_history: String,
}

/// Biosignal collector node – device that gathers physiological data.
/// This is a stub; in reality it would interface with hardware.
#[derive(Debug, Clone)]
pub struct BiosignalCollectorNode {
    pub id: String,
    pub plant_id: Option<String>,
    /// Last reported health metrics (simplified)
    pub last_heart_rate: Option<f64>,
    pub last_respiratory_rate: Option<f64>,
}

/// Region node – a sensor/actuator node covering an area.
#[derive(Debug, Clone)]
pub struct RegionNode {
    pub id: String,
    pub coverage_radius_m: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub associated_plant_ids: Vec<String>,
}

/// Autonomous reclamation plant kit – machinery interface.
/// Provides methods to control and monitor the physical plant.
pub struct AutonomousReclamationKit {
    pub plant_id: String,
    // In a real system, this would contain handles to motors, valves, sensors.
}

impl AutonomousReclamationKit {
    /// Start the reclamation process.
    pub fn start_reclamation(&self) -> Result<(), String> {
        // Simulated hardware command
        println!("[{}] Reclamation started", self.plant_id);
        Ok(())
    }

    /// Stop the reclamation process.
    pub fn stop_reclamation(&self) -> Result<(), String> {
        println!("[{}] Reclamation stopped", self.plant_id);
        Ok(())
    }

    /// Retrieve current efficiency from plant sensors.
    pub fn read_efficiency(&self) -> Result<f64, String> {
        // Simulated reading
        Ok(0.95)
    }
}

// =============================================================================
// Models and analysis functions
// =============================================================================

/// Represents the water reclamation efficiency model.
#[derive(Debug, Default)]
pub struct EfficiencyModel {
    /// Plant ID -> efficiency
    pub plant_efficiencies: HashMap<String, f64>,
    /// Overall system efficiency
    pub system_efficiency: f64,
    /// Per capita availability (gal/day)
    pub per_capita_gal: f64,
    /// Whether O'odham knowledge was incorporated
    pub oodham_integrated: bool,
    /// Timestamp of model update
    pub last_updated: Instant,
}

/// Analyzes Pure-Water-Phoenix expansion reports and updates plant data.
pub fn analyze_expansion(
    plants: &mut [WaterReclamationPlant],
    reports: &[ExpansionReport],
) -> Result<(), String> {
    for report in reports {
        // Find matching plant by ID or location
        if let Some(plant) = plants.iter_mut().find(|p| p.id == report.plant_id) {
            plant.capacity_gpd = report.new_capacity_gpd;
            plant.efficiency = report.reported_efficiency;
            plant.operational = report.operational;
        } else {
            return Err(format!("Plant {} not found", report.plant_id));
        }
    }
    Ok(())
}

/// A simple expansion report structure.
#[derive(Debug)]
pub struct ExpansionReport {
    pub plant_id: String,
    pub new_capacity_gpd: f64,
    pub reported_efficiency: f64,
    pub operational: bool,
    pub report_date: Instant,
}

/// Computes efficiency for a plant considering aquifer interactions.
/// Returns an efficiency factor (0.0-1.0) that may be adjusted based on
/// aquifer stress.
pub fn compute_efficiency(plant: &WaterReclamationPlant, aquifer: &AquiferData) -> f64 {
    // Base efficiency is plant's own efficiency
    let mut eff = plant.efficiency;

    // Adjust based on aquifer health: if aquifer is stressed, efficiency drops.
    if let Some(years) = aquifer.years_until_depletion() {
        if years < 10.0 {
            // Stress factor: linear reduction if less than 10 years left
            let stress = (10.0 - years) / 10.0; // 0.0 to 1.0
            eff *= 1.0 - stress * 0.2; // max 20% penalty
        }
    }

    eff.clamp(0.0, 1.0)
}

/// Integrates O'odham traditional knowledge into the efficiency model.
/// This could adjust recharge estimates or suggest seasonal operational changes.
pub fn integrate_oodham_knowledge(
    knowledge: &OodhamKnowledge,
    model: &mut EfficiencyModel,
    aquifer: &mut AquiferData,
) {
    // Example: adjust recharge rate based on traditional seasonal knowledge
    let current_month = chrono::Local::now().month();
    if knowledge.seasonal_recharge_months.contains(&current_month) {
        // Increase perceived recharge during traditional recharge months
        aquifer.recharge_rate_m3_per_day *= 1.1; // 10% boost
    }

    // Mark that knowledge was integrated
    model.oodham_integrated = true;
}

/// Simulates the water reclamation system over a period.
/// Returns an updated efficiency model.
pub fn simulate_efficiency(
    plants: &[WaterReclamationPlant],
    aquifers: &[AquiferData],
    population: u64,
    duration: Duration,
) -> EfficiencyModel {
    let mut model = EfficiencyModel::default();
    let mut total_output = 0.0;

    for plant in plants {
        if !plant.operational {
            continue;
        }
        // For simplicity, we pick the first aquifer (in reality, spatial matching)
        let aquifer = aquifers.first().unwrap();
        let eff = compute_efficiency(plant, aquifer);
        model
            .plant_efficiencies
            .insert(plant.id.clone(), eff);
        total_output += plant.daily_output();
    }

    model.system_efficiency = total_output / plants.iter().map(|p| p.capacity_gpd).sum::<f64>();
    model.per_capita_gal = total_output / population as f64;
    model.last_updated = Instant::now();

    model
}

// =============================================================================
// Device and machinery interaction stubs
// =============================================================================

/// Connects a biosignal collector node to a plant for health monitoring.
pub fn attach_biosignal_node(
    plant: &mut WaterReclamationPlant,
    node: BiosignalCollectorNode,
) -> Result<(), String> {
    if plant.biosignal_node_id.is_some() {
        return Err("Plant already has a biosignal node".into());
    }
    plant.biosignal_node_id = Some(node.id.clone());
    // In reality, we'd also store the node in a registry
    Ok(())
}

/// Links region nodes to a plant for environmental monitoring.
pub fn link_region_nodes(
    plant: &mut WaterReclamationPlant,
    node_ids: Vec<String>,
) {
    plant.region_node_ids.extend(node_ids);
}

/// Initializes the autonomous reclamation kit for a plant.
pub fn initialize_reclamation_kit(plant_id: &str) -> AutonomousReclamationKit {
    AutonomousReclamationKit {
        plant_id: plant_id.to_string(),
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plant_creation() {
        let plant = WaterReclamationPlant::new("P1", 33.45, -112.07, 10_000_000.0);
        assert_eq!(plant.id, "P1");
        assert_eq!(plant.capacity_gpd, 10_000_000.0);
        assert_eq!(plant.efficiency, 0.0);
    }

    #[test]
    fn test_aquifer_depletion() {
        let aquifer = AquiferData {
            name: "Test".into(),
            storage_m3: 1_000_000.0,
            recharge_rate_m3_per_day: 100.0,
            extraction_rate_m3_per_day: 200.0,
            history: vec![],
        };
        let years = aquifer.years_until_depletion().unwrap();
        assert!((years - (1_000_000.0 / 100.0 / 365.0)).abs() < 1e-6);
    }

    #[test]
    fn test_efficiency_computation() {
        let plant = WaterReclamationPlant {
            id: "P1".into(),
            latitude: 0.0,
            longitude: 0.0,
            capacity_gpd: 1_000_000.0,
            efficiency: 0.95,
            operational: true,
            last_maintenance: None,
            biosignal_node_id: None,
            region_node_ids: vec![],
            bci_types: vec![],
        };
        let aquifer = AquiferData {
            name: "Test".into(),
            storage_m3: 1_000_000.0,
            recharge_rate_m3_per_day: 100.0,
            extraction_rate_m3_per_day: 200.0,
            history: vec![],
        };
        let eff = compute_efficiency(&plant, &aquifer);
        // With depletion <10 years, efficiency should be <0.95
        assert!(eff < 0.95 && eff > 0.85);
    }

    #[test]
    fn test_oodham_integration() {
        let knowledge = OodhamKnowledge {
            seasonal_recharge_months: vec![7, 8, 9], // July-September
            traditional_wells: vec![],
            oral_history: "".into(),
        };
        let mut aquifer = AquiferData {
            name: "Test".into(),
            storage_m3: 1_000_000.0,
            recharge_rate_m3_per_day: 100.0,
            extraction_rate_m3_per_day: 150.0,
            history: vec![],
        };
        let mut model = EfficiencyModel::default();

        // Pretend current month is July (7)
        // To test without chrono, we'd need to mock. Here we'll just call the function
        // and check that recharge rate increased if we set the month manually.
        // For simplicity, we skip the month check and just test that the flag is set.
        integrate_oodham_knowledge(&knowledge, &mut model, &mut aquifer);
        assert!(model.oodham_integrated);
        // Recharge rate should have been increased by 10%
        assert!((aquifer.recharge_rate_m3_per_day - 110.0).abs() < 1e-6);
    }
}
