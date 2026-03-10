//! Purpose:
//!   Compute operating envelopes for lighting, noise, and pesticides that satisfy
//!   BioticTreaties while maintaining human comfort and safety. [file:5]

use std::collections::HashMap;

pub type SpeciesId = String;
pub type TreatyId = String;
pub type SegmentId = String;
pub type ZoneId = String;

/// Normalized score 0–1. [file:6]
pub type Score = f64;

/// Human comfort targets per corridor/zone. [file:6]
#[derive(Debug, Clone)]
pub struct HumanComfortTargets {
    pub min_illuminance_lux: f64,
    pub max_night_dba: f64,
    /// Acceptable pest pressure threshold (0–1).
    pub max_pest_pressure: Score,
}

/// Minimal BioticTreaty slice needed for envelope planning. [file:5]
#[derive(Debug, Clone)]
pub struct BioticTreatyEnv {
    pub treaty_id: TreatyId,
    pub species_ids: Vec<SpeciesId>,
    /// Max lux in habitat corridors.
    pub max_corridor_lux: f64,
    /// Max dBA at night.
    pub max_night_dba: f64,
    /// True if night-time pesticides are required to be minimized.
    pub pesticide_restrictive: bool,
}

/// Species activity probability per time band, per segment. [file:5]
#[derive(Debug, Clone)]
pub struct SpeciesActivity {
    pub species_id: SpeciesId,
    /// Time band identifier, e.g., "18-22", "22-06".
    pub time_band: String,
    pub segment_id: SegmentId,
    /// Probability of presence/activity (0–1).
    pub activity_prob: Score,
}

/// Base infrastructure state for one segment. [file:6]
#[derive(Debug, Clone)]
pub struct SegmentBaseState {
    pub segment_id: SegmentId,
    pub zone_id: ZoneId,
    // Lighting.
    pub current_lux: f64,
    pub spectrum: String,
    pub dimmable: bool,
    pub shieldable: bool,
    // Noise.
    pub current_night_dba: f64,
    pub has_impulsive_sources: bool,
    // Pesticides.
    pub pesticide_regime: String,
    pub near_pollinator_corridor: bool,
}

/// Operating envelope for one segment and time band. [file:5]
#[derive(Debug, Clone)]
pub struct OperatingEnvelope {
    pub segment_id: SegmentId,
    pub time_band: String,
    // Lighting.
    pub max_lux: f64,
    pub allowed_spectra: Vec<String>,
    pub full_off_windows: Vec<(String, String)>, // [start,end]
    pub shielding_required: bool,
    // Noise.
    pub max_dba: f64,
    pub quiet_hours: Vec<(String, String)>,
    // Pesticides.
    pub allowed_windows: Vec<(String, String)>,
    pub no_spray_zones: Vec<String>,
}

/// Violation report for planned/actual operations. [file:5]
#[derive(Debug, Clone)]
pub struct ViolationReport {
    pub segment_id: SegmentId,
    pub time_band: String,
    pub treaty_id: TreatyId,
    pub violation_type: String,
    pub magnitude: f64,
    pub suggested_change: String,
}

/// Planner configuration knobs. [file:6]
#[derive(Debug, Clone)]
pub struct LightNoisePesticidePlannerConfig {
    /// Lux buffer above BioticTreaty max allowed, used when human safety demands more light.
    pub safety_lux_floor: f64,
    /// Minimal quiet-hours length in hours for high-activity bands.
    pub min_quiet_hours_high_activity: f64,
    /// Default pesticide window duration in hours, when allowed.
    pub pesticide_window_hours: f64,
}

impl Default for LightNoisePesticidePlannerConfig {
    fn default() -> Self {
        Self {
            safety_lux_floor: 3.0,
            min_quiet_hours_high_activity: 6.0,
            pesticide_window_hours: 2.0,
        }
    }
}

/// Planner inputs assembled from state modeling layer. [file:5][file:6]
#[derive(Debug, Clone)]
pub struct PlannerInput {
    pub date_iso: String,
    pub segments: Vec<SegmentBaseState>,
    pub treaties: Vec<BioticTreatyEnv>,
    pub human_targets_by_zone: HashMap<ZoneId, HumanComfortTargets>,
    pub species_activity: Vec<SpeciesActivity>,
    pub config: LightNoisePesticidePlannerConfig,
}

