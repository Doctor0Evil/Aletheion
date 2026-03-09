#ifndef AQUIFER_MANAGER_HPP
#define AQUIFER_MANAGER_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t AQUIFER_MANAGER_VERSION = 20260310;
constexpr size_t MAX_AQUIFER_ZONES = 256;
constexpr size_t MAX_RECHARGE_WELLS = 1024;
constexpr size_t MAX_MONITORING_POINTS = 2048;
constexpr size_t MAX_WATER_QUALITY_SAMPLES = 4096;
constexpr double PHOENIX_BASIN_AREA_KM2 = 15700.0;
constexpr double SAFE_YIELD_MCM_YEAR = 350.0;
constexpr double MIN_WATER_TABLE_DEPTH_M = 30.0;
constexpr double TARGET_WATER_TABLE_DEPTH_M = 50.0;

enum class WaterSource : uint8_t {
    STORMWATER = 0, RECLAIMED = 1, RIVER = 2, GROUNDWATER = 3,
    DESALINATED = 4, ATMOSPHERIC = 5, IMPORTED = 6
};

enum class WaterQualityGrade : uint8_t {
    EXCELLENT = 0, GOOD = 1, ACCEPTABLE = 2, MARGINAL = 3, POOR = 4, HAZARDOUS = 5
};

struct AquiferZone {
    uint64_t zone_id;
    char zone_name[64];
    double area_km2;
    double avg_depth_m;
    double water_table_depth_m;
    double storage_capacity_mcm;
    double current_storage_mcm;
    double recharge_rate_m_year;
    double extraction_rate_m_year;
    double subsidence_mm_year;
    WaterQualityGrade quality_grade;
    uint64_t last_survey_ns;
    bool protected_status;
    bool indigenous_territory;
};

struct RechargeWell {
    uint64_t well_id;
    uint32_t aquifer_zone_id;
    double latitude;
    double longitude;
    double depth_m;
    double diameter_m;
    double max_injection_rate_m3h;
    double current_injection_rate_m3h;
    double cumulative_injected_m3;
    WaterSource water_source;
    WaterQualityGrade input_quality;
    uint64_t installation_date_ns;
    uint64_t last_maintenance_ns;
    bool operational;
    bool clogging_detected;
    double clogging_index_0_1;
};

struct MonitoringPoint {
    uint64_t point_id;
    uint32_t aquifer_zone_id;
    double latitude;
    double longitude;
    double depth_m;
    double water_level_m;
    double temperature_celsius;
    double electrical_conductivity_uscm;
    double ph;
    double dissolved_oxygen_mgl;
    double nitrate_mgl;
    double pfas_ngl;
    double pharmaceuticals_ngl;
    uint64_t last_reading_ns;
    bool operational;
    bool telemetry_active;
};

struct WaterQualitySample {
    uint64_t sample_id;
    uint64_t source_id;
    WaterSource source_type;
    uint64_t collected_ns;
    uint64_t analyzed_ns;
    double turbidity_ntu;
    double total_dissolved_solids_mgl;
    double chemical_oxygen_demand_mgl;
    double biological_oxygen_demand_mgl;
    double e_coli_cfu_100ml;
    double pfas_total_ngl;
    double pharmaceuticals_total_ngl;
    double heavy_metals_ugl;
    double radionuclides_pci_l;
    WaterQualityGrade grade;
    bool approved_for_recharge;
};

