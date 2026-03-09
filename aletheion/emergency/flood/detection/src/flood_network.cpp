#ifndef FLOOD_NETWORK_HPP
#define FLOOD_NETWORK_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t FLOOD_NETWORK_VERSION = 20260310;
constexpr size_t MAX_WATER_SENSORS = 4096;
constexpr size_t MAX_WATERSHED_ZONES = 512;
constexpr size_t MAX_EVACUATION_ROUTES = 2048;
constexpr size_t MAX_FLOOD_ALERTS = 32768;
constexpr double FLASH_FLOOD_WARNING_THRESHOLD_IN = 2.0;
constexpr double CRITICAL_FLOOD_LEVEL_FT = 5.0;
constexpr double MONSOON_SEASON_START_MONTH = 6.0;
constexpr double MONSOON_SEASON_END_MONTH = 9.0;

enum class FloodAlertLevel : uint8_t {
    NORMAL = 0, ADVISORY = 1, WATCH = 2, WARNING = 3, EMERGENCY = 4, CRITICAL = 5
};

enum class SensorType : uint8_t {
    WATER_LEVEL = 0, RAINFALL = 1, FLOW_RATE = 2, SOIL_MOISTURE = 3,
    WASH_MONITOR = 4, STORM_DRAIN = 5, RIVER_GAUGE = 6, RESERVOIR = 7
};

struct WaterSensor {
    uint64_t sensor_id;
    SensorType sensor_type;
    double latitude;
    double longitude;
    double elevation_ft;
    double current_reading;
    double threshold_warning;
    double threshold_critical;
    uint64_t last_reading_ns;
    uint64_t last_maintenance_ns;
    bool operational;
    bool telemetry_active;
    uint8_t battery_pct;
    uint8_t signal_strength_dbm;
    uint32_t watershed_zone_id;
    uint64_t total_readings;
    uint64_t failed_readings;
};

struct WatershedZone {
    uint32_t zone_id;
    char zone_name[64];
    double area_km2;
    double avg_slope_pct;
    double soil_absorption_rate;
    double impervious_surface_pct;
    uint32_t sensor_count;
    uint64_t sensors[MAX_WATER_SENSORS / 512];
    FloodAlertLevel current_alert_level;
    double current_water_level_ft;
    double rainfall_accumulation_in;
    uint64_t last_flood_event_ns;
    uint32_t evacuation_route_ids[8];
    uint8_t evacuation_route_count;
    uint32_t vulnerable_population;
    bool evacuation_ordered;
    uint64_t last_evacuation_ns;
};

struct EvacuationRoute {
    uint32_t route_id;
    char route_name[128];
    double start_lat;
    double start_lon;
    double end_lat;
    double end_lon;
    double distance_miles;
    double capacity_vehicles_per_hour;
    double current_traffic_level;
    bool passable;
    bool flooded;
    double water_depth_ft;
    uint64_t last_inspection_ns;
    uint32_t shelter_destination_id;
    bool accessibility_compliant;
    bool emergency_vehicle_priority;
};

struct FloodAlert {
    uint64_t alert_id;
    FloodAlertLevel alert_level;
    uint32_t watershed_zone_id;
    uint64_t issued_at_ns;
    uint64_t expires_at_ns;
    double rainfall_in;
    double water_level_ft;
    double flow_rate_cfs;
    bool evacuation_ordered;
    bool public_notification_sent;
    bool emergency_services_notified;
    uint32_t affected_population;
    uint32_t casualties_reported;
    uint32_t structures_damaged;
    double economic_loss_usd;
};

