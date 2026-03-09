//! Heat–Water–Tree Optimization Engine for Downtown/Central Phoenix. [file:4]
//! Generates multi-objective cooling plans using NSGA-II-style evolutionary search,
//! under hard water constraints and SMART-Chain / ALN governance. [file:4]

use std::collections::HashMap;
use std::time::SystemTime;

use crate::erm_engine::{
    ActuationTargetId,
    ActuationTargetKind,
    ErmActionCommand,
    ErmActionPlan,
    ErmStateSnapshot,
    GovernanceDecision,
    OptimizationAdapter,
    Quantity,
    ResourceDomain,
    ResourceId,
    WorkflowId,
    ZoneId,
};

//
// -------------------------
// Downtown pilot abstractions
// -------------------------
//

/// Identifier for a downtown block in the pilot state model. [file:4]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(pub String);

/// Cooling intervention types allowed by the downtown pilot. [file:4]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterventionKind {
    TreePlanting,
    TreeIrrigation,
    CoolPavement,
    HydrationStation,
    MistingSystem,
}

/// Vulnerability profile for a block, derived from HVI/HEVI and census factors. [file:4]
#[derive(Debug, Clone)]
pub struct BlockVulnerability {
    pub heat_vulnerability_index: f64,
    pub outdoor_worker_density: f64,
    pub low_income_fraction: f64,
    pub elderly_fraction: f64,
}

/// Ecological sensitivity for a block (e.g., proximity to eco corridors). [file:4]
#[derive(Debug, Clone)]
pub struct BlockEcology {
    pub habitat_connectivity_score: f64,
    pub protected_species_weight: f64,
}

/// Static constraints for a candidate downtown plan. [file:4]
#[derive(Debug, Clone)]
pub struct DowntownConstraints {
    /// Daily reclaimed water budget from AWP + canals for cooling. [file:4]
    pub max_cooling_water_m3: f64,
    /// Maximum number of new trees that can be planted in the planning horizon. [file:4]
    pub max_tree_plantings: u32,
    /// Maximum cool pavement lane-kilometers. [file:4]
    pub max_cool_pavement_km: f64,
}

/// Per-block intervention decision variables. [file:4]
#[derive(Debug, Clone)]
pub struct BlockDecision {
    pub block_id: BlockId,
    /// 0..1 fraction of feasible tree spots to plant. [file:4]
    pub tree_planting_fraction: f64,
    /// Irrigation intensity scalar (0..1) relative to baseline. [file:4]
    pub irrigation_scalar: f64,
    /// 0..1 fraction of feasible cool pavement surface. [file:4]
    pub cool_pavement_fraction: f64,
    /// 0..1 deployment level of hydration stations/misters. [file:4]
    pub hydration_level: f64,
}

/// Candidate plan genome for the evolutionary search. [file:4]
#[derive(Debug, Clone)]
pub struct HeatWaterTreeGenome {
    pub decisions: Vec<BlockDecision>,
}

//
// ----------------------
// Objective computations
// ----------------------
//

/// Utility functions for translating genomes into objective scores and resource usage. [file:4]
pub struct ObjectiveEvaluator {
    /// Block vulnerability profiles keyed by block id. [file:4]
    pub vulnerabilities: HashMap<BlockId, BlockVulnerability>,
    /// Block ecological profiles keyed by block id. [file:4]
    pub ecologies: HashMap<BlockId, BlockEcology>,
    /// Mapping from block id to zone id for ERM commands. [file:4][file:6]
    pub block_to_zone: HashMap<BlockId, ZoneId>,
    /// Liters per day of irrigation water per unit of irrigation_scalar and tree_planting_fraction. [file:4]
    pub base_irrigation_l_per_unit: f64,
    /// Cooling effect coefficients for interventions (approximate). [file:4]
    pub canopy_cooling_per_fraction_deg_c: f64,
    pub cool_pavement_cooling_per_fraction_deg_c: f64,
    pub hydration_cooling_per_level_deg_c: f64,
}