class AquiferRechargeManager {
private:
    uint64_t manager_id_;
    char city_code_[8];
    AquiferZone zones_[MAX_AQUIFER_ZONES];
    size_t zone_count_;
    RechargeWell wells_[MAX_RECHARGE_WELLS];
    size_t well_count_;
    MonitoringPoint monitoring_[MAX_MONITORING_POINTS];
    size_t monitoring_count_;
    WaterQualitySample samples_[MAX_WATER_QUALITY_SAMPLES];
    size_t sample_count_;
    double total_recharge_volume_m3_;
    double total_extraction_volume_m3_;
    double net_balance_m3_;
    double avg_water_table_change_m_year_;
    double total_subsidence_mm_;
    uint64_t contamination_events_;
    uint64_t last_contamination_ns_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= zone_count_ * well_count_ * monitoring_count_;
        sum ^= static_cast<uint64_t>(total_recharge_volume_m3_);
        sum ^= static_cast<uint64_t>(total_extraction_volume_m3_);
        sum ^= contamination_events_;
        for (size_t i = 0; i < zone_count_; ++i) {
            sum ^= zones_[i].zone_id * static_cast<uint64_t>(zones_[i].protected_status);
        }
        audit_checksum_ = sum;
    }
    
    WaterQualityGrade ComputeQualityGrade(const WaterQualitySample& sample) {
        if (sample.e_coli_cfu_100ml > 0 || sample.pfas_total_ngl > 10.0 || 
            sample.pharmaceuticals_total_ngl > 50.0 || sample.heavy_metals_ugl > 5.0) {
            return WaterQualityGrade::HAZARDOUS;
        }
        if (sample.turbidity_ntu > 5.0 || sample.total_dissolved_solids_mgl > 500.0) {
            return WaterQualityGrade::POOR;
        }
        if (sample.turbidity_ntu > 2.0 || sample.total_dissolved_solids_mgl > 300.0) {
            return WaterQualityGrade::MARGINAL;
        }
        if (sample.turbidity_ntu > 1.0 || sample.total_dissolved_solids_mgl > 200.0) {
            return WaterQualityGrade::ACCEPTABLE;
        }
        if (sample.turbidity_ntu > 0.5 || sample.total_dissolved_solids_mgl > 100.0) {
            return WaterQualityGrade::GOOD;
        }
        return WaterQualityGrade::EXCELLENT;
    }
    
