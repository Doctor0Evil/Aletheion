
use crate::contracts::*;
use crate::governance::TreatyGuard;
use crate::metrics::ScenarioMetrics;

pub struct HeatWaveOrchestrator<'a> {
    pub workflows: Vec<Box<dyn CityMeshWorkflow + Send + Sync>>,
    pub treaty_guards: Vec<Box<dyn TreatyGuard + Send + Sync>>,
    pub metrics: &'a mut ScenarioMetrics,
}

impl<'a> HeatWaveOrchestrator<'a> {
    pub fn run_tick(&mut self, ctx: &CityContext, global_constraints: &ConstraintsIn, states: &[DomainStateIn]) {
        let mut proposals: Vec<WorkflowOutput> = Vec::new();

        for wf in &self.workflows {
            if let Some(state) = states.iter().find(|s| s.domain_name == wf.workflow_id()) {
                let out = wf.evaluate_tick(ctx, global_constraints, state);
                self.metrics.observe_workflow_output(&out);
                proposals.push(out);
            }
        }

        let approved_actions = self.apply_governance_gates(ctx, global_constraints, &proposals);
        self.dispatch_actions(&approved_actions);
    }

    fn apply_governance_gates(
        &mut self,
        ctx: &CityContext,
        _constraints: &ConstraintsIn,
        outputs: &[WorkflowOutput],
    ) -> Vec<ActionProposal> {
        let mut approved = Vec::new();

        for out in outputs {
            for action in &out.actions {
                let mut blocked_reason: Option<String> = None;

                for guard in &self.treaty_guards {
                    let input = guard_input_from_action(ctx, action);
                    match guard.evaluate(&input) {
                        TreatyDecision::MayRun => {}
                        TreatyDecision::MustHalt(reason) => {
                            blocked_reason = Some(reason);
                            self.metrics.record_treaty_violation(&input);
                            break;
                        }
                    }
                }

                if blocked_reason.is_none() {
                    approved.push(action.clone());
                }
            }
        }

        approved
    }

    fn dispatch_actions(&self, actions: &[ActionProposal]) {
        // integration with infra control planes (water, thermal, mobility, etc.)
        // intentionally left abstract here
        let _ = actions;
    }
}

fn guard_input_from_action(ctx: &CityContext, action: &ActionProposal) -> crate::governance::TreatyScopeCheckInput {
    crate::governance::TreatyScopeCheckInput {
        treaty_id: String::new(),
        operation_kind: action.operation_kind.clone(),
        geo_point_wgs84: (0.0, 0.0),
        roh_0_1: 0.0,
        psr_0_1: 0.0,
        consent_token_present: false,
    }
}