impl ObjectiveEvaluator {
    /// Estimate total daily water use in cubic meters for a genome. [file:4]
    pub fn estimate_water_use_m3(&self, genome: &HeatWaterTreeGenome) -> f64 {
        let mut total_liters = 0.0;
        for decision in &genome.decisions {
            let irrigation = decision.irrigation_scalar.max(0.0).min(1.0);
            let planting = decision.tree_planting_fraction.max(0.0).min(1.0);
            let block_factor = irrigation * planting;
            total_liters += block_factor * self.base_irrigation_l_per_unit;
        }
        total_liters / 1000.0
    }

    /// Compute heat-risk reduction score (higher is better) emphasizing vulnerable blocks. [file:4]
    pub fn compute_heat_risk_score(&self, genome: &HeatWaterTreeGenome) -> f64 {
        let mut score = 0.0;
        for decision in &genome.decisions {
            if let Some(v) = self.vulnerabilities.get(&decision.block_id) {
                let canopy = decision.tree_planting_fraction;
                let cool_pavement = decision.cool_pavement_fraction;
                let hydration = decision.hydration_level;

                let cooling_deg_c = canopy * self.canopy_cooling_per_fraction_deg_c
                    + cool_pavement * self.cool_pavement_cooling_per_fraction_deg_c
                    + hydration * self.hydration_cooling_per_level_deg_c;

                let vulnerability_weight = v.heat_vulnerability_index
                    + 0.5 * v.outdoor_worker_density
                    + 0.3 * v.low_income_fraction
                    + 0.2 * v.elderly_fraction;

                score += cooling_deg_c * vulnerability_weight;
            }
        }
        score
    }

    /// Compute ecological benefit score (higher is better). [file:4]
    pub fn compute_ecology_score(&self, genome: &HeatWaterTreeGenome) -> f64 {
        let mut score = 0.0;
        for decision in &genome.decisions {
            if let Some(e) = self.ecologies.get(&decision.block_id) {
                let canopy = decision.tree_planting_fraction;
                let ecological_gain = canopy * e.habitat_connectivity_score * e.protected_species_weight;
                score += ecological_gain;
            }
        }
        score
    }
}

//
// ---------------------
// NSGA-II style engine
// ---------------------
//

/// Hyperparameters for the evolutionary search. [file:4]
#[derive(Debug, Clone)]
pub struct EvolutionParams {
    pub population_size: usize,
    pub generations: usize,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
}

/// Heat–Water–Tree optimizer configuration. [file:4]
#[derive(Debug, Clone)]
pub struct HeatWaterTreeConfig {
    pub downtown_constraints: DowntownConstraints,
    pub evolution: EvolutionParams,
    /// Workflow id used when emitting ERM plans (e.g., "HEAT_WATER_TREE_DOWNTOWN"). [file:4]
    pub workflow_id: WorkflowId,
    /// Resource id for the daily AWP+canal cooling water portfolio. [file:4][file:6]
    pub cooling_water_resource_id: ResourceId,
}

/// Concrete optimizer implementing the OptimizationAdapter interface. [file:4][file:6]
pub struct HeatWaterTreeOptimizer {
    pub cfg: HeatWaterTreeConfig,
    pub evaluator: ObjectiveEvaluator,
}

impl HeatWaterTreeOptimizer {
    /// Initialize an optimizer with its configuration and evaluator. [file:4]
    pub fn new(cfg: HeatWaterTreeConfig, evaluator: ObjectiveEvaluator) -> Self {
        Self { cfg, evaluator }
    }

    /// Generate an initial random population of genomes. [file:4]
    fn init_population(&self, template: &HeatWaterTreeGenome) -> Vec<HeatWaterTreeGenome> {
        let mut population = Vec::with_capacity(self.cfg.evolution.population_size);
        for i in 0..self.cfg.evolution.population_size {
            let factor = (i as f64 / self.cfg.evolution.population_size as f64).min(1.0);
            let mut genome = template.clone();
            for d in &mut genome.decisions {
                d.tree_planting_fraction = (d.tree_planting_fraction + factor * 0.2).min(1.0);
                d.irrigation_scalar = (d.irrigation_scalar + factor * 0.2).min(1.0);
                d.cool_pavement_fraction = (d.cool_pavement_fraction + factor * 0.1).min(1.0);
                d.hydration_level = (d.hydration_level + factor * 0.3).min(1.0);
            }
            population.push(genome);
        }
        population
    }

