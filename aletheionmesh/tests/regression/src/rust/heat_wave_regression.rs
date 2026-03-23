
use crate::metrics::ScenarioMetrics;
use crate::heat_wave_orchestrator::HeatWaveOrchestrator;
use crate::contracts::{CityContext, ConstraintsIn, DomainStateIn};

pub fn run_heat_wave_regression(trace_id: &str, metrics_thresholds: &ScenarioMetrics) -> bool {
    let mut metrics = ScenarioMetrics::new(trace_id.to_string());
    let mut orchestrator = HeatWaveOrchestrator {
        workflows: Vec::new(),      // injected in real wiring
        treaty_guards: Vec::new(),  // injected in real wiring
        metrics: &mut metrics,
    };

    let replay = crate::trace_loader::load_trace(trace_id);

    for tick in replay.ticks {
        let ctx = CityContext {
            story_id: "HEAT_WAVE_AWP_STRESS_V1".into(),
            phase_id: tick.phase_id.clone(),
            ts_utc_ms: tick.ts_utc_ms,
        };

        let constraints = ConstraintsIn {
            treaty_ids: tick.treaty_ids.clone(),
            ecosafety_corridor_ids: tick.corridor_ids.clone(),
            neurorights_policy_id: tick.neurorights_policy_id.clone(),
            vt_safe_ceiling: tick.vt_safe_ceiling,
        };

        let states: Vec<DomainStateIn> = tick
            .domain_states
            .iter()
            .map(|s| DomainStateIn {
                domain_name: s.domain_name.clone(),
                state_shard_id: s.state_shard_id.clone(),
            })
            .collect();

        orchestrator.run_tick(&ctx, &constraints, &states);
    }

    metrics.check_against_thresholds(metrics_thresholds)
}
