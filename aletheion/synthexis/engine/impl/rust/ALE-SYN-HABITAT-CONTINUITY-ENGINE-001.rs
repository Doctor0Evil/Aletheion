//! This engine computes candidate habitat corridors for SpeciesAgents under BioticTreaties,
//! operating on a built-form graph for Phoenix neighborhoods and corridors.
//!
//! It is designed to plug into:
//!   - aletheion/synthexis/model/SpeciesAgent.*
//!   - aletheion/synthexis/model/BioticTreaty.*
//! and to be called by higher-level workflows under Synthexis CrossSpecies Habitat. [file:5]

use std::collections::{HashMap, HashSet};

/// Identifier types – kept as String for compatibility with ALN and Googolswarm bindings. [file:6]
pub type SpeciesId = String;
pub type TreatyId = String;
pub type NodeId = String;
pub type EdgeId = String;
pub type CorridorId = String;

/// Normalized score 0–1. [file:6]
pub type Score = f64;

/// Basic check to keep scores within [0,1].
fn clamp01(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Minimal mirror of Synthexis SpeciesAgent core fields needed for continuity. [file:6]
#[derive(Debug, Clone)]
pub struct SpeciesAgent {
    pub species_id: SpeciesId,
    /// 0–1 conservation priority weight.
    pub conservation_priority: Score,
    /// 0–1 ecosystem service weight (pollination, pest control, etc.).
    pub ecosystem_service_weight: Score,
    /// Whether the species is light-averse in urban corridors.
    pub light_averse: bool,
    /// Maximum acceptable ALAN (artificial light at night) in lux along movement corridors.
    pub max_corridor_lux: f64,
    /// Maximum acceptable nighttime noise in dBA.
    pub max_night_dba: f64,
    /// Whether pesticide exposure along corridors must be minimized.
    pub pesticide_sensitive: bool,
}

/// Minimal mirror of BioticTreaty envelope for one or more SpeciesAgents. [file:6]
#[derive(Debug, Clone)]
pub struct BioticTreaty {
    pub treaty_id: TreatyId,
    /// Species covered by this treaty.
    pub species_ids: Vec<SpeciesId>,
    /// Treaty's minimum connectivity requirement (0–1).
    pub min_connectivity: Score,
    /// Treaty's maximum acceptable ALAN violation fraction along a corridor (0–1).
    pub max_alan_violation_fraction: Score,
    /// Treaty's maximum acceptable pesticide violation fraction along a corridor (0–1).
    pub max_pesticide_violation_fraction: Score,
    /// Whether this treaty must be enforced as a hard constraint (no actuation if violated).
    pub hard_enforcement: bool,
}

/// Built-form node – parcel, park, canal segment, etc. [file:5]
#[derive(Debug, Clone)]
pub struct BuiltFormNode {
    pub node_id: NodeId,
    /// 0–1 native vegetation fraction.
    pub native_veg_fraction: Score,
    /// 0–1 canopy cover fraction.
    pub canopy_fraction: Score,
    /// Whether node touches riparian or canal habitat.
    pub riparian: bool,
}

/// Built-form edge – potential movement link between nodes. [file:5]
#[derive(Debug, Clone)]
pub struct BuiltFormEdge {
    pub edge_id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    /// Nighttime ALAN (lux) at corridor height.
    pub night_lux: f64,
    /// Nighttime noise (dBA).
    pub night_dba: f64,
    /// True if pesticides are applied frequently along this edge.
    pub pesticide_intense: bool,
    /// 0–1 surface hardness (1 = fully impervious, 0 = fully soft/vegetated).
    pub surface_hardness: Score,
    /// 0–1 tree-lined score (1 = strongly tree-lined).
    pub tree_corridor_score: Score,
    /// Optional precomputed distance/length in meters.
    pub length_m: Option<f64>,
}

/// External corridor constraints injected from ALN. [file:5][file:6]
#[derive(Debug, Clone)]
pub struct CorridorEnvelope {
    pub corridor_id: CorridorId,
    /// Nodes that must be included in any valid corridor (e.g., roosts, nesting patches).
    pub required_nodes: Vec<NodeId>,
    /// Nodes that must be avoided entirely.
    pub forbidden_nodes: Vec<NodeId>,
    /// Edges that must be avoided (e.g., heavy pesticide, critical noise hotspots).
    pub forbidden_edges: Vec<EdgeId>,
}

/// One candidate path for one species under one treaty. [file:5]
#[derive(Debug, Clone)]
pub struct CorridorProposal {
    pub corridor_id: CorridorId,
    pub species_id: SpeciesId,
    pub treaty_id: TreatyId,
    pub path_nodes: Vec<NodeId>,
    pub path_edges: Vec<EdgeId>,
    /// 0–1 connectivity score.
    pub connectivity_score: Score,
    /// 0–1 ALAN safety score (1 = fully within envelopes).
    pub alan_safety_score: Score,
    /// 0–1 pesticide safety score.
    pub pesticide_safety_score: Score,
    /// 0–1 composite score.
    pub composite_score: Score,
    /// True if proposal satisfies hard treaty constraints.
    pub treaty_compliant: bool,
}

/// Bundle of proposals plus diagnostics for debugging and governance. [file:5][file:6]
#[derive(Debug, Clone)]
pub struct HabitatContinuityResult {
    pub proposals: Vec<CorridorProposal>,
    /// For each species, highest composite score achieved.
    pub best_species_scores: HashMap<SpeciesId, Score>,
    /// For each treaty, fraction of species that achieved >= treaty.min_connectivity.
    pub treaty_satisfaction: HashMap<TreatyId, Score>,
}

/// Convenience container for graph. [file:5]
#[derive(Debug, Clone)]
pub struct BuiltFormGraph {
    pub nodes: HashMap<NodeId, BuiltFormNode>,
    pub edges: HashMap<EdgeId, BuiltFormEdge>,
    /// Adjacency keyed by node -> outgoing edge ids.
    pub adjacency: HashMap<NodeId, Vec<EdgeId>>,
}

impl BuiltFormGraph {
    pub fn new(
        nodes: Vec<BuiltFormNode>,
        edges: Vec<BuiltFormEdge>,
    ) -> BuiltFormGraph {
        let mut node_map = HashMap::new();
        for n in nodes {
            node_map.insert(n.node_id.clone(), n);
        }

        let mut edge_map = HashMap::new();
        let mut adjacency: HashMap<NodeId, Vec<EdgeId>> = HashMap::new();
        for e in edges {
            adjacency
                .entry(e.from.clone())
                .or_default()
                .push(e.edge_id.clone());
            edge_map.insert(e.edge_id.clone(), e);
        }

        BuiltFormGraph {
            nodes: node_map,
            edges: edge_map,
            adjacency,
        }
    }
}

/// Engine configuration knobs, to be driven from ALN. [file:6]
#[derive(Debug, Clone)]
pub struct HabitatContinuityConfig {
    /// Maximum length (in edges) of candidate paths.
    pub max_steps_per_path: usize,
    /// Maximum number of candidate paths to score per species.
    pub max_paths_per_species: usize,
    /// Weight for connectivity vs. ALAN vs. pesticide in composite score (must sum to 1).
    pub w_connectivity: f64,
    pub w_alan: f64,
    pub w_pesticide: f64,
}

impl Default for HabitatContinuityConfig {
    fn default() -> Self {
        HabitatContinuityConfig {
            max_steps_per_path: 24,
            max_paths_per_species: 128,
            w_connectivity: 0.5,
            w_alan: 0.25,
            w_pesticide: 0.25,
        }
    }
}

/// Public engine interface – primary entry point from ALN and workflows. [file:5][file:6]
pub fn compute_habitat_continuity(
    graph: &BuiltFormGraph,
    species: &[SpeciesAgent],
    treaties: &[BioticTreaty],
    envelopes: &[CorridorEnvelope],
    cfg: &HabitatContinuityConfig,
) -> HabitatContinuityResult {
    // Index species and treaties for quick lookup.
    let mut species_index: HashMap<SpeciesId, &SpeciesAgent> = HashMap::new();
    for s in species {
        species_index.insert(s.species_id.clone(), s);
    }

    let mut treaties_by_species: HashMap<SpeciesId, Vec<&BioticTreaty>> = HashMap::new();
    for t in treaties {
        for sid in &t.species_ids {
            treaties_by_species.entry(sid.clone()).or_default().push(t);
        }
    }

    // Map envelopes by corridor_id for potential future multi-corridor use.
    let _envelopes_index: HashMap<CorridorId, &CorridorEnvelope> =
        envelopes.iter().map(|e| (e.corridor_id.clone(), e)).collect();

    let mut proposals: Vec<CorridorProposal> = Vec::new();
    let mut best_species_scores: HashMap<SpeciesId, Score> = HashMap::new();
    let mut treaty_satisfaction: HashMap<TreatyId, Score> = HashMap::new();

    // For each species, generate candidate paths and score against its treaties. [file:5]
    for s in species {
        let sid = &s.species_id;

        let treaties_for_species = match treaties_by_species.get(sid) {
            Some(v) if !v.is_empty() => v.clone(),
            _ => continue,
        };

        // Start nodes: all riparian or high-vegetation nodes by default.
        let start_nodes: Vec<&BuiltFormNode> = graph
            .nodes
            .values()
            .filter(|n| n.riparian || n.native_veg_fraction > 0.5)
            .collect();

        let mut species_proposals: Vec<CorridorProposal> = Vec::new();

        for start in start_nodes {
            let mut visited: HashSet<NodeId> = HashSet::new();
            visited.insert(start.node_id.clone());

            dfs_paths_for_species(
                graph,
                s,
                &treaties_for_species,
                start,
                &mut visited,
                &mut Vec::new(),
                cfg,
                &mut species_proposals,
            );

            if species_proposals.len() >= cfg.max_paths_per_species {
                break;
            }
        }

        // Aggregate best scores and per-treaty satisfaction. [file:6]
        let mut best_score_for_species = 0.0_f64;
        let mut per_treaty_hits: HashMap<TreatyId, usize> = HashMap::new();
        let mut per_treaty_total: HashMap<TreatyId, usize> = HashMap::new();

        for p in &species_proposals {
            if p.composite_score > best_score_for_species {
                best_score_for_species = p.composite_score;
            }
            // Treat "connectivity_score >= min_connectivity" as a hit.
            if let Some(t) = treaties_for_species
                .iter()
                .find(|t| t.treaty_id == p.treaty_id)
            {
                let total = per_treaty_total.entry(t.treaty_id.clone()).or_insert(0);
                *total += 1;
                if p.connectivity_score + 1e-9 >= t.min_connectivity {
                    let hits = per_treaty_hits.entry(t.treaty_id.clone()).or_insert(0);
                    *hits += 1;
                }
            }
        }

        if best_score_for_species > 0.0 {
            best_species_scores.insert(sid.clone(), best_score_for_species);
        }

        for (tid, total) in per_treaty_total {
            let hits = per_treaty_hits.get(&tid).copied().unwrap_or(0);
            let fraction = if total == 0 {
                0.0
            } else {
                hits as f64 / total as f64
            };
            let existing = treaty_satisfaction.entry(tid.clone()).or_insert(0.0);
            if fraction > *existing {
                *existing = fraction;
            }
        }

        proposals.extend(species_proposals);
    }

    HabitatContinuityResult {
        proposals,
        best_species_scores,
        treaty_satisfaction,
    }
}

/// Depth-bounded DFS to sample candidate paths for one species. [file:5]
fn dfs_paths_for_species(
    graph: &BuiltFormGraph,
    species: &SpeciesAgent,
    treaties_for_species: &[&BioticTreaty],
    current_node: &BuiltFormNode,
    visited: &mut HashSet<NodeId>,
    path_edges: &mut Vec<EdgeId>,
    cfg: &HabitatContinuityConfig,
    out: &mut Vec<CorridorProposal>,
) {
    if path_edges.len() >= cfg.max_steps_per_path {
        return;
    }

    let outgoing_edges = match graph.adjacency.get(&current_node.node_id) {
        Some(v) => v,
        None => return,
    };

    for edge_id in outgoing_edges {
        if out.len() >= cfg.max_paths_per_species {
            return;
        }

        let edge = match graph.edges.get(edge_id) {
            Some(e) => e,
            None => continue,
        };

        // Basic species-level filter: reject edges that are clearly outside envelopes. [file:6]
        if species.light_averse && edge.night_lux > species.max_corridor_lux {
            continue;
        }
        if edge.night_dba > species.max_night_dba {
            continue;
        }
        if species.pesticide_sensitive && edge.pesticide_intense {
            // We still allow these as "bad edges" for scoring, so do not skip; just note.
        }

        let next_node_id = edge.to.clone();
        if visited.contains(&next_node_id) {
            continue;
        }

        let next_node = match graph.nodes.get(&next_node_id) {
            Some(n) => n,
            None => continue,
        };

        visited.insert(next_node_id.clone());
        path_edges.push(edge.edge_id.clone());

        // For each treaty, build a proposal snapshot. [file:5]
        for treaty in treaties_for_species {
            let proposal = score_path_for_treaty(
                graph,
                species,
                treaty,
                &visited,
                path_edges,
                cfg,
            );
            out.push(proposal);
            if out.len() >= cfg.max_paths_per_species {
                break;
            }
        }

        // Continue DFS deeper.
        dfs_paths_for_species(
            graph,
            species,
            treaties_for_species,
            next_node,
            visited,
            path_edges,
            cfg,
            out,
        );

        // Backtrack.
        path_edges.pop();
        visited.remove(&next_node_id);
    }
}

/// Score a candidate path for a given species and treaty. [file:6]
fn score_path_for_treaty(
    graph: &BuiltFormGraph,
    species: &SpeciesAgent,
    treaty: &BioticTreaty,
    visited_nodes: &HashSet<NodeId>,
    path_edges: &[EdgeId],
    cfg: &HabitatContinuityConfig,
) -> CorridorProposal {
    // Build ordered node list from visited set is lossy; we instead approximate:
    let path_nodes: Vec<NodeId> = visited_nodes.iter().cloned().collect();

    // Compute connectivity: blend path length, riparian coverage, and tree corridor. [file:5]
    let (mut length_m_sum, mut riparian_count, mut tree_score_sum) =
        (0.0_f64, 0_u32, 0.0_f64);

    for nid in &path_nodes {
        if let Some(n) = graph.nodes.get(nid) {
            if n.riparian {
                riparian_count += 1;
            }
        }
    }

    for eid in path_edges {
        if let Some(e) = graph.edges.get(eid) {
            length_m_sum += e.length_m.unwrap_or(1.0);
            tree_score_sum += e.tree_corridor_score;
        }
    }

    let steps = path_edges.len() as f64;
    let avg_tree_score = if steps > 0.0 {
        tree_score_sum / steps
    } else {
        0.0
    };
    let riparian_fraction = if !path_nodes.is_empty() {
        riparian_count as f64 / path_nodes.len() as f64
    } else {
        0.0
    };

    // Normalize length with simple saturating transform (longer, but not unbounded). [file:6]
    let length_norm = 1.0 - (-length_m_sum / 1000.0).exp(); // ~1 km -> ~0.63

    let connectivity_raw = 0.5 * length_norm + 0.3 * avg_tree_score + 0.2 * riparian_fraction;
    let connectivity_score = clamp01(connectivity_raw);

    // ALAN safety – fraction of edges within species & treaty tolerances. [file:6]
    let mut alan_ok = 0_u32;
    let mut pesticide_ok = 0_u32;
    let total_edges = path_edges.len() as f64;

    for eid in path_edges {
        if let Some(e) = graph.edges.get(eid) {
            let light_ok =
                (!species.light_averse || e.night_lux <= species.max_corridor_lux)
                    && e.night_lux <= species.max_corridor_lux * 1.2;

            if light_ok {
                alan_ok += 1;
            }

            let pesticide_good = !e.pesticide_intense || !species.pesticide_sensitive;
            if pesticide_good {
                pesticide_ok += 1;
            }
        }
    }

    let alan_safety_score = if total_edges > 0.0 {
        alan_ok as f64 / total_edges
    } else {
        0.0
    };

    let pesticide_safety_score = if total_edges > 0.0 {
        pesticide_ok as f64 / total_edges
    } else {
        0.0
    };

    // Treaty compliance check via allowed violation fractions. [file:6]
    let alan_violation_fraction = 1.0 - alan_safety_score;
    let pesticide_violation_fraction = 1.0 - pesticide_safety_score;

    let alan_ok_treaty =
        alan_violation_fraction + 1e-9 <= treaty.max_alan_violation_fraction;
    let pesticide_ok_treaty =
        pesticide_violation_fraction + 1e-9 <= treaty.max_pesticide_violation_fraction;

    let treaty_compliant =
        (!treaty.hard_enforcement) || (alan_ok_treaty && pesticide_ok_treaty);

    // Composite score – mix connectivity and safety per config; then weight by species priority. [file:6]
    let composite_raw = cfg.w_connectivity * connectivity_score
        + cfg.w_alan * alan_safety_score
        + cfg.w_pesticide * pesticide_safety_score;

    let composite_weighted = composite_raw
        * clamp01(
            0.5 * species.conservation_priority + 0.5 * species.ecosystem_service_weight,
        );

    let composite_score = clamp01(composite_weighted);

    // CorridorId can be recomputed upstream or left generic here.
    let corridor_id = format!(
        "SYN-HAB-CORRIDOR-{}-{}-{}",
        species.species_id,
        treaty.treaty_id,
        path_nodes.len()
    );

    CorridorProposal {
        corridor_id,
        species_id: species.species_id.clone(),
        treaty_id: treaty.treaty_id.clone(),
        path_nodes,
        path_edges: path_edges.to_vec(),
        connectivity_score,
        alan_safety_score,
        pesticide_safety_score,
        composite_score,
        treaty_compliant,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_graph() -> BuiltFormGraph {
        let n1 = BuiltFormNode {
            node_id: "N1".into(),
            native_veg_fraction: 0.8,
            canopy_fraction: 0.7,
            riparian: true,
        };
        let n2 = BuiltFormNode {
            node_id: "N2".into(),
            native_veg_fraction: 0.6,
            canopy_fraction: 0.4,
            riparian: false,
        };
        let n3 = BuiltFormNode {
            node_id: "N3".into(),
            native_veg_fraction: 0.4,
            canopy_fraction: 0.3,
            riparian: false,
        };

        let e12 = BuiltFormEdge {
            edge_id: "E12".into(),
            from: "N1".into(),
            to: "N2".into(),
            night_lux: 0.5,
            night_dba: 40.0,
            pesticide_intense: false,
            surface_hardness: 0.3,
            tree_corridor_score: 0.9,
            length_m: Some(120.0),
        };

        let e23 = BuiltFormEdge {
            edge_id: "E23".into(),
            from: "N2".into(),
            to: "N3".into(),
            night_lux: 1.0,
            night_dba: 45.0,
            pesticide_intense: true,
            surface_hardness: 0.7,
            tree_corridor_score: 0.4,
            length_m: Some(150.0),
        };

        BuiltFormGraph::new(vec![n1, n2, n3], vec![e12, e23])
    }

    #[test]
    fn smoke_test_continuity_engine_runs() {
        let graph = make_test_graph();

        let species = SpeciesAgent {
            species_id: "bat_myotis".into(),
            conservation_priority: 0.9,
            ecosystem_service_weight: 0.8,
            light_averse: true,
            max_corridor_lux: 3.0,
            max_night_dba: 55.0,
            pesticide_sensitive: true,
        };

        let treaty = BioticTreaty {
            treaty_id: "TREATY-RIPARIAN-BATS-001".into(),
            species_ids: vec!["bat_myotis".into()],
            min_connectivity: 0.4,
            max_alan_violation_fraction: 0.25,
            max_pesticide_violation_fraction: 0.5,
            hard_enforcement: true,
        };

        let env = CorridorEnvelope {
            corridor_id: "ENV-1".into(),
            required_nodes: vec!["N1".into()],
            forbidden_nodes: vec!["N99".into()],
            forbidden_edges: vec!["EXX".into()],
        };

        let cfg = HabitatContinuityConfig::default();

        let result = compute_habitat_continuity(
            &graph,
            &[species],
            &[treaty],
            &[env],
            &cfg,
        );

        assert!(
            !result.proposals.is_empty(),
            "Expected at least one corridor proposal"
        );

        for p in &result.proposals {
            assert!(
                p.composite_score >= 0.0 && p.composite_score <= 1.0,
                "Composite score must be in [0,1]"
            );
        }
    }
}
