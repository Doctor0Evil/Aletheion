use std::collections::HashMap;
use std::time::Duration;

// -------- Core Types --------

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CorridorId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct WorkflowId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct StepId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CapitalId(pub String); // e.g. "Water", "Thermal", "Waste", "Biotic", "Neurobiome", "Somatic", "Treaty"

#[derive(Clone, Debug)]
pub struct Timestamp(pub i64); // unix seconds

#[derive(Clone, Debug)]
pub enum StepOutcome {
    Success,
    SoftDegrade,
    HardStop,
    TreatyBlock,
    SensorUncertain,
}

#[derive(Clone, Debug)]
pub struct WorkflowStepLog {
    pub corridor: CorridorId,
    pub workflow: WorkflowId,
    pub step: StepId,
    pub ts_start: Timestamp,
    pub ts_end: Timestamp,
    pub outcome: StepOutcome,
    pub knowledge_inputs: KnowledgeInputs,
    pub ecosafety_state: EcosafetyState,
    pub resource_state: ResourceState,
}

#[derive(Clone, Debug)]
pub struct KnowledgeInputs {
    pub sensor_count: u32,
    pub sensor_coverage_fraction: f32, // 0.0–1.0 of corridor area/time
    pub model_confidence: f32,         // 0.0–1.0
    pub human_review_fraction: f32,    // 0.0–1.0 decisions with human-in-the-loop
}

#[derive(Clone, Debug)]
pub struct EcosafetyState {
    pub risk_before: f32,   // 0.0–1.0
    pub risk_after: f32,    // 0.0–1.0
    pub treaty_violations: u32,
    pub safe_step_respected: bool,
}

#[derive(Clone, Debug)]
pub struct ResourceState {
    pub capital_deltas: HashMap<CapitalId, f32>, // positive = consumption, negative = restoration
    pub capital_capacity: HashMap<CapitalId, f32>,
}

#[derive(Clone, Debug)]
pub struct SensorReading {
    pub corridor: CorridorId,
    pub ts: Timestamp,
    pub capital: CapitalId,
    pub value: f32,
}

// -------- KER Metrics --------

#[derive(Clone, Debug, Default)]
pub struct KER {
    pub k: f32, // Knowledge
    pub e: f32, // Ecosafety
    pub r: f32, // Resource
}

#[derive(Clone, Debug)]
pub struct KerPerStep {
    pub corridor: CorridorId,
    pub workflow: WorkflowId,
    pub step: StepId,
    pub ts_start: Timestamp,
    pub ts_end: Timestamp,
    pub ker: KER,
}

#[derive(Clone, Debug)]
pub struct KerPerCorridor {
    pub corridor: CorridorId,
    pub window_start: Timestamp,
    pub window_end: Timestamp,
    pub ker: KER,
}

// -------- Configuration --------

#[derive(Clone, Debug)]
pub struct KerWeights {
    pub k_weights: KnowledgeWeights,
    pub e_weights: EcosafetyWeights,
    pub r_weights: ResourceWeights,
}

#[derive(Clone, Debug)]
pub struct KnowledgeWeights {
    pub w_sensor_coverage: f32,
    pub w_model_confidence: f32,
    pub w_human_review: f32,
}

#[derive(Clone, Debug)]
pub struct EcosafetyWeights {
    pub w_risk_reduction: f32,
    pub w_safe_step: f32,
    pub w_treaty_penalty: f32,
}

#[derive(Clone, Debug)]
pub struct ResourceWeights {
    pub capital_importance: HashMap<CapitalId, f32>, // importance of each capital
    pub w_efficiency: f32,
}

impl Default for KerWeights {
    fn default() -> Self {
        Self {
            k_weights: KnowledgeWeights {
                w_sensor_coverage: 0.35,
                w_model_confidence: 0.40,
                w_human_review: 0.25,
            },
            e_weights: EcosafetyWeights {
                w_risk_reduction: 0.60,
                w_safe_step: 0.25,
                w_treaty_penalty: 0.15,
            },
            r_weights: ResourceWeights {
                capital_importance: HashMap::new(),
                w_efficiency: 1.0,
            },
        }
    }
}

// -------- Core Computation --------

pub struct KerComputationPipeline {
    pub weights: KerWeights,
    pub step_window_max: Duration,
}

impl KerComputationPipeline {
    pub fn new(weights: KerWeights, step_window_max: Duration) -> Self {
        Self { weights, step_window_max }
    }

    pub fn compute_ker_for_steps(
        &self,
        logs: &[WorkflowStepLog],
        sensor_readings: &[SensorReading],
    ) -> Vec<KerPerStep> {
        let mut out = Vec::with_capacity(logs.len());
        for log in logs {
            let k = self.compute_k(&log.knowledge_inputs);
            let e = self.compute_e(&log.ecosafety_state);
            let r = self.compute_r(&log.resource_state, sensor_readings, &log.corridor, &log.ts_start, &log.ts_end);

            out.push(KerPerStep {
                corridor: log.corridor.clone(),
                workflow: log.workflow.clone(),
                step: log.step.clone(),
                ts_start: log.ts_start.clone(),
                ts_end: log.ts_end.clone(),
                ker: KER { k, e, r },
            });
        }
        out
    }