    /// Single-point crossover between two genomes. [file:4]
    fn crossover(&self, a: &HeatWaterTreeGenome, b: &HeatWaterTreeGenome) -> HeatWaterTreeGenome {
        let len = a.decisions.len().min(b.decisions.len());
        if len == 0 {
            return a.clone();
        }
        let pivot = len / 2;
        let mut decisions = Vec::with_capacity(len);
        for i in 0..len {
            if i < pivot {
                decisions.push(a.decisions[i].clone());
            } else {
                decisions.push(b.decisions[i].clone());
            }
        }
        HeatWaterTreeGenome { decisions }
    }

    /// Simple mutation nudging decision variables. [file:4]
    fn mutate(&self, genome: &mut HeatWaterTreeGenome) {
        for d in &mut genome.decisions {
            d.tree_planting_fraction = (d.tree_planting_fraction + 0.05).min(1.0);
            d.irrigation_scalar = (d.irrigation_scalar + 0.05).min(1.0);
            d.cool_pavement_fraction = (d.cool_pavement_fraction + 0.02).min(1.0);
            d.hydration_level = (d.hydration_level + 0.05).min(1.0);
        }
    }

    /// Evaluate a genome into objective scores and check hard water constraint. [file:4]
    fn evaluate_genome(&self, genome: &HeatWaterTreeGenome) -> (f64, f64, bool) {
        let water_use_m3 = self.evaluator.estimate_water_use_m3(genome);
        let within_constraint = water_use_m3 <= self.cfg.downtown_constraints.max_cooling_water_m3;
        let heat_score = if within_constraint {
            self.evaluator.compute_heat_risk_score(genome)
        } else {
            -1.0
        };
        let ecology_score = if within_constraint {
            self.evaluator.compute_ecology_score(genome)
        } else {
            -1.0
        };
        (heat_score, ecology_score, within_constraint)
    }

