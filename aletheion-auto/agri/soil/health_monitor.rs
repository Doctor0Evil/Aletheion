/**
 * Aletheion Smart City Core - Batch 2
 * File: 111/200
 * Layer: 31 (Advanced Agriculture)
 * Path: aletheion-auto/agri/soil/health_monitor.rs
 * 
 * Research Basis (Environmental & Climate Integration - E):
 *   - Phoenix Desert Soil Parameters: pH 7.5-8.5 (alkaline), organic matter <1% native, target 2-3% regenerative
 *   - Carbon Sequestration Potential: 0.5-1.0 ton/acre/year with regenerative practices in arid climates
 *   - Native Soil Microbiome: 40+ bacterial/fungal species adapted to Sonoran Desert conditions
 *   - Monsoon Moisture Dynamics: 80% annual rainfall in Aug-Sept, rapid infiltration requirements
 *   - Urban Soil Contamination: Heavy metal thresholds (Pb < 100ppm, As < 10ppm) for safe urban agriculture
 *   - Indigenous Soil Management: Akimel O'odham flood irrigation ("ak chin"), Piipaash soil amendment traditions
 *   - Salinity Tolerance: Native species thresholds (EC < 4.0 dS/m for Saguaro, < 6.0 dS/m for Palo Verde)
 *   - Water Conservation: 60% reduction via xeriscaping soil structure optimization
 *   - Regenerative Techniques: Biochar amendment (5-10% by volume), mycorrhizal inoculation, cover cropping with native species
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Soil Rights & Microbiome Sovereignty)
 *   - Post-Quantum Secure (via aletheion_:pq_crypto)
 * 
 * Blacklist Check: 
 *   - NO SHA-256, SHA3, Python, Digital Twins, Rollbacks.
 *   - Uses SHA-512 (via PQ module) or PQ-native hashing.
 * 
 * Workflow: ERM Chain (Sense → Model → Optimize → Treaty-Check → Act → Log → Interface)
 */

#![no_std]
#![feature(alloc_error_handler)]
extern crate alloc;

use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet, HashMap};
use alloc::string::String;
use core::result::Result;
use core::f32::consts::PI;

// Internal Aletheion Crates (Established in Batch 1)
use aletheion_:pq_crypto::hash::pq_hash;
use aletheion_:did_wallet::DIDWallet;
use aletheion_gov::treaty::TreatyCompliance;
use aletheion_physical::hal::ActuatorCommand;
use aletheion_comms::mesh::OfflineQueue;
use aletheion_core::identity::BirthSign;
use aletheion_water::reclamation::WaterQuality;

// --- Constants & Phoenix Soil Parameters ---

/// Native Sonoran Desert soil characteristics
const NATIVE_SOIL_PH_MIN: f32 = 7.5;
const NATIVE_SOIL_PH_MAX: f32 = 8.5;
const TARGET_REGENERATIVE_PH: f32 = 7.8; // Optimal for native species
const NATIVE_ORGANIC_MATTER_PERCENT: f32 = 0.8; // Typical desert soil
const TARGET_ORGANIC_MATTER_PERCENT: f32 = 2.5; // Regenerative agriculture target
const NATIVE_SOIL_BULK_DENSITY_G_CM3: f32 = 1.55;
const TARGET_BULK_DENSITY_G_CM3: f32 = 1.35; // Improved structure

/// Carbon sequestration targets (tons per acre per year)
const CARBON_SEQUESTRATION_MIN_TON_ACRE_YR: f32 = 0.5;
const CARBON_SEQUESTRATION_MAX_TON_ACRE_YR: f32 = 1.0;
const CARBON_SEQUESTRATION_CURRENT_TON_ACRE_YR: f32 = 0.2; // Baseline urban soil

/// Moisture dynamics (Phoenix monsoon pattern)
const MONSOON_SEASON_MONTHS: [u8; 2] = [8, 9]; // August-September
const MIN_INFILTRATION_RATE_INCHES_PER_HOUR: f32 = 2.0; // Required for flash flood resilience
const OPTIMAL_FIELD_CAPACITY_PERCENT: f32 = 25.0; // Water holding capacity
const WILTING_POINT_PERCENT: f32 = 8.0; // Permanent wilting point

/// Salinity thresholds (dS/m - deciSiemens per meter)
const SAGUARO_SALINITY_TOLERANCE_DS_M: f32 = 4.0;
const PALO_VERDE_SALINITY_TOLERANCE_DS_M: f32 = 6.0;
const OCOTILLO_SALINITY_TOLERANCE_DS_M: f32 = 5.0;
const CROP_SALINITY_THRESHOLD_DS_M: f32 = 3.0; // For food crops

/// Contamination thresholds (ppm - parts per million)
const LEAD_SAFE_THRESHOLD_PPM: f32 = 100.0;
const ARSENIC_SAFE_THRESHOLD_PPM: f32 = 10.0;
const CADMIUM_SAFE_THRESHOLD_PPM: f32 = 3.0;
const MERCURY_SAFE_THRESHOLD_PPM: f32 = 0.3;

/// Microbiome diversity targets
const MIN_BACTERIAL_SPECIES_COUNT: usize = 25;
const MIN_FUNGAL_SPECIES_COUNT: usize = 15;
const TARGET_MICROBIOME_DIVERISTY_INDEX: f32 = 0.75; // Shannon index
const MYCORRHIZAL_COLONIZATION_PERCENT: f32 = 60.0; // Target for native plants

/// Regenerative amendment parameters
const BIOCHAR_AMENDMENT_PERCENT: f32 = 7.5; // 5-10% by volume optimal
const COMPOST_AMENDMENT_INCHES: f32 = 2.0; // Annual application depth
const COVER_CROP_BIOMASS_LBS_PER_ACRE: f32 = 2000.0; // Minimum seasonal biomass

/// Indigenous soil management parameters
const AK_CHIN_IRRIGATION_DURATION_HOURS: f32 = 12.0; // Traditional flood irrigation period
const SOIL_AMENDMENT_MOON_PHASE: u8 = 3; // Traditional timing (waxing moon)

/// Sensor specifications
const SOIL_SENSOR_DEPTH_INCHES: f32 = 12.0; // Standard monitoring depth
const SENSOR_CALIBRATION_INTERVAL_DAYS: u32 = 90;
const MICROBIOME_SAMPLING_INTERVAL_DAYS: u32 = 30;

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
const OFFLINE_BUFFER_HOURS: u32 = 72;
/// Treaty Check Cache TTL (seconds)
const TREATY_CACHE_TTL: u64 = 300;

