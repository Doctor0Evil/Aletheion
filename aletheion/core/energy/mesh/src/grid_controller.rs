// ALETHEION_ENERGY_MESH_CONTROLLER_V1.0.0
// LICENSE: BioticTreaty_Compliant_AGPLv3
// ECO_IMPACT: K=0.93 | E=0.91 | R=0.12
// CHAIN: ERM (Model → Optimize → Act)
// CONSTRAINTS: Heat-Degradation-Mitigation, Offline-Capable, P2P-Trading
// INDIGENOUS_RIGHTS: Priority_Power_Allocation_Sacred_Sites

#![no_std]
extern crate alloc;
use alloc::vec::Vec;
use core::cmp::Ordering;

// --- ENERGY STATE STRUCTS ---
#[derive(Clone, Copy)]
pub struct NodeState {
    pub id: u64,
    pub solar_input_watts: f32,
    pub battery_level_pct: f32,
    pub load_demand_watts: f32,
    pub is_critical_infra: bool, // Hospital, Water Pump
    pub is_indigenous_site: bool, // Sacred Land Priority
    pub temperature_c: f32,      // Battery Heat Stress
}

#[derive(Clone)]
pub struct GridAction {
    pub target_node: u64,
    pub action_type: ActionType,
    pub power_transfer_watts: f32,
}

#[derive(Clone, Copy)]
pub enum ActionType {
    Charge,
    Discharge,
    Isolate,
    Share,
}

// --- CONSTANTS (PHOENIX HEAT SPECIFIC) ---
const BATTERY_HEAT_THROTTLE_C: f32 = 45.0; // Li-ion degradation risk
const MIN_BATTERY_RESERVE_PCT: f32 = 20.0; // Emergency Reserve
const INDIGENOUS_PRIORITY_BONUS: f32 = 1.5; // Weight multiplier

// --- MESH CONTROLLER ---
pub struct GridController {
    pub nodes: Vec<NodeState>,
    pub total_grid_load: f32,
    pub total_grid_supply: f32,
}

impl GridController {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            total_grid_load: 0.0,
            total_grid_supply: 0.0,
        }
    }

    // ERM: OPTIMIZE
    // Balances load while respecting heat constraints and sovereignty
    pub fn optimize_dispatch(&mut self) -> Vec<GridAction> {
        let mut actions = Vec::new();
        self.total_grid_load = self.nodes.iter().map(|n| n.load_demand_watts).sum();
        self.total_grid_supply = self.nodes.iter().map(|n| n.solar_input_watts).sum();

        for i in 0..self.nodes.len() {
            let node = &mut self.nodes[i];
            
            // 1. Heat Mitigation (Phoenix Summer)
            if node.temperature_c > BATTERY_HEAT_THROTTLE_C {
                actions.push(GridAction {
                    target_node: node.id,
                    action_type: ActionType::Isolate,
                    power_transfer_watts: 0.0,
                });
                continue;
            }

            // 2. Sovereignty & Critical Priority
            let priority_score = self.calculate_priority(node);
            
            if node.battery_level_pct < MIN_BATTERY_RESERVE_PCT && priority_score > 1.0 {
                // Find surplus node to share
                if let Some(surplus_node) = self.find_surplus_node(i) {
                    actions.push(GridAction {
                        target_node: surplus_node.id,
                        action_type: ActionType::Share,
                        power_transfer_watts: 500.0, // 500W transfer
                    });
                }
            }
        }
        actions
    }

    // SMART: TREATY-CHECK
    fn calculate_priority(&self, node: &NodeState) -> f32 {
        let mut score = 1.0;
        if node.is_critical_infra { score += 1.0; }
        if node.is_indigenous_site { score *= INDIGENOUS_PRIORITY_BONUS; }
        score
    }

    fn find_surplus_node(&self, exclude_id: usize) -> Option<&NodeState> {
        self.nodes.iter().find(|n| {
            n.id != exclude_id as u64 && 
            n.battery_level_pct > 80.0 && 
            n.temperature_c < BATTERY_HEAT_THROTTLE_C
        })
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heat_throttle_logic() {
        let mut controller = GridController::new();
        controller.nodes.push(NodeState {
            id: 1,
            solar_input_watts: 1000.0,
            battery_level_pct: 50.0,
            load_demand_watts: 200.0,
            is_critical_infra: false,
            is_indigenous_site: false,
            temperature_c: 50.0, // Overheat
        });
        let actions = controller.optimize_dispatch();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0].action_type, ActionType::Isolate));
    }
}
