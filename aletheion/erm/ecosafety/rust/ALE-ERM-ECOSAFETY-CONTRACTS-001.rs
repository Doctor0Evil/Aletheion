// Role
// - Implement runtime ecosafety contracts for cyboquatic nodes and biodegradable materials.
// - Provide safe_step() as the mandatory gate for actuation.
// - Provide FOG-of-Governance (FOG) routing helpers that choose nodes only when corridors are strong.
// - Emit qpudatashard-style violation records that higher layers can sign and append to Googolswarm.
//
// This crate is intended to be consumed via FFI (Lua, WASM) by:
// - aletheion/infra/canals/ALE-INF-CANAL-SEGMENT-WORKFLOW-001.lua
// - aletheion/infra/cyboquatic/* biodegradable node runners
// and referenced by ALN grammar in ALE-ERM-ECOSAFETY-GRAMMAR-001.aln.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

//////////////////////////////////////////////////////////////
// 1. Core types, mirroring ALE-ERM-ECOSAFETY-TYPES-001.rs
//////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct RiskCoord {
    pub name: String,   // e.g. "r_degrade", "r_tox_acute"
    pub value: f64,     // current normalized rx ∈ [0,1]
    pub min_safe: f64,  // safe band lower
    pub max_safe: f64,  // safe band upper
}

#[derive(Debug, Clone)]
pub struct RiskVector {
    pub id: String,             // e.g. "BIO_FLOWVAC_SUBSTRATE_V1"
    pub coords: Vec<RiskCoord>, // ordered but accessed by name
}

#[derive(Debug, Clone)]
pub struct LyapunovResidual {
    pub system_id: String,
    pub t: f64,         // time (seconds or hours)
    pub value: f64,     // V_t
    pub dvalue_dt: f64, // estimated derivative
    pub stable: bool,   // precomputed monotonic flag if available
}