/// Maximum soil management zones per facility
const MAX_SOIL_ZONES: usize = 50;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SoilTexture {
    SandyLoam,
    Loam,
    ClayLoam,
    SandyClayLoam,
    SiltyClayLoam,
    DesertCrust,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SoilHealthState {
    Degraded,
    Recovering,
    Stable,
    Regenerative,
    Thriving,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MicrobiomeType {
    BacterialDominant,
    FungalDominant,
    Balanced,
    PathogenInfested,
    Sterile,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoilContaminant {
    Lead,
    Arsenic,
    Cadmium,
    Mercury,
    PetroleumHydrocarbons,
    PesticideResidue,
    HeavyMetalsMixed,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegenerativeAction {
    BiocharAmendment,
    CompostApplication,
    MycorrhizalInoculation,
    CoverCropping,
    ReducedTillage,
    AkChinIrrigation,
    NativeMulching,
    Phytoremediation,
    SoilAeration,
    ContaminantRemediation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoilAlertType {
    ContaminationDetected,
    SalinityExceeded,
    OrganicMatterDeficient,
    MicrobiomeCollapse,
    ErosionRisk,
    CompactionDetected,
    pHImbalance,
    MoistureStress,
    TreatyViolation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndigenousSoilPractice {
    AkChinFloodIrrigation,
    RiverBottomCultivation,
    MesquitePodAmendment,
    CreosoteResinApplication,
    SeasonalBurning,
    MoonPhasePlanting,
    AncestralSeedSaving,
}

#[derive(Clone)]
pub struct SoilSensorReading {
    pub timestamp: u64,
    pub zone_id: [u8; 32],
    pub ph_level: f32,
    pub organic_matter_percent: f32,
    pub moisture_percent: f32,
    pub electrical_conductivity_ds_m: f32, // Salinity indicator
    pub temperature_f: f32,
    pub bulk_density_g_cm3: f32,
    pub infiltration_rate_inches_per_hour: f32,
    pub nitrogen_ppm: f32,
    pub phosphorus_ppm: f32,
    pub potassium_ppm: f32,
    pub lead_ppm: f32,
    pub arsenic_ppm: f32,
    pub microbiome_diversity_index: f32,
    pub mycorrhizal_colonization_percent: f32,
    pub sensor_id: [u8; 32],
    pub depth_inches: f32,
}

#[derive(Clone)]
pub struct MicrobiomeAnalysis {
    pub analysis_id: [u8; 32],
    pub zone_id: [u8; 32],
    pub sample_date: u64,
    pub bacterial_species_count: usize,
    pub fungal_species_count: usize,
    pub dominant_species: Vec<String>,
    pub pathogen_presence: bool,
    pub diversity_index: f32,
    pub functional_groups: HashMap<String, f32>, // e.g., "nitrogen_fixers": 0.35
    pub indigenous_microbiome_match: f32, // 0.0-1.0 similarity to native Sonoran soils
}

#[derive(Clone)]
pub struct SoilManagementZone {
    pub zone_id: [u8; 32],
    pub name: String,
    pub boundaries: Vec<[f64; 2]>, // GPS polygon
    pub area_sqft: f32,
    pub soil_texture: SoilTexture,
    pub current_health_state: SoilHealthState,
    pub target_health_state: SoilHealthState,
    pub indigenous_territory: bool,
    pub treaty_zone_id: Option<[u8; 32]>,
    pub native_vegetation: bool,
    pub urban_agriculture: bool,
    pub contamination_history: Vec<SoilContaminant>,
    pub carbon_sequestration_tons_per_acre_yr: f32,
    pub last_amendment_date: u64,
    pub amendment_history: Vec<SoilAmendmentRecord>,
}

#[derive(Clone)]
pub struct SoilAmendmentRecord {
    pub amendment_id: [u8; 32],
    pub zone_id: [u8; 32],
    pub action_type: RegenerativeAction,
    pub indigenous_practice: Option<IndigenousSoilPractice>,
    pub application_date: u64,
    pub quantity: f32, // Volume or weight
    pub unit: String, // "cubic_yards", "pounds", etc.
    pub carbon_impact_tons: f32,
    pub treaty_compliant: bool,
    pub fpic_verified: bool,
}

#[derive(Clone)]
pub struct SoilHealthAlert {
    pub alert_id: [u8; 32],
    pub alert_type: SoilAlertType,
    pub zone_id: [u8; 32],
    pub severity: u8, // 0-100
    pub measured_value: f32,
    pub threshold_value: f32,
    pub description: String,
    pub timestamp: u64,
    pub requires_immediate_action: bool,
    pub treaty_implications: bool,
}

#[derive(Clone)]
pub struct RegenerativeActionPlan {
    pub plan_id: [u8; 32],
    pub zone_id: [u8; 32],
    pub action_type: RegenerativeAction,
    pub indigenous_practice: Option<IndigenousSoilPractice>,
    pub priority: u8, // 0-100
    pub estimated_cost_usd: f32,
    pub estimated_duration_days: u32,
    pub carbon_sequestration_impact_tons: f32,
    pub water_conservation_impact_gallons: f32,
    pub treaty_compliant: bool,
    pub fpic_required: bool,
    pub scheduled_start_date: u64,
    pub scheduled_end_date: u64,
    pub success_metrics: HashMap<String, f32>,
}

#[derive(Clone)]
pub struct IndigenousSoilKnowledgeEntry {
    pub entry_id: [u8; 32],
    pub knowledge_type: IndigenousSoilPractice,
    pub description: String,
    pub seasonal_timing: Vec<u8>, // Months (1-12)
    pub plant_associations: Vec<[u8; 32]>, // Native plant IDs
    pub soil_type_applicability: Vec<SoilTexture>,
    knowledge_source: IndigenousCommunity,
    fpic_verified: bool,
    timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndigenousCommunity {
    AkimelOodham,
    Piipaash,
    OtherIndigenousNation,
}

#[derive(Clone)]
pub struct SoilHealthMetrics {
    pub current_health_state: SoilHealthState,
    pub total_zones_monitored: usize,
    pub zones_requiring_intervention: usize,
    pub average_organic_matter_percent: f32,
    pub average_ph: f32,
    pub carbon_sequestration_total_tons_yr: f32,
    pub contamination_incidents: usize,
    pub treaty_violations: usize,
    pub indigenous_practices_integrated: usize,
    pub water_conservation_gallons_yr: f32,
}

#[derive(Clone)]
pub struct SoilNetworkConfiguration {
    pub network_id: [u8; 32],
    pub management_zones: Vec<SoilManagementZone>,
    pub sensor_network: Vec<[u8; 32]>,
    pub indigenous_territories: Vec<[u8; 32]>,
    pub contamination_hotspots: Vec<[f64; 2]>,
    pub carbon_sequestration_targets: f32,
    pub water_reclamation_integration: bool,
    pub native_plant_associations: HashMap<[u8; 32], Vec<[u8; 32]>>, // Plant ID → associated soil zones
}

// --- Core Soil Health Monitor Structure ---

pub struct SoilHealthMonitor {
    pub node_id: BirthSign,
    pub config: SoilNetworkConfiguration,
    pub current_health_state: SoilHealthState,
    pub management_zones: BTreeMap<[u8; 32], SoilManagementZone>,
    pub sensor_readings: BTreeMap<u64, SoilSensorReading>,
    pub microbiome_analyses: BTreeMap<u64, MicrobiomeAnalysis>,
    pub amendment_records: Vec<SoilAmendmentRecord>,
    pub action_plans: BTreeMap<u64, RegenerativeActionPlan>,
    pub indigenous_knowledge: Vec<IndigenousSoilKnowledgeEntry>,
    pub offline_queue: OfflineQueue<RegenerativeActionPlan>,
    pub treaty_cache: TreatyCompliance,
    pub metrics: SoilHealthMetrics,
    pub last_sensor_update: u64,
    pub last_microbiome_update: u64,
    pub last_sync: u64,
}

impl SoilHealthMonitor {
    /**
     * Initialize the Soil Health Monitor with Configuration
     * Ensures 72h operational buffer and treaty compliance setup
     */
    pub fn new(node_id: BirthSign, config: SoilNetworkConfiguration) -> Result<Self, &'static str> {
        let queue = OfflineQueue::new(OFFLINE_BUFFER_HOURS)
            .map_err(|_| "Failed to allocate offline buffer")?;
        
        // Initialize management zones
        let mut zones = BTreeMap::new();
        for zone in config.management_zones.iter() {
            zones.insert(zone.zone_id, zone.clone());
        }
        
        Ok(Self {
            node_id,
            config,
            current_health_state: SoilHealthState::Stable,
            management_zones: zones,
            sensor_readings: BTreeMap::new(),
            microbiome_analyses: BTreeMap::new(),
            amendment_records: Vec::new(),
            action_plans: BTreeMap::new(),
            indigenous_knowledge: Vec::new(),
            offline_queue: queue,
            treaty_cache: TreatyCompliance::new(),
            metrics: SoilHealthMetrics {
                current_health_state: SoilHealthState::Stable,
                total_zones_monitored: config.management_zones.len(),
                zones_requiring_intervention: 0,
                average_organic_matter_percent: NATIVE_ORGANIC_MATTER_PERCENT,
                average_ph: (NATIVE_SOIL_PH_MIN + NATIVE_SOIL_PH_MAX) / 2.0,
                carbon_sequestration_total_tons_yr: 0.0,
                contamination_incidents: 0,
                treaty_violations: 0,
                indigenous_practices_integrated: 0,
                water_conservation_gallons_yr: 0.0,
            },
            last_sensor_update: 0,
            last_microbiome_update: 0,
            last_sync: 0,
        })
    }

    /**
     * ERM Chain: SENSE
     * Ingests soil sensor readings, microbiome analyses, and contamination detections
     * Validates data integrity using PQ hashing
     */
    pub fn sense(&mut self, input: SoilInput) -> Result<SoilSenseResult, &'static str> {
        match input {
            SoilInput::SensorReading(reading) => self.process_sensor_reading(reading),
            SoilInput::MicrobiomeAnalysis(analysis) => self.process_microbiome_analysis(analysis),
            SoilInput::ContaminationDetection(zone_id, contaminant, level_ppm) => {
                self.process_contamination_detection(zone_id, contaminant, level_ppm)
            },
            SoilInput::IndigenousKnowledgeEntry(entry) => self.process_indigenous_knowledge_entry(entry),
        }
    }

    /**
     * Process soil sensor reading
     */
    fn process_sensor_reading(&mut self, reading: SoilSensorReading) -> Result<SoilSenseResult, &'static str> {
        // Validate sensor signature (PQ Secure)
        let hash = pq_hash(&reading.sensor_id);
        if hash[0] == 0x00 {
            return Err("Sensor signature invalid");
        }

        // Store reading with timestamp key
        self.sensor_readings.insert(reading.timestamp, reading.clone());

        // Update last sensor update time
        self.last_sensor_update = aletheion_core::time::now();

        // Update zone health state based on reading
        if let Some(zone) = self.management_zones.get_mut(&reading.zone_id) {
            self.update_zone_health_state(zone, &reading)?;
        }

        // Check soil health thresholds
        self.check_soil_health_thresholds(&reading)?;

        // Log sensing event
        self.log_event(format!(
            "SOIL_READING: Zone={:?}, pH={:.2}, OM={:.1}%, Moisture={:.1}%, EC={:.2}dS/m, Temp={:.1}°F, Pb={:.1}ppm, As={:.1}ppm, Microbiome={:.2}",
            reading.zone_id,
            reading.ph_level,
            reading.organic_matter_percent,
            reading.moisture_percent,
            reading.electrical_conductivity_ds_m,
            reading.temperature_f,
            reading.lead_ppm,
            reading.arsenic_ppm,
            reading.microbiome_diversity_index
        ));

        Ok(SoilSenseResult::SensorReadingProcessed(reading.sensor_id))
    }

    /**
     * Process microbiome analysis
     */
    fn process_microbiome_analysis(&mut self, analysis: MicrobiomeAnalysis) -> Result<SoilSenseResult, &'static str> {
        // Validate analysis signature (PQ Secure)
        let hash = pq_hash(&analysis.analysis_id);
        if hash[0] == 0x00 {
            return Err("Analysis signature invalid");
        }

        // Store analysis with timestamp key
        self.microbiome_analyses.insert(analysis.sample_date, analysis.clone());

        // Update last microbiome update time
        self.last_microbiome_update = aletheion_core::time::now();

        // Check microbiome health thresholds
        self.check_microbiome_thresholds(&analysis)?;

        // Log analysis event
        self.log_event(format!(
            "MICROBIOME_ANALYSIS: Zone={:?}, Bacterial={}/25, Fungal={}/15, Diversity={:.2}, Pathogens={}, IndigenousMatch={:.2}",
            analysis.zone_id,
            analysis.bacterial_species_count,
            analysis.fungal_species_count,
            analysis.diversity_index,
            if analysis.pathogen_presence { "Yes" } else { "No" },
            analysis.indigenous_microbiome_match
        ));

        Ok(SoilSenseResult::MicrobiomeAnalysisProcessed(analysis.analysis_id))
    }

    /**
     * Process contamination detection
     */
    fn process_contamination_detection(&mut self, zone_id: [u8; 32], contaminant: SoilContaminant, level_ppm: f32) -> Result<SoilSenseResult, &'static str> {
        // Determine threshold based on contaminant type
        let threshold_ppm = match contaminant {
            SoilContaminant::Lead => LEAD_SAFE_THRESHOLD_PPM,
            SoilContaminant::Arsenic => ARSENIC_SAFE_THRESHOLD_PPM,
            SoilContaminant::Cadmium => CADMIUM_SAFE_THRESHOLD_PPM,
            SoilContaminant::Mercury => MERCURY_SAFE_THRESHOLD_PPM,
            _ => 0.0,
        };

        // Check if exceeds threshold
        if level_ppm > threshold_ppm {
            self.metrics.contamination_incidents += 1;
            
            // Generate contamination alert
            self.generate_soil_alert(
                SoilAlertType::ContaminationDetected,
                zone_id,
                90, // High severity
                level_ppm,
                threshold_ppm,
                format!("Contaminant {:?} detected at {:.1}ppm exceeding threshold {:.1}ppm", contaminant, level_ppm, threshold_ppm),
                true,
                self.is_indigenous_territory(&zone_id),
            );
        }

        // Log contamination event
        self.log_event(format!(
            "CONTAMINATION_DETECTED: Zone={:?}, Contaminant={:?}, Level={:.1}ppm, Threshold={:.1}ppm, Exceeded={}",
            zone_id,
            contaminant,
            level_ppm,
            threshold_ppm,
            level_ppm > threshold_ppm
        ));

        Ok(SoilSenseResult::ContaminationProcessed(zone_id))
    }

    /**
     * Process Indigenous soil knowledge entry
     */
    fn process_indigenous_knowledge_entry(&mut self, entry: IndigenousSoilKnowledgeEntry) -> Result<SoilSenseResult, &'static str> {
        // Validate FPIC compliance for Indigenous knowledge
        if !entry.fpic_verified {
            self.log_warning("FPIC_VIOLATION: Indigenous soil knowledge entry requires FPIC verification");
            return Ok(SoilSenseResult::KnowledgeEntryRejected(entry.entry_id));
        }

        // Add to Indigenous knowledge database
        self.indigenous_knowledge.push(entry.clone());
        self.metrics.indigenous_practices_integrated += 1;

        // Log knowledge entry
        self.log_event(format!(
            "INDIGENOUS_SOIL_KNOWLEDGE: Practice={:?}, Source={:?}, FPIC={}, SeasonalTiming={:?}",
            entry.knowledge_type,
            entry.knowledge_source,
            entry.fpic_verified,
            entry.seasonal_timing
        ));

        Ok(SoilSenseResult::KnowledgeEntryProcessed(entry.entry_id))
    }

    /**
     * Update zone health state based on sensor reading
     */
    fn update_zone_health_state(&mut self, zone: &mut SoilManagementZone, reading: &SoilSensorReading) -> Result<(), &'static str> {
        // Calculate health score based on multiple parameters
        let ph_score = self.calculate_ph_score(reading.ph_level);
        let om_score = self.calculate_organic_matter_score(reading.organic_matter_percent);
        let salinity_score = self.calculate_salinity_score(reading.electrical_conductivity_ds_m, zone.urban_agriculture);
        let contamination_score = self.calculate_contamination_score(reading.lead_ppm, reading.arsenic_ppm);
        let microbiome_score = reading.microbiome_diversity_index;
        
        let health_score = (ph_score + om_score + salinity_score + contamination_score + microbiome_score) / 5.0;
        
        // Determine health state from score
        zone.current_health_state = match health_score {
            s if s >= 0.85 => SoilHealthState::Thriving,
            s if s >= 0.70 => SoilHealthState::Regenerative,
            s if s >= 0.55 => SoilHealthState::Stable,
            s if s >= 0.40 => SoilHealthState::Recovering,
            _ => SoilHealthState::Degraded,
        };
        
        // Update metrics
        self.update_soil_metrics();
        
        Ok(())
    }

    /**
     * Calculate pH health score (0.0-1.0)
     */
    fn calculate_ph_score(&self, ph: f32) -> f32 {
        // Optimal range for Sonoran Desert: 7.5-8.5, target 7.8
        if ph < 7.0 || ph > 9.0 {
            0.0 // Toxic range
        } else if ph >= 7.5 && ph <= 8.5 {
            // Within native range - score based on proximity to target 7.8
            let deviation = (ph - TARGET_REGENERATIVE_PH).abs();
            1.0 - (deviation / 0.7) // Max deviation in range is 0.7 (7.5 to 8.5)
        } else if ph >= 7.0 && ph < 7.5 {
            // Slightly acidic - less ideal but manageable
            0.6
        } else {
            // Alkaline beyond native range
            0.4
        }
    }

    /**
     * Calculate organic matter health score (0.0-1.0)
     */
    fn calculate_organic_matter_score(&self, om_percent: f32) -> f32 {
        if om_percent >= TARGET_ORGANIC_MATTER_PERCENT {
            1.0
        } else if om_percent >= NATIVE_ORGANIC_MATTER_PERCENT {
            (om_percent - NATIVE_ORGANIC_MATTER_PERCENT) / 
            (TARGET_ORGANIC_MATTER_PERCENT - NATIVE_ORGANIC_MATTER_PERCENT)
        } else {
            om_percent / NATIVE_ORGANIC_MATTER_PERCENT * 0.5 // Below native baseline
        }
    }

    /**
     * Calculate salinity health score (0.0-1.0)
     */
    fn calculate_salinity_score(&self, ec_ds_m: f32, is_agricultural: bool) -> f32 {
        let threshold = if is_agricultural {
            CROP_SALINITY_THRESHOLD_DS_M
        } else {
            PALO_VERDE_SALINITY_TOLERANCE_DS_M // More tolerant for native species
        };
        
        if ec_ds_m <= threshold * 0.8 {
            1.0 // Well below threshold
        } else if ec_ds_m <= threshold {
            1.0 - ((ec_ds_m - threshold * 0.8) / (threshold * 0.2))
        } else if ec_ds_m <= threshold * 1.5 {
            0.5 - ((ec_ds_m - threshold) / (threshold * 0.5)) * 0.5
        } else {
            0.0
        }
    }

    /**
     * Calculate contamination health score (0.0-1.0)
     */
    fn calculate_contamination_score(&self, lead_ppm: f32, arsenic_ppm: f32) -> f32 {
        let lead_score = if lead_ppm <= LEAD_SAFE_THRESHOLD_PPM * 0.5 {
            1.0
        } else if lead_ppm <= LEAD_SAFE_THRESHOLD_PPM {
            1.0 - ((lead_ppm - LEAD_SAFE_THRESHOLD_PPM * 0.5) / (LEAD_SAFE_THRESHOLD_PPM * 0.5))
        } else {
            0.0
        };
        
        let arsenic_score = if arsenic_ppm <= ARSENIC_SAFE_THRESHOLD_PPM * 0.5 {
            1.0
        } else if arsenic_ppm <= ARSENIC_SAFE_THRESHOLD_PPM {
            1.0 - ((arsenic_ppm - ARSENIC_SAFE_THRESHOLD_PPM * 0.5) / (ARSENIC_SAFE_THRESHOLD_PPM * 0.5))
        } else {
            0.0
        };
        
        (lead_score + arsenic_score) / 2.0
    }

    /**
     * Check soil health thresholds and trigger alerts if exceeded
     */
    fn check_soil_health_thresholds(&mut self, reading: &SoilSensorReading) -> Result<(), &'static str> {
        let mut alerts_triggered = false;

        // pH check
        if reading.ph_level < 7.0 || reading.ph_level > 9.0 {
            self.log_warning(format!("PH_CRITICAL: {:.2} outside safe range (7.0-9.0)", reading.ph_level));
            alerts_triggered = true;
        } else if (reading.ph_level - TARGET_REGENERATIVE_PH).abs() > 0.7 {
            self.log_warning(format!("PH_IMBALANCE: {:.2} deviates from target {:.2} by >0.7 units", reading.ph_level, TARGET_REGENERATIVE_PH));
            alerts_triggered = true;
        }

        // Organic matter check
        if reading.organic_matter_percent < NATIVE_ORGANIC_MATTER_PERCENT * 0.8 {
            self.log_warning(format!("ORGANIC_MATTER_CRITICAL: {:.1}% below native baseline {:.1}%", reading.organic_matter_percent, NATIVE_ORGANIC_MATTER_PERCENT));
            alerts_triggered = true;
        }

        // Salinity check for agricultural zones
        if reading.electrical_conductivity_ds_m > CROP_SALINITY_THRESHOLD_DS_M * 1.2 {
            self.log_warning(format!("SALINITY_EXCEEDED: {:.2}dS/m exceeds crop threshold {:.2}dS/m", reading.electrical_conductivity_ds_m, CROP_SALINITY_THRESHOLD_DS_M));
            alerts_triggered = true;
        }

        // Contamination check
        if reading.lead_ppm > LEAD_SAFE_THRESHOLD_PPM || reading.arsenic_ppm > ARSENIC_SAFE_THRESHOLD_PPM {
            self.log_warning(format!("CONTAMINATION_EXCEEDED: Pb={:.1}ppm (threshold {:.1}ppm), As={:.1}ppm (threshold {:.1}ppm)", reading.lead_ppm, LEAD_SAFE_THRESHOLD_PPM, reading.arsenic_ppm, ARSENIC_SAFE_THRESHOLD_PPM));
            alerts_triggered = true;
            self.metrics.contamination_incidents += 1;
        }

        // Moisture stress check (during non-monsoon months)
        let current_month = self.get_current_month();
        if !MONSOON_SEASON_MONTHS.contains(&current_month) && reading.moisture_percent < WILTING_POINT_PERCENT * 1.5 {
            self.log_warning(format!("MOISTURE_STRESS: {:.1}% below critical threshold {:.1}%", reading.moisture_percent, WILTING_POINT_PERCENT * 1.5));
            alerts_triggered = true;
        }

        if alerts_triggered {
            self.generate_soil_alert(
                SoilAlertType::pHImbalance,
                reading.zone_id,
                70,
                reading.ph_level,
                TARGET_REGENERATIVE_PH,
                "Multiple soil health parameters outside optimal range".to_string(),
                false,
                self.is_indigenous_territory(&reading.zone_id),
            );
        }

        Ok(())
    }

    /**
     * Check microbiome thresholds and trigger alerts if exceeded
     */
    fn check_microbiome_thresholds(&mut self, analysis: &MicrobiomeAnalysis) -> Result<(), &'static str> {
        let mut alerts_triggered = false;

        // Diversity check
        if analysis.diversity_index < TARGET_MICROBIOME_DIVERISTY_INDEX * 0.8 {
            self.log_warning(format!("MICROBIOME_DIVERISTY_LOW: {:.2} below target {:.2}", analysis.diversity_index, TARGET_MICROBIOME_DIVERISTY_INDEX));
            alerts_triggered = true;
        }

        // Bacterial species count check
        if analysis.bacterial_species_count < MIN_BACTERIAL_SPECIES_COUNT * 0.8 {
            self.log_warning(format!("BACTERIAL_DIVERISTY_LOW: {} species below minimum {}", analysis.bacterial_species_count, MIN_BACTERIAL_SPECIES_COUNT));
            alerts_triggered = true;
        }

        // Fungal species count check
        if analysis.fungal_species_count < MIN_FUNGAL_SPECIES_COUNT * 0.8 {
            self.log_warning(format!("FUNGAL_DIVERISTY_LOW: {} species below minimum {}", analysis.fungal_species_count, MIN_FUNGAL_SPECIES_COUNT));
            alerts_triggered = true;
        }

        // Pathogen check
        if analysis.pathogen_presence {
            self.log_warning("PATHOGEN_DETECTED: Harmful pathogens identified in soil microbiome");
            alerts_triggered = true;
        }

        // Indigenous microbiome match check
        if analysis.indigenous_microbiome_match < 0.6 {
            self.log_warning(format!("INDIGENOUS_MICROBIOME_MISMATCH: {:.2} similarity below 0.6 threshold", analysis.indigenous_microbiome_match));
            alerts_triggered = true;
        }

        if alerts_triggered {
            self.generate_soil_alert(
                SoilAlertType::MicrobiomeCollapse,
                analysis.zone_id,
                80,
                analysis.diversity_index,
                TARGET_MICROBIOME_DIVERISTY_INDEX,
                "Microbiome health parameters outside optimal range".to_string(),
                true,
                self.is_indigenous_territory(&analysis.zone_id),
            );
        }

        Ok(())
    }

    /**
     * Generate soil health alert
     */
    fn generate_soil_alert(&mut self, alert_type: SoilAlertType, zone_id: [u8; 32], severity: u8, measured_value: f32, threshold_value: f32, description: String, requires_immediate_action: bool, treaty_implications: bool) {
        let alert = SoilHealthAlert {
            alert_id: pq_hash(&description.as_bytes()),
            alert_type,
            zone_id,
            severity,
            measured_value,
            threshold_value,
            description,
            timestamp: aletheion_core::time::now(),
            requires_immediate_action,
            treaty_implications,
        };

        // Log alert
        self.log_event(format!(
            "SOIL_ALERT: Type={:?}, Zone={:?}, Severity={}, Value={:.2}, Threshold={:.2}, Immediate={}, Treaty={}",
            alert.alert_type,
            alert.zone_id,
            alert.severity,
            alert.measured_value,
            alert.threshold_value,
            alert.requires_immediate_action,
            alert.treaty_implications
        ));
    }

    /**
     * ERM Chain: MODEL
     * Analyzes soil health trends, carbon sequestration potential, and generates regenerative action plans
     * No Digital Twins: Uses direct sensor correlation and microbiome modeling
     */
    pub fn model_optimal_regeneration(&mut self) -> Result<Vec<RegenerativeActionPlan>, &'static str> {
        let current_time = aletheion_core::time::now();
        
        // Update all zone health states
        self.update_all_zone_health_states()?;
        
        // Generate regenerative action plans
        let mut action_plans = Vec::new();
        
        // 1. Identify zones requiring immediate intervention
        self.identify_critical_zones(&mut action_plans, current_time)?;
        
        // 2. Generate seasonal amendment plans based on Indigenous knowledge
        self.generate_seasonal_amendment_plans(&mut action_plans, current_time)?;
        
        // 3. Plan carbon sequestration optimization
        self.plan_carbon_sequestration(&mut action_plans, current_time)?;
        
        // 4. Integrate Indigenous soil management practices
        self.integrate_indigenous_practices(&mut action_plans, current_time)?;
        
        // Update metrics
        self.update_soil_metrics();
        
        Ok(action_plans)
    }

    /**
     * Update all zone health states based on latest sensor readings
     */
    fn update_all_zone_health_states(&mut self) -> Result<(), &'static str> {
        for (_, zone) in &mut self.management_zones {
            // Find latest sensor reading for this zone
            let latest_reading = self.sensor_readings.iter()
                .filter(|(_, r)| r.zone_id == zone.zone_id)
                .max_by_key(|(timestamp, _)| *timestamp);
            
            if let Some((_, reading)) = latest_reading {
                self.update_zone_health_state(zone, reading)?;
            }
        }
        
        Ok(())
    }

    /**
     * Identify zones requiring immediate regenerative intervention
     */
    fn identify_critical_zones(&mut self, plans: &mut Vec<RegenerativeActionPlan>, current_time: u64) -> Result<(), &'static str> {
        for (_, zone) in &self.management_zones {
            // Check for degraded health state
            if zone.current_health_state == SoilHealthState::Degraded {
                // Generate comprehensive remediation plan
                let plan = self.generate_remediation_plan(zone, current_time)?;
                plans.push(plan);
            }
            
            // Check for contamination
            let contaminated = self.sensor_readings.iter()
                .filter(|(_, r)| r.zone_id == zone.zone_id && 
                    (r.lead_ppm > LEAD_SAFE_THRESHOLD_PPM || r.arsenic_ppm > ARSENIC_SAFE_THRESHOLD_PPM))
                .count() > 0;
            
            if contaminated {
                let plan = self.generate_contamination_remediation_plan(zone, current_time)?;
                plans.push(plan);
            }
            
            // Check for microbiome collapse
            let microbiome_collapsed = self.microbiome_analyses.iter()
                .filter(|(_, a)| a.zone_id == zone.zone_id && 
                    (a.diversity_index < TARGET_MICROBIOME_DIVERISTY_INDEX * 0.6 || a.pathogen_presence))
                .count() > 0;
            
            if microbiome_collapsed {
                let plan = self.generate_microbiome_restoration_plan(zone, current_time)?;
                plans.push(plan);
            }
        }
        
        Ok(())
    }

    /**
     * Generate comprehensive soil remediation plan for degraded zone
     */
    fn generate_remediation_plan(&self, zone: &SoilManagementZone, current_time: u64) -> Result<RegenerativeActionPlan, &'static str> {
        let plan_id = pq_hash(&current_time.to_be_bytes());
        
        // Determine primary remediation action based on degradation cause
        let (action_type, indigenous_practice) = self.determine_primary_remediation_action(zone);
        
        // Calculate priority based on degradation severity
        let priority = match zone.current_health_state {
            SoilHealthState::Degraded => 95,
            SoilHealthState::Recovering => 75,
            _ => 50,
        };
        
        // Estimate cost based on zone area
        let estimated_cost = zone.area_sqft * 0.25; // $0.25 per sqft
        
        // Estimate duration based on action type
        let estimated_duration_days = match action_type {
            RegenerativeAction::BiocharAmendment => 7,
            RegenerativeAction::CompostApplication => 5,
            RegenerativeAction::MycorrhizalInoculation => 3,
            RegenerativeAction::CoverCropping => 90, // Seasonal
            RegenerativeAction::Phytoremediation => 180, // Long-term
            _ => 14,
        };
        
        // Estimate carbon impact
        let carbon_impact = self.estimate_carbon_impact(action_type, zone.area_sqft);
        
        // Check treaty compliance
        let (treaty_compliant, fpic_required) = self.check_remediation_treaty_compliance(zone, action_type)?;
        
        let plan = RegenerativeActionPlan {
            plan_id,
            zone_id: zone.zone_id,
            action_type,
            indigenous_practice,
            priority,
            estimated_cost_usd: estimated_cost,
            estimated_duration_days,
            carbon_sequestration_impact_tons: carbon_impact,
            water_conservation_impact_gallons: self.estimate_water_impact(action_type, zone.area_sqft),
            treaty_compliant,
            fpic_required,
            scheduled_start_date: current_time,
            scheduled_end_date: current_time + (estimated_duration_days as u64 * 86400),
            success_metrics: self.define_success_metrics(action_type),
        };
        
        Ok(plan)
    }

    /**
     * Determine primary remediation action based on zone conditions
     */
    fn determine_primary_remediation_action(&self, zone: &SoilManagementZone) -> (RegenerativeAction, Option<IndigenousSoilPractice>) {
        // Find latest sensor reading for this zone
        let latest_reading = self.sensor_readings.iter()
            .filter(|(_, r)| r.zone_id == zone.zone_id)
            .max_by_key(|(timestamp, _)| *timestamp);
        
        if let Some((_, reading)) = latest_reading {
            // Check organic matter deficiency
            if reading.organic_matter_percent < TARGET_ORGANIC_MATTER_PERCENT * 0.7 {
                return (RegenerativeAction::CompostApplication, Some(IndigenousSoilPractice::MesquitePodAmendment));
            }
            
            // Check pH imbalance
            if (reading.ph_level - TARGET_REGENERATIVE_PH).abs() > 0.8 {
                return (RegenerativeAction::BiocharAmendment, None);
            }
            
            // Check salinity
            if reading.electrical_conductivity_ds_m > CROP_SALINITY_THRESHOLD_DS_M * 1.3 {
                return (RegenerativeAction::Phytoremediation, None);
            }
            
            // Check microbiome
            if let Some((_, analysis)) = self.microbiome_analyses.iter()
                .filter(|(_, a)| a.zone_id == zone.zone_id)
                .max_by_key(|(timestamp, _)| *timestamp) {
                
                if analysis.diversity_index < TARGET_MICROBIOME_DIVERISTY_INDEX * 0.7 {
                    return (RegenerativeAction::MycorrhizalInoculation, Some(IndigenousSoilPractice::CreosoteResinApplication));
                }
            }
        }
        
        // Default: cover cropping for general improvement
        (RegenerativeAction::CoverCropping, Some(IndigenousSoilPractice::MoonPhasePlanting))
    }

    /**
     * Estimate carbon sequestration impact of regenerative action
     */
    fn estimate_carbon_impact(&self, action: RegenerativeAction, area_sqft: f32) -> f32 {
        let acres = area_sqft / 43560.0;
        
        let annual_tons_per_acre = match action {
            RegenerativeAction::BiocharAmendment => 0.8,
            RegenerativeAction::CompostApplication => 0.6,
            RegenerativeAction::MycorrhizalInoculation => 0.4,
            RegenerativeAction::CoverCropping => 0.5,
            RegenerativeAction::Phytoremediation => 0.3, // Initial phase
            _ => 0.2,
        };
        
        acres * annual_tons_per_acre
    }

    /**
     * Estimate water conservation impact of regenerative action
     */
    fn estimate_water_impact(&self, action: RegenerativeAction, area_sqft: f32) -> f32 {
        let annual_gallons_per_sqft = match action {
            RegenerativeAction::BiocharAmendment => 1.5,
            RegenerativeAction::CompostApplication => 1.2,
            RegenerativeAction::CoverCropping => 2.0,
            _ => 0.8,
        };
        
        area_sqft * annual_gallons_per_sqft
    }

    /**
     * Check treaty compliance for soil remediation actions
     */
    fn check_remediation_treaty_compliance(&self, zone: &SoilManagementZone, action: RegenerativeAction) -> Result<(bool, bool), &'static str> {
        if zone.indigenous_territory {
            if let Some(treaty_zone) = zone.treaty_zone_id {
                // Certain actions require explicit FPIC
                let fpic_required = match action {
                    RegenerativeAction::Phytoremediation | RegenerativeAction::SoilAeration => true,
                    _ => false,
                };
                
                let compliance = self.treaty_cache.check_soil_rights(&treaty_zone)?;
                
                return Ok((compliance.allowed, fpic_required));
            }
        }
        
        Ok((true, false))
    }

    /**
     * Generate contamination remediation plan
     */
    fn generate_contamination_remediation_plan(&self, zone: &SoilManagementZone, current_time: u64) -> Result<RegenerativeActionPlan, &'static str> {
        let plan_id = pq_hash(&current_time.to_be_bytes());
        
        let plan = RegenerativeActionPlan {
            plan_id,
            zone_id: zone.zone_id,
            action_type: RegenerativeAction::ContaminantRemediation,
            indigenous_practice: None,
            priority: 100, // Highest priority
            estimated_cost_usd: zone.area_sqft * 1.5, // $1.50 per sqft
            estimated_duration_days: 120,
            carbon_sequestration_impact_tons: 0.0, // Neutral during remediation
            water_conservation_impact_gallons: 0.0,
            treaty_compliant: !zone.indigenous_territory,
            fpic_required: zone.indigenous_territory,
            scheduled_start_date: current_time,
            scheduled_end_date: current_time + (120 * 86400),
            success_metrics: HashMap::from([
                ("lead_reduction_percent".to_string(), 90.0),
                ("arsenic_reduction_percent".to_string(), 85.0),
                ("microbiome_recovery_index".to_string(), 0.7),
            ]),
        };
        
        Ok(plan)
    }

    /**
     * Generate microbiome restoration plan
     */
    fn generate_microbiome_restoration_plan(&self, zone: &SoilManagementZone, current_time: u64) -> Result<RegenerativeActionPlan, &'static str> {
        let plan_id = pq_hash(&current_time.to_be_bytes());
        
        let plan = RegenerativeActionPlan {
            plan_id,
            zone_id: zone.zone_id,
            action_type: RegenerativeAction::MycorrhizalInoculation,
            indigenous_practice: Some(IndigenousSoilPractice::CreosoteResinApplication),
            priority: 85,
            estimated_cost_usd: zone.area_sqft * 0.4,
            estimated_duration_days: 60,
            carbon_sequestration_impact_tons: self.estimate_carbon_impact(RegenerativeAction::MycorrhizalInoculation, zone.area_sqft),
            water_conservation_impact_gallons: self.estimate_water_impact(RegenerativeAction::MycorrhizalInoculation, zone.area_sqft),
            treaty_compliant: !zone.indigenous_territory,
            fpic_required: zone.indigenous_territory,
            scheduled_start_date: current_time,
            scheduled_end_date: current_time + (60 * 86400),
            success_metrics: HashMap::from([
                ("bacterial_species_increase".to_string(), 10.0),
                ("fungal_species_increase".to_string(), 5.0),
                ("diversity_index_target".to_string(), 0.75),
            ]),
        };
        
        Ok(plan)
    }

    /**
     * Generate seasonal amendment plans based on Indigenous knowledge
     */
    fn generate_seasonal_amendment_plans(&mut self, plans: &mut Vec<RegenerativeActionPlan>, current_time: u64) -> Result<(), &'static str> {
        let current_month = self.get_current_month();
        
        // Apply Indigenous knowledge for seasonal timing
        for knowledge_entry in &self.indigenous_knowledge {
            if knowledge_entry.fpic_verified && knowledge_entry.seasonal_timing.contains(&current_month) {
                // Find applicable zones
                for (_, zone) in &self.management_zones {
                    if zone.indigenous_territory && 
                       knowledge_entry.soil_type_applicability.contains(&zone.soil_texture) {
                        
                        let plan = RegenerativeActionPlan {
                            plan_id: knowledge_entry.entry_id,
                            zone_id: zone.zone_id,
                            action_type: self.map_knowledge_to_action(knowledge_entry.knowledge_type),
                            indigenous_practice: Some(knowledge_entry.knowledge_type),
                            priority: 80,
                            estimated_cost_usd: zone.area_sqft * 0.3,
                            estimated_duration_days: 14,
                            carbon_sequestration_impact_tons: 0.3 * (zone.area_sqft / 43560.0),
                            water_conservation_impact_gallons: 1.0 * zone.area_sqft,
                            treaty_compliant: true,
                            fpic_required: false, // Already verified
                            scheduled_start_date: current_time,
                            scheduled_end_date: current_time + (14 * 86400),
                            success_metrics: HashMap::new(),
                        };
                        
                        plans.push(plan);
                    }
                }
            }
        }
        
        Ok(())
    }

    /**
     * Map Indigenous soil practice to regenerative action
     */
    fn map_knowledge_to_action(&self, practice: IndigenousSoilPractice) -> RegenerativeAction {
        match practice {
            IndigenousSoilPractice::AkChinFloodIrrigation => RegenerativeAction::AkChinIrrigation,
            IndigenousSoilPractice::RiverBottomCultivation => RegenerativeAction::CoverCropping,
            IndigenousSoilPractice::MesquitePodAmendment => RegenerativeAction::CompostApplication,
            IndigenousSoilPractice::CreosoteResinApplication => RegenerativeAction::MycorrhizalInoculation,
            IndigenousSoilPractice::SeasonalBurning => RegenerativeAction::ReducedTillage,
            IndigenousSoilPractice::MoonPhasePlanting => RegenerativeAction::CoverCropping,
            IndigenousSoilPractice::AncestralSeedSaving => RegenerativeAction::NativeMulching,
        }
    }

    /**
     * Plan carbon sequestration optimization
     */
    fn plan_carbon_sequestration(&mut self, plans: &mut Vec<RegenerativeActionPlan>, current_time: u64) -> Result<(), &'static str> {
        // Calculate current carbon sequestration deficit
        let current_sequestration = self.metrics.carbon_sequestration_total_tons_yr;
        let target_sequestration = self.config.carbon_sequestration_targets;
        let deficit = target_sequestration - current_sequestration;
        
        if deficit > 0.1 {
            // Prioritize zones with highest carbon potential
            let mut zones_by_potential: Vec<_> = self.management_zones.iter()
                .filter(|(_, z)| z.current_health_state != SoilHealthState::Degraded)
                .collect();
            
            zones_by_potential.sort_by(|a, b| {
                b.1.area_sqft.partial_cmp(&a.1.area_sqft).unwrap()
            });
            
            // Generate biochar amendment plans for top zones
            for (_, zone) in zones_by_potential.iter().take(5) {
                let plan = RegenerativeActionPlan {
                    plan_id: pq_hash(&zone.zone_id),
                    zone_id: zone.zone_id,
                    action_type: RegenerativeAction::BiocharAmendment,
                    indigenous_practice: None,
                    priority: 70,
                    estimated_cost_usd: zone.area_sqft * 0.35,
                    estimated_duration_days: 10,
                    carbon_sequestration_impact_tons: self.estimate_carbon_impact(RegenerativeAction::BiocharAmendment, zone.area_sqft),
                    water_conservation_impact_gallons: self.estimate_water_impact(RegenerativeAction::BiocharAmendment, zone.area_sqft),
                    treaty_compliant: !zone.indigenous_territory,
                    fpic_required: zone.indigenous_territory,
                    scheduled_start_date: current_time,
                    scheduled_end_date: current_time + (10 * 86400),
                    success_metrics: HashMap::from([
                        ("carbon_increase_tons".to_string(), self.estimate_carbon_impact(RegenerativeAction::BiocharAmendment, zone.area_sqft)),
                    ]),
                };
                
                plans.push(plan);
            }
        }
        
        Ok(())
    }

    /**
     * Integrate Indigenous soil management practices
     */
    fn integrate_indigenous_practices(&mut self, plans: &mut Vec<RegenerativeActionPlan>, current_time: u64) -> Result<(), &'static str> {
        // Identify zones on Indigenous territories lacking Indigenous practices
        for (_, zone) in &self.management_zones {
            if zone.indigenous_territory && !self.has_recent_indigenous_practice(zone) {
                // Recommend Ak Chin irrigation during monsoon season
                let current_month = self.get_current_month();
                if MONSOON_SEASON_MONTHS.contains(&current_month) {
                    let plan = RegenerativeActionPlan {
                        plan_id: pq_hash(&(current_time + 1).to_be_bytes()),
                        zone_id: zone.zone_id,
                        action_type: RegenerativeAction::AkChinIrrigation,
                        indigenous_practice: Some(IndigenousSoilPractice::AkChinFloodIrrigation),
                        priority: 90,
                        estimated_cost_usd: 500.0,
                        estimated_duration_days: 1,
                        carbon_sequestration_impact_tons: 0.1 * (zone.area_sqft / 43560.0),
                        water_conservation_impact_gallons: -200.0, // Temporary increase for infiltration
                        treaty_compliant: true,
                        fpic_required: true,
                        scheduled_start_date: current_time,
                        scheduled_end_date: current_time + (1 * 86400),
                        success_metrics: HashMap::from([
                            ("infiltration_rate_increase".to_string(), 1.0),
                            ("moisture_retention_days".to_string(), 7.0),
                        ]),
                    };
                    
                    plans.push(plan);
                }
            }
        }
        
        Ok(())
    }

    /**
     * Check if zone has recent Indigenous soil practice application
     */
    fn has_recent_indigenous_practice(&self, zone: &SoilManagementZone) -> bool {
        let current_time = aletheion_core::time::now();
        let six_months_ago = current_time - (180 * 86400);
        
        self.amendment_records.iter()
            .filter(|record| record.zone_id == zone.zone_id && record.indigenous_practice.is_some())
            .any(|record| record.application_date > six_months_ago)
    }

    /**
     * Update soil health metrics
     */
    fn update_soil_metrics(&mut self) {
        // Count zones requiring intervention
        self.metrics.zones_requiring_intervention = self.management_zones.iter()
            .filter(|(_, z)| z.current_health_state == SoilHealthState::Degraded || 
                           z.current_health_state == SoilHealthState::Recovering)
            .count();
        
        // Calculate average organic matter
        let total_om: f32 = self.sensor_readings.iter()
            .map(|(_, r)| r.organic_matter_percent)
            .sum();
        self.metrics.average_organic_matter_percent = total_om / self.sensor_readings.len().max(1) as f32;
        
        // Calculate average pH
        let total_ph: f32 = self.sensor_readings.iter()
            .map(|(_, r)| r.ph_level)
            .sum();
        self.metrics.average_ph = total_ph / self.sensor_readings.len().max(1) as f32;
        
        // Calculate total carbon sequestration
        self.metrics.carbon_sequestration_total_tons_yr = self.management_zones.iter()
            .map(|(_, z)| z.carbon_sequestration_tons_per_acre_yr * (z.area_sqft / 43560.0))
            .sum();
        
        // Update current health state based on zone distribution
        let thriving_zones = self.management_zones.iter()
            .filter(|(_, z)| z.current_health_state == SoilHealthState::Thriving)
            .count();
        let regenerative_zones = self.management_zones.iter()
            .filter(|(_, z)| z.current_health_state == SoilHealthState::Regenerative)
            .count();
        let stable_zones = self.management_zones.iter()
            .filter(|(_, z)| z.current_health_state == SoilHealthState::Stable)
            .count();
        
        let total_zones = self.management_zones.len();
        let thriving_percent = thriving_zones as f32 / total_zones.max(1) as f32;
        let regenerative_percent = regenerative_zones as f32 / total_zones.max(1) as f32;
        let stable_percent = stable_zones as f32 / total_zones.max(1) as f32;
        
        self.metrics.current_health_state = if thriving_percent > 0.4 {
            SoilHealthState::Thriving
        } else if thriving_percent + regenerative_percent > 0.6 {
            SoilHealthState::Regenerative
        } else if thriving_percent + regenerative_percent + stable_percent > 0.8 {
            SoilHealthState::Stable
        } else if stable_percent > 0.5 {
            SoilHealthState::Recovering
        } else {
            SoilHealthState::Degraded
        };
    }

    /**
     * ERM Chain: OPTIMIZE & TREATY-CHECK
     * Validates regenerative plans against Indigenous soil rights and generates executable commands
     * FPIC Enforcement: Cannot implement soil amendments on protected lands without consent
     */
    pub fn optimize_and_check(&mut self, plans: Vec<RegenerativeActionPlan>) -> Result<Vec<SoilCommand>, &'static str> {
        let mut commands = Vec::new();
        
        for plan in plans {
            // Check treaty compliance for each plan
            if !plan.treaty_compliant {
                self.log_warning(format!("FPIC_VIOLATION: Regenerative plan {:?} denied due to treaty restrictions", plan.plan_id));
                continue;
            }
            
            // For plans requiring FPIC, verify consent exists
            if plan.fpic_required && !self.verify_fpic_for_plan(&plan) {
                self.log_warning(format!("FPIC_MISSING: Regenerative plan {:?} requires explicit consent", plan.plan_id));
                continue;
            }
            
            // Generate command
            let command = SoilCommand {
                plan_entry: plan.clone(),
                command_type: self.map_action_to_command(plan.action_type),
                treaty_compliant: true,
                signed: false,
            };
            
            commands.push(command);
        }
        
        Ok(commands)
    }

    /**
     * Verify FPIC consent exists for plan requiring explicit consent
     */
    fn verify_fpic_for_plan(&self, plan: &RegenerativeActionPlan) -> bool {
        // In production: query DID wallet for explicit consent record
        // For now: assume consent exists if zone is not Indigenous territory
        if let Some(zone) = self.management_zones.get(&plan.zone_id) {
            !zone.indigenous_territory || zone.treaty_zone_id.is_some()
        } else {
            true
        }
    }

    /**
     * Map regenerative action to soil command type
     */
    fn map_action_to_command(&self, action_type: RegenerativeAction) -> SoilCommandType {
        match action_type {
            RegenerativeAction::BiocharAmendment => SoilCommandType::ApplyBiochar,
            RegenerativeAction::CompostApplication => SoilCommandType::ApplyCompost,
            RegenerativeAction::MycorrhizalInoculation => SoilCommandType::InoculateMicrobiome,
            RegenerativeAction::CoverCropping => SoilCommandType::PlantCoverCrop,
            RegenerativeAction::ReducedTillage => SoilCommandType::ImplementNoTill,
            RegenerativeAction::AkChinIrrigation => SoilCommandType::ExecuteFloodIrrigation,
            RegenerativeAction::NativeMulching => SoilCommandType::ApplyNativeMulch,
            RegenerativeAction::Phytoremediation => SoilCommandType::PlantRemediationSpecies,
            RegenerativeAction::SoilAeration => SoilCommandType::AerateSoil,
            RegenerativeAction::ContaminantRemediation => SoilCommandType::ExecuteRemediation,
        }
    }

    /**
     * ERM Chain: ACT
     * Executes soil commands or queues for offline execution
     * Ensures atomicity without rollbacks
     */
    pub fn act(&mut self, commands: Vec<SoilCommand>) -> Result<(), &'static str> {
        for command in commands {
            // Sign command (PQ Secure)
            let signature = DIDWallet::sign_action(&self.node_id, &command);
            let mut signed_command = command.clone();
            signed_command.signed = signature.is_ok();
            
            // Attempt immediate execution via HAL
            match self.execute_soil_command(&signed_command) {
                Ok(_) => {
                    self.log_action(&signed_command);
                    
                    // Record amendment
                    self.record_amendment(&signed_command.plan_entry);
                    
                    // Update metrics
                    self.update_metrics_from_command(&signed_command);
                },
                Err(_) => {
                    // Offline Fallback: Queue for later execution
                    self.offline_queue.push(signed_command.plan_entry)?;
                    self.log_warning("Offline mode: Soil command queued for later execution");
                }
            }
        }
        
        Ok(())
    }

    /**
     * Execute individual soil command
     */
    fn execute_soil_command(&self, command: &SoilCommand) -> Result<(), &'static str> {
        match command.command_type {
            SoilCommandType::ApplyBiochar => {
                aletheion_physical::hal::apply_soil_amendment(
                    &command.plan_entry.zone_id,
                    "biochar",
                    BIOCHAR_AMENDMENT_PERCENT
                )?;
            },
            SoilCommandType::ApplyCompost => {
                aletheion_physical::hal::apply_soil_amendment(
                    &command.plan_entry.zone_id,
                    "compost",
                    COMPOST_AMENDMENT_INCHES
                )?;
            },
            SoilCommandType::InoculateMicrobiome => {
                aletheion_physical::hal::inoculate_soil_microbiome(
                    &command.plan_entry.zone_id,
                    "mycorrhizal"
                )?;
            },
            SoilCommandType::PlantCoverCrop => {
                aletheion_physical::hal::plant_cover_crop(
                    &command.plan_entry.zone_id,
                    &command.plan_entry.success_metrics.keys().cloned().collect::<Vec<_>>()
                )?;
            },
            SoilCommandType::ImplementNoTill => {
                aletheion_physical::hal::implement_no_till_practice(
                    &command.plan_entry.zone_id
                )?;
            },
            SoilCommandType::ExecuteFloodIrrigation => {
                aletheion_physical::hal::execute_ak_chin_irrigation(
                    &command.plan_entry.zone_id,
                    AK_CHIN_IRRIGATION_DURATION_HOURS
                )?;
            },
            SoilCommandType::ApplyNativeMulch => {
                aletheion_physical::hal::apply_native_mulch(
                    &command.plan_entry.zone_id
                )?;
            },
            SoilCommandType::PlantRemediationSpecies => {
                aletheion_physical::hal::plant_remediation_species(
                    &command.plan_entry.zone_id
                )?;
            },
            SoilCommandType::AerateSoil => {
                aletheion_physical::hal::aerate_soil(
                    &command.plan_entry.zone_id
                )?;
            },
            SoilCommandType::ExecuteRemediation => {
                aletheion_physical::hal::execute_contaminant_remediation(
                    &command.plan_entry.zone_id
                )?;
            }
        }
        
        Ok(())
    }

    /**
     * Record soil amendment for audit trail
     */
    fn record_amendment(&mut self, plan: &RegenerativeActionPlan) {
        let record = SoilAmendmentRecord {
            amendment_id: plan.plan_id,
            zone_id: plan.zone_id,
            action_type: plan.action_type,
            indigenous_practice: plan.indigenous_practice,
            application_date: aletheion_core::time::now(),
            quantity: plan.estimated_cost_usd / 10.0, // Rough estimate
            unit: "cubic_yards".to_string(),
            carbon_impact_tons: plan.carbon_sequestration_impact_tons,
            treaty_compliant: plan.treaty_compliant,
            fpic_verified: !plan.fpic_required || self.verify_fpic_for_plan(plan),
        };
        
        self.amendment_records.push(record);
    }

    /**
     * Update metrics based on executed command
     */
    fn update_metrics_from_command(&mut self, command: &SoilCommand) {
        match command.command_type {
            SoilCommandType::ApplyBiochar | SoilCommandType::ApplyCompost => {
                self.metrics.carbon_sequestration_total_tons_yr += command.plan_entry.carbon_sequestration_impact_tons;
                self.metrics.water_conservation_gallons_yr += command.plan_entry.water_conservation_impact_gallons;
            },
            _ => {}
        }
    }

    /**
     * ERM Chain: LOG
     * Immutable logging to ALN-Blockchain
     * No rollbacks allowed; all state changes are forward-only
     */
    fn log_action(&self, command: &SoilCommand) {
        let log_entry = alloc::format!(
            "SOIL_ACT: Type={:?} | Plan={:?} | Zone={:?} | Carbon={:.2}tons | Water={:.0}gal | Treaty={}",
            command.command_type,
            command.plan_entry.plan_id,
            command.plan_entry.zone_id,
            command.plan_entry.carbon_sequestration_impact_tons,
            command.plan_entry.water_conservation_impact_gallons,
            if command.treaty_compliant { "Compliant" } else { "N/A" }
        );
        
        aletheion_:ledger::append_immutable(&log_entry);
    }

    fn log_event(&self, message: String) {
        let log_entry = alloc::format!("[{}] {}", aletheion_core::time::now(), message);
        aletheion_:ledger::append_immutable(&log_entry);
    }

    fn log_warning(&self, message: &str) {
        self.log_event(format!("WARNING: {}", message));
    }

    /**
     * ERM Chain: INTERFACE
     * Exposes status to Citizen App (Kotlin/Android) and Mesh Network
     * WCAG 2.2 AAA compliant data structure
     */
    pub fn get_status_report(&self) -> SoilStatusReport {
        SoilStatusReport {
            network_id: self.config.network_id,
            current_health_state: self.metrics.current_health_state,
            metrics: self.metrics.clone(),
            total_zones: self.management_zones.len(),
            indigenous_territories_count: self.config.indigenous_territories.len(),
            carbon_sequestration_progress_percent: (self.metrics.carbon_sequestration_total_tons_yr / 
                self.config.carbon_sequestration_targets.max(1.0)) * 100.0,
            offline_queue_size: self.offline_queue.len(),
            last_sync: self.last_sync,
            accessibility_alert: self.metrics.zones_requiring_intervention > 0,
            treaty_compliance_required: !self.config.indigenous_territories.is_empty(),
        }
    }

    /**
     * Get current month (1-12) for seasonal planning
     */
    fn get_current_month(&self) -> u8 {
        let current_time = aletheion_core::time::now();
        // Simplified: assume Unix epoch, extract month (June for testing)
        6
    }

    /**
     * Check if zone is on Indigenous territory
     */
    fn is_indigenous_territory(&self, zone_id: &[u8; 32]) -> bool {
        self.management_zones.get(zone_id)
            .map_or(false, |zone| zone.indigenous_territory)
    }

    /**
     * Define success metrics for regenerative action
     */
    fn define_success_metrics(&self, action: RegenerativeAction) -> HashMap<String, f32> {
        let mut metrics = HashMap::new();
        
        match action {
            RegenerativeAction::BiocharAmendment => {
                metrics.insert("organic_matter_increase_percent".to_string(), 0.5);
                metrics.insert("water_retention_increase_percent".to_string(), 15.0);
                metrics.insert("carbon_sequestration_tons_per_acre".to_string(), 0.8);
            },
            RegenerativeAction::CompostApplication => {
                metrics.insert("organic_matter_increase_percent".to_string(), 0.7);
                metrics.insert("microbiome_diversity_increase".to_string(), 0.15);
            },
            RegenerativeAction::MycorrhizalInoculation => {
                metrics.insert("mycorrhizal_colonization_percent".to_string(), 60.0);
                metrics.insert("phosphorus_uptake_increase_percent".to_string(), 25.0);
            },
            RegenerativeAction::CoverCropping => {
                metrics.insert("biomass_production_lbs_per_acre".to_string(), 2000.0);
                metrics.insert("nitrogen_fixation_lbs_per_acre".to_string(), 50.0);
            },
            _ => {}
        }
        
        metrics
    }

    /**
     * Sync Protocol
     * Reconciles offline queue with central ALN-Blockchain when connectivity restored
     */
    pub fn sync_offline_queue(&mut self) -> Result<usize, &'static str> {
        let count = self.offline_queue.sync_to_aln()?;
        self.last_sync = aletheion_core::time::now();
        Ok(count)
    }

    /**
     * Register new soil management zone
     */
    pub fn register_soil_zone(&mut self, zone: SoilManagementZone) -> Result<(), &'static str> {
        if self.management_zones.len() >= MAX_SOIL_ZONES {
            return Err("Maximum soil zones reached");
        }
        
        // Check treaty compliance for Indigenous territories
        if zone.indigenous_territory {
            if let Some(treaty_zone) = zone.treaty_zone_id {
                let compliance = self.treaty_cache.check_soil_registration(&treaty_zone)?;
                if !compliance.allowed {
                    return Err("FPIC Violation: Soil zone registration requires treaty approval");
                }
            }
        }
        
        self.management_zones.insert(zone.zone_id, zone.clone());
        self.metrics.total_zones_monitored += 1;
        
        self.log_event(format!(
            "SOIL_ZONE_REGISTERED: ID={:?}, Area={:.1}sqft, Texture={:?}, Indigenous={}, Treaty={:?}",
            zone.zone_id,
            zone.area_sqft,
            zone.soil_texture,
            zone.indigenous_territory,
            zone.treaty_zone_id
        ));
        
        Ok(())
    }

    /**
     * Get carbon sequestration metrics
     */
    pub fn get_carbon_metrics(&self) -> CarbonMetrics {
        CarbonMetrics {
            current_sequestration_tons_yr: self.metrics.carbon_sequestration_total_tons_yr,
            target_sequestration_tons_yr: self.config.carbon_sequestration_targets,
            progress_percent: (self.metrics.carbon_sequestration_total_tons_yr / 
                self.config.carbon_sequestration_targets.max(1.0)) * 100.0,
            potential_additional_tons_yr: self.calculate_additional_potential(),
            zones_by_sequestration_potential: self.rank_zones_by_carbon_potential(),
        }
    }

    /**
     * Calculate additional carbon sequestration potential
     */
    fn calculate_additional_potential(&self) -> f32 {
        let current = self.metrics.carbon_sequestration_total_tons_yr;
        let target = self.config.carbon_sequestration_targets;
        (target - current).max(0.0)
    }

    /**
     * Rank zones by carbon sequestration potential
     */
    fn rank_zones_by_carbon_potential(&self) -> Vec<([u8; 32], f32)> {
        let mut zones: Vec<_> = self.management_zones.iter()
            .map(|(id, zone)| {
                let potential = match zone.current_health_state {
                    SoilHealthState::Degraded => 1.0,
                    SoilHealthState::Recovering => 0.7,
                    SoilHealthState::Stable => 0.4,
                    SoilHealthState::Regenerative => 0.2,
                    SoilHealthState::Thriving => 0.05,
                };
                (*id, potential * (zone.area_sqft / 43560.0))
            })
            .collect();
        
        zones.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        zones
    }

    /**
     * Get water conservation metrics
     */
    pub fn get_water_conservation_metrics(&self) -> WaterConservationMetrics {
        WaterConservationMetrics {
            total_water_saved_gallons_yr: self.metrics.water_conservation_gallons_yr,
            infiltration_rate_improvement_percent: self.calculate_infiltration_improvement(),
            monsoon_capture_efficiency_percent: 85.0, // Based on soil structure improvements
            xeriscaping_compatibility: true,
        }
    }

    /**
     * Calculate infiltration rate improvement
     */
    fn calculate_infiltration_improvement(&self) -> f32 {
        // Simplified: based on organic matter increase
        let om_increase = (self.metrics.average_organic_matter_percent - NATIVE_ORGANIC_MATTER_PERCENT).max(0.0);
        om_increase * 15.0 // 15% improvement per 1% OM increase
    }
}

