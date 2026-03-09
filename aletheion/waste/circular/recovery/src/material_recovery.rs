#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const MATERIAL_RECOVERY_VERSION: u32 = 20260310;
pub const MAX_WASTE_STREAMS: usize = 256;
pub const MAX_SORTING_LINES: usize = 64;
pub const MAX_OUTPUT_BALES: usize = 512;
pub const TARGET_RECOVERY_RATE_PCT: f64 = 99.0;
pub const TARGET_CONTAMINATION_PCT: f64 = 1.0;
pub const PHOENIX_WASTE_DIVERSION_TARGET_PCT: f64 = 90.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WasteStreamType {
    MixedRecyclables = 0, Organic = 1, Construction = 2, Hazardous = 3,
    Electronic = 4, Textile = 5, Glass = 6, Metal = 7,
    PlasticPET = 8, PlasticHDPE = 9, PlasticOther = 10, Paper = 11,
    YardWaste = 12, FoodWaste = 13, Bulky = 14, Medical = 15,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialGrade {
    Premium = 0, Grade1 = 1, Grade2 = 2, Grade3 = 3,
    Feedstock = 4, Fuel = 5, Landfill = 6, Hazardous = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct WasteStream {
    pub stream_id: u32,
    pub stream_type: WasteStreamType,
    pub source_facility_id: u32,
    pub mass_kg: f64,
    pub volume_m3: f64,
    pub contamination_pct: f64,
    pub moisture_pct: f64,
    pub collection_date_ns: u64,
    pub processing_priority: u8,
    pub hazardous: bool,
    pub organic: bool,
    pub recyclable: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SortingLine {
    pub line_id: u32,
    pub line_type: u8,
    pub capacity_kg_h: f64,
    pub current_throughput_kg_h: f64,
    pub efficiency_0_1: f64,
    pub downtime_pct: f64,
    pub last_maintenance_ns: u64,
    pub operational: bool,
    pub automated: bool,
    pub ai_vision_enabled: bool,
    pub contamination_detection: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct OutputBale {
    pub bale_id: u32,
    pub material_type: WasteStreamType,
    pub grade: MaterialGrade,
    pub mass_kg: f64,
    pub density_kg_m3: f64,
    pub dimensions_cm: [u16; 3],
    pub contamination_pct: f64,
    pub market_value_usd_ton: f64,
    pub destination_facility_id: u32,
    pub production_date_ns: u64,
    pub quality_certified: bool,
    pub carbon_offset_kg_co2e: f64,
}

pub struct MaterialRecoveryFacility {
    pub facility_id: u32,
    pub city_code: [u8; 8],
    pub location_zone_id: u32,
    pub total_capacity_tpd: f64,
    pub waste_streams: [Option<WasteStream>; MAX_WASTE_STREAMS],
    pub stream_count: usize,
    pub sorting_lines: [Option<SortingLine>; MAX_SORTING_LINES],
    pub line_count: usize,
    pub output_bales: [Option<OutputBale>; MAX_OUTPUT_BALES],
    pub bale_count: usize,
    pub total_input_mass_kg: f64,
    pub total_recovered_mass_kg: f64,
    pub total_contamination_kg: f64,
    pub total_landfill_kg: f64,
    pub total_energy_kwh: f64,
    pub total_water_used_l: f64,
    pub carbon_offset_kg_co2e: f64,
    pub revenue_usd: f64,
    pub operating_cost_usd: f64,
    pub last_audit_ns: u64,
    pub audit_checksum: u64,
}

impl MaterialRecoveryFacility {
    pub fn new(facility_id: u32, city_code: [u8; 8], capacity_tpd: f64, init_ns: u64) -> Self {
        Self {
            facility_id,
            city_code,
            location_zone_id: 0,
            total_capacity_tpd: capacity_tpd,
            waste_streams: Default::default(),
            stream_count: 0,
            sorting_lines: Default::default(),
            line_count: 0,
            output_bales: Default::default(),
            bale_count: 0,
            total_input_mass_kg: 0.0,
            total_recovered_mass_kg: 0.0,
            total_contamination_kg: 0.0,
            total_landfill_kg: 0.0,
            total_energy_kwh: 0.0,
            total_water_used_l: 0.0,
            carbon_offset_kg_co2e: 0.0,
            revenue_usd: 0.0,
            operating_cost_usd: 0.0,
            last_audit_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_waste_stream(&mut self, stream: WasteStream) -> Result<u32, &'static str> {
        if self.stream_count >= MAX_WASTE_STREAMS {
            return Err("STREAM_LIMIT_EXCEEDED");
        }
        self.waste_streams[self.stream_count] = Some(stream);
        self.stream_count += 1;
        self.total_input_mass_kg += stream.mass_kg;
        self.update_audit_checksum();
        Ok(stream.stream_id)
    }
    pub fn register_sorting_line(&mut self, line: SortingLine) -> Result<u32, &'static str> {
        if self.line_count >= MAX_SORTING_LINES {
            return Err("LINE_LIMIT_EXCEEDED");
        }
        self.sorting_lines[self.line_count] = Some(line);
        self.line_count += 1;
        self.update_audit_checksum();
        Ok(line.line_id)
    }
    pub fn process_waste_stream(&mut self, stream_id: u32, recovery_rate: f64, 
                                 contamination_rate: f64, energy_kwh: f64, 
                                 water_l: f64, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.stream_count {
            if let Some(ref mut stream) = self.waste_streams[i] {
                if stream.stream_id == stream_id {
                    let recovered = stream.mass_kg * recovery_rate;
                    let contamination = stream.mass_kg * contamination_rate;
                    let landfill = stream.mass_kg - recovered - contamination;
                    self.total_recovered_mass_kg += recovered;
                    self.total_contamination_kg += contamination;
                    self.total_landfill_kg += landfill;
                    self.total_energy_kwh += energy_kwh;
                    self.total_water_used_l += water_l;
                    self.carbon_offset_kg_co2e += recovered * 1.5;
                    stream.processing_priority = 0;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("STREAM_NOT_FOUND")
    }
    pub fn create_output_bale(&mut self, bale: OutputBale) -> Result<u32, &'static str> {
        if self.bale_count >= MAX_OUTPUT_BALES {
            return Err("BALE_LIMIT_EXCEEDED");
        }
        self.output_bales[self.bale_count] = Some(bale);
        self.bale_count += 1;
        self.revenue_usd += bale.mass_kg / 1000.0 * bale.market_value_usd_ton;
        self.update_audit_checksum();
        Ok(bale.bale_id)
    }
    pub fn compute_recovery_rate(&self) -> f64 {
        if self.total_input_mass_kg == 0.0 { return 0.0; }
        self.total_recovered_mass_kg / self.total_input_mass_kg * 100.0
    }
    pub fn compute_contamination_rate(&self) -> f64 {
        if self.total_input_mass_kg == 0.0 { return 0.0; }
        self.total_contamination_kg / self.total_input_mass_kg * 100.0
    }
    pub fn compute_diversion_rate(&self) -> f64 {
        if self.total_input_mass_kg == 0.0 { return 0.0; }
        (self.total_recovered_mass_kg + self.total_contamination_kg) / 
        self.total_input_mass_kg * 100.0
    }
    pub fn compute_facility_efficiency(&self) -> f64 {
        let recovery_score = (self.compute_recovery_rate() / TARGET_RECOVERY_RATE_PCT).min(1.0);
        let contamination_score = if self.compute_contamination_rate() <= TARGET_CONTAMINATION_PCT {
            1.0
        } else {
            0.5
        };
        let diversion_score = (self.compute_diversion_rate() / PHOENIX_WASTE_DIVERSION_TARGET_PCT).min(1.0);
        let energy_efficiency = if self.total_input_mass_kg > 0.0 {
            (100.0 / (self.total_energy_kwh / self.total_input_mass_kg * 1000.0)).min(1.0)
        } else { 0.0 };
        (recovery_score * 0.35 + contamination_score * 0.25 + 
         diversion_score * 0.25 + energy_efficiency * 0.15).min(1.0)
    }
    pub fn get_facility_status(&self, now_ns: u64) -> FacilityStatus {
        let operational_lines = self.sorting_lines.iter()
            .filter(|l| l.as_ref().map(|line| line.operational).unwrap_or(false))
            .count();
        let premium_bales = self.output_bales.iter()
            .filter(|b| b.as_ref().map(|bale| bale.grade == MaterialGrade::Premium).unwrap_or(false))
            .count();
        FacilityStatus {
            facility_id: self.facility_id,
            total_capacity_tpd: self.total_capacity_tpd,
            total_input_mass_kg: self.total_input_mass_kg,
            total_recovered_mass_kg: self.total_recovered_mass_kg,
            total_contamination_kg: self.total_contamination_kg,
            total_landfill_kg: self.total_landfill_kg,
            recovery_rate_pct: self.compute_recovery_rate(),
            contamination_rate_pct: self.compute_contamination_rate(),
            diversion_rate_pct: self.compute_diversion_rate(),
            facility_efficiency_score: self.compute_facility_efficiency(),
            total_lines: self.line_count,
            operational_lines,
            total_bales: self.bale_count,
            premium_bales,
            total_energy_kwh: self.total_energy_kwh,
            total_water_used_l: self.total_water_used_l,
            carbon_offset_kg_co2e: self.carbon_offset_kg_co2e,
            revenue_usd: self.revenue_usd,
            operating_cost_usd: self.operating_cost_usd,
            net_profit_usd: self.revenue_usd - self.operating_cost_usd,
            last_audit_ns: self.last_audit_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.stream_count as u64).wrapping_mul((self.total_input_mass_kg * 100.0) as u64);
        sum ^= (self.total_recovered_mass_kg as u64);
        sum ^= (self.total_landfill_kg as u64);
        sum ^= (self.revenue_usd as u64);
        for i in 0..self.stream_count {
            if let Some(ref stream) = self.waste_streams[i] {
                sum ^= stream.stream_id as u64;
                sum ^= (stream.contamination_pct * 1e4) as u64;
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.stream_count as u64).wrapping_mul((self.total_input_mass_kg * 100.0) as u64);
        sum ^= (self.total_recovered_mass_kg as u64);
        sum ^= (self.total_landfill_kg as u64);
        sum ^= (self.revenue_usd as u64);
        for i in 0..self.stream_count {
            if let Some(ref stream) = self.waste_streams[i] {
                sum ^= stream.stream_id as u64;
                sum ^= (stream.contamination_pct * 1e4) as u64;
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct FacilityStatus {
    pub facility_id: u32,
    pub total_capacity_tpd: f64,
    pub total_input_mass_kg: f64,
    pub total_recovered_mass_kg: f64,
    pub total_contamination_kg: f64,
    pub total_landfill_kg: f64,
    pub recovery_rate_pct: f64,
    pub contamination_rate_pct: f64,
    pub diversion_rate_pct: f64,
    pub facility_efficiency_score: f64,
    pub total_lines: usize,
    pub operational_lines: usize,
    pub total_bales: usize,
    pub premium_bales: usize,
    pub total_energy_kwh: f64,
    pub total_water_used_l: f64,
    pub carbon_offset_kg_co2e: f64,
    pub revenue_usd: f64,
    pub operating_cost_usd: f64,
    pub net_profit_usd: f64,
    pub last_audit_ns: u64,
    pub last_update_ns: u64,
}

impl FacilityStatus {
    pub fn circularity_index(&self) -> f64 {
        let recovery_component = self.recovery_rate_pct / 100.0;
        let diversion_component = self.diversion_rate_pct / 100.0;
        let quality_component = self.premium_bales as f64 / self.total_bales.max(1) as f64;
        let carbon_component = (self.carbon_offset_kg_co2e / self.total_input_mass_kg.max(1.0)).min(1.0);
        (recovery_component * 0.3 + diversion_component * 0.3 + 
         quality_component * 0.2 + carbon_component * 0.2).min(1.0)
    }
}
