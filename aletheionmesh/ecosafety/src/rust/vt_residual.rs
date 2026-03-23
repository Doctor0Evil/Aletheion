
#[derive(Clone, Debug)]
pub struct EcosafetyResidualParams {
    pub w_r_degrade: f64,
    pub w_r_microplastics: f64,
    pub w_r_tox: f64,
    pub w_r_residualmass: f64,
    pub w_r_shear: f64,
    pub w_r_habitatload: f64,
    pub v_safe_ceiling: f64,
}

#[derive(Clone, Debug)]
pub struct EcosafetyStatePoint {
    pub t_days: f64,
    pub r_degrade_0_1: f64,
    pub r_microplastics_0_1: f64,
    pub r_tox_acute_0_1: f64,
    pub r_tox_chronic_0_1: f64,
    pub r_residualmass_0_1: f64,
    pub r_shear_0_1: f64,
    pub r_habitatload_0_1: f64,
}

#[derive(Clone, Debug)]
pub struct EcosafetyResidual {
    pub t_days: f64,
    pub v_t: f64,
}

impl EcosafetyResidual {
    pub fn compute(p: &EcosafetyResidualParams, s: &EcosafetyStatePoint) -> Self {
        let r_tox = 0.5 * (s.r_tox_acute_0_1 + s.r_tox_chronic_0_1);
        let v_t = p.w_r_degrade * s.r_degrade_0_1
            + p.w_r_microplastics * s.r_microplastics_0_1
            + p.w_r_tox * r_tox
            + p.w_r_residualmass * s.r_residualmass_0_1
            + p.w_r_shear * s.r_shear_0_1
            + p.w_r_habitatload * s.r_habitatload_0_1;

        Self {
            t_days: s.t_days,
            v_t,
        }
    }

    pub fn check_monotone_non_increasing(prev: &Self, next: &Self, eps: f64) -> bool {
        next.v_t <= prev.v_t + eps
    }

    pub fn check_within_safe_ceiling(&self, params: &EcosafetyResidualParams) -> bool {
        self.v_t <= params.v_safe_ceiling
    }
}