class FlashFloodDetectionNetwork {
private:
    uint64_t network_id_;
    char city_code_[8];
    WaterSensor sensors_[MAX_WATER_SENSORS];
    size_t sensor_count_;
    WatershedZone zones_[MAX_WATERSHED_ZONES];
    size_t zone_count_;
    EvacuationRoute routes_[MAX_EVACUATION_ROUTES];
    size_t route_count_;
    FloodAlert alerts_[MAX_FLOOD_ALERTS];
    size_t alert_count_;
    uint64_t total_alerts_issued_;
    uint64_t total_evacuations_ordered_;
    uint64_t total_false_alarms_;
    double average_detection_time_min_;
    double average_response_time_min_;
    uint64_t total_casualties_;
    uint64_t total_structures_damaged_;
    double total_economic_loss_usd_;
    uint64_t last_major_flood_event_ns_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= sensor_count_ * zone_count_ * route_count_;
        sum ^= total_alerts_issued_;
        sum ^= total_evacuations_ordered_;
        sum ^= total_casualties_;
        for (size_t i = 0; i < sensor_count_; ++i) {
            sum ^= sensors_[i].sensor_id * static_cast<uint64_t>(sensors_[i].operational);
        }
        audit_checksum_ = sum;
    }
    
    bool IsMonsoonSeason(uint64_t timestamp_ns) {
        uint64_t days_since_epoch = timestamp_ns / 86400000000000ULL;
        uint64_t day_of_year = days_since_epoch % 365;
        double month = (day_of_year / 30.44) + 1.0;
        return month >= MONSOON_SEASON_START_MONTH && month <= MONSOON_SEASON_END_MONTH;
    }
    
    FloodAlertLevel ComputeAlertLevel(double rainfall_in, double water_level_ft, double flow_rate_cfs) {
        if (rainfall_in >= 4.0 || water_level_ft >= CRITICAL_FLOOD_LEVEL_FT || flow_rate_cfs >= 1000.0) {
            return FloodAlertLevel::CRITICAL;
        }
        if (rainfall_in >= 3.0 || water_level_ft >= 4.0 || flow_rate_cfs >= 750.0) {
            return FloodAlertLevel::EMERGENCY;
        }
        if (rainfall_in >= 2.0 || water_level_ft >= 3.0 || flow_rate_cfs >= 500.0) {
            return FloodAlertLevel::WARNING;
        }
        if (rainfall_in >= 1.5 || water_level_ft >= 2.0 || flow_rate_cfs >= 300.0) {
            return FloodAlertLevel::WATCH;
        }
        if (rainfall_in >= 1.0 || water_level_ft >= 1.0 || flow_rate_cfs >= 150.0) {
            return FloodAlertLevel::ADVISORY;
        }
        return FloodAlertLevel::NORMAL;
    }
    
