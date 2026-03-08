//! Aletheion Ecosafety Types v1
//! File: ALE-ERM-ECOSAFETY-TYPES-001.rs
//! Path: aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-TYPES-001.rs
//!
//! Role
//! - Provide Rust types that mirror ALE-ERM-ECOSAFETY-GRAMMAR-001.aln.
//! - Offer minimal helpers for corridor presence, evaluation shells, and node actions.
//! - Intended to be used by ecosafety contracts, ERM water/MAR/canal/air modules,
//!   and central compliance engines.

use std::collections::HashMap;

/// Post-quantum mode (aligned with SMART-chain validator PQMode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PqMode {
    ClassicalOnly,
    Hybrid,
    PqStrict,
}

impl PqMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CLASSICAL_ONLY" => Some(PqMode::ClassicalOnly),
            "HYBRID" => Some(PqMode::Hybrid),
            "PQ_STRICT" => Some(PqMode::PqStrict),
            _ => None,
        }
    }
}

/// Ecosafety domain tags (aligned with ALN domain-tag).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcosafetyDomain {
    Water,
    Mar,
    Wetland,
    Air,
    Biochar,
    SoftRobot,
}

impl EcosafetyDomain {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "water" => Some(EcosafetyDomain::Water),
            "mar" => Some(EcosafetyDomain::Mar),
            "wetland" => Some(EcosafetyDomain::Wetland),
            "air" => Some(EcosafetyDomain::Air),
            "biochar" => Some(EcosafetyDomain::Biochar),
            "softrobot" => Some(EcosafetyDomain::SoftRobot),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EcosafetyDomain::Water => "water",
            EcosafetyDomain::Mar => "mar",
            EcosafetyDomain::Wetland => "wetland",
            EcosafetyDomain::Air => "air",
            EcosafetyDomain::Biochar => "biochar",
            EcosafetyDomain::SoftRobot => "softrobot",
        }
    }
}

/// Normalized risk coordinate (rx) 0..1 band with safe corridor.
#[derive(Debug, Clone)]
pub struct RiskCoord {
    pub name: String,   // e.g. "PFAS", "R_DEGRADE"
    pub value: f64,     // current normalized value
    pub min_safe: f64,  // lower bound of safe band
    pub max_safe: f64,  // upper bound of safe band
}

impl RiskCoord {
    pub fn is_within_safe(&self) -> bool {
        self.value >= self.min_safe && self.value <= self.max_safe
    }
}

/// Risk vector (rx) = set of coordinates for a system.
#[derive(Debug, Clone)]
pub struct RiskVector {
    pub id: String,                     // e.g. "MARDESERT_PFAS_V1"
    pub coords: HashMap<String, RiskCoord>,
}

impl RiskVector {
    pub fn coord(&self, name: &str) -> Option<&RiskCoord> {
        self.coords.get(name)
    }
}

/// Lyapunov residual description for ecosafety stability checks.
#[derive(Debug, Clone)]
pub struct LyapunovResidual {
    pub system_id: String,
    pub t: f64,
    pub value: f64,
    pub d_value_dt: f64,
    pub stable: bool,
}

impl LyapunovResidual {
    pub fn basic_stability_ok(&self) -> bool {
        self.stable && self.value <= 0.0
    }
}

/// Corridor definition for a cyboquatic / ecosafety system.
#[derive(Debug, Clone)]
pub struct Corridor {
    pub corridor_id: String,
    pub region: String,
    pub domain: EcosafetyDomain,
    pub risk_vector_template_id: String, // aligns to ALN CorridorAtom risk-coords grouping
    pub lyapunov_template_id: Option<String>,
}

/// Status of a corridor evaluation.
#[derive(Debug, Clone)]
pub enum CorridorStatus {
    Satisfied,
    SoftViolation {
        coord_name: String,
        delta: f64,
    },
    HardViolation {
        coord_name: String,
        delta: f64,
    },
}

/// Result of evaluating a corridor against current rx and optional Lyapunov residual.
#[derive(Debug, Clone)]
pub struct CorridorEvalResult {
    pub corridor_id: String,
    pub status: CorridorStatus,
    pub vt: Option<LyapunovResidual>,
}

/// Node-level ecosafety declaration.
#[derive(Debug, Clone)]
pub struct CyboquaticNodeEcosafety {
    pub node_id: String,
    pub domain: EcosafetyDomain,
    pub pq_mode: PqMode,
    pub corridors: Vec<Corridor>,
}

impl CyboquaticNodeEcosafety {
    /// No corridor, no build.
    pub fn require_corridors(&self) -> Result<(), String> {
        if self.corridors.is_empty() {
            Err(format!(
                "Cyboquatic node '{}' missing ecosafety corridors",
                self.node_id
            ))
        } else {
            Ok(())
        }
    }
}

/// Node action decisions based on ecosafety evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeAction {
    Normal,
    Derate { factor: f64 },
    Stop,
}

/// Ecosafety policy (runtime mirror of ALN EcosafetyPolicy).
#[derive(Debug, Clone)]
pub struct EcosafetyPolicy {
    pub policy_id: String,
    pub applies_domain: EcosafetyDomain,
    pub on_soft_violation_derate: bool,
    pub on_hard_violation_stop: bool,
    pub min_corridors_per_node: usize,
    pub enforce_pq_strict: bool,
}

impl EcosafetyPolicy {
    pub fn applies_to(&self, domain: EcosafetyDomain) -> bool {
        self.applies_domain == domain
    }

