// Normalized risk coordinates r_x in [0,1] for environmental parameters

#[derive(Clone, Debug)]
pub struct RiskCoordinate {
    param: String,
    value: f64, // Normalized [0,1]
    gradient: f64, // For adjoint sensitivity
}

impl RiskCoordinate {
    pub fn new(param: &str, raw_value: f64, min: f64, max: f64) -> Self {
        let normalized = (raw_value - min) / (max - min).max(1e-10);
        Self {
            param: param.to_string(),
            value: normalized.clamp(0.0, 1.0),
            gradient: 0.0,
        }
    }

    pub fn value(&self) -> f64 { self.value }
    pub fn is_safe(&self, threshold: f64) -> bool { self.value <= threshold }
}
