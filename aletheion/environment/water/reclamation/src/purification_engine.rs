#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::corridor_kernel::{EcosafetyState, CorridorEnforced, RiskCoordinate, LyapunovResidual};

pub const RECLAMATION_VERSION: u32 = 20260310;
pub const MAX_TREATMENT_STAGES: usize = 8;
pub const TARGET_RECLAMATION_EFFICIENCY: f64 = 0.97;
pub const MAX_FLOW_RATE_M3H: f64 = 500.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreatmentStage {
    Screening = 0, Coagulation = 1, Flocculation = 2, Sedimentation = 3,
    Filtration = 4, RO_Membrane = 5, UV_Disinfection = 6, Advanced_Oxidation = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct WaterQualityParameters {
    pub turbidity_ntu: f64,
    pub total_dissolved_solids_mgl: f64,
    pub ph: f64,
    pub dissolved_oxygen_mgl: f64,
    pub chemical_oxygen_demand_mgl: f64,
    pub biological_oxygen_demand_mgl: f64,
    pub total_nitrogen_mgl: f64,
    pub total_phosphorus_mgl: f64,
    pub e_coli_cfu_100ml: u64,
    pub pfas_ngl: f64,
    pub pharmaceuticals_ngl: f64,
    pub heavy_metals_ugl: f64,
    pub temperature_celsius: f64,
    pub conductivity_uscm: f64,
}

impl WaterQualityParameters {
    pub const fn potable_thresholds() -> Self {
        Self {
            turbidity_ntu: 0.3,
            total_dissolved_solids_mgl: 500.0,
            ph: 7.5,
            dissolved_oxygen_mgl: 8.0,
            chemical_oxygen_demand_mgl: 10.0,
            biological_oxygen_demand_mgl: 3.0,
            total_nitrogen_mgl: 5.0,
            total_phosphorus_mgl: 0.5,
            e_coli_cfu_100ml: 0,
            pfas_ngl: 10.0,
            pharmaceuticals_ngl: 50.0,
            heavy_metals_ugl: 5.0,
            temperature_celsius: 20.0,
            conductivity_uscm: 500.0,
        }
    }
    pub fn to_risk_coordinates(&self, ts: u64) -> [RiskCoordinate; 12] {
        [
            RiskCoordinate::new(201, self.turbidity_ntu / 1.0, 1.0, ts),
            RiskCoordinate::new(202, self.total_dissolved_solids_mgl / 1000.0, 1.0, ts),
            RiskCoordinate::new(203, if self.ph < 6.5 || self.ph > 8.5 { 0.8 } else { 0.2 }, 1.0, ts),
            RiskCoordinate::new(204, if self.dissolved_oxygen_mgl < 5.0 { 0.7 } else { 0.1 }, 1.0, ts),
            RiskCoordinate::new(205, self.chemical_oxygen_demand_mgl / 50.0, 1.0, ts),
            RiskCoordinate::new(206, self.biological_oxygen_demand_mgl / 10.0, 1.0, ts),
            RiskCoordinate::new(207, self.total_nitrogen_mgl / 15.0, 1.0, ts),
            RiskCoordinate::new(208, self.total_phosphorus_mgl / 2.0, 1.0, ts),
            RiskCoordinate::new(209, if self.e_coli_cfu_100ml > 0 { 1.0 } else { 0.0 }, 1.0, ts),
            RiskCoordinate::new(210, self.pfas_ngl / 50.0, 1.0, ts),
            RiskCoordinate::new(211, self.pharmaceuticals_ngl / 200.0, 1.0, ts),
            RiskCoordinate::new(212, self.heavy_metals_ugl / 20.0, 1.0, ts),
        ]
    }
    pub fn compute_efficiency(&self, influent: &WaterQualityParameters) -> f64 {
        if influent.turbidity_ntu == 0.0 { return 1.0; }
        let turbidity_removal = 1.0 - (self.turbidity_ntu / influent.turbidity_ntu);
        let tds_removal = 1.0 - (self.total_dissolved_solids_mgl / influent.total_dissolved_solids_mgl.max(1.0));
        let cod_removal = 1.0 - (self.chemical_oxygen_demand_mgl / influent.chemical_oxygen_demand_mgl.max(1.0));
        (turbidity_removal + tds_removal + cod_removal) / 3.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StagePerformance {
    pub stage: TreatmentStage,
    pub flow_rate_m3h: f64,
    pub pressure_bar: f64,
    pub energy_kwh_m3: f64,
    pub removal_efficiency_0_1: f64,
    pub fouling_index_0_1: f64,
    pub last_cleaning_ns: u64,
    pub operational: bool,
}

pub struct WaterReclamationController {
    pub plant_id: u32,
    pub total_capacity_m3h: f64,
    pub current_flow_m3h: f64,
    pub target_flow_m3h: f64,
    pub stages: [Option<StagePerformance>; MAX_TREATMENT_STAGES],
    pub stage_count: usize,
    pub influent_quality: WaterQualityParameters,
    pub effluent_quality: WaterQualityParameters,
    pub ecosafety: EcosafetyState,
    pub total_volume_processed_m3: f64,
    pub total_energy_kwh: f64,
    pub reclamation_efficiency: f64,
    public_alert_active: bool,
    public_alert_reason: [u8; 64],
    last_maintenance_ns: u64,
    audit_checksum: u64,
}

impl WaterReclamationController {
    pub fn new(plant_id: u32, capacity_m3h: f64, epoch_ns: u64) -> Self {
        let mut ecosafety = EcosafetyState::new(epoch_ns);
        let thresholds = WaterQualityParameters::potable_thresholds();
        for rc in thresholds.to_risk_coordinates(epoch_ns) {
            let _ = ecosafety.corridors.insert(rc);
        }
        Self {
            plant_id,
            total_capacity_m3h: capacity_m3h,
            current_flow_m3h: 0.0,
            target_flow_m3h: capacity_m3h * 0.8,
            stages: Default::default(),
            stage_count: 0,
            influent_quality: WaterQualityParameters::potable_thresholds(),
            effluent_quality: WaterQualityParameters::potable_thresholds(),
            ecosafety,
            total_volume_processed_m3: 0.0,
            total_energy_kwh: 0.0,
            reclamation_efficiency: 0.0,
            public_alert_active: false,
            public_alert_reason: [0u8; 64],
            last_maintenance_ns: epoch_ns,
            audit_checksum: 0,
        }
    }
    pub fn add_treatment_stage(&mut self, stage: StagePerformance) -> Result<(), &'static str> {
        if self.stage_count >= MAX_TREATMENT_STAGES {
            return Err("STAGE_LIMIT_EXCEEDED");
        }
        self.stages[self.stage_count] = Some(stage);
        self.stage_count += 1;
        self.update_audit_checksum();
        Ok(())
    }
    pub fn update_quality_sensors(
        &mut self,
        influent: WaterQualityParameters,
        effluent: WaterQualityParameters,
        now_ns: u64
    ) {
        self.influent_quality = influent;
        self.effluent_quality = effluent;
        self.reclamation_efficiency = effluent.compute_efficiency(&influent);
        for rc in effluent.to_risk_coordinates(now_ns) {
            let _ = self.ecosafety.corridors.insert(rc);
        }
        let vt = self.compute_lyapunov_residual(&effluent);
        self.ecosafety.lyapunov = LyapunovResidual::new(vt.0, vt.1, now_ns);
        if self.reclamation_efficiency < TARGET_RECLAMATION_EFFICIENCY {
            self.public_alert_active = true;
            self.public_alert_reason[..28].copy_from_slice(b"EFFICIENCY_BELOW_TARGET");
        } else {
            self.public_alert_active = false;
        }
    }
    fn compute_lyapunov_residual(&self, quality: &WaterQualityParameters) -> (f64, f64) {
        let mut vt = 0.0;
        for rc in quality.to_risk_coordinates(0) {
            vt += rc.normalized().powi(2);
        }
        let prev_vt = vt * 0.95;
        let vt_dot = (vt - prev_vt) / 3600.0;
        (vt, vt_dot)
    }
    pub fn compute_actuation(&mut self, now_ns: u64) -> (f64, bool) {
        if !self.ecosafety.check(now_ns) {
            self.current_flow_m3h = 0.0;
            self.public_alert_active = true;
            return (0.0, true);
        }
        let allowed_power = self.ecosafety.allowed_power(self.target_flow_m3h);
        let stage_limit = self.stages.iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.operational)
            .map(|s| s.flow_rate_m3h)
            .fold(f64::MAX, f64::min);
        self.current_flow_m3h = allowed_power.min(stage_limit).min(self.total_capacity_m3h);
        let energy_rate = self.stages.iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.energy_kwh_m3 * s.flow_rate_m3h)
            .sum::<f64>();
        self.total_energy_kwh += energy_rate;
        self.total_volume_processed_m3 += self.current_flow_m3h / 3600.0;
        self.update_audit_checksum();
        (self.current_flow_m3h, self.public_alert_active)
    }
    pub fn stage_cleaning_required(&self, stage_idx: usize, now_ns: u64) -> bool {
        if stage_idx >= self.stage_count { return false; }
        if let Some(ref stage) = self.stages[stage_idx] {
            let elapsed_h = (now_ns - stage.last_cleaning_ns) / 3600000000000;
            elapsed_h > 168 || stage.fouling_index_0_1 > 0.7
        } else {
            false
        }
    }
    pub fn execute_stage_cleaning(&mut self, stage_idx: usize, now_ns: u64) -> Result<(), &'static str> {
        if stage_idx >= self.stage_count {
            return Err("STAGE_INDEX_OUT_OF_BOUNDS");
        }
        if let Some(ref mut stage) = self.stages[stage_idx] {
            stage.last_cleaning_ns = now_ns;
            stage.fouling_index_0_1 = 0.1;
            stage.operational = true;
            self.last_maintenance_ns = now_ns;
            self.update_audit_checksum();
            Ok(())
        } else {
            Err("STAGE_NOT_FOUND")
        }
    }
    pub fn get_plant_status(&self, now_ns: u64) -> PlantStatus {
        let operational_stages = self.stages.iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.operational)
            .count();
        let avg_stage_efficiency = self.stages.iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.removal_efficiency_0_1)
            .sum::<f64>() / self.stage_count.max(1) as f64;
        let specific_energy = if self.total_volume_processed_m3 > 0.0 {
            self.total_energy_kwh / self.total_volume_processed_m3
        } else { 0.0 };
        PlantStatus {
            plant_id: self.plant_id,
            current_flow_m3h: self.current_flow_m3h,
            capacity_utilization_pct: (self.current_flow_m3h / self.total_capacity_m3h * 100.0).min(100.0),
            operational_stages,
            total_stages: self.stage_count,
            reclamation_efficiency: self.reclamation_efficiency,
            avg_stage_efficiency,
            specific_energy_kwh_m3: specific_energy,
            public_alert_active: self.public_alert_active,
            ecosafety_halt: self.ecosafety.halt_required,
            total_volume_processed_m3: self.total_volume_processed_m3,
            total_energy_kwh: self.total_energy_kwh,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.plant_id as u64).wrapping_mul((self.reclamation_efficiency * 1e6) as u64);
        sum ^= (self.total_volume_processed_m3 as u64);
        sum ^= (self.total_energy_kwh as u64);
        sum ^= if self.public_alert_active { 1u64 } else { 0u64 };
        for i in 0..self.stage_count {
            if let Some(ref stage) = self.stages[i] {
                sum ^= (stage.stage as u64).wrapping_mul((stage.fouling_index_0_1 * 1e6) as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.plant_id as u64).wrapping_mul((self.reclamation_efficiency * 1e6) as u64);
        sum ^= (self.total_volume_processed_m3 as u64);
        sum ^= (self.total_energy_kwh as u64);
        sum ^= if self.public_alert_active { 1u64 } else { 0u64 };
        for i in 0..self.stage_count {
            if let Some(ref stage) = self.stages[i] {
                sum ^= (stage.stage as u64).wrapping_mul((stage.fouling_index_0_1 * 1e6) as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct PlantStatus {
    pub plant_id: u32,
    pub current_flow_m3h: f64,
    pub capacity_utilization_pct: f64,
    pub operational_stages: usize,
    pub total_stages: usize,
    pub reclamation_efficiency: f64,
    pub avg_stage_efficiency: f64,
    pub specific_energy_kwh_m3: f64,
    pub public_alert_active: bool,
    pub ecosafety_halt: bool,
    pub total_volume_processed_m3: f64,
    pub total_energy_kwh: f64,
}

impl PlantStatus {
    pub fn performance_score(&self) -> f64 {
        let mut score = 0.0;
        score += self.reclamation_efficiency * 0.4;
        score += (self.operational_stages as f64 / self.total_stages.max(1) as f64) * 0.3;
        score += if self.specific_energy_kwh_m3 < 2.0 { 0.2 } else { 0.1 };
        score += if !self.public_alert_active && !self.ecosafety_halt { 0.1 } else { 0.0 };
        score.min(1.0)
    }
}
