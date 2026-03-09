//! Aletheion Phoenix ERM Engine Core
//! Layer_2 (State Modeling System) and Layer_4 (Optimization Engine) bridge for
//! water, thermal, materials, biotic, neurobiome, and energy resources in Phoenix. [file:6]
//! All actions are gated by Birth‑Signs, ALN norms, and Googolswarm‑compatible trust logging. [file:2][file:6]

use std::collections::HashMap;
use std::time::SystemTime;

//
// ------------------------
// Core identifiers & types
// ------------------------
//

/// Opaque identifier for any ERM resource (water portfolio, thermal corridor, etc.). [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

/// Opaque identifier for a Phoenix zone (district, corridor, basin, campus). [file:5][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneId(pub String);

/// Opaque identifier for workflows defined in ALN schemas (e.g., AWP allocation run). [file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkflowId(pub String);

/// Aletheion Legal Norm identifier bound to ALN schemas. [file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlnNormId(pub String);

/// Trust‑layer transaction id for Googolswarm audit records. [file:2][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrustTxId(pub String);

/// Birth‑Sign identifier for a spatial tile / jurisdictional context. [file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

/// High‑level domains the ERM engine can manage. [file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceDomain {
    Water,
    Thermal,
    Materials,
    Biotic,
    Neurobiome,
    Energy,
}

//
// ---------------------
// Governance primitives
// ---------------------
//

/// Constraint enforcement mode derived from Birth‑Signs. [file:2]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintMode {
    Hard,
    HighPenalty,
}

/// Result of pre‑flight governance checks for an action plan. [file:2]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceDecision {
    Allowed,
    AllowedWithPenalties {
        /// Scalar penalty applied in optimization scoring space.
        penalty_score: f64,
        /// Norms that were soft‑violated.
        softened_norms: Vec<AlnNormId>,
    },
    Rejected {
        /// Hard constraints that were violated.
        violated_norms: Vec<AlnNormId>,
        /// Human‑readable explanation for operators.
        reason: String,
    },
}

/// Snapshot of governance context attached to a zone and resources. [file:2]
#[derive(Debug, Clone)]
pub struct GovernanceContext {
    pub birth_sign_id: BirthSignId,
    pub applicable_norms: Vec<AlnNormId>,
    pub constraint_mode: ConstraintMode,
}

//
// --------------------
// ERM state & metrics
// --------------------
//

/// Common scalar with unit metadata for ERM metrics. [file:6]
#[derive(Debug, Clone)]
pub struct Quantity {
    pub value: f64,
    /// Example: "m3", "kWh", "degC", "kg", "index".
    pub unit: String,
}

/// Aggregate resilience metrics for Phoenix resource systems. [file:5][file:6]
#[derive(Debug, Clone, Default)]
pub struct ResilienceMetrics {
    pub water_reuse_fraction: Option<f64>,
    pub groundwater_balance: Option<Quantity>,
    pub heat_risk_index: Option<f64>,
    pub waste_recovery_rate: Option<f64>,
    pub treaty_adherence_index: Option<f64>,
}

/// ERM view of a single resource within a zone at a point in time. [file:6]
#[derive(Debug, Clone)]
pub struct ResourceState {
    pub id: ResourceId,
    pub domain: ResourceDomain,
    pub zone: ZoneId,
    /// Physical / operational quantity (e.g., volume, temperature, load).
    pub level: Quantity,
    /// Optional quality metrics (purity, reliability, ecological impact).
    pub quality: HashMap<String, Quantity>,
    /// Last update time from the state modeling system.
    pub updated_at: SystemTime,
}

/// Full ERM state snapshot across all resources/zones. [file:6]
#[derive(Debug, Clone)]
pub struct ErmStateSnapshot {
    pub resources: HashMap<ResourceId, ResourceState>,
    pub resilience: ResilienceMetrics,
    /// Jurisdictional + governance context keyed by zone.
    pub governance_by_zone: HashMap<ZoneId, GovernanceContext>,
    /// Time at which this snapshot became consistent.
    pub snapshot_time: SystemTime,
}

//
// --------------------
// Actions & actuators
// --------------------
//

/// Category of actuation target in Phoenix infrastructure. [file:5][file:6]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActuationTargetKind {
    AwpPlant,
    CanalGate,
    SewerValve,
    ThermalAsset,
    PumpStation,
    MicrogridNode,
    TreeIrrigationCluster,
    CyboquaticNode,
    CitizenInterfaceSignal,
}

/// Specific actuation endpoint (e.g., named AWP plant, gate id). [file:5]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActuationTargetId(pub String);