/// Planner outputs. [file:5]
#[derive(Debug, Clone)]
pub struct PlannerOutput {
    pub envelopes: Vec<OperatingEnvelope>,
    pub violations: Vec<ViolationReport>,
    pub meta: HashMap<String, String>,
}

/// Main entry point – compute envelopes and violations. [file:5]
pub fn compute_envelopes(input: PlannerInput) -> PlannerOutput {
    // Index activity by (segment, time_band).
    let mut activity_index: HashMap<(SegmentId, String), Vec<SpeciesActivity>> =
        HashMap::new();
    for a in &input.species_activity {
        activity_index
            .entry((a.segment_id.clone(), a.time_band.clone()))
            .or_default()
            .push(a.clone());
    }

    // For simplicity v1: use a fixed set of time bands inferred from activity data. [file:6]
    let mut time_bands: Vec<String> = activity_index
        .keys()
        .map(|(_, tb)| tb.clone())
        .collect();
    time_bands.sort();
    time_bands.dedup();

    // Index treaties (global in v1; could be spatially filtered later).
    let treaties = input.treaties.clone();

    let mut envelopes: Vec<OperatingEnvelope> = Vec::new();
    let mut violations: Vec<ViolationReport> = Vec::new();

    for seg in &input.segments {
        let comfort = match input.human_targets_by_zone.get(&seg.zone_id) {
            Some(c) => c,
            None => continue,
        };

        for tb in &time_bands {
            let key = (seg.segment_id.clone(), tb.clone());
            let activities = activity_index.get(&key).cloned().unwrap_or_default();

            let (env, local_violations) =
                plan_for_segment_time_band(seg, tb, comfort, &treaties, &activities, &input.config);

            envelopes.push(env);
            violations.extend(local_violations);
        }
    }

    let mut meta = HashMap::new();
    meta.insert("date".into(), input.date_iso);
    meta.insert("planner_version".into(), "ALE-SYN-LNP-001".into());

    PlannerOutput {
        envelopes,
        violations,
        meta,
    }
}

