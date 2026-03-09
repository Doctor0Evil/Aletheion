// Aletheion Phoenix – NSGA-II Heat–Water–Tree Optimization Engine
// Destination: workflows/pipelines/heat-water-tree/nsga2_heat_mitigation_engine.rs
// Purpose: compute daily Pareto-optimal action plans for canal flows, AWP reuse, tree irrigation, pavements, and hydration nodes.[file:10][file:11][file:12]

mod domain {
    use std::collections::HashMap;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct CorridorId(pub String);

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct NodeId(pub String);

    #[derive(Clone, Debug)]
    pub struct TimeSlot {
        pub idx: u32,
        pub label: String,
    }

    #[derive(Clone, Debug)]
    pub struct CanalState {
        pub corridor: CorridorId,
        pub max_flow_lps: f64,
        pub min_env_flow_lps: f64,
        pub current_temp_c: f64,
    }

    #[derive(Clone, Debug)]
    pub struct AwpAvailability {
        pub plant_id: String,
        pub max_reclaimed_m3: f64,
        pub corridor_targets: HashMap<CorridorId, f64>,
    }

    #[derive(Clone, Debug)]
    pub struct TreeCluster {
        pub corridor: CorridorId,
        pub node: NodeId,
        pub canopy_m2: f64,
        pub soil_moisture_pct: f64,
        pub heat_risk_score: f64,
    }

    #[derive(Clone, Debug)]
    pub struct PavementCell {
        pub corridor: CorridorId,
        pub node: NodeId,
        pub albedo: f64,
        pub has_cool_pavement: bool,
        pub human_exposure_idx: f64,
    }

    #[derive(Clone, Debug)]
    pub struct HydrationNode {
        pub corridor: CorridorId,
        pub node: NodeId,
        pub daily_capacity_l: f64,
        pub vulnerable_weight: f64,
    }

    #[derive(Clone, Debug)]
    pub struct HeatInputs {
        pub timeslots: Vec<TimeSlot>,
        pub canals: Vec<CanalState>,
        pub awp: Vec<AwpAvailability>,
        pub trees: Vec<TreeCluster>,
        pub pavements: Vec<PavementCell>,
        pub hydration: Vec<HydrationNode>,
    }

    #[derive(Clone, Debug)]
    pub struct CorridorAction {
        pub corridor: CorridorId,
        pub canal_flow_lps: f64,
        pub awp_reuse_m3: f64,
        pub irrigation_mm: f64,
        pub pavement_misting_l: f64,
        pub hydration_alloc_l: f64,
    }

    #[derive(Clone, Debug)]
    pub struct DailyPlan {
        pub timeslot: TimeSlot,
        pub actions: Vec<CorridorAction>,
    }

    #[derive(Clone, Debug)]
    pub struct Candidate {
        pub plans: Vec<DailyPlan>,
        pub f_heat: f64,
        pub f_water: f64,
        pub f_vulnerable: f64,
    }
}

mod constraints {
    use super::domain::*;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    pub struct ConstraintProfile {
        pub max_corridor_awp_m3: HashMap<CorridorId, f64>,
        pub max_irrigation_mm: f64,
        pub max_pavement_misting_l: f64,
        pub min_env_flow_factor: f64,
    }

    impl ConstraintProfile {
        pub fn default_from_inputs(inputs: &HeatInputs) -> Self {
            let mut max_corridor_awp_m3 = HashMap::new();
            for awp in &inputs.awp {
                for (cid, v) in &awp.corridor_targets {
                    *max_corridor_awp_m3.entry(cid.clone()).or_insert(0.0) += *v;
                }
            }
            ConstraintProfile {
                max_corridor_awp_m3,
                max_irrigation_mm: 15.0,
                max_pavement_misting_l: 2000.0,
                min_env_flow_factor: 1.05,
            }
        }