/// Canonical actuation command issued by the ERM engine. [file:5][file:6]
#[derive(Debug, Clone)]
pub struct ErmActionCommand {
    pub workflow_id: WorkflowId,
    pub target_kind: ActuationTargetKind,
    pub target_id: ActuationTargetId,
    /// Arbitrary command payload, e.g. {"gate_open_pct": 30.0}.
    pub payload: HashMap<String, f64>,
    /// Time when this action should be applied.
    pub effective_at: SystemTime,
    /// Zone this action primarily affects.
    pub zone: ZoneId,
}

//
// ----------------------
// Planning & audit types
// ----------------------
//

/// Abstract description of what the engine intends to change. [file:5]
#[derive(Debug, Clone)]
pub struct ErmActionPlan {
    pub id: String,
    pub created_at: SystemTime,
    pub based_on_snapshot: SystemTime,
    pub commands: Vec<ErmActionCommand>,
    /// Objective‑space scores (e.g., { "efficiency": 0.9, "equity": 0.85 }). [file:6]
    pub scores: HashMap<String, f64>,
    /// Governance decision applied to this plan.
    pub governance_decision: GovernanceDecision,
}

/// Status of a command after attempting to apply to real infrastructure. [file:5]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandStatus {
    Pending,
    Applied,
    Failed(String),
    Skipped(String),
}

/// Result of applying a full action plan. [file:6]
#[derive(Debug, Clone)]
pub struct PlanApplicationResult {
    pub plan_id: String,
    pub trust_tx_id: Option<TrustTxId>,
    pub command_statuses: HashMap<usize, CommandStatus>,
}

/// Audit record for a state → plan → actuation cycle. [file:2][file:5]
#[derive(Debug, Clone)]
pub struct ErmAuditRecord {
    pub workflow_id: WorkflowId,
    pub snapshot_time: SystemTime,
    pub plan_id: String,
    pub governance_decision: GovernanceDecision,
    pub trust_tx_id: Option<TrustTxId>,
    pub created_at: SystemTime,
}

//
// ---------------------------
// Extension traits & adapters
// ---------------------------
//

/// Adapter for optimization engines (NSGA‑II, MOEA/D, etc.). [file:6]
pub trait OptimizationAdapter {
    /// Compute candidate plans for a given snapshot and workflow. [file:5][file:6]
    fn propose_plans(
        &self,
        workflow: &WorkflowId,
        snapshot: &ErmStateSnapshot,
    ) -> Vec<ErmActionPlan>;
}

/// Adapter for governance checks using Birth‑Signs and ALN norms. [file:2]
pub trait GovernanceAdapter {
    /// Run pre‑flight governance checks over a candidate plan. [file:2]
    fn evaluate_plan(
        &self,
        plan: &ErmActionPlan,
        snapshot: &ErmStateSnapshot,
    ) -> GovernanceDecision;
}

/// Adapter for pushing actions into real infrastructure controllers. [file:5]
pub trait ActuationAdapter {
    /// Apply a fully‑approved action plan, returning per‑command statuses. [file:5]
    fn apply_plan(&self, plan: &ErmActionPlan) -> HashMap<usize, CommandStatus>;
}

/// Adapter for Googolswarm‑compatible trust logging. [file:2]
pub trait TrustLogAdapter {
    /// Record the lifecycle of a plan and return a trust‑layer transaction id. [file:2]
    fn append_audit_record(&self, record: &ErmAuditRecord) -> Option<TrustTxId>;
}

//
// --------------
// ERM Engine API
// --------------
//

/// Core ERM Engine trait powering all Phoenix resource workflows. [file:5][file:6]
pub trait ErmEngine {
    /// Ingest a new state snapshot from the State Modeling System. [file:6]
    fn ingest_state(&mut self, snapshot: ErmStateSnapshot);

    /// Produce a set of candidate action plans for a workflow using optimization engines. [file:5][file:6]
    fn plan_actions(&self, workflow_id: &WorkflowId) -> Vec<ErmActionPlan>;

    /// Apply a governance‑filtered action plan and push to actuators + trust layer. [file:2][file:5]
    fn apply_actions(&self, plan: ErmActionPlan) -> PlanApplicationResult;

    /// Generate an audit record summarizing a state → plan → actuation cycle. [file:2]
    fn audit_trace(
        &self,
        workflow_id: &WorkflowId,
        snapshot_time: SystemTime,
        plan_result: &PlanApplicationResult,
        decision: GovernanceDecision,
    ) -> ErmAuditRecord;

    /// Retrieve the latest state snapshot known to the engine. [file:6]
    fn latest_snapshot(&self) -> Option<ErmStateSnapshot>;
}

//
// ----------------------
// Phoenix ERM Engine impl
// ----------------------
//

