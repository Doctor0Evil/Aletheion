
use crate::types::{
    CityObjectId,
    RiskScalar,
    EcosafetyCorridorSet,
    EcosafetyGuardPolicy,
    EcosafetyStateSnapshot,
    LyapunovWeightSet,
};

#[derive(Clone, Debug)]
pub struct CityObjectGuard {
    pub object_id: CityObjectId,
    pub corridors: EcosafetyCorridorSet,
    pub policy: EcosafetyGuardPolicy,
    pub weights: LyapunovWeightSet,
}

#[derive(Clone, Debug)]
pub struct GuardInputs {
    pub r_degrade: RiskScalar,
    pub r_residualmass: RiskScalar,
    pub r_microplastics: RiskScalar,
    pub r_tox_acute: RiskScalar,
    pub r_tox_chronic: RiskScalar,
    pub r_shear: RiskScalar,
    pub r_habitatload: RiskScalar,
    pub swarm_coverage_c_t: RiskScalar,
    pub agent_density_rho: f64,
    pub agent_density_max: f64,
    pub last_snapshot: Option<EcosafetyStateSnapshot>,
}

#[derive(Clone, Debug)]
pub enum GuardDecision {
    Allow(EcosafetyStateSnapshot),
    Derate(EcosafetyStateSnapshot),
    Stop(EcosafetyStateSnapshot),
}

impl CityObjectGuard {
    pub fn evaluate(&self, inputs: GuardInputs, ts_utc: i64) -> GuardDecision {
        if self.policy.rule_no_corridor_no_build && self.corridors.is_empty() {
            return GuardDecision::Stop(self.zero_snapshot(ts_utc));
        }

        let r_scalar = Self::aggregate_risk_scalar(&inputs);
        let v_obj_t = self.compute_v_obj(r_scalar, inputs.swarm_coverage_c_t, inputs.agent_density_rho, inputs.agent_density_max);

        let snapshot = EcosafetyStateSnapshot {
            id: Default::default(),
            object_id: self.object_id,
            timestamp_utc: ts_utc,
            risk_scalar_r_t: r_scalar,
            swarm_coverage_c_t: inputs.swarm_coverage_c_t,
            agent_density_rho: inputs.agent_density_rho,
            agent_density_max: inputs.agent_density_max,
            v_obj_t,
            lyapunov_weightset_id: self.weights.id,
        };

        if !self.corridors.within_all(&snapshot) {
            return match self.policy.rule_violated_corridor_action {
                crate::types::GuardViolationAction::DERATE => GuardDecision::Derate(snapshot),
                crate::types::GuardViolationAction::STOP => GuardDecision::Stop(snapshot),
                crate::types::GuardViolationAction::ALERT_AND_STOP => GuardDecision::Stop(snapshot),
            };
        }

        if self.policy.rule_nonincreasing_vobj_required {
            if let Some(prev) = inputs.last_snapshot {
                if v_obj_t > prev.v_obj_t + self.corridors.v_obj_corridor.hard_stop_delta {
                    return GuardDecision::Stop(snapshot);
                }
            }
        }

        GuardDecision::Allow(snapshot)
    }

    fn aggregate_risk_scalar(inputs: &GuardInputs) -> RiskScalar {
        let coords = [
            inputs.r_degrade,
            inputs.r_residualmass,
            inputs.r_microplastics,
            inputs.r_tox_acute,
            inputs.r_tox_chronic,
            inputs.r_shear,
            inputs.r_habitatload,
        ];
        let sum: f64 = coords.iter().map(|c| c.0).sum();
        RiskScalar((sum / coords.len() as f64).clamp(0.0, 1.0))
    }

    fn compute_v_obj(&self, r_t: RiskScalar, c_t: RiskScalar, rho: f64, rho_max: f64) -> f64 {
        let w1 = self.weights.w_risk;
        let w2 = self.weights.w_coverage;
        let w3 = self.weights.w_density;

        let density_term = (rho - rho_max).max(0.0);
        w1 * r_t.0 + w2 * (1.0 - c_t.0) + w3 * density_term
    }

    fn zero_snapshot(&self, ts_utc: i64) -> EcosafetyStateSnapshot {
        EcosafetyStateSnapshot {
            id: Default::default(),
            object_id: self.object_id,
            timestamp_utc: ts_utc,
            risk_scalar_r_t: RiskScalar(0.0),
            swarm_coverage_c_t: RiskScalar(0.0),
            agent_density_rho: 0.0,
            agent_density_max: 0.0,
            v_obj_t: 0.0,
            lyapunov_weightset_id: self.weights.id,
        }
    }
}

pub trait EcosafetyCorridorExt {
    fn within_all(&self, s: &EcosafetyStateSnapshot) -> bool;
    fn is_empty(&self) -> bool;
}

impl EcosafetyCorridorExt for EcosafetyCorridorSet {
    fn within_all(&self, s: &EcosafetyStateSnapshot) -> bool {
        self.r_degrade_corridor.contains(s.risk_scalar_r_t.0)
            && self.r_residualmass_corridor.contains(s.risk_scalar_r_t.0)
            && self.r_microplastics_corridor.contains(s.risk_scalar_r_t.0)
            && self.r_tox_acute_corridor.contains(s.risk_scalar_r_t.0)
            && self.r_tox_chronic_corridor.contains(s.risk_scalar_r_t.0)
            && self.r_shear_corridor.contains(s.risk_scalar_r_t.0)
            && self.r_habitatload_corridor.contains(s.risk_scalar_r_t.0)
            && self.v_obj_corridor.contains(s.v_obj_t)
    }

    fn is_empty(&self) -> bool {
        false
    }
}

impl EcosafetyCorridorSet {
    pub fn downtowncentral_canal_mar_demo() -> Self {
        Self {
            id: Default::default(),
            name: "downtowncentral_canal_mar_demo".into(),
            description: "Demo ecosafety corridors for DowntownCentral canal + MAR vault".into(),
            r_degrade_corridor: crate::types::CorridorScalar::closed(0.0, 0.7, 0.5, 0.02),
            r_residualmass_corridor: crate::types::CorridorScalar::closed(0.0, 0.5, 0.3, 0.02),
            r_microplastics_corridor: crate::types::CorridorScalar::closed(0.0, 0.3, 0.15, 0.01),
            r_tox_acute_corridor: crate::types::CorridorScalar::closed(0.0, 0.25, 0.1, 0.01),
            r_tox_chronic_corridor: crate::types::CorridorScalar::closed(0.0, 0.2, 0.08, 0.01),
            r_shear_corridor: crate::types::CorridorScalar::closed(0.0, 0.6, 0.4, 0.02),
            r_habitatload_corridor: crate::types::CorridorScalar::closed(0.0, 0.4, 0.25, 0.02),
            v_obj_corridor: crate::types::CorridorScalar::closed(0.0, 1.0, 0.5, 0.01),
        }
    }
}
