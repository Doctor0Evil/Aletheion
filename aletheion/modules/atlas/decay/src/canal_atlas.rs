#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const ATLAS_GRID_SIZE_M: f64 = 100.0;
pub const MAX_REACH_SEGMENTS: usize = 1024;
pub const ATLAS_VERSION: u32 = 20260310;

#[derive(Clone, Copy, Debug)]
pub struct GeoCoordinate {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub elevation_m: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct HydraulicParams {
    pub flow_velocity_ms: f64,
    pub turbulence_intensity_0_1: f64,
    pub residence_time_h: f64,
    pub depth_m: f64,
    pub width_m: f64,
    pub slope_0_1: f64,
    pub roughness_n: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ChemicalParams {
    pub dissolved_oxygen_mgl: f64,
    pub nitrogen_total_mgl: f64,
    pub phosphorus_total_mgl: f64,
    pub ph: f64,
    pub conductivity_uscm: f64,
    pub sulfide_h2s_mgl: f64,
    pub heavy_metals_ugl: f64,
    pub temperature_celsius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct BiologicalParams {
    pub biofilm_density_gm2: f64,
    pub microbial_activity_0_1: f64,
    pub decay_constant_day: f64,
    pub algae_coverage_pct: f64,
    pub macroinvertebrate_index_0_10: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct FoulingParams {
    pub sediment_accumulation_mm_year: f64,
    pub grease_fog_kg_m2: f64,
    pub microplastics_count_m2: u64,
    pub biofilm_thickness_mm: f64,
    pub corrosion_rate_mm_year: f64,
}

#[derive(Clone, Debug)]
pub struct ReachSegment {
    pub segment_id: u32,
    pub start_geo: GeoCoordinate,
    pub end_geo: GeoCoordinate,
    pub length_m: f64,
    pub hydraulic: HydraulicParams,
    pub chemical: ChemicalParams,
    pub biological: BiologicalParams,
    pub fouling: FoulingParams,
    pub risk_rx: f64,
    pub last_survey_ns: u64,
    pub survey_confidence_0_1: f64,
}

impl ReachSegment {
    pub fn compute_risk_coordinate(&mut self, now_ns: u64) -> f64 {
        let mut rx = 0.0;
        if self.chemical.dissolved_oxygen_mgl < 3.0 { rx += 0.2; }
        if self.chemical.sulfide_h2s_mgl > 0.5 { rx += 0.15; }
        if self.fouling.grease_fog_kg_m2 > 0.1 { rx += 0.2; }
        if self.fouling.corrosion_rate_mm_year > 1.0 { rx += 0.15; }
        if self.biological.macroinvertebrate_index_0_10 < 5 { rx += 0.15; }
        if self.hydraulic.turbulence_intensity_0_1 > 0.7 { rx += 0.15; }
        self.risk_rx = rx.min(1.0);
        self.last_survey_ns = now_ns;
        self.risk_rx
    }
    pub fn habitat_quality_index(&self) -> f64 {
        let mut hqi = 1.0;
        hqi -= (self.chemical.dissolved_oxygen_mgl < 5.0) as u8 as f64 * 0.2;
        hqi -= (self.chemical.sulfide_h2s_mgl > 0.1) as u8 as f64 * 0.15;
        hqi -= (self.fouling.microplastics_count_m2 > 100) as u8 as f64 * 0.2;
        hqi -= (self.biological.macroinvertebrate_index_0_10 < 7) as u8 as f64 * 0.15;
        hqi -= (self.fouling.biofilm_thickness_mm > 5.0) as u8 as f64 * 0.1;
        hqi.max(0.0)
    }
    pub fn suitable_for_circular_hardware(&self) -> bool {
        self.risk_rx < 0.6 &&
        self.chemical.dissolved_oxygen_mgl >= 3.0 &&
        self.hydraulic.flow_velocity_ms < 2.0 &&
        self.fouling.grease_fog_kg_m2 < 0.3
    }
}

pub struct CanalDecayAtlas {
    pub atlas_id: u32,
    pub city_code: [u8; 8],
    pub segments: [Option<ReachSegment>; MAX_REACH_SEGMENTS],
    pub segment_count: usize,
    pub total_length_km: f64,
    pub last_full_survey_ns: u64,
    pub version: u32,
}

impl CanalDecayAtlas {
    pub fn new(atlas_id: u32, city_code: [u8; 8]) -> Self {
        Self {
            atlas_id,
            city_code,
            segments: Default::default(),
            segment_count: 0,
            total_length_km: 0.0,
            last_full_survey_ns: 0,
            version: ATLAS_VERSION,
        }
    }
    pub fn add_segment(&mut self, segment: ReachSegment) -> Result<(), &'static str> {
        if self.segment_count >= MAX_REACH_SEGMENTS {
            return Err("ATLAS_SEGMENT_LIMIT");
        }
        self.total_length_km += segment.length_m / 1000.0;
        self.segments[self.segment_count] = Some(segment);
        self.segment_count += 1;
        Ok(())
    }
    pub fn update_all_risk_coordinates(&mut self, now_ns: u64) {
        for i in 0..self.segment_count {
            if let Some(ref mut seg) = self.segments[i] {
                seg.compute_risk_coordinate(now_ns);
            }
        }
    }
    pub fn find_high_risk_segments(&self, threshold: f64) -> Vec<u32> {
        let mut result = Vec::new();
        for i in 0..self.segment_count {
            if let Some(ref seg) = self.segments[i] {
                if seg.risk_rx > threshold {
                    result.push(seg.segment_id);
                }
            }
        }
        result
    }
    pub fn find_suitable_hardware_sites(&self) -> Vec<u32> {
        let mut result = Vec::new();
        for i in 0..self.segment_count {
            if let Some(ref seg) = self.segments[i] {
                if seg.suitable_for_circular_hardware() {
                    result.push(seg.segment_id);
                }
            }
        }
        result
    }
    pub fn average_habitat_quality(&self) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.segment_count {
            if let Some(ref seg) = self.segments[i] {
                sum += seg.habitat_quality_index();
            }
        }
        if self.segment_count == 0 { return 0.0; }
        sum / self.segment_count as f64
    }
    pub fn compute_atlas_checksum(&self) -> u64 {
        let mut sum: u64 = 0;
        for i in 0..self.segment_count {
            if let Some(ref seg) = self.segments[i] {
                sum ^= (seg.segment_id as u64).wrapping_mul((seg.risk_rx * 1e6) as u64);
                sum = sum.rotate_left(11);
            }
        }
        sum
    }
}
