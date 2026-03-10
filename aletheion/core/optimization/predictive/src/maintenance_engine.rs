// ALETHEION_PREDICTIVE_MAINTENANCE_ENGINE_V1.0.0
// LICENSE: BioticTreaty_Compliant_AGPLv3
// ECO_IMPACT: K=0.94 | E=0.92 | R=0.10
// CHAIN: ERM (Model → Optimize → Act)
// CONSTRAINTS: Heat-Degradation-Mitigation, Offline-Inference, Zero-Downtime
// INDIGENOUS_RIGHTS: Infrastructure_Priority_For_Sacred_Sites

#![no_std]
extern crate alloc;
use alloc::vec::Vec;

// --- ASSET STATE ---
#[derive(Clone)]
pub struct InfrastructureAsset {
    pub id: u64,
    pub asset_type: AssetType,
    pub temperature_c: f32,
    pub vibration_hz: f32,
    pub efficiency_pct: f32,
    pub age_days: u32,
    pub is_critical_for_indigenous_site: bool,
    pub last_maintenance_days: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AssetType {
    SolarPanel,
    BatteryStorage,
    WaterPump,
    HvacUnit,
    SensorNode,
}

// --- FAILURE PREDICTION ---
#[derive(Clone)]
pub struct FailurePrediction {
    pub asset_id: u64,
    pub failure_probability: f32, // 0.0 - 1.0
    pub estimated_days_to_failure: u32,
    pub recommended_action: Action,
    pub priority_score: f32,
}

#[derive(Clone, Copy)]
pub enum Action {
    Monitor,
    ScheduleMaintenance,
    ImmediateReplace,
    DerateOperation,
}

// --- PHOENIX-SPECIFIC THRESHOLDS ---
const MAX_OPERATING_TEMP_C: f32 = 85.0; // Electronics limit
const EFFICIENCY_DROP_THRESHOLD: f32 = 0.80; // 80% efficiency trigger
const INDIGENOUS_PRIORITY_MULTIPLIER: f32 = 2.0; // Boost priority for sacred sites
const HEAT_DEGRADATION_RATE: f32 = 0.001; // Per degree over 50°C

// --- MAINTENANCE ENGINE ---
pub struct PredictiveMaintenanceEngine {
    pub assets: Vec<InfrastructureAsset>,
    pub predictions: Vec<FailurePrediction>,
    pub maintenance_queue: Vec<u64>,
}

impl PredictiveMaintenanceEngine {
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            predictions: Vec::new(),
            maintenance_queue: Vec::new(),
        }
    }

    // ERM: MODEL
    // Ingests sensor data from all city infrastructure
    pub fn ingest_asset_data(&mut self, asset: InfrastructureAsset) {
        self.assets.push(asset);
    }

    // ERM: OPTIMIZE
    // Predicts failures using heat-degradation models
    pub fn predict_failures(&mut self) {
        self.predictions.clear();
        
        for asset in &self.assets {
            let failure_prob = self.calculate_failure_probability(asset);
            let days_to_failure = self.estimate_days_to_failure(asset, failure_prob);
            let action = self.determine_action(failure_prob, asset);
            let priority = self.calculate_priority(asset, failure_prob);

            if failure_prob > 0.3 { // Only track significant risks
                self.predictions.push(FailurePrediction {
                    asset_id: asset.id,
                    failure_probability: failure_prob,
                    estimated_days_to_failure: days_to_failure,
                    recommended_action: action,
                    priority_score: priority,
                });
            }
        }
        
        // Sort by priority (highest first)
        self.predictions.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());
    }

    // SMART: TREATY-CHECK
    fn calculate_priority(&self, asset: &InfrastructureAsset, failure_prob: f32) -> f32 {
        let mut priority = failure_prob;
        
        // Indigenous sites get priority maintenance
        if asset.is_critical_for_indigenous_site {
            priority *= INDIGENOUS_PRIORITY_MULTIPLIER;
        }
        
        // Critical infrastructure (water, hospital) boosted
        if matches!(asset.asset_type, AssetType::WaterPump | AssetType::HvacUnit) {
            priority *= 1.5;
        }
        
        priority
    }

    fn calculate_failure_probability(&self, asset: &InfrastructureAsset) -> f32 {
        let mut prob = 0.0;
        
        // Heat degradation model (Phoenix specific)
        if asset.temperature_c > 50.0 {
            prob += (asset.temperature_c - 50.0) * HEAT_DEGRADATION_RATE;
        }
        
        // Age factor
        prob += (asset.age_days as f32) * 0.0001;
        
        // Efficiency drop
        if asset.efficiency_pct < EFFICIENCY_DROP_THRESHOLD {
            prob += 0.2;
        }
        
        prob.min(1.0)
    }

    fn estimate_days_to_failure(&self, asset: &InfrastructureAsset, prob: f32) -> u32 {
        if prob > 0.8 { return 0; }
        if prob > 0.5 { return 7; }
        if prob > 0.3 { return 30; }
        365 // Default safe
    }

    fn determine_action(&self, prob: f32, asset: &InfrastructureAsset) -> Action {
        if prob > 0.8 { return Action::ImmediateReplace; }
        if prob > 0.5 { return Action::ScheduleMaintenance; }
        if asset.temperature_c > MAX_OPERATING_TEMP_C { return Action::DerateOperation; }
        Action::Monitor
    }

    // ERM: ACT
    // Generates maintenance queue based on predictions
    pub fn generate_maintenance_queue(&mut self) -> Vec<u64> {
        self.maintenance_queue.clear();
        for pred in &self.predictions {
            if pred.recommended_action != Action::Monitor {
                self.maintenance_queue.push(pred.asset_id);
            }
        }
        self.maintenance_queue.clone()
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indigenous_site_priority() {
        let mut engine = PredictiveMaintenanceEngine::new();
        let asset = InfrastructureAsset {
            id: 1,
            asset_type: AssetType::WaterPump,
            temperature_c: 60.0,
            vibration_hz: 10.0,
            efficiency_pct: 0.85,
            age_days: 1000,
            is_critical_for_indigenous_site: true,
            last_maintenance_days: 200,
        };
        engine.ingest_asset_data(asset);
        engine.predict_failures();
        
        assert!(!engine.predictions.is_empty());
        assert!(engine.predictions[0].priority_score > 0.5); // Boosted priority
    }
}