#[derive(Debug, Clone)]
pub struct Corridor {
    pub corridor_id: String,
    pub domain: String, // "SOFT_ROBOT", "WATER", etc.
    pub risk_vector: RiskVector,
    pub lyapunov_template: Option<LyapunovResidual>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorStatus {
    Satisfied,
    SoftViolation,
    HardViolation,
}

#[derive(Debug, Clone)]
pub struct CorridorEvalResult {
    pub corridor_id: String,
    pub status: CorridorStatus,
    pub offending_coord: Option<RiskCoord>,
    pub delta: Option<f64>,                    // magnitude outside corridor, if any
    pub vt: Option<LyapunovResidual>,          // current residual, if provided
}

#[derive(Debug, Clone)]
pub struct CyboquaticNodeEcosafety {
    pub node_id: String,        // e.g. "DTC_CYBO_SOFTBOT_01"
    pub corridors: Vec<Corridor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeAction {
    Allow,
    Derate, // reduced capacity
    Stop,
}

//////////////////////////////////////////////////////////////
// 2. FOG routing primitives for biodegradable nodes
//////////////////////////////////////////////////////////////

/// Minimal node status as seen by FOG routers.
#[derive(Debug, Clone)]
pub struct NodeShardSummary {
    pub node_id: String,
    pub domain: String, // "SOFT_ROBOT"
    pub geography: String, // e.g. "REGION_DOWNTOWN_CANAL_EDGE"
    pub surplus_energy: f64,  // normalized 0–1
    pub hydraulic_safety: f64, // normalized 0–1
    pub ecosafety_score: f64,  // 0–1 derived from corridor status
    pub weakened_corridors: usize,
}

/// FOG router decision outcome for a single node candidate.
#[derive(Debug, Clone)]
pub enum FogRouteDecision {
    Eligible(NodeShardSummary),
    Excluded(NodeShardSummary, String), // reason
}

/// Policy thresholds for FOG routing.
#[derive(Debug, Clone)]
pub struct FogRoutingPolicy {
    pub min_ecosafety_score: f64,   // e.g. 0.9
    pub max_weakened_corridors: u32,
    pub min_surplus_energy: f64,    // prefer nodes with surplus capacity
    pub min_hydraulic_safety: f64,  // ensure flow stability for cybo nodes
}

impl Default for FogRoutingPolicy {
    fn default() -> Self {
        FogRoutingPolicy {
            min_ecosafety_score: 0.90,
            max_weakened_corridors: 0,
            min_surplus_energy: 0.50,
            min_hydraulic_safety: 0.75,
        }
    }
}

/// Compute an ecosafety_score ∈ [0,1] from corridor evaluations:
/// - 1.0 if all SATISFIED
/// - penalize soft/hard violations.
pub fn compute_ecosafety_score(eval_results: &[CorridorEvalResult]) -> (f64, usize) {
    if eval_results.is_empty() {
        return (0.0, 0);
    }

    let mut score = 1.0;
    let mut weakened = 0usize;

    for r in eval_results {
        match r.status {
            CorridorStatus::Satisfied => {}
            CorridorStatus::SoftViolation => {
                score -= 0.1;
                weakened += 1;
            }
            CorridorStatus::HardViolation => {
                score -= 0.4;
                weakened += 1;
            }
        }
    }

    if score < 0.0 {
        score = 0.0;
    }
    (score, weakened)
}

/// FOG routing: given shard-derived metrics, decide eligibility per node.
pub fn fog_route_nodes(
    candidates: Vec<NodeShardSummary>,
    policy: &FogRoutingPolicy,
) -> Vec<FogRouteDecision> {
    candidates
        .into_iter()
        .map(|c| {
            if c.ecosafety_score < policy.min_ecosafety_score {
                return FogRouteDecision::Excluded(
                    c,
                    format!(
                        "FOG: ecosafety_score {:.3} below threshold {:.3}",
                        c.ecosafety_score, policy.min_ecosafety_score
                    ),
                );
            }
            if c.weakened_corridors as u32 > policy.max_weakened_corridors {
                return FogRouteDecision::Excluded(
                    c,
                    format!(
                        "FOG: weakened_corridors {} exceeds allowed {}",
                        c.weakened_corridors, policy.max_weakened_corridors
                    ),
                );
            }
            if c.surplus_energy < policy.min_surplus_energy {
                return FogRouteDecision::Excluded(
                    c,
                    format!(
                        "FOG: surplus_energy {:.3} below threshold {:.3}",
                        c.surplus_energy, policy.min_surplus_energy
                    ),
                );
            }
            if c.hydraulic_safety < policy.min_hydraulic_safety {
                return FogRouteDecision::Excluded(
                    c,
                    format!(
                        "FOG: hydraulic_safety {:.3} below threshold {:.3}",
                        c.hydraulic_safety, policy.min_hydraulic_safety
                    ),
                );
            }
            FogRouteDecision::Eligible(c)
        })
        .collect()
}

//////////////////////////////////////////////////////////////
// 3. Corridor evaluation + safe_step runtime gate
//////////////////////////////////////////////////////////////

/// Evaluate a corridor against the current RiskVector for a node / timestep.
pub fn eval_corridor(corridor: &Corridor, current: &RiskVector) -> CorridorEvalResult {
    // Index current coords by name for quick lookup.
    let mut by_name: HashMap<&str, &RiskCoord> = HashMap::new();
    for c in &current.coords {
        by_name.insert(c.name.as_str(), c);
    }

    let mut status = CorridorStatus::Satisfied;
    let mut offending: Option<RiskCoord> = None;
    let mut delta: Option<f64> = None;

    for desired in &corridor.risk_vector.coords {
        if let Some(cur) = by_name.get(desired.name.as_str()) {
            // Compare current value to safe band.
            if cur.value < desired.min_safe {
                status = CorridorStatus::HardViolation;
                offending = Some(RiskCoord {
                    name: cur.name.clone(),
                    value: cur.value,
                    min_safe: desired.min_safe,
                    max_safe: desired.max_safe,
                });
                delta = Some(desired.min_safe - cur.value);
                break;
            } else if cur.value > desired.max_safe {
                // Allow Soft vs Hard split if close to boundary; here we keep v1 simple: Hard.
                status = CorridorStatus::HardViolation;
                offending = Some(RiskCoord {
                    name: cur.name.clone(),
                    value: cur.value,
                    min_safe: desired.min_safe,
                    max_safe: desired.max_safe,
                });
                delta = Some(cur.value - desired.max_safe);
                break;
            }
        } else {
            // Missing coordinate acts as HardViolation in v1.
            status = CorridorStatus::HardViolation;
            offending = Some(RiskCoord {
                name: desired.name.clone(),
                value: f64::NAN,
                min_safe: desired.min_safe,
                max_safe: desired.max_safe,
            });
            delta = None;
            break;
        }
    }

    // Lyapunov handled separately; we attach template as-is for now.
    CorridorEvalResult {
        corridor_id: corridor.corridor_id.clone(),
        status,
        offending_coord: offending,
        delta,
        vt: corridor.lyapunov_template.clone(),
    }
}

/// Decide node action from a single corridor evaluation (v1 policy).
pub fn decide_node_action(eval: &CorridorEvalResult) -> NodeAction {
    match eval.status {
        CorridorStatus::Satisfied => NodeAction::Allow,
        CorridorStatus::SoftViolation => NodeAction::Derate,
        CorridorStatus::HardViolation => NodeAction::Stop,
    }
}

/// Check Lyapunov residual monotonicity for a given system.
/// In this crate we only enforce "value <= 1.0" and non-increasing if stable=true.
pub fn check_lyapunov(vt_current: &LyapunovResidual, vt_previous: Option<&LyapunovResidual>) -> bool {
    if vt_current.value > 1.0 {
        return false;
    }
    if let Some(prev) = vt_previous {
        if vt_current.value > prev.value + 1e-6 {
            // Not monotonically non-increasing.
            return false;
        }
    }
    true
}

//////////////////////////////////////////////////////////////
// 4. qpudatashard-style violation records for Googolswarm
//////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum ViolationKind {
    CorridorHardViolation,
    CorridorSoftViolation,
    LyapunovViolation,
}

#[derive(Debug, Clone)]
pub struct QpuDataShard {
    pub shard_id: String,      // e.g. "QPUSHARD-<node>-<ts>"
    pub node_id: String,
    pub corridor_id: String,
    pub violation: ViolationKind,
    pub offending_coord: Option<RiskCoord>,
    pub vt: Option<LyapunovResidual>,
    pub timestamp_s: u64,
}

impl QpuDataShard {
    pub fn new(
        node_id: &str,
        corridor_id: &str,
        violation: ViolationKind,
        offending_coord: Option<RiskCoord>,
        vt: Option<LyapunovResidual>,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let shard_id = format!("QPUSHARD_{}_{}_{}", node_id, corridor_id, ts);
        QpuDataShard {
            shard_id,
            node_id: node_id.to_string(),
            corridor_id: corridor_id.to_string(),
            violation,
            offending_coord,
            vt,
            timestamp_s: ts,
        }
    }
}

//////////////////////////////////////////////////////////////
// 5. safe_step: mandatory runtime gate for biodegradable nodes
//////////////////////////////////////////////////////////////

/// Result returned to orchestrators (Lua, WASM) from safe_step.
#[derive(Debug, Clone)]
pub struct SafeStepResult {
    pub node_id: String,
    pub action: NodeAction,
    pub corridors: Vec<CorridorEvalResult>,
    pub violation_shards: Vec<QpuDataShard>,
}

/// safe_step:
/// - Evaluates all corridors for a node at a given timestep.
/// - Optionally checks Lyapunov residual vs previous value.
/// - Returns Allow / Derate / Stop, plus qpudatashards for any violation.
/// - Orchestrators MUST call safe_step before issuing actuation commands.
///
/// In v1, policy is:
/// - If any HardViolation or LyapunovViolation → Stop, emit shards.
/// - Else if any SoftViolation → Derate.
/// - Else → Allow.
pub fn safe_step(
    node: &CyboquaticNodeEcosafety,
    current_vectors: &HashMap<String, RiskVector>, // key: corridor_id
    vt_current: Option<LyapunovResidual>,
    vt_previous: Option<LyapunovResidual>,
) -> SafeStepResult {
    let mut eval_results = Vec::new();
    let mut shards = Vec::new();
    let mut has_hard = false;
    let mut has_soft = false;
    let mut lyap_bad = false;

    for corridor in &node.corridors {
        let rv = match current_vectors.get(&corridor.corridor_id) {
            Some(v) => eval_corridor(corridor, v),
            None => {
                // Missing measurements treated as HardViolation.
                CorridorEvalResult {
                    corridor_id: corridor.corridor_id.clone(),
                    status: CorridorStatus::HardViolation,
                    offending_coord: None,
                    delta: None,
                    vt: corridor.lyapunov_template.clone(),
                }
            }
        };

        if rv.status == CorridorStatus::HardViolation {
            has_hard = true;
            let shard = QpuDataShard::new(
                &node.node_id,
                &rv.corridor_id,
                ViolationKind::CorridorHardViolation,
                rv.offending_coord.clone(),
                rv.vt.clone(),
            );
            shards.push(shard);
        } else if rv.status == CorridorStatus::SoftViolation {
            has_soft = true;
            let shard = QpuDataShard::new(
                &node.node_id,
                &rv.corridor_id,
                ViolationKind::CorridorSoftViolation,
                rv.offending_coord.clone(),
                rv.vt.clone(),
            );
            shards.push(shard);
        }

        eval_results.push(rv);
    }

    // Lyapunov check if we have current residual.
    if let Some(vt_cur) = vt_current.as_ref() {
        let vt_prev_ref = vt_previous.as_ref();
        if !check_lyapunov(vt_cur, vt_prev_ref) {
            lyap_bad = true;
            let shard = QpuDataShard::new(
                &node.node_id,
                "GLOBAL_VT",
                ViolationKind::LyapunovViolation,
                None,
                Some(vt_cur.clone()),
            );
            shards.push(shard);
        }
    }

    let action = if has_hard || lyap_bad {
        NodeAction::Stop
    } else if has_soft {
        NodeAction::Derate
    } else {
        NodeAction::Allow
    };

    SafeStepResult {
        node_id: node.node_id.clone(),
        action,
        corridors: eval_results,
        violation_shards: shards,
    }
}

//////////////////////////////////////////////////////////////
// 6. Lua / WASM FFI surface (typed, minimal)
// (binding code lives in a separate file; here we define
//  C-friendly wrappers to be called from orchestrators.)
//////////////////////////////////////////////////////////////

// The FFI surface assumes the host (Lua/WASM) is responsible for:
// - Maintaining a registry of CyboquaticNodeEcosafety and Corridor defs.
// - Passing a flattened JSON blob or binary to reconstruct current RiskVectors.
// For brevity, we expose only a JSON-based FFI function. A strongly typed
// binding layer can wrap this JSON channel with statically checked types.

#[cfg(feature = "ffi_json")]
mod ffi_json {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FfiRiskCoord {
        pub name: String,
        pub value: f64,
        pub min_safe: f64,
        pub max_safe: f64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FfiRiskVector {
        pub corridor_id: String,
        pub coords: Vec<FfiRiskCoord>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FfiNodeStepInput {
        pub node_id: String,
        pub domain: String,
        pub geography: String,
        pub corridors: Vec<FfiRiskVector>,
        pub vt_current: Option<LyapunovResidual>,
        pub vt_previous: Option<LyapunovResidual>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct FfiSafeStepOutput {
        pub node_id: String,
        pub action: String, // "ALLOW" | "DERATE" | "STOP"
        pub violation_shards: Vec<QpuDataShard>,
    }

    /// WARNING: JSON FFI is intended as a compatibility layer.
    /// Strongly-typed bindings should wrap this for production.
    #[no_mangle]
    pub extern "C" fn ecosafety_safe_step_json(
        input_json_ptr: *const u8,
        input_len: usize,
        output_json_ptr: *mut u8,
        output_len: *mut usize,
    ) -> i32 {
        // Safety: host must provide valid pointers; we do minimal checks.
        if input_json_ptr.is_null() || output_json_ptr.is_null() || output_len.is_null() {
            return -1;
        }
        let input_slice = unsafe { std::slice::from_raw_parts(input_json_ptr, input_len) };
        let input_str = match std::str::from_utf8(input_slice) {
            Ok(s) => s,
            Err(_) => return -2,
        };

        let parsed: FfiNodeStepInput = match serde_json::from_str(input_str) {
            Ok(v) => v,
            Err(_) => return -3,
        };

        // Build node + corridors.
        let mut corridors = Vec::new();
        let mut current_vectors = HashMap::new();

        for c in &parsed.corridors {
            let mut rv_coords = Vec::new();
            let mut desired_coords = Vec::new();
            for rc in &c.coords {
                // For simplicity, treat current and desired safe bands as identical;
                // in a full implementation, safe bands would be loaded from config.
                rv_coords.push(RiskCoord {
                    name: rc.name.clone(),
                    value: rc.value,
                    min_safe: rc.min_safe,
                    max_safe: rc.max_safe,
                });
                desired_coords.push(RiskCoord {
                    name: rc.name.clone(),
                    value: rc.value, // ignored
                    min_safe: rc.min_safe,
                    max_safe: rc.max_safe,
                });
            }

            let rv = RiskVector {
                id: c.corridor_id.clone(),
                coords: rv_coords,
            };

            let corridor = Corridor {
                corridor_id: c.corridor_id.clone(),
                domain: parsed.domain.clone(),
                risk_vector: RiskVector {
                    id: c.corridor_id.clone(),
                    coords: desired_coords,
                },
                lyapunov_template: parsed.vt_current.clone(),
            };

            corridors.push(corridor);
            current_vectors.insert(c.corridor_id.clone(), rv);
        }

        let node = CyboquaticNodeEcosafety {
            node_id: parsed.node_id.clone(),
            corridors,
        };

        let vt_prev = parsed.vt_previous.clone();
        let vt_cur = parsed.vt_current.clone();

        let res = safe_step(&node, &current_vectors, vt_cur, vt_prev);

        let action_str = match res.action {
            NodeAction::Allow => "ALLOW",
            NodeAction::Derate => "DERATE",
            NodeAction::Stop => "STOP",
        }
        .to_string();

        let ffi_out = FfiSafeStepOutput {
            node_id: res.node_id,
            action: action_str,
            violation_shards: res.violation_shards,
        };

        let out_json = match serde_json::to_string(&ffi_out) {
            Ok(s) => s,
            Err(_) => return -4,
        };

        let bytes = out_json.as_bytes();
        let out_len_needed = bytes.len();

        unsafe {
            *output_len = out_len_needed;
            let out_slice = std::slice::from_raw_parts_mut(output_json_ptr, out_len_needed);
            out_slice.copy_from_slice(bytes);
        }

        0
    }
}

//////////////////////////////////////////////////////////////
// 7. DowntownCentral-specific helpers (biodegradable nodes)
//////////////////////////////////////////////////////////////

/// Convenience helper for DowntownCentral biodegradable nodes:
/// - Runs safe_step.
/// - Converts SafeStepResult into a NodeShardSummary suitable for FOG routing,
///   using simple heuristics for surplus_energy and hydraulic_safety (to be
///   refined with real telemetry).
pub fn downtown_bio_safe_step_and_shard(
    node: &CyboquaticNodeEcosafety,
    current_vectors: &HashMap<String, RiskVector>,
    vt_current: Option<LyapunovResidual>,
    vt_previous: Option<LyapunovResidual>,
    geography: &str,    // e.g. "REGION_DOWNTOWN_CORE"
    surplus_energy: f64,
    hydraulic_safety: f64,
) -> (SafeStepResult, NodeShardSummary) {
    let ss = safe_step(node, current_vectors, vt_current, vt_previous);

    // Derive ecosafety_score from corridor results.
    let (eco_score, weakened) = compute_ecosafety_score(&ss.corridors);

    let shard_summary = NodeShardSummary {
        node_id: ss.node_id.clone(),
        domain: "SOFT_ROBOT".to_string(),
        geography: geography.to_string(),
        surplus_energy,
        hydraulic_safety,
        ecosafety_score: eco_score,
        weakened_corridors: weakened,
    };

    (ss, shard_summary)
}

//////////////////////////////////////////////////////////////
// 8. Tests (unit-level, no external IO)
//////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn make_corridor(id: &str, min: f64, max: f64) -> Corridor {
        let coord = RiskCoord {
            name: "r_degrade".to_string(),
            value: 0.0,
            min_safe: min,
            max_safe: max,
        };
        Corridor {
            corridor_id: id.to_string(),
            domain: "SOFT_ROBOT".to_string(),
            risk_vector: RiskVector {
                id: id.to_string(),
                coords: vec![coord],
            },
            lyapunov_template: None,
        }
    }