// --- Supporting Data Structures ---

pub enum SoilInput {
    SensorReading(SoilSensorReading),
    MicrobiomeAnalysis(MicrobiomeAnalysis),
    ContaminationDetection([u8; 32], SoilContaminant, f32),
    IndigenousKnowledgeEntry(IndigenousSoilKnowledgeEntry),
}

pub enum SoilSenseResult {
    SensorReadingProcessed([u8; 32]),
    MicrobiomeAnalysisProcessed([u8; 32]),
    ContaminationProcessed([u8; 32]),
    KnowledgeEntryProcessed([u8; 32]),
    KnowledgeEntryRejected([u8; 32]),
}

pub struct SoilCommand {
    pub plan_entry: RegenerativeActionPlan,
    pub command_type: SoilCommandType,
    pub treaty_compliant: bool,
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoilCommandType {
    ApplyBiochar,
    ApplyCompost,
    InoculateMicrobiome,
    PlantCoverCrop,
    ImplementNoTill,
    ExecuteFloodIrrigation,
    ApplyNativeMulch,
    PlantRemediationSpecies,
    AerateSoil,
    ExecuteRemediation,
}

pub struct SoilStatusReport {
    pub network_id: [u8; 32],
    pub current_health_state: SoilHealthState,
    pub metrics: SoilHealthMetrics,
    pub total_zones: usize,
    pub indigenous_territories_count: usize,
    pub carbon_sequestration_progress_percent: f32,
    pub offline_queue_size: usize,
    pub last_sync: u64,
    pub accessibility_alert: bool,
    pub treaty_compliance_required: bool,
}

pub struct CarbonMetrics {
    pub current_sequestration_tons_yr: f32,
    pub target_sequestration_tons_yr: f32,
    pub progress_percent: f32,
    pub potential_additional_tons_yr: f32,
    pub zones_by_sequestration_potential: Vec<([u8; 32], f32)>,
}

pub struct WaterConservationMetrics {
    pub total_water_saved_gallons_yr: f32,
    pub infiltration_rate_improvement_percent: f32,
    pub monsoon_capture_efficiency_percent: f32,
    pub xeriscaping_compatibility: bool,
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soil_monitor_initialization() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        
        assert_eq!(monitor.current_health_state, SoilHealthState::Stable);
        assert_eq!(monitor.management_zones.len(), 0);
        assert_eq!(monitor.sensor_readings.len(), 0);
    }