/// Configuration for Phoenix‑specific deployment of the ERM engine. [file:5][file:6]
#[derive(Debug, Clone)]
pub struct PhoenixErmConfig {
    /// Named AWP facilities (e.g., Cave Creek, North Gateway, 91st Ave). [file:5]
    pub awp_plants: Vec<String>,
    /// Canal system segments referenced by workflows. [file:5]
    pub canal_segments: Vec<String>,
    /// Sewer basins used in pollutant and flow monitoring workflows. [file:5]
    pub sewer_basins: Vec<String>,
    /// Named districts or corridors for ERM zoning. [file:5][file:6]
    pub zones: Vec<ZoneId>,
}

/// Phoenix implementation of the ERM Engine. [file:5][file:6]
pub struct PhoenixErmEngine<'a> {
    pub config: PhoenixErmConfig,
    optimization: &'a dyn OptimizationAdapter,
    governance: &'a dyn GovernanceAdapter,
    actuation: &'a dyn ActuationAdapter,
    trust_log: &'a dyn TrustLogAdapter,
    latest: Option<ErmStateSnapshot>,
}

impl<'a> PhoenixErmEngine<'a> {
    /// Construct a new Phoenix ERM engine instance. [file:6]
    pub fn new(
        config: PhoenixErmConfig,
        optimization: &'a dyn OptimizationAdapter,
        governance: &'a dyn GovernanceAdapter,
        actuation: &'a dyn ActuationAdapter,
        trust_log: &'a dyn TrustLogAdapter,
    ) -> Self {
        Self {
            config,
            optimization,
            governance,
            actuation,
            trust_log,
            latest: None,
        }
    }

    /// Internal helper to get a snapshot or fail with a consistent error. [file:6]
    fn require_snapshot(&self) -> &ErmStateSnapshot {
        self.latest
            .as_ref()
            .expect("PhoenixErmEngine: state snapshot not yet ingested")
    }
}

impl<'a> ErmEngine for PhoenixErmEngine<'a> {
    fn ingest_state(&mut self, snapshot: ErmStateSnapshot) {
        // In a full implementation, this is where reconciliation, validation,
        // and Digital Twin Exclusion Protocol checks occur before acceptance. [file:6]
        self.latest = Some(snapshot);
    }

    fn plan_actions(&self, workflow_id: &WorkflowId) -> Vec<ErmActionPlan> {
        let snapshot = self.require_snapshot();
        let mut plans = self.optimization.propose_plans(workflow_id, snapshot);

        // Apply governance evaluation to each plan before returning. [file:2]
        for plan in plans.iter_mut() {
            let decision = self.governance.evaluate_plan(plan, snapshot);
            plan.governance_decision = decision;
        }

        plans
    }

    fn apply_actions(&self, mut plan: ErmActionPlan) -> PlanApplicationResult {
        let snapshot_time = plan.based_on_snapshot;
        let decision = plan.governance_decision.clone();

        // Only allow application if governance has not rejected the plan. [file:2]
        let command_statuses = match &decision {
            GovernanceDecision::Rejected { reason, .. } => {
                let mut statuses = HashMap::new();
                for (idx, _) in plan.commands.iter().enumerate() {
                    statuses.insert(idx, CommandStatus::Skipped(reason.clone()));
                }
                statuses
            }
            _ => self.actuation.apply_plan(&plan),
        };

        // Create preliminary audit record (trust tx id filled in by adapter). [file:2]
        let workflow_id = plan
            .commands
            .get(0)
            .map(|c| c.workflow_id.clone())
            .unwrap_or_else(|| WorkflowId("unknown".into()));

        let audit_record = ErmAuditRecord {
            workflow_id,
            snapshot_time,
            plan_id: plan.id.clone(),
            governance_decision: decision.clone(),
            trust_tx_id: None,
            created_at: SystemTime::now(),
        };

        let trust_tx_id = self.trust_log.append_audit_record(&audit_record);

        PlanApplicationResult {
            plan_id: plan.id.clone(),
            trust_tx_id,
            command_statuses,
        }
    }

    fn audit_trace(
        &self,
        workflow_id: &WorkflowId,
        snapshot_time: SystemTime,
        plan_result: &PlanApplicationResult,
        decision: GovernanceDecision,
    ) -> ErmAuditRecord {
        ErmAuditRecord {
            workflow_id: workflow_id.clone(),
            snapshot_time,
            plan_id: plan_result.plan_id.clone(),
            governance_decision: decision,
            trust_tx_id: plan_result.trust_tx_id.clone(),
            created_at: SystemTime::now(),
        }
    }

    fn latest_snapshot(&self) -> Option<ErmStateSnapshot> {
        self.latest.clone()
    }
}