    #[test]
    fn test_eval_corridor_satisfied() {
        let corridor = make_corridor("C1", 0.4, 0.95);
        let current = RiskCoord {
            name: "r_degrade".to_string(),
            value: 0.7,
            min_safe: 0.4,
            max_safe: 0.95,
        };
        let rv = RiskVector {
            id: "C1".to_string(),
            coords: vec![current],
        };
        let res = eval_corridor(&corridor, &rv);
        assert_eq!(res.status, CorridorStatus::Satisfied);
    }

    #[test]
    fn test_eval_corridor_hard_violation() {
        let corridor = make_corridor("C1", 0.4, 0.95);
        let current = RiskCoord {
            name: "r_degrade".to_string(),
            value: 0.2,
            min_safe: 0.4,
            max_safe: 0.95,
        };
        let rv = RiskVector {
            id: "C1".to_string(),
            coords: vec![current],
        };
        let res = eval_corridor(&corridor, &rv);
        assert_eq!(res.status, CorridorStatus::HardViolation);
        assert!(res.offending_coord.is_some());
    }

    #[test]
    fn test_safe_step_stop_on_violation() {
        let corridor = make_corridor("C1", 0.4, 0.95);
        let node = CyboquaticNodeEcosafety {
            node_id: "NODE1".to_string(),
            corridors: vec![corridor],
        };
        let current = RiskCoord {
            name: "r_degrade".to_string(),
            value: 0.2,
            min_safe: 0.4,
            max_safe: 0.95,
        };
        let rv = RiskVector {
            id: "C1".to_string(),
            coords: vec![current],
        };
        let mut map = HashMap::new();
        map.insert("C1".to_string(), rv);
        let res = safe_step(&node, &map, None, None);
        assert_eq!(res.action, NodeAction::Stop);
        assert!(!res.violation_shards.is_empty());
    }

