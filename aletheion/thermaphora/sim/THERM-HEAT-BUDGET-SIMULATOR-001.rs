// Thermaphora HeatBudgetSimulator v1 (Phoenix desert microclimate)

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Fraction01 = f32;
pub type Celsius = f32;
pub type Minutes = f32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizenId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockId(pub String);

// Mirror of core model subsets (kept local for now)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonHeatFactors {
    pub age_years: u8,
    pub bmi: f32,
    pub acclimatized_to_heat: bool,
    pub baseline_fitness01: Fraction01,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationHeatModifier {
    pub med_code: String,
    pub increases_dehydration01: Fraction01,
    pub impairs_sweating01: Fraction01,
    pub alters_thermoregulation01: Fraction01,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySegment {
    pub interval: TimeInterval,
    pub description: String,
    pub metabolic_rate_met: f32,
    pub clothing_clo: f32,
    pub is_outdoors: bool,
    pub block_id: BlockId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroclimateField {
    pub block_id: BlockId,
    pub air_temp_c: Celsius,
    pub mean_radiant_temp_c: Celsius,
    pub relative_humidity01: Fraction01,
    pub wind_speed_ms: f32,
    pub shade_fraction01: Fraction01,
    pub surface_temp_c: Celsius,
    pub has_cool_pavement: bool,
    pub tree_canopy_fraction01: Fraction01,
    pub evap_cooling_capacity01: Fraction01,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatBudgetProfile {
    pub citizen_id: CitizenId,
    pub date_iso: String,
    pub person_factors: PersonHeatFactors,
    pub meds: Vec<MedicationHeatModifier>,
    pub activities: Vec<ActivitySegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentHeatLoad {
    pub interval: TimeInterval,
    pub block_id: BlockId,
    pub predicted_core_temp_delta_c: f32,
    pub heat_strain_index01: Fraction01,
    pub dehydration_risk_index01: Fraction01,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatBudgetResult {
    pub citizen_id: CitizenId,
    pub date_iso: String,
    pub segments: Vec<SegmentHeatLoad>,
    pub max_heat_strain_index01: Fraction01,
    pub minutes_above_safe_threshold: Minutes,
    pub recovery_time_minutes: Minutes,
}

// Safe threshold tuned for desert-heat policy discussions[web:31][file:1]
const SAFE_STRAIN_THRESHOLD: Fraction01 = 0.6;

fn duration_minutes(interval: &TimeInterval) -> Minutes {
    let start = (interval.start.hour as i32) * 60 + (interval.start.minute as i32);
    let end = (interval.end.hour as i32) * 60 + (interval.end.minute as i32);
    (end - start).max(0) as f32
}

// Very simplified heat load estimator; later versions can plug full heat balance.[file:1]
fn estimate_segment_heat_load(
    activity: &ActivitySegment,
    person: &PersonHeatFactors,
    meds: &[MedicationHeatModifier],
    micro: &MicroclimateField,
) -> SegmentHeatLoad {
    let duration_min = duration_minutes(&activity.interval);

    let t_a = micro.air_temp_c;
    let t_r = micro.mean_radiant_temp_c;
    let rh = micro.relative_humidity01;
    let v = micro.wind_speed_ms;
    let shade = micro.shade_fraction01;
    let evap = micro.evap_cooling_capacity01;

    let met = activity.metabolic_rate_met;
    let clo = activity.clothing_clo;

    let mut load = 0.0_f32;

    let env_temp = 0.5 * t_a + 0.5 * t_r;

    if env_temp > 26.0 {
        load += (env_temp - 26.0) / 10.0;
    }
    if rh > 0.5 {
        load += (rh - 0.5) * 0.8;
    }
    if met > 1.5 {
        load += (met - 1.5) / 2.0;
    }
    if clo > 0.7 {
        load += (clo - 0.7) * 0.5;
    }

    load -= 0.4 * shade;
    load -= 0.3 * evap;
    load -= 0.1 * v;

    if !person.acclimatized_to_heat {
        load += 0.2;
    }

    let mut med_penalty = 0.0_f32;
    for m in meds {
        med_penalty += 0.3 * (m.increases_dehydration01 + m.impairs_sweating01 + m.alters_thermoregulation01);
    }
    load += med_penalty;

    let heat_strain_index = load.clamp(0.0, 1.5).min(1.0);

    let dehydration_index =
        (met / 3.0 + duration_min / 480.0 + med_penalty).clamp(0.0, 1.0);

    let core_temp_delta_c = 0.5 * heat_strain_index;

    SegmentHeatLoad {
        interval: activity.interval.clone(),
        block_id: micro.block_id.clone(),
        predicted_core_temp_delta_c: core_temp_delta_c,
        heat_strain_index01: heat_strain_index,
        dehydration_risk_index01: dehydration_index,
    }
}

pub struct HeatBudgetSimulator;

impl HeatBudgetSimulator {
    pub fn simulate_day(
        profile: &HeatBudgetProfile,
        fields: &HashMap<String, MicroclimateField>,
    ) -> HeatBudgetResult {
        let mut segments = Vec::new();
        let mut max_strain = 0.0_f32;
        let mut minutes_above = 0.0_f32;

        for activity in &profile.activities {
            let key = activity.block_id.0.clone();
            if let Some(field) = fields.get(&key) {
                let seg = estimate_segment_heat_load(
                    activity,
                    &profile.person_factors,
                    &profile.meds,
                    field,
                );
                max_strain = max_strain.max(seg.heat_strain_index01);
                if seg.heat_strain_index01 > SAFE_STRAIN_THRESHOLD {
                    minutes_above += duration_minutes(&seg.interval);
                }
                segments.push(seg);
            }
        }

        let recovery = if max_strain <= SAFE_STRAIN_THRESHOLD {
            0.0
        } else {
            (max_strain - SAFE_STRAIN_THRESHOLD) * 60.0
        };

        HeatBudgetResult {
            citizen_id: profile.citizen_id.clone(),
            date_iso: profile.date_iso.clone(),
            segments,
            max_heat_strain_index01: max_strain,
            minutes_above_safe_threshold: minutes_above,
            recovery_time_minutes: recovery,
        }
    }
}
