#ifndef DISTRICT_ENERGY_HPP
#define DISTRICT_ENERGY_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t DISTRICT_ENERGY_VERSION = 20260310;
constexpr size_t MAX_ENERGY_ZONES = 512;
constexpr size_t MAX_GENERATION_UNITS = 2048;
constexpr size_t MAX_STORAGE_SYSTEMS = 1024;
constexpr size_t MAX_CONSUMERS = 131072;
constexpr double PHOENIX_PEAK_DEMAND_MW = 3500.0;
constexpr double RENEWABLE_TARGET_PCT = 0.75;
constexpr double GRID_STABILITY_THRESHOLD = 0.95;

enum class GenerationType : uint8_t {
    SOLAR_PV = 0, SOLAR_THERMAL = 1, WIND = 2, BATTERY = 3,
    NATURAL_GAS = 4, GEOTHERMAL = 5, BIOMASS = 6, HYDROGEN = 7,
    NUCLEAR = 8, COAL = 9, DIESEL = 10, FUEL_CELL = 11
};

enum class EnergyZoneStatus : uint8_t {
    NORMAL = 0, HIGH_DEMAND = 1, LOW_SUPPLY = 2, CRITICAL = 3,
    ISLANDED = 4, BLACKOUT = 5, RESTORING = 6, STABLE = 7
};

struct GenerationUnit {
    uint64_t unit_id;
    GenerationType generation_type;
    char unit_name[128];
    double capacity_mw;
    double current_output_mw;
    double efficiency_pct;
    double capacity_factor;
    uint64_t installation_date_ns;
    uint64_t last_maintenance_ns;
    uint64_t next_maintenance_ns;
    bool operational;
    bool renewable;
    bool dispatchable;
    uint32_t energy_zone_id;
    double carbon_intensity_g_co2_kwh;
    double marginal_cost_usd_mwh;
};

struct EnergyStorage {
    uint64_t storage_id;
    char storage_type[32];
    double capacity_mwh;
    double current_soc_mwh;
    double max_charge_rate_mw;
    double max_discharge_rate_mw;
    double round_trip_efficiency;
    uint64_t cycle_count;
    double state_of_health;
    uint64_t installation_date_ns;
    bool operational;
    bool grid_services_capable;
    bool emergency_backup;
    uint32_t energy_zone_id;
};

struct EnergyConsumer {
    uint64_t consumer_id;
    char consumer_type[32];
    double average_demand_kw;
    double peak_demand_kw;
    double current_demand_kw;
    uint64_t billing_account_id;
    uint32_t energy_zone_id;
    bool demand_response_enrolled;
    bool time_of_use_pricing;
    bool solar_equipped;
    bool battery_equipped;
    bool low_income_assistance;
    double monthly_consumption_kwh;
    double target_consumption_kwh;
    bool target_compliant;
};

struct EnergyZone {
    uint32_t zone_id;
    char zone_name[64];
    double total_generation_mw;
    double total_consumption_mw;
    double net_balance_mw;
    double frequency_hz;
    double voltage_v;
    EnergyZoneStatus status;
    uint32_t generation_unit_count;
    uint32_t storage_system_count;
    uint32_t consumer_count;
    double renewable_percentage;
    bool island_capable;
    bool is_islanded;
    uint64_t last_optimization_ns;
    double stability_score;
};