/// Plan envelopes for one segment and time band. [file:5]
fn plan_for_segment_time_band(
    seg: &SegmentBaseState,
    time_band: &str,
    comfort: &HumanComfortTargets,
    treaties: &[BioticTreatyEnv],
    activities: &[SpeciesActivity],
    cfg: &LightNoisePesticidePlannerConfig,
) -> (OperatingEnvelope, Vec<ViolationReport>) {
    let mut violations: Vec<ViolationReport> = Vec::new();

    // Aggregate max treaty constraints for any active species. [file:5]
    let max_activity = activities
        .iter()
        .map(|a| a.activity_prob)
        .fold(0.0_f64, f64::max);

    let is_high_activity = max_activity > 0.5;

    let mut treaty_max_lux = f64::INFINITY;
    let mut treaty_max_dba = f64::INFINITY;
    let mut pesticide_restrictive = false;

    for t in treaties {
        // Simple v1: apply all treaties; a future version would spatially filter.
        if t.max_corridor_lux < treaty_max_lux {
            treaty_max_lux = t.max_corridor_lux;
        }
        if t.max_night_dba < treaty_max_dba {
            treaty_max_dba = t.max_night_dba;
        }
        if t.pesticide_restrictive {
            pesticide_restrictive = true;
        }
    }

    // Lighting: obey BioticTreaty unless it jeopardizes human minimum safety illuminance. [file:5][file:6]
    let mut max_lux = treaty_max_lux.min(comfort.min_illuminance_lux.max(cfg.safety_lux_floor));

    if max_lux.is_infinite() {
        max_lux = comfort.min_illuminance_lux.max(cfg.safety_lux_floor);
    }

    let shielding_required = seg.shieldable && is_high_activity;

    let allowed_spectra = if is_high_activity {
        vec!["amber".into(), "warm_white".into()]
    } else {
        vec![seg.spectrum.clone()]
    };

    let full_off_windows = if is_high_activity {
        vec![("22:00".into(), "04:00".into())]
    } else {
        Vec::new()
    };

    // Noise: recommended max dBA is the minimum of treaty and comfort, with quiet hours. [file:5]
    let mut max_dba = treaty_max_dba.min(comfort.max_night_dba);
    if max_dba.is_infinite() {
        max_dba = comfort.max_night_dba;
    }

    let quiet_hours = if is_high_activity {
        vec![("21:00".into(), "06:00".into())]
    } else {
        vec![("23:00".into(), "05:00".into())]
    };

    // Pesticides: if near pollinator corridors or high activity, restrict timing. [file:5]
    let allowed_windows = if pesticide_restrictive || seg.near_pollinator_corridor || is_high_activity {
        vec![("00:00".into(), "03:00".into())]
    } else {
        vec![("00:00".into(), "05:00".into())]
    };

    let mut no_spray_zones = Vec::new();
    if seg.near_pollinator_corridor {
        no_spray_zones.push("pollinator-corridor-buffer".into());
    }

    // Violation heuristics vs current state. [file:5]
    // Lighting violation if current lux exceeds planned max by >10%.
    if seg.current_lux > max_lux * 1.1 {
        violations.push(ViolationReport {
            segment_id: seg.segment_id.clone(),
            time_band: time_band.into(),
            treaty_id: "ANY".into(),
            violation_type: "LightingLuxExceedsEnvelope".into(),
            magnitude: seg.current_lux - max_lux,
            suggested_change: format!("Dim segment {} by at least {:.1} lux", seg.segment_id, seg.current_lux - max_lux),
        });
    }

    // Noise violation if current dBA exceeds planned max by >5 dB.
    if seg.current_night_dba > max_dba + 5.0 {
        violations.push(ViolationReport {
            segment_id: seg.segment_id.clone(),
            time_band: time_band.into(),
            treaty_id: "ANY".into(),
            violation_type: "NoiseExceedsEnvelope".into(),
            magnitude: seg.current_night_dba - max_dba,
            suggested_change: format!("Retime or relocate loud operations on {}", seg.segment_id),
        });
    }

    // Pesticide violation if regimen is "broad-spectrum" and activity is high. [file:5]
    if seg.pesticide_regime.to_lowercase().contains("broad") && is_high_activity {
        violations.push(ViolationReport {
            segment_id: seg.segment_id.clone(),
            time_band: time_band.into(),
            treaty_id: "ANY".into(),
            violation_type: "PesticideRegimeTooBroad".into(),
            magnitude: max_activity,
            suggested_change: "Switch to targeted or reduced-toxicity agents outside pollinator activity windows".into(),
        });
    }

    let env = OperatingEnvelope {
        segment_id: seg.segment_id.clone(),
        time_band: time_band.into(),
        max_lux,
        allowed_spectra,
        full_off_windows,
        shielding_required,
        max_dba,
        quiet_hours,
        allowed_windows,
        no_spray_zones,
    };

    (env, violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_smoke_test() {
        let seg = SegmentBaseState {
            segment_id: "SEG-001".into(),
            zone_id: "ZONE-A".into(),
            current_lux: 10.0,
            spectrum: "cool_white".into(),
            dimmable: true,
            shieldable: true,
            current_night_dba: 60.0,
            has_impulsive_sources: true,
            pesticide_regime: "broad-spectrum".into(),
            near_pollinator_corridor: true,
        };

        let treaty = BioticTreatyEnv {
            treaty_id: "T-BATS-POLLINATORS-001".into(),
            species_ids: vec!["bat_myotis".into(), "bee_native".into()],
            max_corridor_lux: 3.0,
            max_night_dba: 45.0,
            pesticide_restrictive: true,
        };

        let comfort = HumanComfortTargets {
            min_illuminance_lux: 5.0,
            max_night_dba: 55.0,
            max_pest_pressure: 0.6,
        };

        let activity = SpeciesActivity {
            species_id: "bat_myotis".into(),
            time_band: "22-06".into(),
            segment_id: "SEG-001".into(),
            activity_prob: 0.9,
        };

        let mut human_targets = HashMap::new();
        human_targets.insert("ZONE-A".into(), comfort);

        let input = PlannerInput {
            date_iso: "2026-03-09".into(),
            segments: vec![seg],
            treaties: vec![treaty],
            human_targets_by_zone: human_targets,
            species_activity: vec![activity],
            config: LightNoisePesticidePlannerConfig::default(),
        };

        let out = compute_envelopes(input);
        assert!(!out.envelopes.is_empty());
        assert!(!out.violations.is_empty());
    }
}