    pub fn aggregate_ker_per_corridor(
        &self,
        step_kers: &[KerPerStep],
        window_start: &Timestamp,
        window_end: &Timestamp,
    ) -> Vec<KerPerCorridor> {
        let mut buckets: HashMap<CorridorId, Vec<&KerPerStep>> = HashMap::new();
        for s in step_kers {
            if s.ts_start.0 < window_start.0 || s.ts_end.0 > window_end.0 {
                continue;
            }
            buckets.entry(s.corridor.clone()).or_default().push(s);
        }

        let mut result = Vec::new();
        for (corridor, entries) in buckets {
            if entries.is_empty() {
                continue;
            }
            let mut sum_k = 0.0;
            let mut sum_e = 0.0;
            let mut sum_r = 0.0;
            for e in entries {
                sum_k += e.ker.k;
                sum_e += e.ker.e;
                sum_r += e.ker.r;
            }
            let n = entries.len() as f32;
            result.push(KerPerCorridor {
                corridor,
                window_start: window_start.clone(),
                window_end: window_end.clone(),
                ker: KER {
                    k: sum_k / n,
                    e: sum_e / n,
                    r: sum_r / n,
                },
            });
        }
        result
    }

    fn compute_k(&self, ki: &KnowledgeInputs) -> f32 {
        let w = &self.weights.k_weights;

        let sensor_term = ki.sensor_coverage_fraction.clamp(0.0, 1.0);
        let model_term = ki.model_confidence.clamp(0.0, 1.0);
        let human_term = ki.human_review_fraction.clamp(0.0, 1.0);

        let mut score =
            w.w_sensor_coverage * sensor_term +
            w.w_model_confidence * model_term +
            w.w_human_review * human_term;

        if ki.sensor_count == 0 {
            score *= 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    fn compute_e(&self, es: &EcosafetyState) -> f32 {
        let w = &self.weights.e_weights;

        let risk_before = es.risk_before.clamp(0.0, 1.0);
        let risk_after = es.risk_after.clamp(0.0, 1.0);
        let risk_reduction = (risk_before - risk_after).max(0.0);

        let safe_step_term = if es.safe_step_respected { 1.0 } else { 0.0 };

        let treaty_penalty = (es.treaty_violations as f32 * 0.25).min(1.0);
        let treaty_term = 1.0 - treaty_penalty;

        let score =
            w.w_risk_reduction * risk_reduction +
            w.w_safe_step * safe_step_term +
            w.w_treaty_penalty * treaty_term;

        score.clamp(0.0, 1.0)
    }

    fn compute_r(
        &self,
        rs: &ResourceState,
        _sensor_readings: &[SensorReading],
        _corridor: &CorridorId,
        _start: &Timestamp,
        _end: &Timestamp,
    ) -> f32 {
        let w = &self.weights.r_weights;
        if rs.capital_deltas.is_empty() || rs.capital_capacity.is_empty() {
            return 0.5;
        }

        let mut num = 0.0;
        let mut den = 0.0;

        for (capital, delta) in &rs.capital_deltas {
            let capacity = *rs.capital_capacity.get(capital).unwrap_or(&1.0);
            if capacity <= 0.0 {
                continue;
            }
            let importance = *w.capital_importance.get(capital).unwrap_or(&1.0);
            let usage_fraction = (delta / capacity).max(-1.0).min(1.0);

            let capital_score = if usage_fraction <= 0.0 {
                (1.0 + usage_fraction) * 0.5 + 0.5
            } else {
                1.0 - usage_fraction
            };

            num += capital_score * importance;
            den += importance;
        }

        if den == 0.0 {
            return 0.5;
        }

        let efficiency = (num / den).clamp(0.0, 1.0);
        (efficiency * w.w_efficiency).clamp(0.0, 1.0)
    }
}

// -------- Example Constructors --------

pub fn default_capital_importance() -> HashMap<CapitalId, f32> {
    let mut m = HashMap::new();
    m.insert(CapitalId("Water".into()), 1.0);
    m.insert(CapitalId("Thermal".into()), 1.0);
    m.insert(CapitalId("Waste".into()), 0.8);
    m.insert(CapitalId("Biotic".into()), 1.2);
    m.insert(CapitalId("Neurobiome".into()), 1.3);
    m.insert(CapitalId("Somatic".into()), 1.1);
    m.insert(CapitalId("Treaty".into()), 1.4);
    m
}

pub fn new_default_pipeline() -> KerComputationPipeline {
    let mut weights = KerWeights::default();
    weights.r_weights.capital_importance = default_capital_importance();
    KerComputationPipeline::new(weights, Duration::from_secs(6 * 3600))
}