class DistrictEnergyManagementSystem {
private:
    uint64_t system_id_;
    char city_code_[8];
    EnergyZone zones_[MAX_ENERGY_ZONES];
    size_t zone_count_;
    GenerationUnit generators_[MAX_GENERATION_UNITS];
    size_t generator_count_;
    EnergyStorage storage_[MAX_STORAGE_SYSTEMS];
    size_t storage_count_;
    EnergyConsumer consumers_[MAX_CONSUMERS];
    size_t consumer_count_;
    double total_generation_mw_;
    double total_consumption_mw_;
    double total_renewable_mw_;
    double system_frequency_hz_;
    double grid_stability_index_;
    double renewable_percentage_;
    uint64_t blackout_events_;
    uint64_t last_blackout_ns_;
    double average_restoration_time_min_;
    uint64_t demand_response_events_;
    double carbon_intensity_g_co2_kwh_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= zone_count_ * generator_count_ * storage_count_;
        sum ^= static_cast<uint64_t>(total_generation_mw_ * 1000);
        sum ^= blackout_events_;
        for (size_t i = 0; i < generator_count_; ++i) {
            sum ^= generators_[i].unit_id * static_cast<uint64_t>(generators_[i].operational);
        }
        audit_checksum_ = sum;
    }
    
    double ComputeRenewablePercentage() {
        if (total_generation_mw_ == 0.0) return 0.0;
        return total_renewable_mw_ / total_generation_mw_;
    }
    
    double ComputeCarbonIntensity() {
        double weighted_carbon = 0.0;
        double total_output = 0.0;
        for (size_t i = 0; i < generator_count_; ++i) {
            if (generators_[i].operational) {
                weighted_carbon += generators_[i].current_output_mw * generators_[i].carbon_intensity_g_co2_kwh;
                total_output += generators_[i].current_output_mw;
            }
        }
        if (total_output == 0.0) return 0.0;
        return weighted_carbon / total_output;
    }
    