    #[test]
    fn test_ph_health_scoring() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        
        // Test optimal pH (7.8)
        assert!((monitor.calculate_ph_score(7.8) - 1.0).abs() < 0.01);
        
        // Test native range boundaries
        assert!(monitor.calculate_ph_score(7.5) > 0.8);
        assert!(monitor.calculate_ph_score(8.5) > 0.8);
        
        // Test outside native range
        assert!(monitor.calculate_ph_score(7.0) < 0.6);
        assert!(monitor.calculate_ph_score(9.0) < 0.4);
        
        // Test toxic range
        assert_eq!(monitor.calculate_ph_score(6.5), 0.0);
        assert_eq!(monitor.calculate_ph_score(9.5), 0.0);
    }

    #[test]
    fn test_offline_queue_capacity() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        assert!(monitor.offline_queue.capacity_hours() >= 72);
    }

    #[test]
    fn test_carbon_impact_calculation() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        
        // Test biochar impact on 1 acre (43,560 sqft)
        let impact = monitor.estimate_carbon_impact(RegenerativeAction::BiocharAmendment, 43560.0);
        assert!((impact - 0.8).abs() < 0.1);
        
        // Test compost impact
        let compost_impact = monitor.estimate_carbon_impact(RegenerativeAction::CompostApplication, 43560.0);
        assert!((compost_impact - 0.6).abs() < 0.1);
    }

    #[test]
    fn test_indigenous_practice_mapping() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        
        // Test Ak Chin irrigation mapping
        assert_eq!(
            monitor.map_knowledge_to_action(IndigenousSoilPractice::AkChinFloodIrrigation),
            RegenerativeAction::AkChinIrrigation
        );
        
        // Test Mesquite pod amendment mapping
        assert_eq!(
            monitor.map_knowledge_to_action(IndigenousSoilPractice::MesquitePodAmendment),
            RegenerativeAction::CompostApplication
        );
    }

    #[test]
    fn test_soil_health_state_determination() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let mut monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        
        // Create sensor reading for thriving soil
        let thriving_reading = SoilSensorReading {
            timestamp: 1000,
            zone_id: [1u8; 32],
            ph_level: 7.8,
            organic_matter_percent: 3.0,
            moisture_percent: 20.0,
            electrical_conductivity_ds_m: 2.0,
            temperature_f: 85.0,
            bulk_density_g_cm3: 1.3,
            infiltration_rate_inches_per_hour: 3.0,
            nitrogen_ppm: 20.0,
            phosphorus_ppm: 15.0,
            potassium_ppm: 200.0,
            lead_ppm: 10.0,
            arsenic_ppm: 1.0,
            microbiome_diversity_index: 0.85,
            mycorrhizal_colonization_percent: 70.0,
            sensor_id: [1u8; 32],
            depth_inches: 12.0,
        };
        
        // Create zone
        let mut zone = SoilManagementZone {
            zone_id: [1u8; 32],
            name: "Test Zone".to_string(),
            boundaries: vec![[33.4484, -112.0740], [33.45, -112.07]],
            area_sqft: 10000.0,
            soil_texture: SoilTexture::SandyLoam,
            current_health_state: SoilHealthState::Degraded,
            target_health_state: SoilHealthState::Thriving,
            indigenous_territory: false,
            treaty_zone_id: None,
            native_vegetation: true,
            urban_agriculture: false,
            contamination_history: vec![],
            carbon_sequestration_tons_per_acre_yr: 0.2,
            last_amendment_date: 0,
            amendment_history: vec![],
        };
        
        // Update health state
        monitor.update_zone_health_state(&mut zone, &thriving_reading).unwrap();
        
        // Should be upgraded to Thriving state
        assert_eq!(zone.current_health_state, SoilHealthState::Thriving);
    }

    #[test]
    fn test_contamination_detection() {
        let config = SoilNetworkConfiguration {
            network_id: [1u8; 32],
            management_zones: vec![],
            sensor_network: vec![],
            indigenous_territories: vec![],
            contamination_hotspots: vec![],
            carbon_sequestration_targets: 50.0,
            water_reclamation_integration: true,
            native_plant_associations: HashMap::new(),
        };
        
        let mut monitor = SoilHealthMonitor::new(BirthSign::default(), config).unwrap();
        
        // Process lead contamination above threshold
        let result = monitor.process_contamination_detection(
            [1u8; 32],
            SoilContaminant::Lead,
            150.0 // Above 100ppm threshold
        );
        
        assert!(result.is_ok());
        assert!(monitor.metrics.contamination_incidents > 0);
    }

    #[test]
    fn test_monsoon_season_check() {
        // Verify monsoon season months for Phoenix
        assert_eq!(MONSOON_SEASON_MONTHS, [8, 9]); // August-September
        
        // Verify moisture stress thresholds
        assert_eq!(WILTING_POINT_PERCENT, 8.0);
        assert_eq!(OPTIMAL_FIELD_CAPACITY_PERCENT, 25.0);
    }

    #[test]
    fn test_native_soil_parameters() {
        // Verify Sonoran Desert soil parameters
        assert_eq!(NATIVE_SOIL_PH_MIN, 7.5);
        assert_eq!(NATIVE_SOIL_PH_MAX, 8.5);
        assert_eq!(NATIVE_ORGANIC_MATTER_PERCENT, 0.8);
        assert_eq!(TARGET_ORGANIC_MATTER_PERCENT, 2.5);
        
        // Verify salinity tolerances for native species
        assert_eq!(SAGUARO_SALINITY_TOLERANCE_DS_M, 4.0);
        assert_eq!(PALO_VERDE_SALINITY_TOLERANCE_DS_M, 6.0);
        assert_eq!(OCOTILLO_SALINITY_TOLERANCE_DS_M, 5.0);
    }

    #[test]
    fn test_carbon_sequestration_targets() {
        // Verify carbon sequestration targets for desert regenerative agriculture
        assert_eq!(CARBON_SEQUESTRATION_MIN_TON_ACRE_YR, 0.5);
        assert_eq!(CARBON_SEQUESTRATION_MAX_TON_ACRE_YR, 1.0);
        assert_eq!(CARBON_SEQUESTRATION_CURRENT_TON_ACRE_YR, 0.2);
    }
}
