//! Biodegradable Cyboquatic Node Ecosafety Contract [file:1][file:3]
//! Implements \"no corridor, no build\" and \"violated corridor → derate/stop\"
//! for soft, partially biodegradable aquatic robots in Phoenix canals, MAR vaults,
//! wetlands, and corridor-managed reaches. [file:1][file:3]

use std::collections::HashMap;
use std::time::SystemTime;

//
// ----------------------
// Shared ecosafety types
// ----------------------
//

/// Normalized risk coordinate rx ∈ [0,1] for a specific dimension. [file:1]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskCoordinate {
    pub value: f64,          // 0.0 = best, 1.0 = worst
    pub label: &'static str, // e.g. \"r_degrade\", \"r_microplastics\" [file:1]
}

impl RiskCoordinate {
    pub fn new(label: &'static str, value: f64) -> Self {
        let clamped = if value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        };
        Self { value: clamped, label }
    }
}

/// Core biodegradable-material rx dimensions from ecosafety spine. [file:1]
#[derive(Debug, Clone)]
pub struct BiodegradableRiskVector {
    pub r_degrade: RiskCoordinate,
    pub r_residual_mass: RiskCoordinate,
    pub r_microplastics: RiskCoordinate,
    pub r_tox_acute: RiskCoordinate,
    pub r_tox_chronic: RiskCoordinate,
}

impl BiodegradableRiskVector {
    /// Simple weighted aggregation into a Lyapunov-like residual V_t. [file:1]
    pub fn v_t(&self) -> f64 {
        let w_deg = 0.25;
        let w_res = 0.15;
        let w_micro = 0.25;
        let w_tox_a = 0.15;
        let w_tox_c = 0.20;

        w_deg * self.r_degrade.value
            + w_res * self.r_residual_mass.value
            + w_micro * self.r_microplastics.value
            + w_tox_a * self.r_tox_acute.value
            + w_tox_c * self.r_tox_chronic.value
    }
}

/// System-wide ecosafety corridor for biodegradable nodes. [file:1]
#[derive(Debug, Clone)]
pub struct EcosafetyCorridor {
    pub id: String,
    /// Upper bounds for each rx dimension (inclusive). [file:1]
    pub max_r_degrade: f64,
    pub max_r_residual_mass: f64,
    pub max_r_microplastics: f64,
    pub max_r_tox_acute: f64,
    pub max_r_tox_chronic: f64,
    /// Maximum allowed V_t inside safe interior. [file:1]
    pub max_v_t: f64,
}

impl EcosafetyCorridor {
    pub fn check(&self, rv: &BiodegradableRiskVector) -> CorridorCheck {
        let mut violations = Vec::new();

        if rv.r_degrade.value > self.max_r_degrade {
            violations.push(CorridorViolation::RDegrade {
                value: rv.r_degrade.value,
                max: self.max_r_degrade,
            });
        }
        if rv.r_residual_mass.value > self.max_r_residual_mass {
            violations.push(CorridorViolation::RResidualMass {
                value: rv.r_residual_mass.value,
                max: self.max_r_residual_mass,
            });
        }
        if rv.r_microplastics.value > self.max_r_microplastics {
            violations.push(CorridorViolation::RMicroplastics {
                value: rv.r_microplastics.value,
                max: self.max_r_microplastics,
            });
        }
        if rv.r_tox_acute.value > self.max_r_tox_acute {
            violations.push(CorridorViolation::RToxAcute {
                value: rv.r_tox_acute.value,
                max: self.max_r_tox_acute,
            });
        }
        if rv.r_tox_chronic.value > self.max_r_tox_chronic {
            violations.push(CorridorViolation::RToxChronic {
                value: rv.r_tox_chronic.value,
                max: self.max_r_tox_chronic,
            });
        }

        let vt = rv.v_t();
        if vt > self.max_v_t {
            violations.push(CorridorViolation::VtResidual { value: vt, max: self.max_v_t });
        }

        if violations.is_empty() {
            CorridorCheck::Within
        } else {
            CorridorCheck::Violated { violations }
        }
    }
}

/// Result of checking risk vector against a corridor. [file:1]
#[derive(Debug, Clone)]
pub enum CorridorCheck {
    Within,
    Violated { violations: Vec<CorridorViolation> },
}

/// Detailed violation reasons for debugging and audit. [file:1]
#[derive(Debug, Clone)]
pub enum CorridorViolation {
    RDegrade { value: f64, max: f64 },
    RResidualMass { value: f64, max: f64 },
    RMicroplastics { value: f64, max: f64 },
    RToxAcute { value: f64, max: f64 },
    RToxChronic { value: f64, max: f64 },
    VtResidual { value: f64, max: f64 },
}