public:
    DistrictEnergyManagementSystem(uint64_t system_id, const char* city_code, uint64_t init_ns)
        : system_id_(system_id), zone_count_(0), generator_count_(0),
          storage_count_(0), consumer_count_(0), total_generation_mw_(0.0),
          total_consumption_mw_(0.0), total_renewable_mw_(0.0),
          system_frequency_hz_(60.0), grid_stability_index_(1.0),
          renewable_percentage_(0.0), blackout_events_(0), last_blackout_ns_(0),
          average_restoration_time_min_(0.0), demand_response_events_(0),
          carbon_intensity_g_co2_kwh_(0.0), audit_checksum_(0),
          last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterGenerationUnit(const GenerationUnit& unit) {
        if (generator_count_ >= MAX_GENERATION_UNITS) return false;
        generators_[generator_count_] = unit;
        if (unit.operational) {
            total_generation_mw_ += unit.current_output_mw;
            if (unit.renewable) total_renewable_mw_ += unit.current_output_mw;
        }
        generator_count_++;
        renewable_percentage_ = ComputeRenewablePercentage();
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterEnergyStorage(const EnergyStorage& storage) {
        if (storage_count_ >= MAX_STORAGE_SYSTEMS) return false;
        storage_[storage_count_] = storage;
        storage_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterEnergyConsumer(const EnergyConsumer& consumer) {
        if (consumer_count_ >= MAX_CONSUMERS) return false;
        consumers_[consumer_count_] = consumer;
        total_consumption_mw_ += consumer.current_demand_kw / 1000.0;
        consumer_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterEnergyZone(const EnergyZone& zone) {
        if (zone_count_ >= MAX_ENERGY_ZONES) return false;
        zones_[zone_count_] = zone;
        zone_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    void UpdateConsumerDemand(uint64_t consumer_id, double demand_kw, uint64_t now_ns) {
        for (size_t i = 0; i < consumer_count_; ++i) {
            if (consumers_[i].consumer_id == consumer_id) {
                double delta = (demand_kw - consumers_[i].current_demand_kw) / 1000.0;
                consumers_[i].current_demand_kw = demand_kw;
                total_consumption_mw_ += delta;
                consumers_[i].target_compliant = demand_kw <= consumers_[i].target_consumption_kwh;
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    void OptimizeEnergyDispatch(uint64_t now_ns) {
        double demand = total_consumption_mw_;
        double supply = total_generation_mw_;
        double imbalance = supply - demand;
        if (std::abs(imbalance) > supply * 0.05) {
            system_frequency_hz_ = 60.0 + (imbalance / 100.0) * 0.1;
        }
        if (std::abs(system_frequency_hz_ - 60.0) > 0.5) {
            blackout_events_++;
            last_blackout_ns_ = now_ns;
        }
        grid_stability_index_ = 1.0 - (std::abs(system_frequency_hz_ - 60.0) / 0.5);
        renewable_percentage_ = ComputeRenewablePercentage();
        carbon_intensity_g_co2_kwh_ = ComputeCarbonIntensity();
        last_optimization_ns_ = now_ns;
        UpdateAuditChecksum();
    }
    
    void ActivateDemandResponse(uint64_t zone_id, uint64_t now_ns) {
        for (size_t i = 0; i < consumer_count_; ++i) {
            if (consumers_[i].energy_zone_id == zone_id && 
                consumers_[i].demand_response_enrolled) {
                consumers_[i].current_demand_kw *= 0.8;
                total_consumption_mw_ -= consumers_[i].average_demand_kw * 0.2 / 1000.0;
            }
        }
        demand_response_events_++;
        UpdateAuditChecksum();
    }
    
    double ComputeGridStabilityIndex() {
        double frequency_stability = 1.0 - (std::abs(system_frequency_hz_ - 60.0) / 0.5);
        double supply_demand_balance = 1.0 - (std::abs(total_generation_mw_ - total_consumption_mw_) / 
                                              total_generation_mw_.max(0.001));
        double renewable_stability = renewable_percentage_ >= RENEWABLE_TARGET_PCT ? 1.0 : 
                                     renewable_percentage_ / RENEWABLE_TARGET_PCT;
        grid_stability_index_ = (frequency_stability * 0.4 + supply_demand_balance * 0.35 + 
                                 renewable_stability * 0.25).max(0.0);
        return grid_stability_index_;
    }
    
    struct SystemStatus {
        uint64_t system_id;
        char city_code[8];
        size_t total_zones;
        size_t stable_zones;
        size_t total_generators;
        size_t operational_generators;
        size_t total_storage;
        size_t operational_storage;
        size_t total_consumers;
        double total_generation_mw;
        double total_consumption_mw;
        double net_balance_mw;
        double system_frequency_hz;
        double renewable_percentage;
        double carbon_intensity_g_co2_kwh;
        double grid_stability_index;
        uint64_t blackout_events;
        uint64_t demand_response_events;
        uint64_t last_blackout_ns;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    SystemStatus GetStatus(uint64_t now_ns) {
        SystemStatus status;
        status.system_id = system_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_zones = zone_count_;
        status.stable_zones = 0;
        for (size_t i = 0; i < zone_count_; ++i) {
            if (zones_[i].status == EnergyZoneStatus::STABLE || 
                zones_[i].status == EnergyZoneStatus::NORMAL) {
                status.stable_zones++;
            }
        }
        status.total_generators = generator_count_;
        status.operational_generators = 0;
        for (size_t i = 0; i < generator_count_; ++i) {
            if (generators_[i].operational) status.operational_generators++;
        }
        status.total_storage = storage_count_;
        status.operational_storage = 0;
        for (size_t i = 0; i < storage_count_; ++i) {
            if (storage_[i].operational) status.operational_storage++;
        }
        status.total_consumers = consumer_count_;
        status.total_generation_mw = total_generation_mw_;
        status.total_consumption_mw = total_consumption_mw_;
        status.net_balance_mw = total_generation_mw_ - total_consumption_mw_;
        status.system_frequency_hz = system_frequency_hz_;
        status.renewable_percentage = renewable_percentage_;
        status.carbon_intensity_g_co2_kwh = carbon_intensity_g_co2_kwh_;
        status.grid_stability_index = ComputeGridStabilityIndex();
        status.blackout_events = blackout_events_;
        status.demand_response_events = demand_response_events_;
        status.last_blackout_ns = last_blackout_ns_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    double ComputeEnergyResilienceIndex() {
        SystemStatus status = GetStatus(last_optimization_ns_);
        double generation_availability = status.operational_generators / status.total_generators.max(1);
        double storage_readiness = status.operational_storage / status.total_storage.max(1);
        double stability_score = status.grid_stability_index;
        double renewable_score = status.renewable_percentage;
        double blackout_penalty = status.blackout_events > 0 ? 0.15 : 0.0;
        return (generation_availability * 0.30 + storage_readiness * 0.25 + 
                stability_score * 0.25 + renewable_score * 0.20 - blackout_penalty).max(0.0);
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= zone_count_ * generator_count_ * storage_count_;
        sum ^= static_cast<uint64_t>(total_generation_mw_ * 1000);
        sum ^= blackout_events_;
        for (size_t i = 0; i < generator_count_; ++i) {
            sum ^= generators_[i].unit_id * static_cast<uint64_t>(generators_[i].operational);
        }
        return sum == audit_checksum_;
    }
};

#endif