    /// Very small Pareto filter: keep non-dominated genomes only. [file:4]
    fn non_dominated_filter(
        &self,
        candidates: &[(HeatWaterTreeGenome, f64, f64)],
    ) -> Vec<(HeatWaterTreeGenome, f64, f64)> {
        let mut result = Vec::new();
        'outer: for (i, (g_i, h_i, e_i)) in candidates.iter().enumerate() {
            for (j, (_, h_j, e_j)) in candidates.iter().enumerate() {
                if j == i {
                    continue;
                }
                let dominates = h_j >= h_i && e_j >= e_i && (h_j > h_i || e_j > e_i);
                if dominates {
                    continue 'outer;
                }
            }
            result.push((g_i.clone(), *h_i, *e_i));
        }
        result
    }

    /// Run a coarse NSGA-II-like search and return a Pareto set of genomes. [file:4]
    fn search(&self, template: &HeatWaterTreeGenome) -> Vec<(HeatWaterTreeGenome, f64, f64)> {
        let mut population = self.init_population(template);
        for _gen in 0..self.cfg.evolution.generations {
            let mut evaluated: Vec<(HeatWaterTreeGenome, f64, f64)> = population
                .iter()
                .map(|g| {
                    let (h, e, ok) = self.evaluate_genome(g);
                    if ok {
                        (g.clone(), h, e)
                    } else {
                        (g.clone(), -1.0, -1.0)
                    }
                })
                .collect();

            evaluated = self.non_dominated_filter(&evaluated);

            let mut next_pop = Vec::new();
            for chunk in evaluated.chunks(2) {
                if chunk.len() == 1 {
                    next_pop.push(chunk[0].0.clone());
                } else {
                    let child = self.crossover(&chunk[0].0, &chunk[1].0);
                    let mut mutated = child.clone();
                    self.mutate(&mut mutated);
                    next_pop.push(mutated);
                }
            }
            if next_pop.is_empty() {
                break;
            }
            population = next_pop;
        }

        population
            .into_iter()
            .map(|g| {
                let (h, e, _) = self.evaluate_genome(&g);
                (g, h, e)
            })
            .collect()
    }

    /// Convert a genome into an ERM action plan for the downtown zone. [file:4][file:6]
    fn genome_to_plan(
        &self,
        genome: &HeatWaterTreeGenome,
        heat_score: f64,
        ecology_score: f64,
        snapshot: &ErmStateSnapshot,
    ) -> ErmActionPlan {
        let now = SystemTime::now();
        let mut commands = Vec::new();

        let mut total_irrigation_m3 = 0.0;
        for decision in &genome.decisions {
            if let Some(zone) = self.evaluator.block_to_zone.get(&decision.block_id) {
                let irrigation = decision.irrigation_scalar.max(0.0).min(1.0);
                let planting = decision.tree_planting_fraction.max(0.0).min(1.0);
                let block_water_m3 =
                    irrigation * planting * (self.evaluator.base_irrigation_l_per_unit / 1000.0);
                total_irrigation_m3 += block_water_m3;

                let mut payload = HashMap::new();
                payload.insert("tree_planting_fraction".into(), planting);
                payload.insert("irrigation_scalar".into(), irrigation);
                payload.insert(
                    "cool_pavement_fraction".into(),
                    decision.cool_pavement_fraction.max(0.0).min(1.0),
                );
                payload.insert(
                    "hydration_level".into(),
                    decision.hydration_level.max(0.0).min(1.0),
                );

                let cmd = ErmActionCommand {
                    workflow_id: self.cfg.workflow_id.clone(),
                    target_kind: ActuationTargetKind::TreeIrrigationCluster,
                    target_id: ActuationTargetId(format!("BLOCK_{}", decision.block_id.0)),
                    payload,
                    effective_at: now,
                    zone: zone.clone(),
                };
                commands.push(cmd);
            }
        }

        let mut scores = HashMap::new();
        scores.insert("heat_risk_reduction".into(), heat_score);
        scores.insert("ecological_benefit".into(), ecology_score);
        scores.insert("water_use_m3".into(), total_irrigation_m3);

        ErmActionPlan {
            id: format!(
                "HEAT_WATER_TREE_PLAN_{}_{}",
                self.cfg.workflow_id.0,
                now
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            created_at: now,
            based_on_snapshot: snapshot.snapshot_time,
            commands,
            scores,
            governance_decision: GovernanceDecision::Allowed,
        }
    }

    /// Build a minimal template genome from the state snapshot and block mapping. [file:4][file:6]
    fn build_template_genome(&self, snapshot: &ErmStateSnapshot) -> HeatWaterTreeGenome {
        let mut decisions = Vec::new();
        for (block_id, _) in self.evaluator.vulnerabilities.iter() {
            if let Some(zone) = self.evaluator.block_to_zone.get(block_id) {
                let _zone = zone.clone();
                decisions.push(BlockDecision {
                    block_id: block_id.clone(),
                    tree_planting_fraction: 0.1,
                    irrigation_scalar: 0.5,
                    cool_pavement_fraction: 0.0,
                    hydration_level: 0.0,
                });
            }
        }
        HeatWaterTreeGenome { decisions }
    }
}

impl OptimizationAdapter for HeatWaterTreeOptimizer {
    /// Produce a Pareto set of downtown cooling plans as ERM action plans. [file:4][file:6]
    fn propose_plans(
        &self,
        workflow: &WorkflowId,
        snapshot: &ErmStateSnapshot,
    ) -> Vec<ErmActionPlan> {
        if *workflow != self.cfg.workflow_id {
            return Vec::new();
        }

        let template = self.build_template_genome(snapshot);
        let genomes = self.search(&template);

        let mut plans = Vec::new();
        for (genome, heat_score, ecology_score) in genomes {
            if heat_score < 0.0 || ecology_score < 0.0 {
                continue;
            }
            let plan = self.genome_to_plan(&genome, heat_score, ecology_score, snapshot);
            plans.push(plan);
        }

        plans
    }
}