        pub fn is_feasible(&self, candidate: &Candidate, inputs: &HeatInputs) -> bool {
            use std::collections::HashMap;
            let mut corridor_awp_sum: HashMap<CorridorId, f64> = HashMap::new();

            for plan in &candidate.plans {
                for act in &plan.actions {
                    if act.irrigation_mm < 0.0
                        || act.irrigation_mm > self.max_irrigation_mm
                        || act.pavement_misting_l < 0.0
                        || act.pavement_misting_l > self.max_pavement_misting_l
                        || act.awp_reuse_m3 < 0.0
                        || act.canal_flow_lps < 0.0
                    {
                        return false;
                    }
                    *corridor_awp_sum.entry(act.corridor.clone()).or_insert(0.0) += act.awp_reuse_m3;
                }
            }

            for (cid, used) in &corridor_awp_sum {
                if let Some(max_allowed) = self.max_corridor_awp_m3.get(cid) {
                    if used > max_allowed + 1e-6 {
                        return false;
                    }
                }
            }

            for c in &inputs.canals {
                let mut max_flow_used = 0.0;
                for plan in &candidate.plans {
                    for act in &plan.actions {
                        if act.corridor == c.corridor {
                            if act.canal_flow_lps > max_flow_used {
                                max_flow_used = act.canal_flow_lps;
                            }
                        }
                    }
                }
                let min_env_flow = c.min_env_flow_lps * self.min_env_flow_factor;
                if max_flow_used > c.max_flow_lps - min_env_flow + 1e-6 {
                    return false;
                }
            }
            true
        }
    }
}

mod objectives {
    use super::domain::*;

    pub fn evaluate_candidate(candidate: &mut Candidate, inputs: &HeatInputs) {
        let mut heat_score = 0.0;
        let mut water_use = 0.0;
        let mut vulnerable_cooling = 0.0;

        for plan in &candidate.plans {
            for act in &plan.actions {
                let mut tree_score = 0.0;
                for t in &inputs.trees {
                    if t.corridor == act.corridor {
                        let dryness = (100.0 - t.soil_moisture_pct).max(0.0) / 100.0;
                        let irrigation_effect = (act.irrigation_mm / 10.0).min(1.0);
                        tree_score += t.heat_risk_score * dryness * (1.0 - irrigation_effect);
                    }
                }

                let mut pavement_score = 0.0;
                for p in &inputs.pavements {
                    if p.corridor == act.corridor {
                        let mist_factor = (act.pavement_misting_l / 500.0).min(1.0);
                        let cool_factor = if p.has_cool_pavement { 0.8 } else { 1.0 };
                        pavement_score += p.human_exposure_idx * cool_factor * (1.0 - mist_factor);
                    }
                }

                heat_score += tree_score + pavement_score;

                water_use += act.awp_reuse_m3
                    + act.irrigation_mm * 0.001 * 100.0
                    + act.pavement_misting_l * 0.001
                    + act.hydration_alloc_l * 0.001;

                for h in &inputs.hydration {
                    if h.corridor == act.corridor {
                        let fraction = (act.hydration_alloc_l / h.daily_capacity_l).min(1.0);
                        vulnerable_cooling += h.vulnerable_weight * fraction;
                    }
                }
            }
        }

        candidate.f_heat = heat_score;
        candidate.f_water = water_use;
        candidate.f_vulnerable = -vulnerable_cooling;
    }
}

mod nsga2 {
    use super::constraints::ConstraintProfile;
    use super::domain::*;
    use super::objectives::evaluate_candidate;
    use rand::prelude::*;
    use std::cmp::Ordering;

    #[derive(Clone, Debug)]
    pub struct Nsga2Config {
        pub population_size: usize,
        pub generations: usize,
        pub crossover_prob: f64,
        pub mutation_prob: f64,
    }

    pub fn run_nsga2(inputs: &HeatInputs, profile: &ConstraintProfile, cfg: &Nsga2Config) -> Vec<Candidate> {
        let mut rng = StdRng::from_entropy();
        let mut pop = init_population(inputs, profile, cfg.population_size, &mut rng);

        for _ in 0..cfg.generations {
            for ind in &mut pop {
                evaluate_candidate(ind, inputs);
            }
            let offspring = make_offspring(inputs, profile, &pop, cfg, &mut rng);
            let mut combined = Vec::with_capacity(pop.len() + offspring.len());
            combined.extend(pop);
            combined.extend(offspring);
            for ind in &mut combined {
                evaluate_candidate(ind, inputs);
            }
            pop = select_next_generation(combined, cfg.population_size);
        }

        for ind in &mut pop {
            evaluate_candidate(ind, inputs);
        }
        extract_pareto_front(&pop)
    }

    fn init_population(
        inputs: &HeatInputs,
        profile: &ConstraintProfile,
        size: usize,
        rng: &mut StdRng,
    ) -> Vec<Candidate> {
        let mut pop = Vec::with_capacity(size);
        while pop.len() < size {
            let mut cand = random_candidate(inputs, rng);
            if profile.is_feasible(&cand, inputs) {
                pop.push(cand);
            }
        }
        pop
    }