/// High-level ecosafety decision used by node contracts. [file:1]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcosafetyDecision {
    Allow,
    Derate,
    Stop,
}

//
// ---------------------
// Node identity & state
// ---------------------
//

/// Unique identifier for a cyboquatic node. [file:1]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

/// Where the node is allowed to operate (corridor-bound). [file:3]
#[derive(Debug, Clone)]
pub struct CorridorBinding {
    pub corridor_id: String,
    pub segment_id: String, // e.g. canal reach or MAR vault id. [file:3]
}

/// High-level lifecycle state of a node. [file:1]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeLifecycle {
    Registered,
    Deployed,
    Parked,
    Derated,
    Decommissioned,
}

/// Operational intent requested from higher-level schedulers. [file:3]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeIntent {
    Idle,
    Inspect,
    Sample,
    PumpAssist,
    FlowSteering,
}

/// Telemetry bundle sufficient for ecosafety evaluation. [file:1]
#[derive(Debug, Clone)]
pub struct NodeTelemetry {
    pub timestamp: SystemTime,
    /// Material risk vector derived from lab/field models (rx). [file:1]
    pub material_risk: BiodegradableRiskVector,
    /// Additional scalar tags like local temperature, DO, PFAS index. [file:1][file:3]
    pub scalars: HashMap<String, f64>,
}

/// Node configuration including design-time ecolimits. [file:1][file:3]
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: NodeId,
    pub corridor_binding: Option<CorridorBinding>,
    pub design_corridor: EcosafetyCorridor,
    /// Maximum safe duty cycle (0..1) used for derating. [file:1]
    pub max_duty_cycle: f64,
}

//
// ----------------------------
// Contract trait & evaluation
// ----------------------------
//

/// Ecosafety contract interface for biodegradable cyboquatic nodes. [file:1]
pub trait BiodegradableNodeContract {
    /// Called once when a node is first introduced to the system. [file:1]
    fn register_node(&mut self, config: NodeConfig) -> Result<(), ContractError>;

    /// Update node health/material state based on telemetry. [file:1]
    fn update_health_state(
        &mut self,
        id: &NodeId,
        telemetry: NodeTelemetry,
    ) -> Result<EcosafetyDecision, ContractError>;

    /// Request permission for a specific action/intent at this step. [file:1][file:3]
    fn request_action(
        &mut self,
        id: &NodeId,
        intent: NodeIntent,
    ) -> Result<EcosafetyDecision, ContractError>;

    /// Report a node failure and trigger immediate stop. [file:1]
    fn report_failure(&mut self, id: &NodeId, reason: String) -> Result<(), ContractError>;

    /// Permanently remove a node from service. [file:1]
    fn decommission(&mut self, id: &NodeId) -> Result<(), ContractError>;
}

/// Contract-level error codes (not transport errors). [file:1]
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error(\"node not found: {0}\")]
    NodeNotFound(String),

    #[error(\"no corridor binding present (no corridor, no build) for node {0}\")]
    NoCorridorBinding(String),

    #[error(\"invalid lifecycle transition for node {0}\")]
    InvalidLifecycle(String),

    #[error(\"ecosafety corridor violation requires STOP for node {0}\")]
    HardViolation(String),
}

//
// ------------------------------
// In-memory contract implementation
// ------------------------------
//

/// Internal state tracked per node for ecosafety decisions. [file:1]
#[derive(Debug, Clone)]
struct NodeRuntimeState {
    config: NodeConfig,
    lifecycle: NodeLifecycle,
    last_risk: Option<BiodegradableRiskVector>,
    last_decision: EcosafetyDecision,
    last_updated: Option<SystemTime>,
}

impl NodeRuntimeState {
    fn new(config: NodeConfig) -> Self {
        Self {
            config,
            lifecycle: NodeLifecycle::Registered,
            last_risk: None,
            last_decision: EcosafetyDecision::Allow,
            last_updated: None,
        }
    }
}

/// In-memory contract engine; may be wrapped by corridor/FOG routing layers. [file:1]
pub struct InMemoryBiodegradableNodeContract {
    nodes: HashMap<NodeId, NodeRuntimeState>,
}

impl InMemoryBiodegradableNodeContract {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Evaluate risk vector against node's design corridor and derive decision. [file:1]
    fn evaluate_corridor(
        config: &NodeConfig,
        rv: &BiodegradableRiskVector,
    ) -> EcosafetyDecision {
        match config.design_corridor.check(rv) {
            CorridorCheck::Within => EcosafetyDecision::Allow,
            CorridorCheck::Violated { violations } => {
                let hard_violation = violations.iter().any(|v| match v {
                    CorridorViolation::RToxAcute { .. }
                    | CorridorViolation::RToxChronic { .. }
                    | CorridorViolation::VtResidual { .. } => true,
                    _ => false,
                });

                if hard_violation {
                    EcosafetyDecision::Stop
                } else {
                    EcosafetyDecision::Derate
                }
            }
        }
    }

