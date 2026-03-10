// Module: Core governance framework for cyboquatic systems with K/E/R scoring, risk coordinates, and Lyapunov residuals

pub mod risk_coordinates;
pub mod lyapunov_residual;
pub mod aln_contracts;
pub mod ecosafety_grammar;
pub mod deployment_gates;

use crate::risk_coordinates::{RiskCoordinate, NormalizedRisk};
use crate::lyapunov_residual::{LyapunovResidual, LyapunovState};
use crate::aln_contracts::{ALNContract, ContractViolationResponse};
use crate::ecosafety_grammar::{EcosafetyGrammar, GrammarRule};
use crate::deployment_gates::{KERThresholds, DeploymentGate};
use aletheion_core::qpudatashard::QpuDataShard;
use aletheion_core::environmental_state::EnvironmentalState;
use std::collections::HashMap;

// Primary framework struct integrating all governance components
pub struct CyboquaticGovernanceFramework {
    contracts: HashMap<String, ALNContract>,
    risk_bounds: HashMap<String, (f64, f64)>, // (min, max) for each r_x
    lyapunov_state: LyapunovResidual,
    ker_thresholds: KERThresholds,
    grammar: EcosafetyGrammar,
}

impl CyboquaticGovernanceFramework {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            risk_bounds: HashMap::new(),
            lyapunov_state: LyapunovResidual::default(),
            ker_thresholds: KERThresholds::default_2026_targets(),
            grammar: EcosafetyGrammar::init_spine(),
        }
    }

    // Ingest qpudatashard and update internal state
    pub fn ingest_shard(&mut self, shard: &QpuDataShard) -> bool {
        let prev_vt = self.lyapunov_state.current_residual();
        self.lyapunov_state.update_from_shard(shard);
        let new_vt = self.lyapunov_state.current_residual();
        new_vt <= prev_vt // Enforce non-increasing invariant
    }

    // Validate action against all corridors and invariants
    pub fn validate_action(
        &self,
        action: &ControlAction,
        state: &EnvironmentalState,
    ) -> Result<(), ContractViolationResponse> {
        // Check risk coordinates against bounds
        for (param, rx) in state.risk_coordinates.iter() {
            if let Some((min, max)) = self.risk_bounds.get(param) {
                if *rx.value() < *min || *rx.value() > *max {
                    return Err(ContractViolationResponse::DerateOrStop);
                }
            }
        }

        // Grammar-based safety check
        self.grammar.validate_action(action, state)?;

        // Deployment gate check
        self.ker_thresholds.check(state)?;

        // Lyapunov stability check
        let projected_vt = self.lyapunov_state.project(action, state);
        if projected_vt > self.lyapunov_state.current_residual() {
            return Err(ContractViolationResponse::Stop("Lyapunov increasing".to_string()));
        }

        Ok(())
    }

    // Tighten corridors based on operational feedback (reduce R toward 0.10)
    pub fn tighten_corridors(&mut self, feedback: &OperationalFeedback) {
        for (param, rx) in feedback.improved_risks.iter() {
            if let Some(bounds) = self.risk_bounds.get_mut(param) {
                bounds.1 = bounds.1 * 0.95; // Shrink upper bound
            }
        }
    }
}
