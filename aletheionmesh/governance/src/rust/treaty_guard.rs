
#[derive(Clone, Debug)]
pub struct TreatyScopeCheckInput {
    pub treaty_id: String,
    pub operation_kind: String,
    pub geo_point_wgs84: (f64, f64),
    pub roh_0_1: f64,
    pub psr_0_1: f64,
    pub consent_token_present: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreatyDecision {
    MayRun,
    MustHalt(String),
}

pub trait TreatyGuard {
    fn evaluate(&self, input: &TreatyScopeCheckInput) -> TreatyDecision;
}
