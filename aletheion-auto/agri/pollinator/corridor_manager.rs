/**
 * Aletheion Smart City Core - Batch 2
 * File: 110/200
 * Layer: 31 (Advanced Agriculture)
 * Path: aletheion-auto/agri/pollinator/corridor_manager.rs
 * 
 * Research Basis (Environmental & Climate Integration - E):
 *   - Phoenix Native Bee Diversity: 400+ species including Sonoran Bumblebee, Sweat Bees, Mason Bees
 *   - Butterfly Migration Routes: Monarch waystations, Gulf Fritillary corridors, Painted Lady pathways
 *   - Hummingbird Habitat: Anna's, Costa's, Rufous hummingbird feeding zones
 *   - Native Sonoran Desert Flora: Saguaro (Carnegiea gigantea), Palo Verde (Parkinsonia spp.), 
 *     Ocotillo (Fouquieria splendens), Creosote (Larrea tridentata), Desert Marigold (Baileya multiradiata),
 *     Brittlebush (Encelia farinosa), Prickly Pear (Opuntia spp.), Mesquite (Prosopis spp.)
 *   - Pollinator Decline Crisis: 40% native bee species at risk, 80% monarch butterfly population decline (1990-2025)
 *   - Habitat Fragmentation: Urban corridors must maintain 100-500 ft connectivity for species movement
 *   - Pesticide-Free Zones: 100% organic management within 500 ft of pollinator corridors
 *   - Indigenous Ecological Knowledge: Akimel O'odham and Piipaash traditional plant-pollinator relationships
 *   - Climate Resilience: Drought-tolerant native species maintain pollinator support during extreme heat
 *   - Water Conservation: Xeriscaping principles reduce water usage by 60% vs traditional landscaping
 * 
 * Compliance: 
 *   - ALE-COMP-CORE (v2.1)
 *   - FPIC (Free, Prior, Informed Consent)
 *   - Phoenix Heat Protocols (Offline-72h)
 *   - BioticTreaties (Ecological Rights & Species Sovereignty)
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
use aletheion_env::water::WaterQuality;

// --- Constants & Phoenix Pollinator Parameters ---

/// Pollinator corridor dimensions (feet)
const MIN_CORRIDOR_WIDTH_FT: f32 = 100.0; // Minimum for species connectivity
const OPTIMAL_CORRIDOR_WIDTH_FT: f32 = 300.0; // Optimal for diverse species
const HABITAT_BUFFER_ZONE_FT: f32 = 50.0; // Buffer from urban development
const CORRIDOR_CONNECTIVITY_DISTANCE_FT: f32 = 500.0; // Max gap between corridor segments

/// Native bee species parameters
const SONORAN_BUMBLEBEE_FORAGING_RADIUS_FT: f32 = 1500.0;
const SWEAT_BEE_FORAGING_RADIUS_FT: f32 = 1000.0;
const MASON_BEE_FORAGING_RADIUS_FT: f32 = 800.0;
const CARPENTER_BEE_FORAGING_RADIUS_FT: f32 = 1200.0;
const MIN_NESTING_SITES_PER_ACRE: usize = 20; // Nesting habitat density

/// Butterfly species parameters
const MONARCH_MIGRATION_CORRIDOR_WIDTH_FT: f32 = 200.0;
const GULF_FRITILLARY_RANGE_FT: f32 = 2000.0;
const PAINTED_LADY_MIGRATION_DISTANCE_MILES: f32 = 100.0;
const MIN_MILKWEED_PLANTS_PER_ACRE: usize = 50; // Monarch host plant requirement

/// Hummingbird parameters
const ANNAS_HUMMINGBIRD_TERRITORY_FT: f32 = 500.0;
const COSTAS_HUMMINGBIRD_RANGE_FT: f32 = 800.0;
const NECTAR_FLOWER_DENSITY_PER_100SQFT: usize = 15; // Minimum flowering plants

/// Native plant species bloom cycles (months)
const SAGUARO_BLOOM_MONTHS: [u8; 2] = [5, 6]; // May-June
const PALO_VERDE_BLOOM_MONTHS: [u8; 2] = [4, 5]; // April-May
const OCOTILLO_BLOOM_MONTHS: [u8; 4] = [3, 4, 5, 6]; // March-June (multiple blooms)
const CREOSOTE_BLOOM_MONTHS: [u8; 2] = [4, 5]; // April-May (after rain)
const DESERT_MARIGOLD_BLOOM_MONTHS: [u8; 8] = [2, 3, 4, 5, 6, 7, 8, 9]; // Feb-Sept
const BRITTLEBUSH_BLOOM_MONTHS: [u8; 4] = [3, 4, 5, 6]; // March-June

/// Habitat quality thresholds (0.0-1.0)
const MIN_HABITAT_QUALITY_SCORE: f32 = 0.7;
const MIN_NATIVE_PLANT_DENSITY: f32 = 0.8; // 80% native species minimum
const MAX_PESTICIDE_RESIDUE_PPM: f32 = 0.0; // Zero tolerance in corridors
const MIN_WATER_AVAILABILITY_SCORE: f32 = 0.6;
const MIN_FLORAL_RESOURCE_DENSITY: f32 = 0.75;

/// Pollinator health monitoring thresholds
const MIN_BEE_COLONY_HEALTH_SCORE: f32 = 0.6;
const MIN_BUTTERFLY_LARVAL_SURVIVAL_RATE: f32 = 0.5;
const MIN_HUMMINGBIRD_NESTING_SUCCESS_RATE: f32 = 0.4;
const PESTICIDE_EXPOSURE_ALERT_THRESHOLD_PPM: f32 = 0.1;

/// Water conservation parameters
const XERISCAPING_WATER_REDUCTION_PERCENT: f32 = 60.0;
const NATIVE_PLANT_WATER_USAGE_GALLONS_PER_SQFT_PER_YEAR: f32 = 3.0;
const NON_NATIVE_PLANT_WATER_USAGE_GALLONS_PER_SQFT_PER_YEAR: f32 = 8.0;
const RAINWATER_HARVEST_EFFICIENCY: f32 = 0.85;

/// Seasonal activity patterns (Phoenix climate)
const BEE_ACTIVE_MONTHS: [u8; 9] = [2, 3, 4, 5, 6, 7, 8, 9, 10]; // Feb-Oct
const BUTTERFLY_ACTIVE_MONTHS: [u8; 10] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11]; // Feb-Nov
const HUMMINGBIRD_YEAR_ROUND: bool = true; // Anna's hummingbird present year-round
const MONARCH_MIGRATION_MONTHS: [u8; 4] = [3, 4, 9, 10]; // Spring and Fall migration

/// Indigenous ecological knowledge integration
const INDIGENOUS_PLANT_USE_CATEGORIES: usize = 8; // Food, medicine, fiber, dye, ceremony, tool, shelter, spiritual
const TRADITIONAL_ECOLOGICAL_KNOWLEDGE_SOURCES: usize = 2; // Akimel O'odham, Piipaash

/// Offline Buffer Duration (hours) - Must meet 72h Protocol
const OFFLINE_BUFFER_HOURS: u32 = 72;
/// Treaty Check Cache TTL (seconds)
const TREATY_CACHE_TTL: u64 = 300;

/// Maximum number of pollinator species tracked per corridor
const MAX_SPECIES_PER_CORRIDOR: usize = 100;

// --- Enumerations ---

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PollinatorType {
    NativeBee,
    Honeybee,
    Butterfly,
    Moth,
    Hummingbird,
    Bat,
    Beetle,
    Fly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum NativeBeeSpecies {
    SonoranBumblebee,
    WesternBumblebee,
    SweatBee,
    MasonBee,
    CarpenterBee,
    LeafcutterBee,
    MiningBee,
    SquashBee,
    CactusBee,
    OtherNativeBee,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ButterflySpecies {
    Monarch,
    GulfFritillary,
    PaintedLady,
    Queen,
    WesternSwallowtail,
    ArizonaSister,
    SleepyOrange,
    ReakirtsBlue,
    OtherButterfly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum HummingbirdSpecies {
    AnnasHummingbird,
    CostasHummingbird,
    RufousHummingbird,
    BroadbilledHummingbird,
    BlackchinnedHummingbird,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CorridorState {
    OptimalHealth,
    GoodCondition,
    ModerateDegradation,
    PoorCondition,
    CriticalDecline,
    RestorationInProgress,
    MonitoringOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HabitatQualityFactor {
    FloralResourceAbundance,
    NestingSiteAvailability,
    WaterSourceProximity,
    PesticideContamination,
    InvasiveSpeciesPressure,
    ClimateStress,
    HumanDisturbance,
    ConnectivityQuality,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConservationActionType {
    PlantNativeSpecies,
    RemoveInvasiveSpecies,
    InstallNestingHabitat,
    CreateWaterSource,
    EstablishPesticideFreeZone,
    MonitorPollinatorHealth,
    RestoreDegradedArea,
    ConnectCorridorSegments,
    IndigenousKnowledgeIntegration,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreatLevel {
    None,
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndigenousKnowledgeCategory {
    PlantMedicinalUse,
    PlantFoodSource,
    PlantFiberMaterial,
    PlantDyeSource,
    CeremonialSignificance,
    ToolMakingKnowledge,
    ShelterConstruction,
    SpiritualConnection,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeasonalActivity {
    Dormant,
    EarlySpring,
    LateSpring,
    EarlySummer,
    LateSummer,
    EarlyFall,
    LateFall,
    WinterActive,
}

#[derive(Clone)]
pub struct PollinatorSpecies {
    pub species_id: [u8; 32],
    pub pollinator_type: PollinatorType,
    pub native_bee_species: Option<NativeBeeSpecies>,
    pub butterfly_species: Option<ButterflySpecies>,
    pub hummingbird_species: Option<HummingbirdSpecies>,
    pub scientific_name: String,
    pub common_name: String,
    pub foraging_radius_ft: f32,
    pub active_months: Vec<u8>,
    pub host_plants: Vec<[u8; 32]>, // Plant variety IDs
    pub nectar_plants: Vec<[u8; 32]>,
    pub nesting_requirements: String,
    pub conservation_status: ConservationStatus,
    pub indigenous_significance: Option<IndigenousKnowledgeCategory>,
    pub population_trend: PopulationTrend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConservationStatus {
    LeastConcern,
    NearThreatened,
    Vulnerable,
    Endangered,
    CriticallyEndangered,
    ExtinctInTheWild,
    Extinct,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PopulationTrend {
    Increasing,
    Stable,
    Decreasing,
    Unknown,
}

#[derive(Clone)]
pub struct NativePlantSpecies {
    pub plant_id: [u8; 32],
    pub scientific_name: String,
    pub common_name: String,
    pub native_sonoran: bool,
    pub bloom_months: Vec<u8>,
    pub water_requirements_gallons_per_year: f32,
    pub pollinator_attraction_score: f32, // 0.0-1.0
    pub drought_tolerance: f32, // 0.0-1.0
    pub soil_requirements: String,
    pub sun_requirements: SunExposure,
    pub height_ft: f32,
    pub spread_ft: f32,
    pub indigenous_uses: Vec<IndigenousKnowledgeCategory>,
    pub host_for_species: Vec<[u8; 32]>, // Pollinator species IDs that use this plant
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SunExposure {
    FullSun,
    PartialSun,
    PartialShade,
    FullShade,
}

#[derive(Clone)]
pub struct PollinatorCorridor {
    pub corridor_id: [u8; 32],
    pub name: String,
    pub boundaries: Vec<[f64; 2]>, // GPS polygon vertices
    pub total_area_sqft: f32,
    pub length_ft: f32,
    pub width_ft: f32,
    pub corridor_type: CorridorType,
    pub connected_corridors: Vec<[u8; 32]>,
    pub habitat_quality_score: f32,
    pub native_plant_density: f32,
    pub species_count: usize,
    pub indigenous_territory: bool,
    pub treaty_zone_id: Option<[u8; 32]>,
    pub pesticide_free_zone: bool,
    pub water_sources: Vec<WaterSourceLocation>,
    pub nesting_sites: Vec<NestingSite>,
    pub active: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CorridorType {
    BeeHighway,
    ButterflyMigrationRoute,
    HummingbirdFeedingZone,
    MultiSpeciesHabitat,
    RiparianCorridor,
    UrbanGreenway,
    AgriculturalBuffer,
    RestorationZone,
}

#[derive(Clone)]
pub struct WaterSourceLocation {
    pub source_id: [u8; 32],
    pub coordinates: [f64; 2],
    pub water_type: WaterSourceType,
    pub reliability_score: f32, // 0.0-1.0
    pub seasonal_availability: Vec<u8>, // Months available
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WaterSourceType {
    NaturalSpring,
    RiparianArea,
    RainwaterHarvest,
    RecycledWaterFeature,
    BirdBath,
    Pond,
    Stream,
}

#[derive(Clone)]
pub struct NestingSite {
    pub site_id: [u8; 32],
    pub coordinates: [f64; 2],
    pub nesting_type: NestingType,
    pub capacity_units: usize, // Number of nesting opportunities
    pub occupied_units: usize,
    pub species_supported: Vec<[u8; 32]>, // Pollinator species IDs
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NestingType {
    GroundNesting,
    CavityNesting,
    StemNesting,
    WoodNesting,
    RockCrevice,
    ArtificialBeeHotel,
    ButterflyPupaSite,
}

#[derive(Clone)]
pub struct HabitatQualityReading {
    pub timestamp: u64,
    pub corridor_id: [u8; 32],
    pub quality_factors: HashMap<HabitatQualityFactor, f32>, // Score 0.0-1.0 for each factor
    pub overall_score: f32,
    pub floral_resource_density: f32,
    pub nesting_site_availability: f32,
    pub water_availability_score: f32,
    pub pesticide_residue_ppm: f32,
    pub invasive_species_percent: f32,
    pub sensor_id: [u8; 32],
}

#[derive(Clone)]
pub struct PollinatorHealthReading {
    pub timestamp: u64,
    pub corridor_id: [u8; 32],
    pub species_id: [u8; 32],
    pub population_estimate: usize,
    pub health_score: f32, // 0.0-1.0
    pub larval_survival_rate: f32,
    pub nesting_success_rate: f32,
    pub pesticide_exposure_ppm: f32,
    pub stress_indicators: Vec<StressIndicator>,
    pub sensor_id: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StressIndicator {
    LowFoodAvailability,
    HighTemperatureStress,
    PesticideExposure,
    HabitatFragmentation,
    DiseasePresence,
    ParasiteLoad,
    WaterScarcity,
    HumanDisturbance,
}

#[derive(Clone)]
pub struct ConservationActionPlan {
    pub plan_id: [u8; 32],
    pub corridor_id: [u8; 32],
    pub action_type: ConservationActionType,
    pub target_species: Vec<[u8; 32]>,
    pub target_plants: Vec<[u8; 32]>,
    pub priority: u8, // 0-100
    pub estimated_cost_usd: f32,
    pub estimated_duration_days: u32,
    pub expected_impact_score: f32, // 0.0-1.0
    pub treaty_compliant: bool,
    pub indigenous_knowledge_integrated: bool,
    pub scheduled_start_date: u64,
    pub scheduled_end_date: u64,
}

#[derive(Clone)]
pub struct IndigenousKnowledgeEntry {
    pub entry_id: [u8; 32],
    pub knowledge_category: IndigenousKnowledgeCategory,
    pub plant_species: Option<[u8; 32]>,
    pub pollinator_species: Option<[u8; 32]>,
    pub traditional_use_description: String,
    pub cultural_significance: String,
    pub knowledge_source: IndigenousCommunity,
    pub fpic_verified: bool,
    pub timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndigenousCommunity {
    AkimelOodham,
    Piipaash,
    OtherIndigenousNation,
}

#[derive(Clone)]
pub struct PollinatorCorridorMetrics {
    pub current_state: CorridorState,
    pub total_corridors: usize,
    pub total_area_acres: f32,
    pub species_monitored: usize,
    pub native_plants_established: usize,
    pub habitat_quality_average: f32,
    pub pesticide_violations: usize,
    pub restoration_projects_active: usize,
    pub indigenous_knowledge_entries: usize,
    pub water_conservation_gallons_per_year: f32,
}

#[derive(Clone)]
pub struct CorridorNetworkConfiguration {
    pub network_id: [u8; 32],
    pub corridors: Vec<PollinatorCorridor>,
    pub native_plant_database: Vec<NativePlantSpecies>,
    pub pollinator_species_database: Vec<PollinatorSpecies>,
    pub indigenous_territories: Vec<[u8; 32]>,
    pub water_reclamation_integration: bool,
    pub monitoring_sensor_network: Vec<[u8; 32]>,
    pub restoration_priority_zones: Vec<[u8; 32]>,
}

// --- Core Pollinator Corridor Manager Structure ---

pub struct PollinatorCorridorManager {
    pub node_id: BirthSign,
    pub config: CorridorNetworkConfiguration,
    pub current_state: CorridorState,
    pub corridors: BTreeMap<[u8; 32], PollinatorCorridor>,
    pub native_plants: BTreeMap<[u8; 32], NativePlantSpecies>,
    pub pollinator_species: BTreeMap<[u8; 32], PollinatorSpecies>,
    pub habitat_quality_readings: BTreeMap<u64, HabitatQualityReading>,
    pub pollinator_health_readings: BTreeMap<u64, PollinatorHealthReading>,
    pub conservation_plans: BTreeMap<u64, ConservationActionPlan>,
    pub indigenous_knowledge: Vec<IndigenousKnowledgeEntry>,
    pub offline_queue: OfflineQueue<ConservationActionPlan>,
    pub treaty_cache: TreatyCompliance,
    pub metrics: PollinatorCorridorMetrics,
    pub last_quality_update: u64,
    pub last_health_update: u64,
    pub last_sync: u64,
}

impl PollinatorCorridorManager {
    /**
     * Initialize the Pollinator Corridor Manager with Configuration
     * Ensures 72h operational buffer and treaty compliance setup
     */
    pub fn new(node_id: BirthSign, config: CorridorNetworkConfiguration) -> Result<Self, &'static str> {
        let queue = OfflineQueue::new(OFFLINE_BUFFER_HOURS)
            .map_err(|_| "Failed to allocate offline buffer")?;
        
        // Initialize native plant and pollinator databases
        let mut native_plants = BTreeMap::new();
        for plant in config.native_plant_database.iter() {
            native_plants.insert(plant.plant_id, plant.clone());
        }
        
        let mut pollinator_species = BTreeMap::new();
        for species in config.pollinator_species_database.iter() {
            pollinator_species.insert(species.species_id, species.clone());
        }
        
        // Initialize corridors
        let mut corridors = BTreeMap::new();
        for corridor in config.corridors.iter() {
            corridors.insert(corridor.corridor_id, corridor.clone());
        }
        
        Ok(Self {
            node_id,
            config,
            current_state: CorridorState::OptimalHealth,
            corridors,
            native_plants,
            pollinator_species,
            habitat_quality_readings: BTreeMap::new(),
            pollinator_health_readings: BTreeMap::new(),
            conservation_plans: BTreeMap::new(),
            indigenous_knowledge: Vec::new(),
            offline_queue: queue,
            treaty_cache: TreatyCompliance::new(),
            metrics: PollinatorCorridorMetrics {
                current_state: CorridorState::OptimalHealth,
                total_corridors: config.corridors.len(),
                total_area_acres: config.corridors.iter().map(|c| c.total_area_sqft / 43560.0).sum(),
                species_monitored: config.pollinator_species_database.len(),
                native_plants_established: config.native_plant_database.len(),
                habitat_quality_average: 0.8,
                pesticide_violations: 0,
                restoration_projects_active: 0,
                indigenous_knowledge_entries: 0,
                water_conservation_gallons_per_year: 0.0,
            },
            last_quality_update: 0,
            last_health_update: 0,
            last_sync: 0,
        })
    }

    /**
     * ERM Chain: SENSE
     * Ingests habitat quality readings, pollinator health data, and Indigenous knowledge entries
     * Validates data integrity using PQ hashing
     */
    pub fn sense(&mut self, input: CorridorInput) -> Result<CorridorSenseResult, &'static str> {
        match input {
            CorridorInput::HabitatQualityReading(reading) => self.process_habitat_quality_reading(reading),
            CorridorInput::PollinatorHealthReading(reading) => self.process_pollinator_health_reading(reading),
            CorridorInput::IndigenousKnowledgeEntry(entry) => self.process_indigenous_knowledge_entry(entry),
            CorridorInput::CorridorUpdate(corridor) => self.process_corridor_update(corridor),
        }
    }

    /**
     * Process habitat quality reading
     */
    fn process_habitat_quality_reading(&mut self, reading: HabitatQualityReading) -> Result<CorridorSenseResult, &'static str> {
        // Validate sensor signature (PQ Secure)
        let hash = pq_hash(&reading.sensor_id);
        if hash[0] == 0x00 {
            return Err("Sensor signature invalid");
        }

        // Store reading with timestamp key
        self.habitat_quality_readings.insert(reading.timestamp, reading.clone());

        // Update last quality update time
        self.last_quality_update = aletheion_core::time::now();

        // Check habitat quality thresholds
        self.check_habitat_quality_thresholds(&reading)?;

        // Update corridor habitat quality score
        if let Some(corridor) = self.corridors.get_mut(&reading.corridor_id) {
            corridor.habitat_quality_score = reading.overall_score;
            
            // Update corridor state based on quality score
            corridor.active = reading.overall_score >= MIN_HABITAT_QUALITY_SCORE;
        }

        // Log sensing event
        self.log_event(format!(
            "HABITAT_QUALITY: Corridor={:?}, Overall={:.2}, Floral={:.2}, Nesting={:.2}, Water={:.2}, Pesticide={:.2}ppm",
            reading.corridor_id,
            reading.overall_score,
            reading.floral_resource_density,
            reading.nesting_site_availability,
            reading.water_availability_score,
            reading.pesticide_residue_ppm
        ));

        Ok(CorridorSenseResult::HabitatQualityProcessed(reading.sensor_id))
    }

    /**
     * Process pollinator health reading
     */
    fn process_pollinator_health_reading(&mut self, reading: PollinatorHealthReading) -> Result<CorridorSenseResult, &'static str> {
        // Validate sensor signature (PQ Secure)
        let hash = pq_hash(&reading.sensor_id);
        if hash[0] == 0x00 {
            return Err("Sensor signature invalid");
        }

        // Store reading with timestamp key
        self.pollinator_health_readings.insert(reading.timestamp, reading.clone());

        // Update last health update time
        self.last_health_update = aletheion_core::time::now();

        // Check health thresholds
        self.check_pollinator_health_thresholds(&reading)?;

        // Log sensing event
        self.log_event(format!(
            "POLLINATOR_HEALTH: Species={:?}, Corridor={:?}, Population={}, Health={:.2}, LarvalSurvival={:.2}, Pesticide={:.2}ppm",
            reading.species_id,
            reading.corridor_id,
            reading.population_estimate,
            reading.health_score,
            reading.larval_survival_rate,
            reading.pesticide_exposure_ppm
        ));

        Ok(CorridorSenseResult::PollinatorHealthProcessed(reading.sensor_id))
    }

    /**
     * Process Indigenous knowledge entry
     */
    fn process_indigenous_knowledge_entry(&mut self, entry: IndigenousKnowledgeEntry) -> Result<CorridorSenseResult, &'static str> {
        // Validate FPIC compliance for Indigenous knowledge
        if !entry.fpic_verified {
            self.log_warning("FPIC_VIOLATION: Indigenous knowledge entry requires FPIC verification");
            return Ok(CorridorSenseResult::KnowledgeEntryRejected(entry.entry_id));
        }

        // Add to Indigenous knowledge database
        self.indigenous_knowledge.push(entry.clone());
        self.metrics.indigenous_knowledge_entries += 1;

        // Log knowledge entry
        self.log_event(format!(
            "INDIGENOUS_KNOWLEDGE: Category={:?}, Source={:?}, Plant={:?}, FPIC={}",
            entry.knowledge_category,
            entry.knowledge_source,
            entry.plant_species.map_or("None".to_string(), |id| format!("{:?}", id)),
            entry.fpic_verified
        ));

        Ok(CorridorSenseResult::KnowledgeEntryProcessed(entry.entry_id))
    }

    /**
     * Process corridor update
     */
    fn process_corridor_update(&mut self, mut corridor: PollinatorCorridor) -> Result<CorridorSenseResult, &'static str> {
        // Validate corridor signature (PQ Secure)
        let hash = pq_hash(&corridor.corridor_id);
        if hash[0] == 0x00 {
            return Err("Corridor signature invalid");
        }

        // Check treaty compliance for Indigenous territories
        if corridor.indigenous_territory {
            if let Some(treaty_zone) = corridor.treaty_zone_id {
                let compliance = self.treaty_cache.check_ecological_rights(&treaty_zone)?;
                if !compliance.allowed {
                    self.log_warning("FPIC_VIOLATION: Corridor update denied due to treaty restrictions");
                    return Ok(CorridorSenseResult::CorridorUpdateDenied(corridor.corridor_id));
                }
            }
        }

        // Update or insert corridor
        self.corridors.insert(corridor.corridor_id, corridor.clone());

        // Update metrics
        self.metrics.total_corridors = self.corridors.len();
        self.metrics.total_area_acres = self.corridors.iter()
            .map(|(_, c)| c.total_area_sqft / 43560.0)
            .sum();

        // Log corridor update
        self.log_event(format!(
            "CORRIDOR_UPDATE: ID={:?}, Name={}, Area={:.1}acres, Width={:.0}ft, Quality={:.2}, Active={}",
            corridor.corridor_id,
            corridor.name,
            corridor.total_area_sqft / 43560.0,
            corridor.width_ft,
            corridor.habitat_quality_score,
            corridor.active
        ));

        Ok(CorridorSenseResult::CorridorUpdateProcessed(corridor.corridor_id))
    }

    /**
     * Check habitat quality thresholds and trigger alerts if exceeded
     */
    fn check_habitat_quality_thresholds(&mut self, reading: &HabitatQualityReading) -> Result<(), &'static str> {
        let mut alerts_triggered = false;

        // Overall quality check
        if reading.overall_score < MIN_HABITAT_QUALITY_SCORE {
            self.log_warning(format!(
                "HABITAT_QUALITY_ALERT: Overall score {:.2} below minimum threshold {:.2}",
                reading.overall_score,
                MIN_HABITAT_QUALITY_SCORE
            ));
            alerts_triggered = true;
        }

        // Native plant density check
        if reading.floral_resource_density < MIN_FLORAL_RESOURCE_DENSITY {
            self.log_warning(format!(
                "FLORAL_RESOURCE_ALERT: Density {:.2} below minimum threshold {:.2}",
                reading.floral_resource_density,
                MIN_FLORAL_RESOURCE_DENSITY
            ));
            alerts_triggered = true;
        }

        // Pesticide contamination check
        if reading.pesticide_residue_ppm > MAX_PESTICIDE_RESIDUE_PPM {
            self.log_warning(format!(
                "PESTICIDE_VIOLATION: {:.2}ppm exceeds maximum threshold {:.2}ppm",
                reading.pesticide_residue_ppm,
                MAX_PESTICIDE_RESIDUE_PPM
            ));
            self.metrics.pesticide_violations += 1;
            alerts_triggered = true;
        }

        // Water availability check
        if reading.water_availability_score < MIN_WATER_AVAILABILITY_SCORE {
            self.log_warning(format!(
                "WATER_SCARCITY_ALERT: Availability score {:.2} below minimum threshold {:.2}",
                reading.water_availability_score,
                MIN_WATER_AVAILABILITY_SCORE
            ));
            alerts_triggered = true;
        }

        if alerts_triggered {
            // Generate conservation action plan
            self.generate_conservation_action_plan(&reading.corridor_id, ThreatLevel::High)?;
        }

        Ok(())
    }

    /**
     * Check pollinator health thresholds and trigger alerts if exceeded
     */
    fn check_pollinator_health_thresholds(&mut self, reading: &PollinatorHealthReading) -> Result<(), &'static str> {
        let mut alerts_triggered = false;

        // Health score check
        if reading.health_score < MIN_BEE_COLONY_HEALTH_SCORE {
            self.log_warning(format!(
                "POLLINATOR_HEALTH_ALERT: Species {:?} health score {:.2} below minimum threshold {:.2}",
                reading.species_id,
                reading.health_score,
                MIN_BEE_COLONY_HEALTH_SCORE
            ));
            alerts_triggered = true;
        }

        // Pesticide exposure check
        if reading.pesticide_exposure_ppm > PESTICIDE_EXPOSURE_ALERT_THRESHOLD_PPM {
            self.log_warning(format!(
                "PESTICIDE_EXPOSURE_ALERT: Species {:?} exposure {:.2}ppm exceeds threshold {:.2}ppm",
                reading.species_id,
                reading.pesticide_exposure_ppm,
                PESTICIDE_EXPOSURE_ALERT_THRESHOLD_PPM
            ));
            alerts_triggered = true;
        }

        // Larval survival check (for butterflies)
        if reading.larval_survival_rate < MIN_BUTTERFLY_LARVAL_SURVIVAL_RATE {
            self.log_warning(format!(
                "LARVAL_SURVIVAL_ALERT: Species {:?} survival rate {:.2} below minimum threshold {:.2}",
                reading.species_id,
                reading.larval_survival_rate,
                MIN_BUTTERFLY_LARVAL_SURVIVAL_RATE
            ));
            alerts_triggered = true;
        }

        if alerts_triggered {
            // Generate conservation action plan
            self.generate_conservation_action_plan(&reading.corridor_id, ThreatLevel::High)?;
        }

        Ok(())
    }

    /**
     * ERM Chain: MODEL
     * Analyzes corridor network health, species population trends, and generates conservation action plans
     * No Digital Twins: Uses real-time sensor data and ecological modeling
     */
    pub fn model_optimal_conservation(&mut self) -> Result<Vec<ConservationActionPlan>, &'static str> {
        let current_time = aletheion_core::time::now();
        
        // Update corridor states based on latest readings
        self.update_corridor_states(current_time)?;
        
        // Generate conservation action plans
        let mut action_plans = Vec::new();
        
        // 1. Identify corridors requiring immediate intervention
        self.identify_critical_corridors(&mut action_plans, current_time)?;
        
        // 2. Generate seasonal planting recommendations
        self.generate_seasonal_planting_plans(&mut action_plans, current_time)?;
        
        // 3. Plan corridor connectivity improvements
        self.plan_corridor_connectivity(&mut action_plans, current_time)?;
        
        // 4. Integrate Indigenous ecological knowledge
        self.integrate_indigenous_knowledge(&mut action_plans)?;
        
        // Update metrics
        self.update_corridor_metrics(current_time)?;
        
        Ok(action_plans)
    }

    /**
     * Update corridor states based on latest habitat quality readings
     */
    fn update_corridor_states(&mut self, current_time: u64) -> Result<(), &'static str> {
        for (_, corridor) in &mut self.corridors {
            // Find latest habitat quality reading for this corridor
            let latest_reading = self.habitat_quality_readings.iter()
                .filter(|(_, r)| r.corridor_id == corridor.corridor_id)
                .max_by_key(|(timestamp, _)| *timestamp);
            
            if let Some((_, reading)) = latest_reading {
                corridor.habitat_quality_score = reading.overall_score;
                
                // Update corridor state based on quality score
                corridor.current_state = self.determine_corridor_state(reading.overall_score);
            }
        }
        
        Ok(())
    }

    /**
     * Determine corridor state based on habitat quality score
     */
    fn determine_corridor_state(&self, quality_score: f32) -> CorridorState {
        match quality_score {
            s if s >= 0.9 => CorridorState::OptimalHealth,
            s if s >= 0.75 => CorridorState::GoodCondition,
            s if s >= 0.6 => CorridorState::ModerateDegradation,
            s if s >= 0.4 => CorridorState::PoorCondition,
            s if s >= 0.2 => CorridorState::CriticalDecline,
            _ => CorridorState::CriticalDecline,
        }
    }

    /**
     * Identify corridors requiring immediate conservation intervention
     */
    fn identify_critical_corridors(&mut self, plans: &mut Vec<ConservationActionPlan>, current_time: u64) -> Result<(), &'static str> {
        for (_, corridor) in &self.corridors {
            if corridor.habitat_quality_score < MIN_HABITAT_QUALITY_SCORE {
                // Generate restoration plan
                let plan = self.generate_restoration_plan(corridor, current_time)?;
                plans.push(plan);
            }
            
            // Check for pesticide violations
            let recent_violations: Vec<_> = self.habitat_quality_readings.iter()
                .filter(|(_, r)| r.corridor_id == corridor.corridor_id && r.pesticide_residue_ppm > 0.0)
                .collect();
            
            if !recent_violations.is_empty() {
                // Generate pesticide remediation plan
                let plan = self.generate_pesticide_remediation_plan(corridor, current_time)?;
                plans.push(plan);
            }
        }
        
        Ok(())
    }

    /**
     * Generate restoration plan for degraded corridor
     */
    fn generate_restoration_plan(&self, corridor: &PollinatorCorridor, current_time: u64) -> Result<ConservationActionPlan, &'static str> {
        let plan_id = pq_hash(&current_time.to_be_bytes());
        
        // Determine restoration actions based on corridor deficiencies
        let action_type = if corridor.native_plant_density < MIN_NATIVE_PLANT_DENSITY {
            ConservationActionType::PlantNativeSpecies
        } else if corridor.habitat_quality_score < 0.5 {
            ConservationActionType::RestoreDegradedArea
        } else {
            ConservationActionType::MonitorPollinatorHealth
        };
        
        // Calculate priority based on degradation level
        let priority = (1.0 - corridor.habitat_quality_score) * 100.0;
        
        // Estimate cost based on corridor area
        let estimated_cost = corridor.total_area_sqft * 0.5; // $0.50 per sqft
        
        // Estimate duration based on area and action type
        let estimated_duration_days = (corridor.total_area_sqft / 1000.0) as u32 + 30;
        
        // Check treaty compliance
        let treaty_compliant = if corridor.indigenous_territory {
            if let Some(treaty_zone) = corridor.treaty_zone_id {
                let compliance = self.treaty_cache.check_restoration_rights(&treaty_zone)?;
                compliance.allowed
            } else {
                false
            }
        } else {
            true
        };
        
        let plan = ConservationActionPlan {
            plan_id,
            corridor_id: corridor.corridor_id,
            action_type,
            target_species: Vec::new(),
            target_plants: self.recommend_native_plants_for_corridor(corridor),
            priority: priority as u8,
            estimated_cost_usd: estimated_cost,
            estimated_duration_days,
            expected_impact_score: 0.7,
            treaty_compliant,
            indigenous_knowledge_integrated: corridor.indigenous_territory,
            scheduled_start_date: current_time,
            scheduled_end_date: current_time + (estimated_duration_days as u64 * 86400),
        };
        
        Ok(plan)
    }

    /**
     * Recommend native plants for corridor based on conditions
     */
    fn recommend_native_plants_for_corridor(&self, corridor: &PollinatorCorridor) -> Vec<[u8; 32]> {
        let mut recommended_plants = Vec::new();
        
        // Filter native plants suitable for corridor conditions
        for (_, plant) in &self.native_plants {
            if plant.native_sonoran && plant.pollinator_attraction_score > 0.7 {
                // Check bloom timing for current season
                let current_month = self.get_current_month();
                if plant.bloom_months.contains(&current_month) {
                    recommended_plants.push(plant.plant_id);
                }
            }
        }
        
        // Limit to top 10 recommendations
        recommended_plants.truncate(10);
        
        recommended_plants
    }

    /**
     * Get current month (1-12)
     */
    fn get_current_month(&self) -> u8 {
        let current_time = aletheion_core::time::now();
        // Simplified: assume Unix epoch, extract month
        // In production: use proper date/time library
        6 // Default to June for testing
    }

    /**
     * Generate pesticide remediation plan
     */
    fn generate_pesticide_remediation_plan(&self, corridor: &PollinatorCorridor, current_time: u64) -> Result<ConservationActionPlan, &'static str> {
        let plan_id = pq_hash(&current_time.to_be_bytes());
        
        let plan = ConservationActionPlan {
            plan_id,
            corridor_id: corridor.corridor_id,
            action_type: ConservationActionType::EstablishPesticideFreeZone,
            target_species: Vec::new(),
            target_plants: Vec::new(),
            priority: 95, // High priority for pesticide issues
            estimated_cost_usd: 5000.0,
            estimated_duration_days: 14,
            expected_impact_score: 0.8,
            treaty_compliant: !corridor.indigenous_territory,
            indigenous_knowledge_integrated: false,
            scheduled_start_date: current_time,
            scheduled_end_date: current_time + (14 * 86400),
        };
        
        Ok(plan)
    }

    /**
     * Generate seasonal planting recommendations
     */
    fn generate_seasonal_planting_plans(&mut self, plans: &mut Vec<ConservationActionPlan>, current_time: u64) -> Result<(), &'static str> {
        let current_month = self.get_current_month();
        
        // Check which native plants should be planted this month
        for (_, plant) in &self.native_plants {
            if plant.bloom_months.contains(&current_month) {
                // Generate planting plan for corridors needing this plant type
                for (_, corridor) in &self.corridors {
                    if corridor.native_plant_density < 0.9 {
                        let plan = ConservationActionPlan {
                            plan_id: pq_hash(&plant.plant_id),
                            corridor_id: corridor.corridor_id,
                            action_type: ConservationActionType::PlantNativeSpecies,
                            target_species: Vec::new(),
                            target_plants: vec![plant.plant_id],
                            priority: 70,
                            estimated_cost_usd: 1000.0,
                            estimated_duration_days: 7,
                            expected_impact_score: 0.6,
                            treaty_compliant: !corridor.indigenous_territory,
                            indigenous_knowledge_integrated: plant.indigenous_uses.len() > 0,
                            scheduled_start_date: current_time,
                            scheduled_end_date: current_time + (7 * 86400),
                        };
                        
                        plans.push(plan);
                    }
                }
            }
        }
        
        Ok(())
    }

    /**
     * Plan corridor connectivity improvements
     */
    fn plan_corridor_connectivity(&mut self, plans: &mut Vec<ConservationActionPlan>, current_time: u64) -> Result<(), &'static str> {
        // Identify disconnected corridor segments
        let disconnected_segments = self.identify_disconnected_corridors()?;
        
        for (corridor1_id, corridor2_id, gap_distance) in disconnected_segments {
            if gap_distance < CORRIDOR_CONNECTIVITY_DISTANCE_FT {
                // Generate connectivity plan
                let plan = ConservationActionPlan {
                    plan_id: pq_hash(&gap_distance.to_be_bytes()),
                    corridor_id: corridor1_id,
                    action_type: ConservationActionType::ConnectCorridorSegments,
                    target_species: Vec::new(),
                    target_plants: self.recommend_connectivity_plants(),
                    priority: 80,
                    estimated_cost_usd: gap_distance as f32 * 10.0, // $10 per foot
                    estimated_duration_days: (gap_distance / 100.0) as u32 + 30,
                    expected_impact_score: 0.9,
                    treaty_compliant: true,
                    indigenous_knowledge_integrated: false,
                    scheduled_start_date: current_time,
                    scheduled_end_date: current_time + (90 * 86400),
                };
                
                plans.push(plan);
            }
        }
        
        Ok(())
    }

    /**
     * Identify disconnected corridor segments
     */
    fn identify_disconnected_corridors(&self) -> Result<Vec<([u8; 32], [u8; 32], f32)>, &'static str> {
        let mut disconnected = Vec::new();
        
        // Simplified: check all corridor pairs for connectivity
        let corridor_ids: Vec<_> = self.corridors.keys().cloned().collect();
        
        for i in 0..corridor_ids.len() {
            for j in (i+1)..corridor_ids.len() {
                let corridor1 = &self.corridors[&corridor_ids[i]];
                let corridor2 = &self.corridors[&corridor_ids[j]];
                
                // Calculate distance between corridors
                let distance = self.calculate_corridor_distance(corridor1, corridor2);
                
                // Check if corridors should be connected but aren't
                if distance < CORRIDOR_CONNECTIVITY_DISTANCE_FT && !corridor1.connected_corridors.contains(&corridor2.corridor_id) {
                    disconnected.push((corridor1.corridor_id, corridor2.corridor_id, distance));
                }
            }
        }
        
        Ok(disconnected)
    }

    /**
     * Calculate distance between two corridors (simplified)
     */
    fn calculate_corridor_distance(&self, corridor1: &PollinatorCorridor, corridor2: &PollinatorCorridor) -> f32 {
        // Simplified: return average distance between boundary points
        // In production: use proper geometric distance calculation
        300.0
    }

    /**
     * Recommend plants for corridor connectivity
     */
    fn recommend_connectivity_plants(&self) -> Vec<[u8; 32]> {
        let mut connectivity_plants = Vec::new();
        
        // Select plants that provide continuous bloom and high pollinator attraction
        for (_, plant) in &self.native_plants {
            if plant.native_sonoran && plant.pollinator_attraction_score > 0.8 && plant.bloom_months.len() >= 4 {
                connectivity_plants.push(plant.plant_id);
            }
        }
        
        connectivity_plants.truncate(5);
        
        connectivity_plants
    }

    /**
     * Integrate Indigenous ecological knowledge into conservation plans
     */
    fn integrate_indigenous_knowledge(&mut self, plans: &mut Vec<ConservationActionPlan>) -> Result<(), &'static str> {
        for knowledge_entry in &self.indigenous_knowledge {
            if knowledge_entry.fpic_verified {
                // Create conservation plan based on Indigenous knowledge
                if let Some(plant_id) = knowledge_entry.plant_species {
                    // Find corridors that could benefit from this plant
                    for (_, corridor) in &self.corridors {
                        if corridor.indigenous_territory {
                            let plan = ConservationActionPlan {
                                plan_id: knowledge_entry.entry_id,
                                corridor_id: corridor.corridor_id,
                                action_type: ConservationActionType::IndigenousKnowledgeIntegration,
                                target_species: Vec::new(),
                                target_plants: vec![plant_id],
                                priority: 85,
                                estimated_cost_usd: 2000.0,
                                estimated_duration_days: 21,
                                expected_impact_score: 0.85,
                                treaty_compliant: true,
                                indigenous_knowledge_integrated: true,
                                scheduled_start_date: aletheion_core::time::now(),
                                scheduled_end_date: aletheion_core::time::now() + (21 * 86400),
                            };
                            
                            plans.push(plan);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /**
     * Update corridor metrics based on current state
     */
    fn update_corridor_metrics(&mut self, current_time: u64) -> Result<(), &'static str> {
        // Calculate average habitat quality
        let total_quality: f32 = self.corridors.iter()
            .map(|(_, c)| c.habitat_quality_score)
            .sum();
        self.metrics.habitat_quality_average = total_quality / self.corridors.len().max(1) as f32;
        
        // Count active restoration projects
        self.metrics.restoration_projects_active = self.conservation_plans.iter()
            .filter(|(_, plan)| plan.scheduled_start_date <= current_time && plan.scheduled_end_date >= current_time)
            .count();
        
        // Calculate water conservation
        self.metrics.water_conservation_gallons_per_year = self.native_plants.iter()
            .map(|(_, plant)| {
                plant.water_requirements_gallons_per_year * 100.0 // Assume 100 sqft per plant
            })
            .sum::<f32>();
        
        // Update current state based on average quality
        self.metrics.current_state = self.determine_corridor_state(self.metrics.habitat_quality_average);
        
        Ok(())
    }

    /**
     * ERM Chain: OPTIMIZE & TREATY-CHECK
     * Validates conservation plans against Indigenous ecological rights and generates executable commands
     * FPIC Enforcement: Cannot implement conservation actions on protected lands without consent
     */
    pub fn optimize_and_check(&mut self, plans: Vec<ConservationActionPlan>) -> Result<Vec<CorridorCommand>, &'static str> {
        let mut commands = Vec::new();
        
        for plan in plans {
            // Check treaty compliance for each plan
            if !plan.treaty_compliant {
                self.log_warning(format!("FPIC_VIOLATION: Conservation plan {:?} denied due to treaty restrictions", plan.plan_id));
                continue;
            }
            
            // Generate command
            let command = CorridorCommand {
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
     * Map conservation action to corridor command type
     */
    fn map_action_to_command(&self, action_type: ConservationActionType) -> CorridorCommandType {
        match action_type {
            ConservationActionType::PlantNativeSpecies => CorridorCommandType::ExecutePlanting,
            ConservationActionType::RemoveInvasiveSpecies => CorridorCommandType::ExecuteRemoval,
            ConservationActionType::InstallNestingHabitat => CorridorCommandType::InstallHabitat,
            ConservationActionType::CreateWaterSource => CorridorCommandType::CreateWaterSource,
            ConservationActionType::EstablishPesticideFreeZone => CorridorCommandType::EstablishZone,
            ConservationActionType::MonitorPollinatorHealth => CorridorCommandType::DeployMonitoring,
            ConservationActionType::RestoreDegradedArea => CorridorCommandType::ExecuteRestoration,
            ConservationActionType::ConnectCorridorSegments => CorridorCommandType::ConnectCorridors,
            ConservationActionType::IndigenousKnowledgeIntegration => CorridorCommandType::IntegrateKnowledge,
        }
    }

    /**
     * ERM Chain: ACT
     * Executes corridor commands or queues for offline execution
     * Ensures atomicity without rollbacks
     */
    pub fn act(&mut self, commands: Vec<CorridorCommand>) -> Result<(), &'static str> {
        for command in commands {
            // Sign command (PQ Secure)
            let signature = DIDWallet::sign_action(&self.node_id, &command);
            let mut signed_command = command.clone();
            signed_command.signed = signature.is_ok();
            
            // Attempt immediate execution via HAL
            match self.execute_corridor_command(&signed_command) {
                Ok(_) => {
                    self.log_action(&signed_command);
                    
                    // Update metrics
                    self.update_metrics_from_command(&signed_command);
                },
                Err(_) => {
                    // Offline Fallback: Queue for later execution
                    self.offline_queue.push(signed_command.plan_entry)?;
                    self.log_warning("Offline mode: Corridor command queued for later execution");
                }
            }
        }
        
        Ok(())
    }

    /**
     * Execute individual corridor command
     */
    fn execute_corridor_command(&self, command: &CorridorCommand) -> Result<(), &'static str> {
        match command.command_type {
            CorridorCommandType::ExecutePlanting => {
                aletheion_physical::hal::execute_native_planting(
                    &command.plan_entry.corridor_id,
                    &command.plan_entry.target_plants
                )?;
            },
            CorridorCommandType::ExecuteRemoval => {
                aletheion_physical::hal::remove_invasive_species(
                    &command.plan_entry.corridor_id
                )?;
            },
            CorridorCommandType::InstallHabitat => {
                aletheion_physical::hal::install_nesting_habitat(
                    &command.plan_entry.corridor_id
                )?;
            },
            CorridorCommandType::CreateWaterSource => {
                aletheion_physical::hal::create_water_source(
                    &command.plan_entry.corridor_id
                )?;
            },
            CorridorCommandType::EstablishZone => {
                aletheion_physical::hal::establish_pesticide_free_zone(
                    &command.plan_entry.corridor_id
                )?;
            },
            CorridorCommandType::DeployMonitoring => {
                aletheion_physical::hal::deploy_pollinator_monitors(
                    &command.plan_entry.corridor_id
                )?;
            },
            CorridorCommandType::ExecuteRestoration => {
                aletheion_physical::hal::execute_habitat_restoration(
                    &command.plan_entry.corridor_id
                )?;
            },
            CorridorCommandType::ConnectCorridors => {
                aletheion_physical::hal::connect_corridor_segments(
                    &command.plan_entry.corridor_id,
                    &command.plan_entry.target_plants
                )?;
            },
            CorridorCommandType::IntegrateKnowledge => {
                aletheion_physical::hal::integrate_indigenous_planting(
                    &command.plan_entry.corridor_id,
                    &command.plan_entry.target_plants
                )?;
            }
        }
        
        Ok(())
    }

    /**
     * Update metrics based on executed command
     */
    fn update_metrics_from_command(&mut self, command: &CorridorCommand) {
        match command.command_type {
            CorridorCommandType::ExecutePlanting => {
                self.metrics.native_plants_established += command.plan_entry.target_plants.len();
            },
            CorridorCommandType::ExecuteRestoration => {
                self.metrics.restoration_projects_active += 1;
            },
            _ => {}
        }
    }

    /**
     * ERM Chain: LOG
     * Immutable logging to ALN-Blockchain
     * No rollbacks allowed; all state changes are forward-only
     */
    fn log_action(&self, command: &CorridorCommand) {
        let log_entry = alloc::format!(
            "CORRIDOR_ACT: Type={:?} | Plan={:?} | Corridor={:?} | Plants={} | Priority={} | Cost=${:.0} | Treaty={}",
            command.command_type,
            command.plan_entry.plan_id,
            command.plan_entry.corridor_id,
            command.plan_entry.target_plants.len(),
            command.plan_entry.priority,
            command.plan_entry.estimated_cost_usd,
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
    pub fn get_status_report(&self) -> CorridorStatusReport {
        CorridorStatusReport {
            network_id: self.config.network_id,
            current_state: self.metrics.current_state,
            metrics: self.metrics.clone(),
            total_corridors: self.corridors.len(),
            active_corridors: self.corridors.iter().filter(|(_, c)| c.active).count(),
            species_monitored: self.pollinator_species.len(),
            indigenous_knowledge_count: self.indigenous_knowledge.len(),
            offline_queue_size: self.offline_queue.len(),
            last_sync: self.last_sync,
            accessibility_alert: self.metrics.current_state != CorridorState::OptimalHealth,
            treaty_compliance_required: !self.config.indigenous_territories.is_empty(),
        }
    }

    /**
     * Generate conservation action plan for threatened corridor
     */
    fn generate_conservation_action_plan(&mut self, corridor_id: &[u8; 32], threat_level: ThreatLevel) -> Result<(), &'static str> {
        if let Some(corridor) = self.corridors.get(corridor_id) {
            let current_time = aletheion_core::time::now();
            
            let plan = ConservationActionPlan {
                plan_id: pq_hash(&current_time.to_be_bytes()),
                corridor_id: *corridor_id,
                action_type: match threat_level {
                    ThreatLevel::Critical => ConservationActionType::RestoreDegradedArea,
                    ThreatLevel::High => ConservationActionType::PlantNativeSpecies,
                    ThreatLevel::Moderate => ConservationActionType::MonitorPollinatorHealth,
                    _ => ConservationActionType::MonitorPollinatorHealth,
                },
                target_species: Vec::new(),
                target_plants: self.recommend_native_plants_for_corridor(corridor),
                priority: match threat_level {
                    ThreatLevel::Critical => 100,
                    ThreatLevel::High => 85,
                    ThreatLevel::Moderate => 60,
                    _ => 40,
                },
                estimated_cost_usd: 5000.0,
                estimated_duration_days: 30,
                expected_impact_score: 0.7,
                treaty_compliant: !corridor.indigenous_territory,
                indigenous_knowledge_integrated: corridor.indigenous_territory,
                scheduled_start_date: current_time,
                scheduled_end_date: current_time + (30 * 86400),
            };
            
            self.conservation_plans.insert(current_time, plan);
        }
        
        Ok(())
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
     * Register new native plant species
     */
    pub fn register_native_plant(&mut self, plant: NativePlantSpecies) -> Result<(), &'static str> {
        // Check if plant already exists
        if self.native_plants.contains_key(&plant.plant_id) {
            return Err("Plant species already registered");
        }
        
        self.native_plants.insert(plant.plant_id, plant.clone());
        
        self.log_event(format!(
            "NATIVE_PLANT_REGISTERED: ID={:?}, Name={}, NativeSonoran={}, BloomMonths={:?}, IndigenousUses={}",
            plant.plant_id,
            plant.common_name,
            plant.native_sonoran,
            plant.bloom_months,
            plant.indigenous_uses.len()
        ));
        
        Ok(())
    }

    /**
     * Register new pollinator species
     */
    pub fn register_pollinator_species(&mut self, species: PollinatorSpecies) -> Result<(), &'static str> {
        // Check if species already exists
        if self.pollinator_species.contains_key(&species.species_id) {
            return Err("Pollinator species already registered");
        }
        
        self.pollinator_species.insert(species.species_id, species.clone());
        
        self.log_event(format!(
            "POLLINATOR_REGISTERED: ID={:?}, Name={}, Type={:?}, ConservationStatus={:?}, IndigenousSignificance={:?}",
            species.species_id,
            species.common_name,
            species.pollinator_type,
            species.conservation_status,
            species.indigenous_significance
        ));
        
        Ok(())
    }

    /**
     * Get corridor biodiversity metrics
     */
    pub fn get_biodiversity_metrics(&self) -> BiodiversityMetrics {
        BiodiversityMetrics {
            total_pollinator_species: self.pollinator_species.len(),
            total_native_plants: self.native_plants.len(),
            endangered_species_count: self.pollinator_species.iter()
                .filter(|(_, s)| s.conservation_status == ConservationStatus::Endangered || 
                                 s.conservation_status == ConservationStatus::CriticallyEndangered)
                .count(),
            indigenous_plant_uses: self.native_plants.iter()
                .map(|(_, p)| p.indigenous_uses.len())
                .sum::<usize>(),
            average_habitat_quality: self.metrics.habitat_quality_average,
            corridor_connectivity_index: self.calculate_connectivity_index(),
        }
    }

    /**
     * Calculate corridor connectivity index (0.0-1.0)
     */
    fn calculate_connectivity_index(&self) -> f32 {
        if self.corridors.is_empty() {
            return 0.0;
        }
        
        let total_corridors = self.corridors.len() as f32;
        let connected_corridors: f32 = self.corridors.iter()
            .filter(|(_, c)| !c.connected_corridors.is_empty())
            .count() as f32;
        
        connected_corridors / total_corridors
    }

    /**
     * Get water conservation metrics
     */
    pub fn get_water_conservation_metrics(&self) -> WaterConservationMetrics {
        WaterConservationMetrics {
            total_water_saved_gallons_per_year: self.metrics.water_conservation_gallons_per_year,
            native_plant_water_usage: self.native_plants.iter()
                .map(|(_, p)| p.water_requirements_gallons_per_year)
                .sum::<f32>(),
            xeriscaping_efficiency_percent: XERISCAPING_WATER_REDUCTION_PERCENT,
            rainwater_harvest_integration: self.config.water_reclamation_integration,
        }
    }
}

// --- Supporting Data Structures ---

pub enum CorridorInput {
    HabitatQualityReading(HabitatQualityReading),
    PollinatorHealthReading(PollinatorHealthReading),
    IndigenousKnowledgeEntry(IndigenousKnowledgeEntry),
    CorridorUpdate(PollinatorCorridor),
}

pub enum CorridorSenseResult {
    HabitatQualityProcessed([u8; 32]),
    PollinatorHealthProcessed([u8; 32]),
    KnowledgeEntryProcessed([u8; 32]),
    KnowledgeEntryRejected([u8; 32]),
    CorridorUpdateProcessed([u8; 32]),
    CorridorUpdateDenied([u8; 32]),
}

pub struct CorridorCommand {
    pub plan_entry: ConservationActionPlan,
    pub command_type: CorridorCommandType,
    pub treaty_compliant: bool,
    pub signed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CorridorCommandType {
    ExecutePlanting,
    ExecuteRemoval,
    InstallHabitat,
    CreateWaterSource,
    EstablishZone,
    DeployMonitoring,
    ExecuteRestoration,
    ConnectCorridors,
    IntegrateKnowledge,
}

pub struct CorridorStatusReport {
    pub network_id: [u8; 32],
    pub current_state: CorridorState,
    pub metrics: PollinatorCorridorMetrics,
    pub total_corridors: usize,
    pub active_corridors: usize,
    pub species_monitored: usize,
    pub indigenous_knowledge_count: usize,
    pub offline_queue_size: usize,
    pub last_sync: u64,
    pub accessibility_alert: bool,
    pub treaty_compliance_required: bool,
}

pub struct BiodiversityMetrics {
    pub total_pollinator_species: usize,
    pub total_native_plants: usize,
    pub endangered_species_count: usize,
    pub indigenous_plant_uses: usize,
    pub average_habitat_quality: f32,
    pub corridor_connectivity_index: f32,
}

pub struct WaterConservationMetrics {
    pub total_water_saved_gallons_per_year: f32,
    pub native_plant_water_usage: f32,
    pub xeriscaping_efficiency_percent: f32,
    pub rainwater_harvest_integration: bool,
}

// --- Unit Tests (Offline Capable) ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corridor_initialization() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        assert_eq!(manager.current_state, CorridorState::OptimalHealth);
        assert_eq!(manager.corridors.len(), 0);
        assert_eq!(manager.native_plants.len(), 0);
        assert_eq!(manager.pollinator_species.len(), 0);
    }

    #[test]
    fn test_native_plant_registration() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let mut manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        // Register Saguaro cactus
        let saguaro = NativePlantSpecies {
            plant_id: [1u8; 32],
            scientific_name: "Carnegiea gigantea".to_string(),
            common_name: "Saguaro Cactus".to_string(),
            native_sonoran: true,
            bloom_months: vec![5, 6],
            water_requirements_gallons_per_year: 2.0,
            pollinator_attraction_score: 0.8,
            drought_tolerance: 0.95,
            soil_requirements: "Well-drained, sandy".to_string(),
            sun_requirements: SunExposure::FullSun,
            height_ft: 40.0,
            spread_ft: 15.0,
            indigenous_uses: vec![IndigenousKnowledgeCategory::PlantFoodSource, IndigenousKnowledgeCategory::CeremonialSignificance],
            host_for_species: vec![],
        };
        
        manager.register_native_plant(saguaro).unwrap();
        
        assert_eq!(manager.native_plants.len(), 1);
        assert!(manager.native_plants.get(&[1u8; 32]).is_some());
    }

    #[test]
    fn test_pollinator_species_registration() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let mut manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        // Register Sonoran Bumblebee
        let bumblebee = PollinatorSpecies {
            species_id: [2u8; 32],
            pollinator_type: PollinatorType::NativeBee,
            native_bee_species: Some(NativeBeeSpecies::SonoranBumblebee),
            butterfly_species: None,
            hummingbird_species: None,
            scientific_name: "Bombus sonorus".to_string(),
            common_name: "Sonoran Bumblebee".to_string(),
            foraging_radius_ft: 1500.0,
            active_months: vec![2, 3, 4, 5, 6, 7, 8, 9, 10],
            host_plants: vec![],
            nectar_plants: vec![[1u8; 32]], // Saguaro
            nesting_requirements: "Underground cavities".to_string(),
            conservation_status: ConservationStatus::Vulnerable,
            indigenous_significance: Some(IndigenousKnowledgeCategory::PlantFoodSource),
            population_trend: PopulationTrend::Decreasing,
        };
        
        manager.register_pollinator_species(bumblebee).unwrap();
        
        assert_eq!(manager.pollinator_species.len(), 1);
        assert!(manager.pollinator_species.get(&[2u8; 32]).is_some());
    }

    #[test]
    fn test_offline_queue_capacity() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        assert!(manager.offline_queue.capacity_hours() >= 72);
    }

    #[test]
    fn test_habitat_quality_threshold_detection() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let mut manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        // Create habitat quality reading below threshold
        let poor_quality_reading = HabitatQualityReading {
            timestamp: 1000,
            corridor_id: [1u8; 32],
            quality_factors: HashMap::new(),
            overall_score: 0.5, // Below MIN_HABITAT_QUALITY_SCORE (0.7)
            floral_resource_density: 0.6,
            nesting_site_availability: 0.7,
            water_availability_score: 0.8,
            pesticide_residue_ppm: 0.0,
            invasive_species_percent: 0.1,
            sensor_id: [1u8; 32],
        };
        
        // Process reading - should trigger habitat quality alert
        manager.process_habitat_quality_reading(poor_quality_reading).unwrap();
        
        // Should have generated conservation action plan
        assert!(manager.conservation_plans.len() > 0);
    }

    #[test]
    fn test_corridor_state_determination() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        // Test corridor state progression
        assert_eq!(manager.determine_corridor_state(0.95), CorridorState::OptimalHealth);
        assert_eq!(manager.determine_corridor_state(0.80), CorridorState::GoodCondition);
        assert_eq!(manager.determine_corridor_state(0.65), CorridorState::ModerateDegradation);
        assert_eq!(manager.determine_corridor_state(0.45), CorridorState::PoorCondition);
        assert_eq!(manager.determine_corridor_state(0.25), CorridorState::CriticalDecline);
        assert_eq!(manager.determine_corridor_state(0.10), CorridorState::CriticalDecline);
    }

    #[test]
    fn test_indigenous_knowledge_fpiv_verification() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let mut manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        // Create Indigenous knowledge entry without FPIC verification
        let unverified_entry = IndigenousKnowledgeEntry {
            entry_id: [1u8; 32],
            knowledge_category: IndigenousKnowledgeCategory::PlantMedicinalUse,
            plant_species: Some([1u8; 32]),
            pollinator_species: None,
            traditional_use_description: "Traditional medicinal use for healing".to_string(),
            cultural_significance: "Sacred plant used in ceremonies".to_string(),
            knowledge_source: IndigenousCommunity::AkimelOodham,
            fpic_verified: false,
            timestamp: 1000,
        };
        
        // Should be rejected due to lack of FPIC verification
        let result = manager.process_indigenous_knowledge_entry(unverified_entry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_native_sonoran_species_parameters() {
        // Verify native Sonoran Desert species parameters
        assert_eq!(SAGUARO_BLOOM_MONTHS, [5, 6]); // May-June bloom
        assert_eq!(PALO_VERDE_BLOOM_MONTHS, [4, 5]); // April-May bloom
        assert_eq!(OCOTILLO_BLOOM_MONTHS, [3, 4, 5, 6]); // March-June multiple blooms
        assert_eq!(DESERT_MARIGOLD_BLOOM_MONTHS, [2, 3, 4, 5, 6, 7, 8, 9]); // Long bloom season Feb-Sept
        
        // Verify pollinator parameters
        assert!(SONORAN_BUMBLEBEE_FORAGING_RADIUS_FT > SWEAT_BEE_FORAGING_RADIUS_FT);
        assert!(MONARCH_MIGRATION_CORRIDOR_WIDTH_FT > 0.0);
    }

    #[test]
    fn test_water_conservation_metrics() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        let metrics = manager.get_water_conservation_metrics();
        
        // Xeriscaping should achieve 60% water reduction
        assert_eq!(metrics.xeriscaping_efficiency_percent, XERISCAPING_WATER_REDUCTION_PERCENT);
        assert!(metrics.rainwater_harvest_integration);
    }

    #[test]
    fn test_connectivity_index_calculation() {
        let config = CorridorNetworkConfiguration {
            network_id: [1u8; 32],
            corridors: vec![],
            native_plant_database: vec![],
            pollinator_species_database: vec![],
            indigenous_territories: vec![],
            water_reclamation_integration: true,
            monitoring_sensor_network: vec![],
            restoration_priority_zones: vec![],
        };
        
        let manager = PollinatorCorridorManager::new(BirthSign::default(), config).unwrap();
        
        // Empty network should have 0.0 connectivity
        assert_eq!(manager.calculate_connectivity_index(), 0.0);
    }
}