    fn random_candidate(inputs: &HeatInputs, rng: &mut StdRng) -> Candidate {
        use super::domain::*;
        let mut plans = Vec::new();
        for ts in &inputs.timeslots {
            let mut actions = Vec::new();
            for canal in &inputs.canals {
                let corridor = canal.corridor.clone();
                let canal_flow_lps = rng.gen_range(0.0..canal.max_flow_lps * 0.5);
                let awp_reuse_m3 = rng.gen_range(0.0..50.0);
                let irrigation_mm = rng.gen_range(0.0..10.0);
                let pavement_misting_l = rng.gen_range(0.0..1000.0);
                let hydration_alloc_l = rng.gen_range(0.0..500.0);
                actions.push(CorridorAction {
                    corridor,
                    canal_flow_lps,
                    awp_reuse_m3,
                    irrigation_mm,
                    pavement_misting_l,
                    hydration_alloc_l,
                });
            }
            plans.push(DailyPlan {
                timeslot: ts.clone(),
                actions,
            });
        }
        Candidate {
            plans,
            f_heat: 0.0,
            f_water: 0.0,
            f_vulnerable: 0.0,
        }
    }

    fn make_offspring(
        inputs: &HeatInputs,
        profile: &ConstraintProfile,
        pop: &Vec<Candidate>,
        cfg: &Nsga2Config,
        rng: &mut StdRng,
    ) -> Vec<Candidate> {
        let mut offspring = Vec::with_capacity(pop.len());
        while offspring.len() < pop.len() {
            let p1 = tournament(pop, rng);
            let p2 = tournament(pop, rng);
            let mut c1 = p1.clone();
            let mut c2 = p2.clone();
            if rng.gen::<f64>() < cfg.crossover_prob {
                crossover(&mut c1, &mut c2, rng);
            }
            if rng.gen::<f64>() < cfg.mutation_prob {
                mutate(inputs, &mut c1, rng);
            }
            if rng.gen::<f64>() < cfg.mutation_prob {
                mutate(inputs, &mut c2, rng);
            }
            if profile.is_feasible(&c1, inputs) {
                offspring.push(c1);
            }
            if offspring.len() < pop.len() && profile.is_feasible(&c2, inputs) {
                offspring.push(c2);
            }
        }
        offspring
    }

    fn tournament(pop: &Vec<Candidate>, rng: &mut StdRng) -> &Candidate {
        let i1 = rng.gen_range(0..pop.len());
        let i2 = rng.gen_range(0..pop.len());
        if dominates(&pop[i1], &pop[i2]) {
            &pop[i1]
        } else {
            &pop[i2]
        }
    }

    fn dominates(a: &Candidate, b: &Candidate) -> bool {
        let mut better_or_equal = true;
        let mut strictly_better = false;

        let objs_a = [a.f_heat, a.f_water, a.f_vulnerable];
        let objs_b = [b.f_heat, b.f_water, b.f_vulnerable];

        for i in 0..objs_a.len() {
            if objs_a[i] > objs_b[i] + 1e-9 {
                better_or_equal = false;
            }
            if objs_a[i] < objs_b[i] - 1e-9 {
                strictly_better = true;
            }
        }
        better_or_equal && strictly_better
    }

    fn crossover(c1: &mut Candidate, c2: &mut Candidate, rng: &mut StdRng) {
        let n_ts = c1.plans.len().min(c2.plans.len());
        if n_ts == 0 {
            return;
        }
        let cut = rng.gen_range(0..n_ts);
        for i in cut..n_ts {
            std::mem::swap(&mut c1.plans[i], &mut c2.plans[i]);
        }
    }

    fn mutate(inputs: &HeatInputs, c: &mut Candidate, rng: &mut StdRng) {
        if c.plans.is_empty() {
            return;
        }
        let ts_idx = rng.gen_range(0..c.plans.len());
        if c.plans[ts_idx].actions.is_empty() {
            return;
        }
        let act_idx = rng.gen_range(0..c.plans[ts_idx].actions.len());
        let act = &mut c.plans[ts_idx].actions[act_idx];
        let canal = inputs
            .canals
            .iter()
            .find(|c| c.corridor == act.corridor)
            .unwrap_or(&inputs.canals[0]);
        act.canal_flow_lps = (act.canal_flow_lps + rng.gen_range(-10.0..10.0))
            .clamp(0.0, canal.max_flow_lps * 0.7);
        act.awp_reuse_m3 = (act.awp_reuse_m3 + rng.gen_range(-5.0..5.0)).clamp(0.0, 80.0);
        act.irrigation_mm = (act.irrigation_mm + rng.gen_range(-3.0..3.0)).clamp(0.0, 20.0);
        act.pavement_misting_l = (act.pavement_misting_l + rng.gen_range(-200.0..200.0))
            .clamp(0.0, 2500.0);
        act.hydration_alloc_l = (act.hydration_alloc_l + rng.gen_range(-100.0..100.0))
            .clamp(0.0, 800.0);
    }