public:
    FlashFloodDetectionNetwork(uint64_t network_id, const char* city_code, uint64_t init_ns)
        : network_id_(network_id), sensor_count_(0), zone_count_(0),
          route_count_(0), alert_count_(0), total_alerts_issued_(0),
          total_evacuations_ordered_(0), total_false_alarms_(0),
          average_detection_time_min_(0.0), average_response_time_min_(0.0),
          total_casualties_(0), total_structures_damaged_(0),
          total_economic_loss_usd_(0.0), last_major_flood_event_ns_(0),
          audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterWaterSensor(const WaterSensor& sensor) {
        if (sensor_count_ >= MAX_WATER_SENSORS) return false;
        sensors_[sensor_count_] = sensor;
        sensor_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterWatershedZone(const WatershedZone& zone) {
        if (zone_count_ >= MAX_WATERSHED_ZONES) return false;
        zones_[zone_count_] = zone;
        zone_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterEvacuationRoute(const EvacuationRoute& route) {
        if (route_count_ >= MAX_EVACUATION_ROUTES) return false;
        routes_[route_count_] = route;
        route_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    void ProcessSensorReading(uint64_t sensor_id, double reading, uint64_t now_ns) {
        for (size_t i = 0; i < sensor_count_; ++i) {
            if (sensors_[i].sensor_id == sensor_id) {
                sensors_[i].current_reading = reading;
                sensors_[i].last_reading_ns = now_ns;
                sensors_[i].total_readings++;
                if (reading > sensors_[i].threshold_critical) {
                    TriggerFloodAlert(sensors_[i].watershed_zone_id, now_ns);
                }
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    void TriggerFloodAlert(uint32_t zone_id, uint64_t now_ns) {
        for (size_t i = 0; i < zone_count_; ++i) {
            if (zones_[i].zone_id == zone_id) {
                WatershedZone& zone = zones_[i];
                FloodAlertLevel alert_level = ComputeAlertLevel(
                    zone.rainfall_accumulation_in,
                    zone.current_water_level_ft,
                    0.0
                );
                if (alert_level >= FloodAlertLevel::WARNING) {
                    if (alert_count_ < MAX_FLOOD_ALERTS) {
                        FloodAlert alert;
                        alert.alert_id = alert_count_;
                        alert.alert_level = alert_level;
                        alert.watershed_zone_id = zone_id;
                        alert.issued_at_ns = now_ns;
                        alert.expires_at_ns = now_ns + 21600000000000ULL;
                        alert.rainfall_in = zone.rainfall_accumulation_in;
                        alert.water_level_ft = zone.current_water_level_ft;
                        alert.flow_rate_cfs = 0.0;
                        alert.evacuation_ordered = alert_level >= FloodAlertLevel::EMERGENCY;
                        alert.public_notification_sent = false;
                        alert.emergency_services_notified = false;
                        alert.affected_population = zone.vulnerable_population;
                        alert.casualties_reported = 0;
                        alert.structures_damaged = 0;
                        alert.economic_loss_usd = 0.0;
                        alerts_[alert_count_] = alert;
                        alert_count_++;
                        total_alerts_issued_++;
                        if (alert.evacuation_ordered) {
                            total_evacuations_ordered_++;
                            zone.evacuation_ordered = true;
                            zone.last_evacuation_ns = now_ns;
                        }
                    }
                }
                zone.current_alert_level = alert_level;
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    void UpdateEvacuationRouteStatus(uint32_t route_id, bool passable, bool flooded, double water_depth_ft, uint64_t now_ns) {
        for (size_t i = 0; i < route_count_; ++i) {
            if (routes_[i].route_id == route_id) {
                routes_[i].passable = passable;
                routes_[i].flooded = flooded;
                routes_[i].water_depth_ft = water_depth_ft;
                routes_[i].last_inspection_ns = now_ns;
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    double ComputeDetectionAccuracy() {
        if (total_alerts_issued_ == 0) return 1.0;
        double true_positives = total_alerts_issued_ - total_false_alarms_;
        return true_positives / total_alerts_issued_;
    }
    
    struct NetworkStatus {
        uint64_t network_id;
        char city_code[8];
        size_t total_sensors;
        size_t operational_sensors;
        size_t total_zones;
        size_t zones_with_alerts;
        size_t total_routes;
        size_t passable_routes;
        size_t flooded_routes;
        size_t total_alerts;
        size_t active_alerts;
        uint64_t total_alerts_issued;
        uint64_t total_evacuations_ordered;
        uint64_t total_false_alarms;
        double detection_accuracy;
        double average_detection_time_min;
        double average_response_time_min;
        uint64_t total_casualties;
        uint64_t total_structures_damaged;
        double total_economic_loss_usd;
        uint64_t last_major_flood_event_ns;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    NetworkStatus GetStatus(uint64_t now_ns) {
        NetworkStatus status;
        status.network_id = network_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_sensors = sensor_count_;
        status.operational_sensors = 0;
        for (size_t i = 0; i < sensor_count_; ++i) {
            if (sensors_[i].operational && sensors_[i].telemetry_active) {
                status.operational_sensors++;
            }
        }
        status.total_zones = zone_count_;
        status.zones_with_alerts = 0;
        for (size_t i = 0; i < zone_count_; ++i) {
            if (zones_[i].current_alert_level >= FloodAlertLevel::ADVISORY) {
                status.zones_with_alerts++;
            }
        }
        status.total_routes = route_count_;
        status.passable_routes = 0;
        status.flooded_routes = 0;
        for (size_t i = 0; i < route_count_; ++i) {
            if (routes_[i].passable) status.passable_routes++;
            if (routes_[i].flooded) status.flooded_routes++;
        }
        status.total_alerts = alert_count_;
        status.active_alerts = 0;
        for (size_t i = 0; i < alert_count_; ++i) {
            if (now_ns < alerts_[i].expires_at_ns) status.active_alerts++;
        }
        status.total_alerts_issued = total_alerts_issued_;
        status.total_evacuations_ordered = total_evacuations_ordered_;
        status.total_false_alarms = total_false_alarms_;
        status.detection_accuracy = ComputeDetectionAccuracy();
        status.average_detection_time_min = average_detection_time_min_;
        status.average_response_time_min = average_response_time_min_;
        status.total_casualties = total_casualties_;
        status.total_structures_damaged = total_structures_damaged_;
        status.total_economic_loss_usd = total_economic_loss_usd_;
        status.last_major_flood_event_ns = last_major_flood_event_ns_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    double ComputeFloodResilienceIndex(uint64_t now_ns) {
        NetworkStatus status = GetStatus(now_ns);
        double sensor_coverage = status.operational_sensors / status.total_sensors.max(1);
        double route_availability = status.passable_routes / status.total_routes.max(1);
        double detection_score = status.detection_accuracy;
        double evacuation_readiness = 1.0 - (status.flooded_routes / status.total_routes.max(1));
        double casualty_penalty = status.total_casualties > 0 ? 0.2 : 0.0;
        return (sensor_coverage * 0.30 + route_availability * 0.25 + 
                detection_score * 0.25 + evacuation_readiness * 0.20 - casualty_penalty).max(0.0);
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= sensor_count_ * zone_count_ * route_count_;
        sum ^= total_alerts_issued_;
        sum ^= total_evacuations_ordered_;
        sum ^= total_casualties_;
        for (size_t i = 0; i < sensor_count_; ++i) {
            sum ^= sensors_[i].sensor_id * static_cast<uint64_t>(sensors_[i].operational);
        }
        return sum == audit_checksum_;
    }
    
    void OptimizeNetwork(uint64_t now_ns) {
        last_optimization_ns_ = now_ns;
        if (IsMonsoonSeason(now_ns)) {
            for (size_t i = 0; i < sensor_count_; ++i) {
                sensors_[i].threshold_warning *= 0.8;
                sensors_[i].threshold_critical *= 0.8;
            }
        }
        UpdateAuditChecksum();
    }
};

#endif
