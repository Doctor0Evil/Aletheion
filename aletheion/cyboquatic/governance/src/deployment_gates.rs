// K/E/R thresholds for production activation

#[derive(Clone, Debug)]
pub struct KERThresholds {
    pub k_min: f64, // 0.90
    pub e_min: f64, // 0.90
    pub r_max: f64, // 0.13
}

impl KERThresholds {
    pub fn default_2026_targets() -> Self {
        Self { k_min: 0.90, e_min: 0.90, r_max: 0.13 }
    }

    pub fn check(&self, state: &EnvironmentalState) -> Result<(), String> {
        if state.k_score < self.k_min || state.e_score < self.e_min || state.r_score > self.r_max {
            Err("KER thresholds violated".to_string())
        } else {
            Ok(())
        }
    }
}

// Supporting types from core
use aletheion_core::control_action::ControlAction;
use aletheion_core::operational_feedback::OperationalFeedback;
