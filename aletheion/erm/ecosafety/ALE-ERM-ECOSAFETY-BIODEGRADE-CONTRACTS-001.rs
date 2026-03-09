#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalStatus {
    Satisfied,
    SoftViolation,
    HardViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeAction {
    Allow,
    Derate,
    Stop,
}

#[derive(Debug, Clone)]
pub struct RiskCoord {
    pub name: String,
    pub value: f64,
    pub min_safe: f64,
    pub max_safe: f64,
}

#[derive(Debug, Clone)]
pub struct RiskVector {
    pub id: String,
    pub coords: Vec<RiskCoord>,
}

#[derive(Debug, Clone)]
pub struct LyapunovResidual {
    pub system_id: String,
    pub t: f64,
    pub value: f64,
    pub dvalue_dt: f64,
    pub stable: bool,
}

#[derive(Debug, Clone)]
pub struct BiodegradeNodeContext {
    pub node_id: String,
    pub corridor_id: String,
    pub risk_vector: RiskVector,
    pub vt: Option<LyapunovResidual>,
}

pub fn eval_corridor(ctx: &BiodegradeNodeContext) -> EvalStatus {
    let mut soft = false;
    for c in &ctx.risk_vector.coords {
        if c.value < c.min_safe || c.value > c.max_safe {
            return EvalStatus::HardViolation;
        }
        let span = (c.max_safe - c.min_safe).max(1e-6);
        let dist = (c.value - c.min_safe).min(c.max_safe - c.value).abs();
        if dist / span < 0.05 {
            soft = true;
        }
    }
    if soft {
        EvalStatus::SoftViolation
    } else {
        EvalStatus::Satisfied
    }
}

pub fn decide_node_action(status: EvalStatus) -> NodeAction {
    match status {
        EvalStatus::Satisfied => NodeAction::Allow,
        EvalStatus::SoftViolation => NodeAction::Derate,
        EvalStatus::HardViolation => NodeAction::Stop,
    }
}

pub fn check_lyapunov_monotone(history: &[LyapunovResidual], tol: f64) -> bool {
    if history.is_empty() {
        return true;
    }
    let mut last = history[0].value;
    for h in &history[1..] {
        if h.value > last + tol {
            return false;
        }
        last = h.value;
    }
    true
}

#[derive(Debug, Clone)]
pub struct SafeStepDecision {
    pub action: NodeAction,
    pub status: EvalStatus,
    pub reason: Option<String>,
}

pub fn safestep(
    ctx: &BiodegradeNodeContext,
    vt_history: Option<&[LyapunovResidual]>,
    vt_tol: f64,
) -> SafeStepDecision {
    let status = eval_corridor(ctx);
    let action = decide_node_action(status);

    if let Some(hist) = vt_history {
        if !check_lyapunov_monotone(hist, vt_tol) {
            return SafeStepDecision {
                action: NodeAction::Stop,
                status: EvalStatus::HardViolation,
                reason: Some("Lyapunov residual not monotone".to_string()),
            };
        }
    }

    let reason = match status {
        EvalStatus::Satisfied => None,
        EvalStatus::SoftViolation => Some("soft corridor edge".to_string()),
        EvalStatus::HardViolation => Some("hard corridor violation".to_string()),
    };

    SafeStepDecision {
        action,
        status,
        reason,
    }
}
