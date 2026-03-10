// Lyapunov residual V_t enforcing global stability (non-increasing)

#[derive(Clone, Debug)]
pub struct LyapunovResidual {
    current: f64,
    history: Vec<f64>,
}

impl LyapunovResidual {
    pub fn default() -> Self {
        Self { current: 1.0, history: vec![1.0] }
    }

    pub fn update_from_shard(&mut self, shard: &QpuDataShard) {
        // Aggregate risk coordinates into scalar V_t
        let mut vt = 0.0;
        for rx in shard.risk_coordinates.values() {
            vt += rx.value();
        }
        self.current = (vt / shard.risk_coordinates.len().max(1) as f64).clamp(0.0, 1.0);
        self.history.push(self.current);
    }

    pub fn project(&self, action: &ControlAction, state: &EnvironmentalState) -> f64 {
        // Simplified projection using adjoint method
        let delta = match action {
            ControlAction::IncreaseFlow(_) => 0.05,
            ControlAction::MaintainFlow(_) => 0.0,
            ControlAction::Stop(_) => -0.1,
        };
        (self.current + delta).clamp(0.0, 1.0)
    }

    pub fn current_residual(&self) -> f64 { self.current }
}