public:
    AquiferRechargeManager(uint64_t manager_id, const char* city_code, uint64_t init_ns)
        : manager_id_(manager_id), zone_count_(0), well_count_(0),
          monitoring_count_(0), sample_count_(0), total_recharge_volume_m3_(0.0),
          total_extraction_volume_m3_(0.0), net_balance_m3_(0.0),
          avg_water_table_change_m_year_(0.0), total_subsidence_mm_(0.0),
          contamination_events_(0), last_contamination_ns_(0),
          audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterAquiferZone(const AquiferZone& zone) {
        if (zone_count_ >= MAX_AQUIFER_ZONES) return false;
        zones_[zone_count_] = zone;
        zone_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterRechargeWell(const RechargeWell& well) {
        if (well_count_ >= MAX_RECHARGE_WELLS) return false;
        wells_[well_count_] = well;
        well_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterMonitoringPoint(const MonitoringPoint& point) {
        if (monitoring_count_ >= MAX_MONITORING_POINTS) return false;
        monitoring_[monitoring_count_] = point;
        monitoring_count_++;
        return true;
    }
    
    bool RecordWaterQualitySample(const WaterQualitySample& sample) {
        if (sample_count_ >= MAX_WATER_QUALITY_SAMPLES) return false;
        WaterQualitySample recorded = sample;
        recorded.grade = ComputeQualityGrade(sample);
        recorded.approved_for_recharge = (recorded.grade <= WaterQualityGrade::ACCEPTABLE);
        samples_[sample_count_] = recorded;
        sample_count_++;
        if (recorded.grade == WaterQualityGrade::HAZARDOUS) {
            contamination_events_++;
            last_contamination_ns_ = sample.analyzed_ns;
        }
        UpdateAuditChecksum();
        return true;
    }
    
    void UpdateWellInjection(uint64_t well_id, double injection_rate_m3h, uint64_t now_ns) {
        for (size_t i = 0; i < well_count_; ++i) {
            if (wells_[i].well_id == well_id && wells_[i].operational) {
                double delta = (injection_rate_m3h - wells_[i].current_injection_rate_m3h) * 0.1;
                wells_[i].current_injection_rate_m3h = injection_rate_m3h;
                wells_[i].cumulative_injected_m3 += delta * 3600.0;
                total_recharge_volume_m3_ += delta * 3600.0;
                wells_[i].clogging_index_0_1 = fmin(1.0, wells_[i].clogging_index_0_1 + 0.001);
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    void ComputeAquiferBalance() {
        net_balance_m3_ = total_recharge_volume_m3_ - total_extraction_volume_m3_;
        double total_zone_storage = 0.0;
        double total_zone_capacity = 0.0;
        for (size_t i = 0; i < zone_count_; ++i) {
            total_zone_storage += zones_[i].current_storage_mcm;
            total_zone_capacity += zones_[i].storage_capacity_mcm;
        }
        if (total_zone_capacity > 0.0) {
            double storage_ratio = total_zone_storage / total_zone_capacity;
            if (storage_ratio < 0.5) {
                avg_water_table_change_m_year_ = -2.0;
            } else if (storage_ratio > 0.8) {
                avg_water_table_change_m_year_ = 1.5;
            } else {
                avg_water_table_change_m_year_ = 0.0;
            }
        }
    }
    
    double ComputeSustainableYield() {
        double total_recharge = 0.0;
        for (size_t i = 0; i < well_count_; ++i) {
            if (wells_[i].operational) {
                total_recharge += wells_[i].cumulative_injected_m3 / 1000000.0;
            }
        }
        double natural_recharge = PHOENIX_BASIN_AREA_KM2 * 0.05;
        double sustainable = (total_recharge + natural_recharge) * 0.9;
        return fmin(sustainable, SAFE_YIELD_MCM_YEAR);
    }
    
    bool DetectContamination(uint64_t monitoring_point_id, uint64_t now_ns) {
        for (size_t i = 0; i < monitoring_count_; ++i) {
            if (monitoring_[i].point_id == monitoring_point_id) {
                if (monitoring_[i].pfas_ngl > 10.0 || 
                    monitoring_[i].pharmaceuticals_ngl > 50.0 ||
                    monitoring_[i].nitrate_mgl > 10.0) {
                    contamination_events_++;
                    last_contamination_ns_ = now_ns;
                    UpdateAuditChecksum();
                    return true;
                }
            }
        }
        return false;
    }
    
    struct ManagerStatus {
        uint64_t manager_id;
        char city_code[8];
        size_t total_zones;
        size_t protected_zones;
        size_t total_wells;
        size_t operational_wells;
        size_t total_monitoring;
        size_t active_monitoring;
        double total_recharge_volume_m3;
        double total_extraction_volume_m3;
        double net_balance_m3;
        double avg_water_table_change_m_year;
        double sustainable_yield_mcm_year;
        double current_yield_mcm_year;
        double yield_sustainability_ratio;
        uint64_t contamination_events;
        uint64_t last_contamination_ns;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    ManagerStatus GetStatus(uint64_t now_ns) {
        ManagerStatus status;
        status.manager_id = manager_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_zones = zone_count_;
        status.protected_zones = 0;
        for (size_t i = 0; i < zone_count_; ++i) {
            if (zones_[i].protected_status) status.protected_zones++;
        }
        status.total_wells = well_count_;
        status.operational_wells = 0;
        for (size_t i = 0; i < well_count_; ++i) {
            if (wells_[i].operational) status.operational_wells++;
        }
        status.total_monitoring = monitoring_count_;
        status.active_monitoring = 0;
        for (size_t i = 0; i < monitoring_count_; ++i) {
            if (monitoring_[i].operational && monitoring_[i].telemetry_active) {
                status.active_monitoring++;
            }
        }
        status.total_recharge_volume_m3 = total_recharge_volume_m3_;
        status.total_extraction_volume_m3 = total_extraction_volume_m3_;
        status.net_balance_m3 = net_balance_m3_;
        status.avg_water_table_change_m_year = avg_water_table_change_m_year_;
        status.sustainable_yield_mcm_year = ComputeSustainableYield();
        status.current_yield_mcm_year = total_extraction_volume_m3_ / 1000000.0;
        status.yield_sustainability_ratio = status.current_yield_mcm_year / 
                                           status.sustainable_yield_mcm_year.max(0.001);
        status.contamination_events = contamination_events_;
        status.last_contamination_ns = last_contamination_ns_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= zone_count_ * well_count_ * monitoring_count_;
        sum ^= static_cast<uint64_t>(total_recharge_volume_m3_);
        sum ^= static_cast<uint64_t>(total_extraction_volume_m3_);
        sum ^= contamination_events_;
        for (size_t i = 0; i < zone_count_; ++i) {
            sum ^= zones_[i].zone_id * static_cast<uint64_t>(zones_[i].protected_status);
        }
        return sum == audit_checksum_;
    }
    
    void OptimizeRechargeSchedule(uint64_t now_ns) {
        last_optimization_ns_ = now_ns;
        ComputeAquiferBalance();
        UpdateAuditChecksum();
    }
};

#endif