    #[test]
    fn test_compute_ecosafety_score() {
        let corridor = make_corridor("C1", 0.4, 0.95);
        let current = RiskCoord {
            name: "r_degrade".to_string(),
            value: 0.7,
            min_safe: 0.4,
            max_safe: 0.95,
        };
        let rv = RiskVector {
            id: "C1".to_string(),
            coords: vec![current],
        };
        let eval = eval_corridor(&corridor, &rv);
        let (score, weakened) = compute_ecosafety_score(&[eval]);
        assert_eq!(weakened, 0);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_check_lyapunov() {
        let prev = LyapunovResidual {
            system_id: "S".to_string(),
            t: 0.0,
            value: 0.5,
            dvalue_dt: -0.1,
            stable: true,
        };
        let cur_ok = LyapunovResidual {
            system_id: "S".to_string(),
            t: 1.0,
            value: 0.4,
            dvalue_dt: -0.1,
            stable: true,
        };
        let cur_bad = LyapunovResidual {
            system_id: "S".to_string(),
            t: 1.0,
            value: 0.6,
            dvalue_dt: 0.1,
            stable: true,
        };
        assert!(check_lyapunov(&cur_ok, Some(&prev)));
        assert!(!check_lyapunov(&cur_bad, Some(&prev)));
    }
}
