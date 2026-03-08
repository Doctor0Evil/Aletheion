// Somaplex v2: SomaticRouteEngine core types + scoring for joint-load and fall-risk aware routes.

use std::time::Duration;

#[derive(Clone, Debug)]
pub struct CitizenId(pub String);

#[derive(Clone, Debug)]
pub struct SegmentId(pub String);

#[derive(Clone, Debug)]
pub struct RouteId(pub String);

#[derive(Clone, Debug)]
pub enum SurfaceKind {
    Asphalt,
    CoolPavement,
    PackedSoil,
    Tile,
    Ramp,
    Stairs,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Microclimate {
    pub shade_fraction: f32,    // 0.0–1.0
    pub est_temp_c:    f32,     // approximate felt temperature.
    pub humidity_pct:  f32,
}

#[derive(Clone, Debug)]
pub struct SegmentSomaticCost {
    pub joint_load_score: f32,  // per-segment, 0.0–1.0
    pub fall_risk_score:  f32,  // per-segment, 0.0–1.0
}

#[derive(Clone, Debug)]
pub struct RouteSegment {
    pub segment_id:  SegmentId,
    pub length_m:   f32,
    pub duration_s: f32,
    pub surface:    SurfaceKind,
    pub microclimate: Microclimate,
    pub somatic_cost: SegmentSomaticCost,
}

#[derive(Clone, Debug)]
pub struct SomaticRoute {
    pub route_id:      RouteId,
    pub citizen:       CitizenId,
    pub segments:      Vec<RouteSegment>,
    pub total_time_s:  f32,
    pub total_length_m:f32,
    pub agg_joint_load:f32,
    pub agg_fall_risk: f32,
    pub shade_weighted:f32, // share of travel time under shade.
}

#[derive(Clone, Debug)]
pub struct SomaticRoutePreferences {
    pub max_time:           Duration,
    pub max_joint_load:     f32,
    pub max_fall_risk:      f32,
    pub min_shade_fraction: f32,
}

impl SomaticRoute {
    pub fn from_segments(
        route_id: RouteId,
        citizen: CitizenId,
        segments: Vec<RouteSegment>,
    ) -> Self {
        let mut total_time_s = 0.0;
        let mut total_len_m = 0.0;
        let mut agg_joint = 0.0;
        let mut agg_fall = 0.0;
        let mut shade_weighted = 0.0;

        for seg in &segments {
            total_time_s += seg.duration_s;
            total_len_m  += seg.length_m;
            agg_joint    += seg.somatic_cost.joint_load_score * seg.duration_s;
            agg_fall     += seg.somatic_cost.fall_risk_score  * seg.duration_s;
            shade_weighted += seg.microclimate.shade_fraction * seg.duration_s;
        }

        let time_norm = if total_time_s > 0.0 { total_time_s } else { 1.0 };
        SomaticRoute {
            route_id,
            citizen,
            segments,
            total_time_s,
            total_length_m: total_len_m,
            agg_joint_load: agg_joint / time_norm,
            agg_fall_risk:  agg_fall / time_norm,
            shade_weighted: shade_weighted / time_norm,
        }
    }

    pub fn satisfies(&self, prefs: &SomaticRoutePreferences) -> bool {
        self.total_time_s <= prefs.max_time.as_secs_f32()
            && self.agg_joint_load <= prefs.max_joint_load
            && self.agg_fall_risk  <= prefs.max_fall_risk
            && self.shade_weighted >= prefs.min_shade_fraction
    }
}

// Simple scoring helpers that upstream routing engines can call.

pub fn score_joint_load(surface: &SurfaceKind, slope_deg: f32) -> f32 {
    let base = match surface {
        SurfaceKind::Ramp       => 0.2,
        SurfaceKind::CoolPavement => 0.3,
        SurfaceKind::Tile       => 0.4,
        SurfaceKind::PackedSoil => 0.5,
        SurfaceKind::Asphalt    => 0.6,
        SurfaceKind::Stairs     => 0.9,
        SurfaceKind::Unknown    => 0.7,
    };
    let slope_penalty = (slope_deg / 20.0).clamp(0.0, 0.4);
    (base + slope_penalty).clamp(0.0, 1.0)
}

pub fn score_fall_risk(surface: &SurfaceKind, lighting_lux: f32) -> f32 {
    let surf = match surface {
        SurfaceKind::Ramp       => 0.2,
        SurfaceKind::CoolPavement => 0.3,
        SurfaceKind::Tile       => 0.5,
        SurfaceKind::PackedSoil => 0.5,
        SurfaceKind::Asphalt    => 0.4,
        SurfaceKind::Stairs     => 0.8,
        SurfaceKind::Unknown    => 0.6,
    };
    let light_penalty = if lighting_lux < 10.0 {
        0.3
    } else if lighting_lux < 30.0 {
        0.15
    } else {
        0.0
    };
    (surf + light_penalty).clamp(0.0, 1.0)
}