    /// Enforce \"no corridor, no build\" at registration time. [file:1][file:3]
    fn ensure_corridor_binding(config: &NodeConfig) -> Result<(), ContractError> {
        if config.corridor_binding.is_none() {
            return Err(ContractError::NoCorridorBinding(config.id.0.clone()));
        }
        Ok(())
    }
}

impl BiodegradableNodeContract for InMemoryBiodegradableNodeContract {
    fn register_node(&mut self, config: NodeConfig) -> Result<(), ContractError> {
        Self::ensure_corridor_binding(&config)?;
        let id = config.id.clone();
        let state = NodeRuntimeState::new(config);
        self.nodes.insert(id, state);
        Ok(())
    }

    fn update_health_state(
        &mut self,
        id: &NodeId,
        telemetry: NodeTelemetry,
    ) -> Result<EcosafetyDecision, ContractError> {
        let state = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| ContractError::NodeNotFound(id.0.clone()))?;

        if state.lifecycle == NodeLifecycle::Decommissioned {
            return Err(ContractError::InvalidLifecycle(id.0.clone()));
        }

        let decision = Self::evaluate_corridor(&state.config, &telemetry.material_risk);

        match decision {
            EcosafetyDecision::Allow => {
                if state.lifecycle == NodeLifecycle::Registered {
                    state.lifecycle = NodeLifecycle::Deployed;
                } else if state.lifecycle == NodeLifecycle::Derated {
                    state.lifecycle = NodeLifecycle::Deployed;
                }
            }
            EcosafetyDecision::Derate => {
                if matches!(
                    state.lifecycle,
                    NodeLifecycle::Registered | NodeLifecycle::Deployed
                ) {
                    state.lifecycle = NodeLifecycle::Derated;
                }
            }
            EcosafetyDecision::Stop => {
                state.lifecycle = NodeLifecycle::Parked;
            }
        }

        state.last_risk = Some(telemetry.material_risk);
        state.last_decision = decision.clone();
        state.last_updated = Some(telemetry.timestamp);

        Ok(decision)
    }

    fn request_action(
        &mut self,
        id: &NodeId,
        intent: NodeIntent,
    ) -> Result<EcosafetyDecision, ContractError> {
        let state = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| ContractError::NodeNotFound(id.0.clone()))?;

        if state.lifecycle == NodeLifecycle::Decommissioned {
            return Err(ContractError::InvalidLifecycle(id.0.clone()));
        }

        if let Some(rv) = &state.last_risk {
            let decision = Self::evaluate_corridor(&state.config, rv);

            match decision {
                EcosafetyDecision::Allow => {
                    if state.lifecycle == NodeLifecycle::Registered {
                        state.lifecycle = NodeLifecycle::Deployed;
                    }
                }
                EcosafetyDecision::Derate => {
                    state.lifecycle = NodeLifecycle::Derated;
                }
                EcosafetyDecision::Stop => {
                    state.lifecycle = NodeLifecycle::Parked;
                }
            }

            let gated_decision = match intent {
                NodeIntent::Idle => EcosafetyDecision::Allow,
                NodeIntent::Inspect | NodeIntent::Sample => decision.clone(),
                NodeIntent::PumpAssist | NodeIntent::FlowSteering => match decision {
                    EcosafetyDecision::Allow => EcosafetyDecision::Derate,
                    other => other,
                },
            };

            if gated_decision == EcosafetyDecision::Stop {
                return Err(ContractError::HardViolation(id.0.clone()));
            }

            state.last_decision = gated_decision.clone();
            Ok(gated_decision)
        } else {
            state.lifecycle = NodeLifecycle::Parked;
            Ok(EcosafetyDecision::Derate)
        }
    }

    fn report_failure(&mut self, id: &NodeId, _reason: String) -> Result<(), ContractError> {
        let state = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| ContractError::NodeNotFound(id.0.clone()))?;
        state.lifecycle = NodeLifecycle::Parked;
        state.last_decision = EcosafetyDecision::Stop;
        Ok(())
    }

    fn decommission(&mut self, id: &NodeId) -> Result<(), ContractError> {
        let state = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| ContractError::NodeNotFound(id.0.clone()))?;
        state.lifecycle = NodeLifecycle::Decommissioned;
        Ok(())
    }
}
