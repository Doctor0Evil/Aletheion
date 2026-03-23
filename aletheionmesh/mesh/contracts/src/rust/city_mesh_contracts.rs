
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityContext {
    pub story_id: String,
    pub phase_id: String,
    pub ts_utc_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstraintsIn {
    pub treaty_ids: Vec<String>,
    pub ecosafety_corridor_ids: Vec<String>,
    pub neurorights_policy_id: Option<String>,
    pub vt_safe_ceiling: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainStateIn {
    pub domain_name: String,      // "WATER", "THERMAL", "MOBILITY", etc.
    pub state_shard_id: String,   // qpudatashard / ALN shard identifier
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionProposal {
    pub action_id: String,
    pub domain_name: String,
    pub target_object_id: String,
    pub operation_kind: String,
    pub magnitude: f64,
    pub window_start_ms: i64,
    pub window_end_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub delta_heatbudget: Option<f64>,
    pub delta_careload: Option<f64>,
    pub delta_vt: Option<f64>,
    pub treaty_violation_risk: Option<f64>,
    pub fpic_error_risk: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowOutput {
    pub actions: Vec<ActionProposal>,
    pub impact: ImpactSummary,
    pub status_code: String,
    pub status_message: String,
}

pub trait CityMeshWorkflow {
    fn workflow_id(&self) -> &'static str;

    fn evaluate_tick(
        &self,
        ctx: &CityContext,
        constraints: &ConstraintsIn,
        state: &DomainStateIn,
    ) -> WorkflowOutput;
}