    pub fn check_node_metadata(&self, node: &CyboquaticNodeEcosafety) -> Result<(), String> {
        if !self.applies_to(node.domain) {
            return Ok(());
        }
        if node.corridors.len() < self.min_corridors_per_node {
            return Err(format!(
                "Node '{}' has {} corridors, policy '{}' requires at least {}",
                node.node_id,
                node.corridors.len(),
                self.policy_id,
                self.min_corridors_per_node
            ));
        }
        if self.enforce_pq_strict && node.pq_mode != PqMode::PqStrict {
            return Err(format!(
                "Node '{}' domain '{}' must be PQ_STRICT under policy '{}'",
                node.node_id,
                node.domain.as_str(),
                self.policy_id
            ));
        }
        Ok(())
    }
}

/// Decide node action given a corridor evaluation and policy.
/// V1: simple mapping – can be extended as corridors/policies grow.
pub fn decide_node_action(policy: &EcosafetyPolicy, eval: &CorridorEvalResult) -> NodeAction {
    match &eval.status {
        CorridorStatus::Satisfied => NodeAction::Normal,
        CorridorStatus::SoftViolation { .. } => {
            if policy.on_soft_violation_derate {
                NodeAction::Derate { factor: 0.5 }
            } else {
                NodeAction::Normal
            }
        }
        CorridorStatus::HardViolation { .. } => {
            if policy.on_hard_violation_stop {
                NodeAction::Stop
            } else {
                NodeAction::Derate { factor: 0.0 }
            }
        }
    }
}

/// Shell for evaluating a corridor against current risk vector.
/// NOTE: V1 leaves numeric thresholds to the caller; here we only
/// implement the minimum shape needed for CI and contracts.
pub fn eval_corridor(
    corridor: &Corridor,
    current: &RiskVector,
    thresholds: &HashMap<String, (f64, f64)>,
) -> CorridorEvalResult {
    // Default: satisfied unless a coord is out of band.
    let mut status = CorridorStatus::Satisfied;

    for (name, (min_safe, max_safe)) in thresholds.iter() {
        if let Some(coord) = current.coord(name) {
            if coord.value < *min_safe {
                status = CorridorStatus::HardViolation {
                    coord_name: name.clone(),
                    delta: coord.value - *min_safe,
                };
                break;
            }
            if coord.value > *max_safe {
                status = CorridorStatus::HardViolation {
                    coord_name: name.clone(),
                    delta: coord.value - *max_safe,
                };
                break;
            }
        }
    }

    CorridorEvalResult {
        corridor_id: corridor.corridor_id.clone(),
        status,
        vt: None,
    }
}

/// Basic Lyapunov check wrapper.
pub fn check_lyapunov(vt: &LyapunovResidual) -> bool {
    vt.basic_stability_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_corridors_fails_on_empty() {
        let node = CyboquaticNodeEcosafety {
            node_id: "MAR_VAULT_TEST".to_string(),
            domain: EcosafetyDomain::Mar,
            pq_mode: PqMode::PqStrict,
            corridors: Vec::new(),
        };
        assert!(node.require_corridors().is_err());
    }

    #[test]
    fn policy_enforces_pq_strict_and_min_corridors() {
        let policy = EcosafetyPolicy {
            policy_id: "ECOSAFETY_POLICY_DOWNTOWN_V1".to_string(),
            applies_domain: EcosafetyDomain::Mar,
            on_soft_violation_derate: true,
            on_hard_violation_stop: true,
            min_corridors_per_node: 1,
            enforce_pq_strict: true,
        };

        let node_ok = CyboquaticNodeEcosafety {
            node_id: "MAR_VAULT_OK".to_string(),
            domain: EcosafetyDomain::Mar,
            pq_mode: PqMode::PqStrict,
            corridors: vec![Corridor {
                corridor_id: "MARDESERT_PFAS_V1".to_string(),
                region: "REGION_DOWNTOWN_CORE".to_string(),
                domain: EcosafetyDomain::Mar,
                risk_vector_template_id: "MARDESERT_PFAS_V1".to_string(),
                lyapunov_template_id: Some("MAR_DESERT_VT_V1".to_string()),
            }],
        };

        assert!(policy.check_node_metadata(&node_ok).is_ok());

        let node_bad_pq = CyboquaticNodeEcosafety {
            node_id: "MAR_VAULT_BAD".to_string(),
            domain: EcosafetyDomain::Mar,
            pq_mode: PqMode::Hybrid,
            corridors: node_ok.corridors.clone(),
        };

        assert!(policy.check_node_metadata(&node_bad_pq).is_err());
    }

    #[test]
    fn decide_node_action_maps_soft_and_hard_violations() {
        let policy = EcosafetyPolicy {
            policy_id: "POLICY".to_string(),
            applies_domain: EcosafetyDomain::Wetland,
            on_soft_violation_derate: true,
            on_hard_violation_stop: true,
            min_corridors_per_node: 1,
            enforce_pq_strict: false,
        };

        let soft = CorridorEvalResult {
            corridor_id: "C".to_string(),
            status: CorridorStatus::SoftViolation {
                coord_name: "PFAS".to_string(),
                delta: -0.1,
            },
            vt: None,
        };
        let hard = CorridorEvalResult {
            corridor_id: "C".to_string(),
            status: CorridorStatus::HardViolation {
                coord_name: "PFAS".to_string(),
                delta: 0.5,
            },
            vt: None,
        };

        assert_eq!(
            decide_node_action(&policy, &soft),
            NodeAction::Derate { factor: 0.5 }
        );
        assert_eq!(decide_node_action(&policy, &hard), NodeAction::Stop);
    }
}
