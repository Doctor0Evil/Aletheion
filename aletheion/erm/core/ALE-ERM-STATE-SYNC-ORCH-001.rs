use std::time::Duration;

#[derive(Debug, Clone)]
pub struct EdgeSnapshot {
    pub ts_millis: i64,
    pub source_id: String,
    pub payload_json: String,
    pub channel: EdgeChannel,
}

#[derive(Debug, Clone)]
pub enum EdgeChannel {
    Water,
    Thermal,
    Energy,
    Materials,
}

#[derive(Debug, Clone)]
pub struct StateSyncConfig {
    pub max_lag_millis: i64,
    pub water_loss_tolerance: f64,
    pub thermal_loss_tolerance: f64,
    pub energy_loss_tolerance: f64,
    pub materials_loss_tolerance: f64,
}

#[derive(Debug, Clone)]
pub struct StateSyncResult {
    pub applied: bool,
    pub reason: Option<String>,
}

pub trait StateModel {
    fn apply_snapshot(&mut self, snap: &EdgeSnapshot) -> Result<(), String>;
    fn reconcile(&mut self) -> Result<(), String>;
}

pub struct StateSyncOrchestrator<M: StateModel> {
    pub model: M,
    pub cfg: StateSyncConfig,
}

impl<M: StateModel> StateSyncOrchestrator<M> {
    pub fn new(model: M, cfg: StateSyncConfig) -> Self {
        Self { model, cfg }
    }

    pub fn handle_snapshot(&mut self, snap: EdgeSnapshot) -> StateSyncResult {
        if self.is_stale(&snap) {
            return StateSyncResult {
                applied: false,
                reason: Some(format!("stale snapshot from {}", snap.source_id)),
            };
        }

        if let Err(e) = self.model.apply_snapshot(&snap) {
            return StateSyncResult {
                applied: false,
                reason: Some(format!("apply failed: {}", e)),
            };
        }

        match self.model.reconcile() {
            Ok(()) => StateSyncResult {
                applied: true,
                reason: None,
            },
            Err(e) => StateSyncResult {
                applied: false,
                reason: Some(format!("reconcile failed: {}", e)),
            },
        }
    }

    fn is_stale(&self, snap: &EdgeSnapshot) -> bool {
        let now = Self::now_millis();
        now - snap.ts_millis > self.cfg.max_lag_millis
    }

    fn now_millis() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_millis(0));
        now.as_millis() as i64
    }
}
