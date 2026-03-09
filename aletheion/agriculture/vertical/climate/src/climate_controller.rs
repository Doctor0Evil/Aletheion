#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const VERTICAL_FARM_VERSION: u32 = 20260310;
pub const MAX_GROW_ZONES: usize = 512;
pub const MAX_CROP_TYPES: usize = 128;
pub const MAX_ENVIRONMENTAL_SENSORS: usize = 2048;
pub const PHOENIX_TARGET_YIELD_KG_M2_YEAR: f64 = 85.0;
pub const WATER_USAGE_TARGET_L_KG: f64 = 5.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CropType {
    LeafyGreen = 0, Herb = 1, Fruit = 2, Root = 3,
    Microgreen = 4, Mushroom = 5, Sprout = 6, Flower = 7,
    NativeDesert = 8, Medicinal = 9, Grain = 10, Legume = 11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GrowthStage {
    Germination = 0, Seedling = 1, Vegetative = 2, Flowering = 3,
    Fruiting = 4, Maturation = 5, Harvest = 6, Rest = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct EnvironmentalConditions {
    pub temperature_celsius: f64,
    pub humidity_pct: f64,
    pub co2_ppm: u16,
    pub light_intensity_umol_m2_s: u16,
    pub light_spectrum_r: f64,
    pub light_spectrum_g: f64,
    pub light_spectrum_b: f64,
    pub light_spectrum_fr: f64,
    pub photoperiod_h: u8,
    pub air_velocity_ms: f64,
    pub vapour_pressure_deficit_kpa: f64,
    pub nutrient_ph: f64,
    pub nutrient_ec_ms_cm: f64,
    pub dissolved_oxygen_mgl: f64,
}

impl EnvironmentalConditions {
    pub fn optimal_for_crop(&self, crop: CropType, stage: GrowthStage) -> f64 {
        let mut score = 1.0;
        match crop {
            CropType::LeafyGreen => {
                if self.temperature_celsius < 18.0 || self.temperature_celsius > 24.0 { score -= 0.2; }
                if self.humidity_pct < 60.0 || self.humidity_pct > 70.0 { score -= 0.15; }
                if self.co2_ppm < 800 || self.co2_ppm > 1200 { score -= 0.1; }
                if self.light_intensity_umol_m2_s < 200 || self.light_intensity_umol_m2_s > 400 { score -= 0.15; }
            }
            CropType::Fruit => {
                if self.temperature_celsius < 22.0 || self.temperature_celsius > 28.0 { score -= 0.2; }
                if self.humidity_pct < 50.0 || self.humidity_pct > 65.0 { score -= 0.15; }
                if self.light_intensity_umol_m2_s < 400 || self.light_intensity_umol_m2_s > 800 { score -= 0.2; }
            }
            CropType::NativeDesert => {
                if self.temperature_celsius < 25.0 || self.temperature_celsius > 40.0 { score -= 0.15; }
                if self.humidity_pct < 30.0 || self.humidity_pct > 50.0 { score -= 0.1; }
                if self.light_intensity_umol_m2_s < 600 { score -= 0.2; }
            }
            _ => {}
        }
        match stage {
            GrowthStage::Germination => {
                if self.humidity_pct < 80.0 { score -= 0.2; }
                if self.light_intensity_umol_m2_s > 100 { score -= 0.1; }
            }
            GrowthStage::Flowering | GrowthStage::Fruiting => {
                if self.light_spectrum_fr < 0.3 { score -= 0.15; }
            }
            _ => {}
        }
        score.max(0.0)
    }
    pub fn compute_vpd(&mut self) {
        let temp_k = self.temperature_celsius + 273.15;
        let sat_vapor_pressure = 0.6108 * f64::exp((17.27 * self.temperature_celsius) / (self.temperature_celsius + 237.3));
        let actual_vapor_pressure = sat_vapor_pressure * (self.humidity_pct / 100.0);
        self.vapour_pressure_deficit_kpa = sat_vapor_pressure - actual_vapor_pressure;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GrowZone {
    pub zone_id: u32,
    pub crop_type: CropType,
    pub growth_stage: GrowthStage,
    pub planting_date_ns: u64,
    pub expected_harvest_ns: u64,
    pub area_m2: f64,
    pub current_yield_kg: f64,
    pub water_used_l: f64,
    pub energy_used_kwh: f64,
    pub environmental: EnvironmentalConditions,
    pub health_index_0_1: f64,
    pub pest_detection: bool,
    pub disease_detection: bool,
    pub operational: bool,
}

impl GrowZone {
    pub fn days_to_harvest(&self, now_ns: u64) -> u32 {
        if now_ns >= self.expected_harvest_ns { return 0; }
        ((self.expected_harvest_ns - now_ns) / 86400000000000) as u32
    }
    pub fn water_efficiency(&self) -> f64 {
        if self.current_yield_kg == 0.0 { return 0.0; }
        WATER_USAGE_TARGET_L_KG / (self.water_used_l / self.current_yield_kg)
    }
    pub fn requires_intervention(&self) -> bool {
        self.health_index_0_1 < 0.6 || self.pest_detection || self.disease_detection || !self.operational
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NutrientRecipe {
    pub recipe_id: u32,
    pub crop_type: CropType,
    pub growth_stage: GrowthStage,
    pub nitrogen_ppm: f64,
    pub phosphorus_ppm: f64,
    pub potassium_ppm: f64,
    pub calcium_ppm: f64,
    pub magnesium_ppm: f64,
    pub sulfur_ppm: f64,
    pub iron_ppm: f64,
    pub manganese_ppm: f64,
    pub zinc_ppm: f64,
    pub copper_ppm: f64,
    pub boron_ppm: f64,
    pub molybdenum_ppm: f64,
    pub ph_target: f64,
    pub ec_target_ms_cm: f64,
}

pub struct VerticalFarmController {
    pub farm_id: u32,
    pub city_code: [u8; 8],
    pub location_zone_id: u32,
    pub total_area_m2: f64,
    pub grow_zones: [Option<GrowZone>; MAX_GROW_ZONES],
    pub zone_count: usize,
    pub nutrient_recipes: [Option<NutrientRecipe>; MAX_CROP_TYPES],
    pub recipe_count: usize,
    pub total_yield_kg: f64,
    pub total_water_used_l: f64,
    pub total_energy_kwh: f64,
    pub water_recycled_pct: f64,
    pub co2_sequestered_kg: f64,
    pub pesticide_free: bool,
    pub organic_certified: bool,
    pub last_harvest_ns: u64,
    pub audit_checksum: u64,
}

impl VerticalFarmController {
    pub fn new(farm_id: u32, city_code: [u8; 8], total_area_m2: f64, init_ns: u64) -> Self {
        Self {
            farm_id,
            city_code,
            location_zone_id: 0,
            total_area_m2,
            grow_zones: Default::default(),
            zone_count: 0,
            nutrient_recipes: Default::default(),
            recipe_count: 0,
            total_yield_kg: 0.0,
            total_water_used_l: 0.0,
            total_energy_kwh: 0.0,
            water_recycled_pct: 95.0,
            co2_sequestered_kg: 0.0,
            pesticide_free: true,
            organic_certified: false,
            last_harvest_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_grow_zone(&mut self, zone: GrowZone) -> Result<u32, &'static str> {
        if self.zone_count >= MAX_GROW_ZONES {
            return Err("ZONE_LIMIT_EXCEEDED");
        }
        self.grow_zones[self.zone_count] = Some(zone);
        self.zone_count += 1;
        self.update_audit_checksum();
        Ok(zone.zone_id)
    }
    pub fn register_nutrient_recipe(&mut self, recipe: NutrientRecipe) -> Result<u32, &'static str> {
        if self.recipe_count >= MAX_CROP_TYPES {
            return Err("RECIPE_LIMIT_EXCEEDED");
        }
        self.nutrient_recipes[self.recipe_count] = Some(recipe);
        self.recipe_count += 1;
        self.update_audit_checksum();
        Ok(recipe.recipe_id)
    }
    pub fn update_zone_environment(&mut self, zone_id: u32, conditions: EnvironmentalConditions, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.zone_count {
            if let Some(ref mut zone) = self.grow_zones[i] {
                if zone.zone_id == zone_id {
                    zone.environmental = conditions;
                    zone.environmental.compute_vpd();
                    let optimality = conditions.optimal_for_crop(zone.crop_type, zone.growth_stage);
                    zone.health_index_0_1 = (zone.health_index_0_1 * 0.7 + optimality * 0.3).min(1.0);
                    zone.operational = zone.health_index_0_1 > 0.5;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("ZONE_NOT_FOUND")
    }
    pub fn record_harvest(&mut self, zone_id: u32, yield_kg: f64, water_l: f64, energy_kwh: f64, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.zone_count {
            if let Some(ref mut zone) = self.grow_zones[i] {
                if zone.zone_id == zone_id {
                    zone.current_yield_kg = yield_kg;
                    zone.water_used_l = water_l;
                    zone.energy_used_kwh = energy_kwh;
                    self.total_yield_kg += yield_kg;
                    self.total_water_used_l += water_l;
                    self.total_energy_kwh += energy_kwh;
                    self.co2_sequestered_kg += yield_kg * 2.5;
                    zone.growth_stage = GrowthStage::Rest;
                    self.last_harvest_ns = now_ns;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("ZONE_NOT_FOUND")
    }
    pub fn detect_pest_outbreak(&mut self, zone_id: u32, confidence: f64, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.zone_count {
            if let Some(ref mut zone) = self.grow_zones[i] {
                if zone.zone_id == zone_id {
                    if confidence > 0.7 {
                        zone.pest_detection = true;
                        zone.health_index_0_1 *= 0.7;
                    }
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("ZONE_NOT_FOUND")
    }
    pub fn compute_farm_efficiency(&self) -> f64 {
        if self.total_yield_kg == 0.0 { return 0.0; }
        let yield_per_area = self.total_yield_kg / self.total_area_m2;
        let yield_score = (yield_per_area / PHOENIX_TARGET_YIELD_KG_M2_YEAR).min(1.0);
        let water_efficiency = self.total_yield_kg / self.total_water_used_l.max(1.0);
        let water_score = (water_efficiency / (1.0 / WATER_USAGE_TARGET_L_KG)).min(1.0);
        let energy_efficiency = self.total_yield_kg / self.total_energy_kwh.max(1.0);
        let energy_score = energy_efficiency.min(1.0);
        (yield_score * 0.4 + water_score * 0.35 + energy_score * 0.25).min(1.0)
    }
    pub fn get_farm_status(&self, now_ns: u64) -> FarmStatus {
        let operational_zones = self.grow_zones.iter()
            .filter(|z| z.as_ref().map(|zone| zone.operational).unwrap_or(false))
            .count();
        let zones_requiring_attention = self.grow_zones.iter()
            .filter(|z| z.as_ref().map(|zone| zone.requires_intervention()).unwrap_or(false))
            .count();
        let avg_health = self.grow_zones.iter()
            .filter_map(|z| z.as_ref())
            .map(|z| z.health_index_0_1)
            .sum::<f64>() / self.zone_count.max(1) as f64;
        FarmStatus {
            farm_id: self.farm_id,
            total_zones: self.zone_count,
            operational_zones,
            zones_requiring_attention,
            total_area_m2: self.total_area_m2,
            total_yield_kg: self.total_yield_kg,
            total_water_used_l: self.total_water_used_l,
            total_energy_kwh: self.total_energy_kwh,
            water_efficiency_l_kg: if self.total_yield_kg > 0.0 { self.total_water_used_l / self.total_yield_kg } else { 0.0 },
            energy_efficiency_kwh_kg: if self.total_yield_kg > 0.0 { self.total_energy_kwh / self.total_yield_kg } else { 0.0 },
            farm_efficiency_score: self.compute_farm_efficiency(),
            average_zone_health: avg_health,
            water_recycled_pct: self.water_recycled_pct,
            co2_sequestered_kg: self.co2_sequestered_kg,
            pesticide_free: self.pesticide_free,
            last_harvest_ns: self.last_harvest_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.zone_count as u64).wrapping_mul((self.total_yield_kg * 100.0) as u64);
        sum ^= (self.total_water_used_l as u64);
        sum ^= (self.total_energy_kwh as u64);
        for i in 0..self.zone_count {
            if let Some(ref zone) = self.grow_zones[i] {
                sum ^= zone.zone_id as u64;
                sum ^= (zone.health_index_0_1 * 1e6) as u64;
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.zone_count as u64).wrapping_mul((self.total_yield_kg * 100.0) as u64);
        sum ^= (self.total_water_used_l as u64);
        sum ^= (self.total_energy_kwh as u64);
        for i in 0..self.zone_count {
            if let Some(ref zone) = self.grow_zones[i] {
                sum ^= zone.zone_id as u64;
                sum ^= (zone.health_index_0_1 * 1e6) as u64;
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct FarmStatus {
    pub farm_id: u32,
    pub total_zones: usize,
    pub operational_zones: usize,
    pub zones_requiring_attention: usize,
    pub total_area_m2: f64,
    pub total_yield_kg: f64,
    pub total_water_used_l: f64,
    pub total_energy_kwh: f64,
    pub water_efficiency_l_kg: f64,
    pub energy_efficiency_kwh_kg: f64,
    pub farm_efficiency_score: f64,
    pub average_zone_health: f64,
    pub water_recycled_pct: f64,
    pub co2_sequestered_kg: f64,
    pub pesticide_free: bool,
    pub last_harvest_ns: u64,
    pub last_update_ns: u64,
}

impl FarmStatus {
    pub fn sustainability_index(&self) -> f64 {
        let water_score = if self.water_efficiency_l_kg <= WATER_USAGE_TARGET_L_KG { 1.0 } else { 0.6 };
        let energy_score = if self.energy_efficiency_kwh_kg < 5.0 { 1.0 } else { 0.7 };
        let health_score = self.average_zone_health;
        let recycling_bonus = self.water_recycled_pct / 100.0 * 0.1;
        (water_score * 0.35 + energy_score * 0.25 + health_score * 0.3 + recycling_bonus).min(1.0)
    }
}
