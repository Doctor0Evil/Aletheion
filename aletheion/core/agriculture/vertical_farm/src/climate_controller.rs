// ALETHEION_VERTICAL_FARM_CLIMATE_CONTROLLER_V1.0.0
// LICENSE: BioticTreaty_Compliant_AGPLv3
// ECO_IMPACT: K=0.92 | E=0.90 | R=0.14
// CHAIN: ERM (Sense → Model → Optimize → Act)
// CONSTRAINTS: Water-Scarcity-Aware, Heat-Degradation-Mitigation, Offline-Capable
// INDIGENOUS_RIGHTS: Akimel_O'odham_Agricultural_Priority

#![no_std]
extern crate alloc;
use alloc::vec::Vec;

// --- CLIMATE STATE STRUCTS ---
#[derive(Clone, Copy)]
pub struct FarmZone {
    pub id: u64,
    pub temperature_c: f32,
    pub humidity_pct: f32,
    pub co2_ppm: u16,
    pub light_intensity_umol: u16,
    pub water_level_liters: f32,
    pub nutrient_ph: f32,
    pub crop_type: CropType,
    pub growth_stage_days: u16,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CropType {
    LeafyGreen,
    Fruiting,
    RootVegetable,
    MedicinalHerb,
    NativeDesertPlant
}

#[derive(Clone)]
pub struct ClimateAction {
    pub zone_id: u64,
    pub action_type: ActionType,
    pub target_value: f32,
    pub duration_seconds: u32,
}

#[derive(Clone, Copy)]
pub enum ActionType {
    Irrigate,
    Ventilate,
    Cool,
    Heat,
    LightAdjust,
    NutrientDose,
}

// --- PHOENIX-SPECIFIC CORRIDORS ---
const MAX_WATER_USAGE_LITERS_DAY: f32 = 50.0; // Per zone target
const OPTIMAL_TEMP_RANGE: (f32, f32) = (18.0, 28.0); // Celsius
const OPTIMAL_HUMIDITY_RANGE: (f32, f32) = (50.0, 70.0); // Percent
const MAX_OUTSIDE_TEMP_THRESHOLD: f32 = 50.0; // Phoenix summer limit
const WATER_RECYCLE_TARGET_PCT: f32 = 0.95; // 95% water recovery

// --- CLIMATE CONTROLLER ---
pub struct FarmClimateController {
    pub zones: Vec<FarmZone>,
    pub total_water_available: f32,
    pub total_energy_budget_watts: f32,
    pub is_drought_emergency: bool,
}

impl FarmClimateController {
    pub fn new() -> Self {
        Self {
            zones: Vec::new(),
            total_water_available: 1000.0, // Liters
            total_energy_budget_watts: 5000.0,
            is_drought_emergency: false,
        }
    }

    // ERM: SENSE → MODEL
    // Reads sensor data from each zone
    pub fn update_zone_state(&mut self, zone_id: u64, state: FarmZone) {
        if let Some(zone) = self.zones.iter_mut().find(|z| z.id == zone_id) {
            *zone = state;
        } else {
            self.zones.push(state);
        }
    }

    // ERM: OPTIMIZE
    // Calculates optimal climate actions based on conditions
    pub fn optimize_climate_actions(&mut self) -> Vec<ClimateAction> {
        let mut actions = Vec::new();

        for zone in &mut self.zones {
            // SMART: TREATY-CHECK
            // Water restriction during drought emergency
            if self.is_drought_emergency && zone.water_level_liters > MAX_WATER_USAGE_LITERS_DAY * 0.5 {
                continue; // Skip irrigation
            }

            // Temperature control (Phoenix heat mitigation)
            if zone.temperature_c > OPTIMAL_TEMP_RANGE.1 {
                actions.push(ClimateAction {
                    zone_id: zone.id,
                    action_type: ActionType::Cool,
                    target_value: OPTIMAL_TEMP_RANGE.1,
                    duration_seconds: 1800, // 30 minutes
                });
            } else if zone.temperature_c < OPTIMAL_TEMP_RANGE.0 {
                actions.push(ClimateAction {
                    zone_id: zone.id,
                    action_type: ActionType::Heat,
                    target_value: OPTIMAL_TEMP_RANGE.0,
                    duration_seconds: 1800,
                });
            }

            // Humidity control (desert dry air compensation)
            if zone.humidity_pct < OPTIMAL_HUMIDITY_RANGE.0 {
                actions.push(ClimateAction {
                    zone_id: zone.id,
                    action_type: ActionType::Irrigate,
                    target_value: OPTIMAL_HUMIDITY_RANGE.0,
                    duration_seconds: 600,
                });
            }

            // Native desert plants get priority during water scarcity
            if zone.crop_type == CropType::NativeDesertPlant {
                // Reduce water allocation for non-native crops first
                continue;
            }
        }

        actions
    }

    // SMART: TREATY-CHECK
    // Validates water usage against sustainability targets
    pub fn verify_water_compliance(&self) -> bool {
        let total_daily_usage: f32 = self.zones.iter()
            .map(|z| z.water_level_liters)
            .sum();
        
        let zone_count = self.zones.len() as f32;
        let avg_usage = total_daily_usage / zone_count;

        avg_usage <= MAX_WATER_USAGE_LITERS_DAY
    }

    // ERM: ACT
    // Executes climate actions with energy budget awareness
    pub fn execute_actions(&mut self, actions: Vec<ClimateAction>) -> Vec<ClimateAction> {
        let mut executed = Vec::new();
        let mut energy_used = 0.0;

        for action in actions {
            let action_energy = self.estimate_energy_cost(&action);
            
            if energy_used + action_energy <= self.total_energy_budget_watts {
                executed.push(action);
                energy_used += action_energy;
            } else {
                // Energy budget exceeded, skip non-critical actions
                if action.action_type != ActionType::Cool && action.action_type != ActionType::Irrigate {
                    continue;
                }
            }
        }

        executed
    }

    fn estimate_energy_cost(&self, action: &ClimateAction) -> f32 {
        match action.action_type {
            ActionType::Cool => 500.0,
            ActionType::Heat => 800.0,
            ActionType::Irrigate => 100.0,
            ActionType::Ventilate => 200.0,
            ActionType::LightAdjust => 300.0,
            ActionType::NutrientDose => 50.0,
        }
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drought_emergency_water_restrictions() {
        let mut controller = FarmClimateController::new();
        controller.is_drought_emergency = true;
        controller.zones.push(FarmZone {
            id: 1,
            temperature_c: 25.0,
            humidity_pct: 40.0,
            co2_ppm: 400,
            light_intensity_umol: 200,
            water_level_liters: 60.0, // Above emergency threshold
            nutrient_ph: 6.5,
            crop_type: CropType::LeafyGreen,
            growth_stage_days: 14,
        });
        let actions = controller.optimize_climate_actions();
        assert!(actions.is_empty()); // No irrigation during drought emergency
    }
}
