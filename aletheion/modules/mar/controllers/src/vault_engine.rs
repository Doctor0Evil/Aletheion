#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::corridor_kernel::{EcosafetyState, CorridorEnforced, RiskCoordinate, LyapunovResidual};

pub const MAR_VAULT_LENGTH_M: f64 = 30.0;
pub const MAR_VAULT_WIDTH_M: f64 = 4.0;
pub const MAR_VAULT_DEPTH_M: f64 = 3.5;
pub const MAR_MAX_FLOW_M3H: f64 = 150.0;
pub const SAT_CORRIDOR_VERSION: u32 = 20260310;

#[repr(u16)]
pub enum MarRiskId {
    PFAS = 1,
    Pharmaceuticals = 2,
    NutrientsN = 3,
    NutrientsP = 4,
    Temperature = 5,
    FoulingIndex = 6,
    SurchargeHead = 7,
    Turbidity = 8,
    DO = 9,
    pH = 10,
}

#[derive(Clone, Debug)]
pub struct SatCorridor {
    pub pfas_ngl: f64,
    pub pharma_ngl: f64,
    pub nitrogen_mgl: f64,
    pub phosphorus_mgl: f64,
    pub temp_celsius: f64,
    pub fouling_0_1: f64,
    pub surcharge_m: f64,
    pub turbidity_ntu: f64,
    pub do_mgl: f64,
    pub ph: f64,
}

impl SatCorridor {
    pub const fn thresholds() -> Self {
        Self {
            pfas_ngl: 10.0,
            pharma_ngl: 50.0,
            nitrogen_mgl: 10.0,
            phosphorus_mgl: 1.0,
            temp_celsius: 35.0,
            fouling_0_1: 0.7,
            surcharge_m: 0.5,
            turbidity_ntu: 5.0,
            do_mgl: 2.0,
            ph: 8.5,
        }
    }
    pub fn to_risk_coordinates(&self, ts: u64) -> [RiskCoordinate; 10] {
        [
            RiskCoordinate::new(MarRiskId::PFAS as u16, self.pfas_ngl / 100.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::Pharmaceuticals as u16, self.pharma_ngl / 200.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::NutrientsN as u16, self.nitrogen_mgl / 15.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::NutrientsP as u16, self.phosphorus_mgl / 2.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::Temperature as u16, self.temp_celsius / 45.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::FoulingIndex as u16, self.fouling_0_1, 1.0, ts),
            RiskCoordinate::new(MarRiskId::SurchargeHead as u16, self.surcharge_m / 1.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::Turbidity as u16, self.turbidity_ntu / 10.0, 1.0, ts),
            RiskCoordinate::new(MarRiskId::DO as u16, if self.do_mgl < 2.0 { (2.0 - self.do_mgl) / 2.0 } else { 0.0 }, 1.0, ts),
            RiskCoordinate::new(MarRiskId::pH as u16, if self.ph > 8.5 { (self.ph - 7.0) / 3.0 } else { (7.0 - self.ph) / 3.0 }, 1.0, ts),
        ]
    }
}

pub struct MarVaultController {
    pub vault_id: u32,
    pub flow_rate_m3h: f64,
    pub target_flow_m3h: f64,
    pub pump_speed_pct: f64,
    pub valve_position_pct: f64,
    pub backwash_interval_h: u32,
    pub last_backwash_ns: u64,
    pub ecosafety: EcosafetyState,
    pub sat_current: SatCorridor,
    pub sat_history: [SatCorridor; 24],
    pub history_idx: usize,
}

impl MarVaultController {
    pub fn new(vault_id: u32, epoch_ns: u64) -> Self {
        let mut ecosafety = EcosafetyState::new(epoch_ns);
        let thresholds = SatCorridor::thresholds();
        for rc in thresholds.to_risk_coordinates(epoch_ns) {
            let _ = ecosafety.corridors.insert(rc);
        }
        Self {
            vault_id,
            flow_rate_m3h: 0.0,
            target_flow_m3h: 100.0,
            pump_speed_pct: 0.0,
            valve_position_pct: 0.0,
            backwash_interval_h: 6,
            last_backwash_ns: epoch_ns,
            ecosafety,
            sat_current: SatCorridor::thresholds(),
            sat_history: [SatCorridor::thresholds(); 24],
            history_idx: 0,
        }
    }
    pub fn update_sensors(&mut self, sat: SatCorridor, now_ns: u64) {
        self.sat_current = sat;
        self.sat_history[self.history_idx] = sat;
        self.history_idx = (self.history_idx + 1) % 24;
        for rc in sat.to_risk_coordinates(now_ns) {
            let _ = self.ecosafety.corridors.insert(rc);
        }
        let vt = self.compute_lyapunov_residual();
        self.ecosafety.lyapunov = LyapunovResidual::new(vt.0, vt.1, now_ns);
    }
    fn compute_lyapunov_residual(&self) -> (f64, f64) {
        let mut vt = 0.0;
        for i in 0..self.ecosafety.corridors.count {
            let rc = self.ecosafety.corridors.coordinates[i].unwrap();
            vt += rc.normalized().powi(2);
        }
        let prev_vt = if self.history_idx > 0 {
            let prev = &self.sat_history[(self.history_idx + 23) % 24];
            let mut sum = 0.0;
            for rc in prev.to_risk_coordinates(0) {
                sum += rc.normalized().powi(2);
            }
            sum
        } else { vt };
        let vt_dot = (vt - prev_vt) / 3600.0;
        (vt, vt_dot)
    }
    pub fn compute_actuation(&mut self, now_ns: u64) -> (f64, f64) {
        if !self.ecosafety.check(now_ns) {
            self.pump_speed_pct = 0.0;
            self.valve_position_pct = 0.0;
            self.flow_rate_m3h = 0.0;
            return (0.0, 0.0);
        }
        let allowed_power = self.ecosafety.allowed_power(self.target_flow_m3h);
        let demand_factor = if self.sat_current.fouling_0_1 > 0.5 { 0.7 }
            else if self.sat_current.turbidity_ntu > 3.0 { 0.8 }
            else { 1.0 };
        self.pump_speed_pct = (allowed_power * demand_factor / MAR_MAX_FLOW_M3H * 100.0).min(100.0);
        self.valve_position_pct = self.pump_speed_pct;
        self.flow_rate_m3h = allowed_power * demand_factor;
        (self.pump_speed_pct, self.valve_position_pct)
    }
    pub fn backwash_required(&self, now_ns: u64) -> bool {
        let elapsed_h = (now_ns - self.last_backwash_ns) / 3_600_000_000_000;
        elapsed_h >= self.backwash_interval_h as u64 || self.sat_current.fouling_0_1 > 0.8
    }
    pub fn execute_backwash(&mut self, now_ns: u64) {
        self.last_backwash_ns = now_ns;
        self.pump_speed_pct = 120.0;
        self.valve_position_pct = 100.0;
    }
}

impl CorridorEnforced for MarVaultController {
    fn ecosafety_state(&self) -> &EcosafetyState { &self.ecosafety }
    fn ecosafety_state_mut(&mut self) -> &mut EcosafetyState { &mut self.ecosafety }
}