    fn select_next_generation(mut combined: Vec<Candidate>, target_size: usize) -> Vec<Candidate> {
        let ranks = fast_nondominated_sort(&combined);
        let mut next_pop = Vec::with_capacity(target_size);
        let mut i = 0;
        while i < ranks.len() && next_pop.len() + ranks[i].len() <= target_size {
            let mut front = ranks[i]
                .iter()
                .map(|&idx| combined[idx].clone())
                .collect::<Vec<_>>();
            crowding_sort(&mut front);
            next_pop.extend(front);
            i += 1;
        }
        if next_pop.len() < target_size && i < ranks.len() {
            let mut last_front = ranks[i]
                .iter()
                .map(|&idx| combined[idx].clone())
                .collect::<Vec<_>>();
            crowding_sort(&mut last_front);
            let remaining = target_size - next_pop.len();
            next_pop.extend(last_front.into_iter().take(remaining));
        }
        next_pop
    }

    fn fast_nondominated_sort(pop: &Vec<Candidate>) -> Vec<Vec<usize>> {
        let n = pop.len();
        let mut s: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut n_dom: Vec<usize> = vec![0; n];
        let mut fronts: Vec<Vec<usize>> = Vec::new();
        let mut first_front: Vec<usize> = Vec::new();

        for p in 0..n {
            for q in 0..n {
                if p == q {
                    continue;
                }
                if dominates(&pop[p], &pop[q]) {
                    s[p].push(q);
                } else if dominates(&pop[q], &pop[p]) {
                    n_dom[p] += 1;
                }
            }
            if n_dom[p] == 0 {
                first_front.push(p);
            }
        }
        fronts.push(first_front);

        let mut i = 0;
        while !fronts[i].is_empty() {
            let mut next_front: Vec<usize> = Vec::new();
            for &p in &fronts[i] {
                for &q in &s[p] {
                    if n_dom[q] > 0 {
                        n_dom[q] -= 1;
                        if n_dom[q] == 0 {
                            next_front.push(q);
                        }
                    }
                }
            }
            if !next_front.is_empty() {
                fronts.push(next_front);
            }
            i += 1;
            if i >= fronts.len() {
                break;
            }
        }
        fronts
    }

    fn crowding_sort(front: &mut Vec<Candidate>) {
        if front.len() <= 2 {
            return;
        }

        let mut idx: Vec<usize> = (0..front.len()).collect();
        for m in 0..3 {
            idx.sort_by(|&a, &b| {
                let va = get_obj(&front[a], m);
                let vb = get_obj(&front[b], m);
                va.partial_cmp(&vb).unwrap_or(Ordering::Equal)
            });
            let min = get_obj(&front[idx[0]], m);
            let max = get_obj(&front[idx[idx.len() - 1]], m);
            if (max - min).abs() < 1e-12 {
                continue;
            }
        }
    }

    fn get_obj(c: &Candidate, idx: usize) -> f64 {
        match idx {
            0 => c.f_heat,
            1 => c.f_water,
            _ => c.f_vulnerable,
        }
    }

    fn extract_pareto_front(pop: &Vec<Candidate>) -> Vec<Candidate> {
        let ranks = fast_nondominated_sort(pop);
        if ranks.is_empty() {
            return Vec::new();
        }
        ranks[0].iter().map(|&i| pop[i].clone()).collect()
    }
}

pub mod engine {
    use super::constraints::ConstraintProfile;
    use super::domain::*;
    use super::nsga2::{run_nsga2, Nsga2Config};

    pub struct HeatWaterTreeEngine {
        profile: ConstraintProfile,
        cfg: Nsga2Config,
    }

    impl HeatWaterTreeEngine {
        pub fn new(inputs: &HeatInputs) -> Self {
            let profile = ConstraintProfile::default_from_inputs(inputs);
            let cfg = Nsga2Config {
                population_size: 60,
                generations: 40,
                crossover_prob: 0.9,
                mutation_prob: 0.4,
            };
            HeatWaterTreeEngine { profile, cfg }
        }

        pub fn optimize(&self, inputs: &HeatInputs) -> Vec<Candidate> {
            run_nsga2(inputs, &self.profile, &self.cfg)
        }
    }
}
