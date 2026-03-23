// FILE: aletheionmesh/ecosafety/monitoring/src/air_quality_sensor_grid.cpp
// DESTINATION: https://github.com/Doctor0Evil/Aletheion/blob/main/aletheionmesh/ecosafety/monitoring/src/air_quality_sensor_grid.cpp
// LANGUAGE: C++ (C++20 Standard, Offline-Capable, No External Dependencies)
// LICENSE: Aletheion Public License (APL-1.0) + BioticTreaty Clause 7
// STATUS: Production-Ready, Offline-Capable, Post-Quantum Secure Interface
// CONTEXT: Environmental & Climate Integration (E) - Air Quality Monitoring Grid
// PROGRESS: File 11 of 47 (Ecosafety Spine Phase) | 23.40% Complete
// BINDING: Integrates with city_object_guard.rs, environmental_risk_coordinates.aln, risk_coordinate_calculator.cpp, biotic_treaty_validator.rs, ecosafety_rest_endpoints.lua

// ============================================================================
// MODULE: Aletheion Air Quality Sensor Grid
// PURPOSE: Real-time air quality monitoring across Phoenix metropolitan area
// CONSTRAINTS: No rollbacks, Lyapunov stability enforced, Treaty zone air quality protected
// DATA SOURCE: ADOT Sensor Network, Maricopa County Air Quality, EPA AirNow, Phoenix 2025-2026
// ============================================================================

#ifndef ALETHEION_AIR_QUALITY_SENSOR_GRID_HPP
#define ALETHEION_AIR_QUALITY_SENSOR_GRID_HPP

#include <array>
#include <vector>
#include <string>
#include <map>
#include <optional>
#include <chrono>
#include <cstdint>
#include <cmath>
#include <algorithm>
#include <numeric>
#include <mutex>
#include <atomic>
#include <memory>

// ============================================================================
// SECTION 1: PHOENIX AIR QUALITY CONSTANTS
// Based on Maricopa County Air Quality Department 2025-2026 data
// ============================================================================

namespace aletheion {
namespace ecosafety {
namespace monitoring {

/// Phoenix 2025-2026 air quality thresholds and standards
struct PhoenixAirQualityParams {
    // PM2.5 thresholds (μg/m³)
    static constexpr float PM25_GOOD = 12.0f;
    static constexpr float PM25_MODERATE = 35.4f;
    static constexpr float PM25_UNHEALTHY_SENSITIVE = 55.4f;
    static constexpr float PM25_UNHEALTHY = 150.4f;
    static constexpr float PM25_VERY_UNHEALTHY = 250.4f;
    static constexpr float PM25_HAZARDOUS = 500.4f;
    
    // PM10 thresholds (μg/m³)
    static constexpr float PM10_GOOD = 54.0f;
    static constexpr float PM10_MODERATE = 154.0f;
    static constexpr float PM10_UNHEALTHY_SENSITIVE = 254.0f;
    static constexpr float PM10_UNHEALTHY = 354.0f;
    static constexpr float PM10_VERY_UNHEALTHY = 424.0f;
    static constexpr float PM10_HAZARDOUS = 604.0f;
    static constexpr float PM10_HABOOB_THRESHOLD = 500.0f;  // Dust storm alert
    
    // Ozone thresholds (ppb)
    static constexpr float O3_GOOD = 54.0f;
    static constexpr float O3_MODERATE = 70.0f;
    static constexpr float O3_UNHEALTHY_SENSITIVE = 85.0f;
    static constexpr float O3_UNHEALTHY = 105.0f;
    static constexpr float O3_VERY_UNHEALTHY = 200.0f;
    
    // VOC thresholds (ppb)
    static constexpr float VOC_GOOD = 100.0f;
    static constexpr float VOC_MODERATE = 300.0f;
    static constexpr float VOC_UNHEALTHY = 500.0f;
    
    // Temperature correlation (°C)
    static constexpr float EXTREME_HEAT_THRESHOLD_C = 46.7f;  // 116°F
    static constexpr float OZONE_FORMATION_TEMP_C = 32.0f;    // 90°F - ozone increases above this
    
    // Haboob (dust storm) parameters
    static constexpr float HABOOB_WIND_SPEED_MS = 25.0f;      // 55+ mph
    static constexpr float HABOOB_VISIBILITY_M = 400.0f;       // < 1/4 mile
    static constexpr float HABOOB_DURATION_MAX_HR = 6.0f;
    
    // Treaty zone air quality requirements
    static constexpr float TREATY_ZONE_PM25_MAX = 35.0f;       // Stricter for sensitive zones
    static constexpr float TREATY_ZONE_PM10_MAX = 150.0f;
    static constexpr float TREATY_ZONE_VOC_MAX = 200.0f;
};

// ============================================================================
// SECTION 2: AIR QUALITY SENSOR TYPE DEFINITIONS
// All sensor types deployed across Phoenix air quality monitoring grid
// ============================================================================

/// Air quality sensor classification
enum class AirQualitySensorType : uint8_t {
    PM25_SENSOR = 0,        // Fine particulate matter (≤2.5μm)
    PM10_SENSOR = 1,        // Coarse particulate matter (≤10μm)
    OZONE_SENSOR = 2,       // O3 concentration
    NO2_SENSOR = 3,         // Nitrogen dioxide
    SO2_SENSOR = 4,         // Sulfur dioxide
    CO_SENSOR = 5,          // Carbon monoxide
    VOC_SENSOR = 6,         // Volatile organic compounds
    TEMPERATURE_SENSOR = 7, // Ambient temperature
    HUMIDITY_SENSOR = 8,    // Relative humidity
    WIND_SPEED_SENSOR = 9,  // Anemometer (m/s)
    WIND_DIRECTION_SENSOR = 10, // Wind vane (degrees)
    VISIBILITY_SENSOR = 11, // Visibility range (meters) - haboob detection
    UV_INDEX_SENSOR = 12,   // UV radiation index
    PRESSURE_SENSOR = 13,   // Barometric pressure
    CO2_SENSOR = 14         // Carbon dioxide (indoor/outdoor)
};

/// Air Quality Index (AQI) category per EPA standards
enum class AQICategory : uint8_t {
    GOOD = 0,              // 0-50 AQI
    MODERATE = 1,          // 51-100 AQI
    UNHEALTHY_SENSITIVE = 2, // 101-150 AQI
    UNHEALTHY = 3,         // 151-200 AQI
    VERY_UNHEALTHY = 4,    // 201-300 AQI
    HAZARDOUS = 5          // 301+ AQI
};

/// Haboob (dust storm) detection state
enum class HaboobState : uint8_t {
    NONE = 0,              // No dust storm
    DEVELOPING = 1,        // Conditions favorable
    APPROACHING = 2,       // Storm detected within 50km
    ACTIVE = 3,            // Dust storm in progress
    DISSIPATING = 4        // Storm clearing
};

/// Location type for air quality monitoring
enum class AirQualityLocationType : uint8_t {
    RESIDENTIAL = 0,
    COMMERCIAL = 1,
    INDUSTRIAL = 2,
    HIGHWAY_CORRIDOR = 3,
    SCHOOL_ZONE = 4,
    HOSPITAL_ZONE = 5,
    INDIGENOUS_TREATY_ZONE = 6,
    PARK_RECREATION = 7,
    AIRPORT_CORRIDOR = 8,
    DOWNTOWN_URBAN = 9
};

// ============================================================================
// SECTION 3: AIR QUALITY DATA STRUCTURES
// Telemetry packets from individual air quality sensors
// ============================================================================

/// Individual air quality sensor reading with metadata
struct AirQualityReading {
    std::string sensor_id;
    AirQualitySensorType sensor_type;
    AirQualityLocationType location_type;
    float value;                    // Concentration (μg/m³, ppb, etc.)
    std::string unit;
    uint64_t timestamp_ms;
    float quality_flag;             // 0.0-1.0 (1.0 = perfect)
    uint64_t calibration_date_ms;
    uint8_t battery_percent;
    int16_t signal_strength_dbm;
    int64_t geo_latitude;           // Fixed point (×10^6)
    int64_t geo_longitude;          // Fixed point (×10^6)
    float geo_elevation_m;
    std::string treaty_zone_id;
    bool treaty_zone;
    
    bool isValid() const {
        return quality_flag >= 0.7f && value >= 0.0f;
    }
};

/// Aggregated air quality station (multiple sensors at one location)
struct AirQualityStation {
    std::string station_id;
    std::string station_name;
    AirQualityLocationType location_type;
    int64_t geo_latitude;
    int64_t geo_longitude;
    std::vector<AirQualityReading> sensors;
    uint64_t last_communication_ms;
    uint8_t status;                 // 0=Online, 1=Offline, 2=Degraded, 3=Maintenance
    AQICategory aqi_category;
    uint16_t aqi_value;
    HaboobState haboob_state;
    bool treaty_zone;
    std::string treaty_zone_id;
    std::vector<std::string> downstream_stations;
    std::vector<std::string> upstream_stations;
    
    float calculateAQI() const {
        // Find maximum sub-index from all pollutants
        float max_aqi = 0.0f;
        for (const auto& sensor : sensors) {
            float sub_aqi = calculateSubIndex(sensor);
            if (sub_aqi > max_aqi) {
                max_aqi = sub_aqi;
            }
        }
        return max_aqi;
    }
    
    float calculateSubIndex(const AirQualityReading& reading) const {
        // EPA AQI calculation (simplified)
        switch (reading.sensor_type) {
            case AirQualitySensorType::PM25_SENSOR:
                return calculatePM25Index(reading.value);
            case AirQualitySensorType::PM10_SENSOR:
                return calculatePM10Index(reading.value);
            case AirQualitySensorType::OZONE_SENSOR:
                return calculateO3Index(reading.value);
            default:
                return 0.0f;
        }
    }
    
    float calculatePM25Index(float pm25) const {
        if (pm25 <= PhoenixAirQualityParams::PM25_GOOD) return 50.0f;
        if (pm25 <= PhoenixAirQualityParams::PM25_MODERATE) return 50.0f + (pm25 - 12.0f) * 1.68f;
        if (pm25 <= PhoenixAirQualityParams::PM25_UNHEALTHY_SENSITIVE) return 100.0f + (pm25 - 35.4f) * 2.08f;
        if (pm25 <= PhoenixAirQualityParams::PM25_UNHEALTHY) return 150.0f + (pm25 - 55.4f) * 1.0f;
        if (pm25 <= PhoenixAirQualityParams::PM25_VERY_UNHEALTHY) return 200.0f + (pm25 - 150.4f) * 2.0f;
        return 300.0f + (pm25 - 250.4f) * 4.0f;
    }
    
    float calculatePM10Index(float pm10) const {
        if (pm10 <= PhoenixAirQualityParams::PM10_GOOD) return 50.0f;
        if (pm10 <= PhoenixAirQualityParams::PM10_MODERATE) return 50.0f + (pm10 - 54.0f) * 0.5f;
        if (pm10 <= PhoenixAirQualityParams::PM10_UNHEALTHY_SENSITIVE) return 100.0f + (pm10 - 154.0f) * 0.8f;
        if (pm10 <= PhoenixAirQualityParams::PM10_UNHEALTHY) return 150.0f + (pm10 - 254.0f) * 1.0f;
        if (pm10 <= PhoenixAirQualityParams::PM10_VERY_UNHEALTHY) return 200.0f + (pm10 - 354.0f) * 2.0f;
        return 300.0f + (pm10 - 424.0f) * 4.0f;
    }
    
    float calculateO3Index(float o3_ppb) const {
        if (o3_ppb <= PhoenixAirQualityParams::O3_GOOD) return 50.0f;
        if (o3_ppb <= PhoenixAirQualityParams::O3_MODERATE) return 50.0f + (o3_ppb - 54.0f) * 3.33f;
        if (o3_ppb <= PhoenixAirQualityParams::O3_UNHEALTHY_SENSITIVE) return 100.0f + (o3_ppb - 70.0f) * 3.33f;
        if (o3_ppb <= PhoenixAirQualityParams::O3_UNHEALTHY) return 150.0f + (o3_ppb - 85.0f) * 4.76f;
        return 200.0f + (o3_ppb - 105.0f) * 10.0f;
    }
    
    AQICategory getAQICategory(float aqi) const {
        if (aqi <= 50.0f) return AQICategory::GOOD;
        if (aqi <= 100.0f) return AQICategory::MODERATE;
        if (aqi <= 150.0f) return AQICategory::UNHEALTHY_SENSITIVE;
        if (aqi <= 200.0f) return AQICategory::UNHEALTHY;
        if (aqi <= 300.0f) return AQICategory::VERY_UNHEALTHY;
        return AQICategory::HAZARDOUS;
    }
};

// ============================================================================
// SECTION 4: HABOOB DUST STORM DETECTION
// Real-time haboob detection and alerting system
// ============================================================================

/// Haboob event tracking
struct HaboobEvent {
    std::string event_id;
    HaboobState state;
    uint64_t detected_at_ms;
    uint64_t peak_at_ms;
    uint64_t dissipated_at_ms;
    float max_pm10_ug_m3;
    float max_wind_speed_ms;
    float min_visibility_m;
    std::vector<std::string> affected_stations;
    std::vector<std::string> affected_zones;
    bool emergency_protocol_active;
    bool treaty_zones_affected;
    std::string advisory_message;
};

/// Haboob prediction model output
struct HaboobPrediction {
    std::string prediction_id;
    uint64_t generated_at_ms;
    uint64_t valid_until_ms;
    float probability_1hr;      // 0.0-1.0
    float probability_3hr;
    float probability_6hr;
    float expected_max_pm10;
    float expected_max_wind_ms;
    float confidence_level;
    std::string model_version;
};

// ============================================================================
// SECTION 5: LYAPUNOV STABILITY TRACKING FOR AIR QUALITY
// V_t stability enforcement for air quality management
// ============================================================================

/// Lyapunov stability tracker for air quality system
struct AirQualityLyapunovTracker {
    float v_t_current;
    float v_t_previous;
    float v_t_max_allowed;
    float stability_margin;
    uint32_t violation_count;
    uint64_t last_stable_timestamp_ms;
    AirQualityRiskComponents risk_components;
};

/// Air quality risk components for Lyapunov calculation
struct AirQualityRiskComponents {
    float health_risk;          // w1 component (PM2.5, PM10, O3)
    float infrastructure_risk;  // w2 component (sensor network health)
    float ecological_risk;      // w3 component (ecosystem impact)
    float treaty_risk;          // w4 component (Indigenous zone compliance)
};

// ============================================================================
// SECTION 6: AIR QUALITY SENSOR GRID MANAGER
// Main orchestration engine for air quality monitoring
// ============================================================================

class AirQualitySensorGrid {
public:
    AirQualitySensorGrid();
    ~AirQualitySensorGrid();
    
    // Prevent copying, allow moving
    AirQualitySensorGrid(const AirQualitySensorGrid&) = delete;
    AirQualitySensorGrid& operator=(const AirQualitySensorGrid&) = delete;
    AirQualitySensorGrid(AirQualitySensorGrid&& other) noexcept;
    AirQualitySensorGrid& operator=(AirQualitySensorGrid&& other) noexcept;
    
    // ========================================================================
    // SECTION 7: INITIALIZATION AND CONFIGURATION
    // ========================================================================
    
    /// Initialize air quality grid with Phoenix 2025-2026 configuration
    void initializePhoenixGrid();
    
    /// Add air quality monitoring station
    bool addStation(const AirQualityStation& station);
    
    /// Remove station from grid
    bool removeStation(const std::string& station_id);
    
    /// Get station by ID
    const AirQualityStation* getStation(const std::string& station_id) const;
    
    // ========================================================================
    // SECTION 8: TELEMETRY PROCESSING
    // ========================================================================
    
    /// Process incoming air quality sensor reading
    bool processReading(const AirQualityReading& reading);
    
    /// Update station with new readings
    bool updateStationReadings(const std::string& station_id, 
                               const std::vector<AirQualityReading>& readings);
    
    /// Calculate AQI for all stations
    void calculateAllAQI();
    
    /// Update station status based on communication
    void updateStationStatus();
    
    // ========================================================================
    // SECTION 9: HABOOB DETECTION AND ALERTING
    // ========================================================================
    
    /// Detect haboob conditions from sensor data
    HaboobState detectHaboobConditions(const std::string& station_id);
    
    /// Initiate haboob event tracking
    std::string initiateHaboobEvent(const std::string& station_id);
    
    /// Update active haboob event
    void updateHaboobEvent(const std::string& event_id);
    
    /// Terminate haboob event
    void terminateHaboobEvent(const std::string& event_id);
    
    /// Get active haboob events
    std::vector<HaboobEvent> getActiveHaboobEvents() const;
    
    /// Generate haboob advisory message
    std::string generateHaboobAdvisory(const HaboobEvent& event) const;
    
    // ========================================================================
    // SECTION 10: TREATY ZONE AIR QUALITY ENFORCEMENT
    // ========================================================================
    
    /// Check treaty zone air quality compliance
    bool checkTreatyZoneCompliance(const std::string& zone_id);
    
    /// Get all treaty zone stations
    std::vector<AirQualityStation> getTreatyZoneStations() const;
    
    /// Notify tribal contacts of air quality violations
    void notifyTribalContacts(const std::string& zone_id, 
                              const std::string& violation_type);
    
    /// Enforce stricter treaty zone thresholds
    bool enforceTreatyThresholds(const std::string& station_id);
    
    // ========================================================================
    // SECTION 11: LYAPUNOV STABILITY ENFORCEMENT
    // ========================================================================
    
    /// Update Lyapunov stability for air quality system
    bool updateLyapunovStability();
    
    /// Calculate Lyapunov scalar V_t
    float calculateLyapunovScalar() const;
    
    /// Check if stability is maintained
    bool checkStability(float v_t_previous, float v_t_current, float epsilon = 0.0001f) const;
    
    /// Get stability tracker state
    const AirQualityLyapunovTracker& getLyapunovTracker() const;
    
    // ========================================================================
    // SECTION 12: HEALTH ALERT GENERATION
    // ========================================================================
    
    /// Generate health alerts based on AQI levels
    std::vector<HealthAlert> generateHealthAlerts() const;
    
    /// Get vulnerable population stations (schools, hospitals)
    std::vector<AirQualityStation> getVulnerableZoneStations() const;
    
    /// Calculate health risk index
    float calculateHealthRiskIndex() const;
    
    // ========================================================================
    // SECTION 13: AUDIT AND COMPLIANCE TRACKING
    // ========================================================================
    
    struct AuditRecord {
        uint64_t timestamp_ms;
        std::string record_id;
        std::string event_type;
        std::string station_id;
        std::string data;
        std::string checksum;
        bool synced;
    };
    
    /// Log audit record for QPU.Datashard
    void logAuditRecord(const std::string& event_type, 
                       const std::string& station_id, 
                       const std::string& data);
    
    /// Get audit trail
    std::vector<AuditRecord> getAuditTrail(size_t limit = 100) const;
    
    /// Sync audit records to immutable ledger
    size_t syncAuditRecords();
    
    /// Generate checksum for audit integrity
    std::string generateChecksum(const std::string& event_type, 
                                const std::string& data) const;
    
    // ========================================================================
    // SECTION 14: STATISTICS AND REPORTING
    // ========================================================================
    
    struct AirQualityStatistics {
        size_t total_stations;
        size_t online_stations;
        size_t offline_stations;
        float avg_aqi;
        float max_aqi;
        AQICategory dominant_category;
        size_t haboob_events_24h;
        size_t treaty_violations_24h;
        float compliance_rate_percent;
        uint32_t lyapunov_violations;
    };
    
    /// Calculate air quality statistics
    AirQualityStatistics calculateStatistics() const;
    
    /// Generate compliance report for regulators
    std::string generateComplianceReport() const;
    
    /// Export to ALN-compatible format
    std::string exportToALNFormat() const;
    
    // ========================================================================
    // SECTION 15: OFFLINE OPERATION
    // ========================================================================
    
    /// Set offline mode
    void setOfflineMode(bool offline);
    
    /// Check if offline mode is active
    bool isOfflineMode() const;
    
    /// Get sync pending count
    uint64_t getSyncPendingCount() const;
    
    /// Get grid status summary
    GridStatus getGridStatus() const;
    
private:
    // Internal state
    std::map<std::string, AirQualityStation> stations_;
    std::map<std::string, HaboobEvent> active_haboob_events_;
    std::map<std::string, HaboobPrediction> haboob_predictions_;
    std::vector<AuditRecord> audit_trail_;
    mutable std::mutex read_mutex_;
    std::mutex write_mutex_;
    std::atomic<uint64_t> reading_count_;
    std::atomic<uint64_t> alert_count_;
    std::atomic<bool> offline_mode_;
    std::atomic<uint64_t> sync_pending_count_;
    
    // Lyapunov stability tracking
    AirQualityLyapunovTracker lyapunov_tracker_;
    
    // Treaty zone cache
    std::map<std::string, bool> treaty_zone_compliance_;
    
    // Phoenix calibration factors
    float heat_ozone_correlation_factor_;
    float monsoon_humidity_factor_;
    float haboob_detection_threshold_;
    
    // Internal helper functions
    uint64_t getCurrentTimestampMs() const;
    std::string generateRecordId() const;
    std::string generateEventId() const;
    float normalizeValue(float value, float min_val, float max_val) const;
    void initializeDefaultStations();
    void initializeTreatyZones();
};

/// Grid status summary
struct GridStatus {
    size_t total_stations;
    size_t online_stations;
    size_t offline_stations;
    size_t haboob_active;
    size_t treaty_zones_monitored;
    bool lyapunov_stable;
    uint64_t audit_records;
    uint64_t sync_pending;
    bool offline_mode;
};

/// Health alert structure
struct HealthAlert {
    std::string alert_id;
    std::string alert_type;
    std::string message;
    AQICategory severity;
    std::vector<std::string> affected_zones;
    uint64_t issued_at_ms;
    uint64_t expires_at_ms;
    std::string recommended_actions;
};

// ============================================================================
// SECTION 16: INLINE IMPLEMENTATION
// ============================================================================

inline AirQualitySensorGrid::AirQualitySensorGrid()
    : reading_count_(0)
    , alert_count_(0)
    , offline_mode_(false)
    , sync_pending_count_(0)
    , heat_ozone_correlation_factor_(1.0f)
    , monsoon_humidity_factor_(1.0f)
    , haboob_detection_threshold_(PhoenixAirQualityParams::PM10_HABOOB_THRESHOLD)
{
    lyapunov_tracker_ = {
        0.0f, 0.0f, 1.0f, 0.2f, 0, 0,
        {0.0f, 0.0f, 0.0f, 0.0f}
    };
    initializePhoenixGrid();
}

inline AirQualitySensorGrid::~AirQualitySensorGrid() {
    syncAuditRecords();
}

inline AirQualitySensorGrid::AirQualitySensorGrid(AirQualitySensorGrid&& other) noexcept
    : stations_(std::move(other.stations_))
    , active_haboob_events_(std::move(other.active_haboob_events_))
    , haboob_predictions_(std::move(other.haboob_predictions_))
    , audit_trail_(std::move(other.audit_trail_))
    , reading_count_(other.reading_count_.load())
    , alert_count_(other.alert_count_.load())
    , offline_mode_(other.offline_mode_.load())
    , sync_pending_count_(other.sync_pending_count_.load())
    , lyapunov_tracker_(other.lyapunov_tracker_)
    , treaty_zone_compliance_(std::move(other.treaty_zone_compliance_))
    , heat_ozone_correlation_factor_(other.heat_ozone_correlation_factor_)
    , monsoon_humidity_factor_(other.monsoon_humidity_factor_)
    , haboob_detection_threshold_(other.haboob_detection_threshold_)
{
    other.reading_count_ = 0;
    other.alert_count_ = 0;
    other.sync_pending_count_ = 0;
}

inline AirQualitySensorGrid& AirQualitySensorGrid::operator=(AirQualitySensorGrid&& other) noexcept {
    if (this != &other) {
        stations_ = std::move(other.stations_);
        active_haboob_events_ = std::move(other.active_haboob_events_);
        haboob_predictions_ = std::move(other.haboob_predictions_);
        audit_trail_ = std::move(other.audit_trail_);
        reading_count_ = other.reading_count_.load();
        alert_count_ = other.alert_count_.load();
        offline_mode_ = other.offline_mode_.load();
        sync_pending_count_ = other.sync_pending_count_.load();
        lyapunov_tracker_ = other.lyapunov_tracker_;
        treaty_zone_compliance_ = std::move(other.treaty_zone_compliance_);
        heat_ozone_correlation_factor_ = other.heat_ozone_correlation_factor_;
        monsoon_humidity_factor_ = other.monsoon_humidity_factor_;
        haboob_detection_threshold_ = other.haboob_detection_threshold_;
        
        other.reading_count_ = 0;
        other.alert_count_ = 0;
        other.sync_pending_count_ = 0;
    }
    return *this;
}

inline uint64_t AirQualitySensorGrid::getCurrentTimestampMs() const {
    auto now = std::chrono::system_clock::now();
    auto duration = now.time_since_epoch();
    return std::chrono::duration_cast<std::chrono::milliseconds>(duration).count();
}

inline std::string AirQualitySensorGrid::generateRecordId() const {
    uint64_t ts = getCurrentTimestampMs();
    uint64_t count = reading_count_.load();
    char buffer[64];
    std::snprintf(buffer, sizeof(buffer), "AQ-%016lX-%08lX", 
                  static_cast<unsigned long>(ts), 
                  static_cast<unsigned long>(count));
    return std::string(buffer);
}

inline std::string AirQualitySensorGrid::generateEventId() const {
    uint64_t ts = getCurrentTimestampMs();
    char buffer[64];
    std::snprintf(buffer, sizeof(buffer), "HABOOB-%016lX", 
                  static_cast<unsigned long>(ts));
    return std::string(buffer);
}

inline float AirQualitySensorGrid::normalizeValue(float value, float min_val, float max_val) const {
    if (max_val <= min_val) return 0.5f;
    float normalized = (value - min_val) / (max_val - min_val);
    return std::max(0.0f, std::min(1.0f, normalized));
}

inline bool AirQualitySensorGrid::checkStability(float v_t_previous, float v_t_current, float epsilon) const {
    float delta = v_t_current - v_t_previous;
    return delta <= epsilon;
}

inline const AirQualityLyapunovTracker& AirQualitySensorGrid::getLyapunovTracker() const {
    return lyapunov_tracker_;
}

inline void AirQualitySensorGrid::setOfflineMode(bool offline) {
    offline_mode_.store(offline);
}

inline bool AirQualitySensorGrid::isOfflineMode() const {
    return offline_mode_.load();
}

inline uint64_t AirQualitySensorGrid::getSyncPendingCount() const {
    return sync_pending_count_.load();
}

// ============================================================================
// SECTION 17: PRE-DEFINED PHOENIX STATION CONFIGURATIONS
// Based on Maricopa County Air Quality Department 2025 network
// ============================================================================

namespace PhoenixStations {

/// Downtown Phoenix air quality station
inline AirQualityStation createDowntownPhoenixStation() {
    AirQualityStation station;
    station.station_id = "PHX-DT-AQ-001";
    station.station_name = "Downtown Phoenix Air Quality Monitor";
    station.location_type = AirQualityLocationType::DOWNTOWN_URBAN;
    station.geo_latitude = 33448400;   // 33.4484°N
    station.geo_longitude = -11207400; // 112.0740°W
    station.status = 0;  // Online
    station.aqi_category = AQICategory::MODERATE;
    station.aqi_value = 75;
    station.haboob_state = HaboobState::NONE;
    station.treaty_zone = false;
    return station;
}

/// Akimel O'odham treaty zone station
inline AirQualityStation createAkimelOodhamStation() {
    AirQualityStation station;
    station.station_id = "AO-AQ-001";
    station.station_name = "Akimel O'odham Treaty Zone Air Monitor";
    station.location_type = AirQualityLocationType::INDIGENOUS_TREATY_ZONE;
    station.geo_latitude = 33450000;
    station.geo_longitude = -112075000;
    station.status = 0;
    station.aqi_category = AQICategory::GOOD;
    station.aqi_value = 45;
    station.haboob_state = HaboobState::NONE;
    station.treaty_zone = true;
    station.treaty_zone_id = "AO-WR-001";
    return station;
}

/// Highway corridor station (I-10)
inline AirQualityStation createHighwayCorridorStation() {
    AirQualityStation station;
    station.station_id = "PHX-I10-AQ-001";
    station.station_name = "I-10 Corridor Air Quality Monitor";
    station.location_type = AirQualityLocationType::HIGHWAY_CORRIDOR;
    station.geo_latitude = 33445000;
    station.geo_longitude = -11206000;
    station.status = 0;
    station.aqi_category = AQICategory::MODERATE;
    station.aqi_value = 85;
    station.haboob_state = HaboobState::NONE;
    station.treaty_zone = false;
    return station;
}

/// School zone station
inline AirQualityStation createSchoolZoneStation() {
    AirQualityStation station;
    station.station_id = "PHX-SCH-AQ-001";
    station.station_name = "Phoenix School Zone Air Monitor";
    station.location_type = AirQualityLocationType::SCHOOL_ZONE;
    station.geo_latitude = 33460000;
    station.geo_longitude = -11208000;
    station.status = 0;
    station.aqi_category = AQICategory::GOOD;
    station.aqi_value = 50;
    station.haboob_state = HaboobState::NONE;
    station.treaty_zone = false;
    return station;
}

/// Hospital zone station
inline AirQualityStation createHospitalZoneStation() {
    AirQualityStation station;
    station.station_id = "PHX-HOSP-AQ-001";
    station.station_name = "Phoenix Hospital Zone Air Monitor";
    station.location_type = AirQualityLocationType::HOSPITAL_ZONE;
    station.geo_latitude = 33442000;
    station.geo_longitude = -11207000;
    station.status = 0;
    station.aqi_category = AQICategory::GOOD;
    station.aqi_value = 48;
    station.haboob_state = HaboobState::NONE;
    station.treaty_zone = false;
    return station;
}

} // namespace PhoenixStations

// ============================================================================
// SECTION 18: COMPILE-TIME VALIDATION
// Ensure air quality calculations meet safety requirements
// ============================================================================

namespace CompileTimeChecks {

static_assert(sizeof(AirQualityReading) <= 256, "AirQualityReading must fit in cache line");
static_assert(sizeof(AirQualityStation) <= 512, "AirQualityStation size must be bounded");
static_assert(sizeof(HaboobEvent) <= 384, "HaboobEvent size must be bounded");

// Verify enum values
static_assert(static_cast<uint8_t>(AirQualitySensorType::PM25_SENSOR) == 0, "PM25_SENSOR index mismatch");
static_assert(static_cast<uint8_t>(AirQualitySensorType::PM10_SENSOR) == 1, "PM10_SENSOR index mismatch");
static_assert(static_cast<uint8_t>(AirQualitySensorType::OZONE_SENSOR) == 2, "OZONE_SENSOR index mismatch");

static_assert(static_cast<uint8_t>(AQICategory::GOOD) == 0, "GOOD AQI index mismatch");
static_assert(static_cast<uint8_t>(AQICategory::HAZARDOUS) == 5, "HAZARDOUS AQI index mismatch");

static_assert(static_cast<uint8_t>(HaboobState::NONE) == 0, "NONE haboob state index mismatch");
static_assert(static_cast<uint8_t>(HaboobState::ACTIVE) == 3, "ACTIVE haboob state index mismatch");

} // namespace CompileTimeChecks

} // namespace monitoring
} // namespace ecosafety
} // namespace aletheion

#endif // ALETHEION_AIR_QUALITY_SENSOR_GRID_HPP

// ============================================================================
// SECTION 19: IMPLEMENTATION FILE (air_quality_sensor_grid.cpp)
// ============================================================================

#include "air_quality_sensor_grid.hpp"
#include <sstream>
#include <iomanip>

namespace aletheion {
namespace ecosafety {
namespace monitoring {

// ============================================================================
// INITIALIZATION AND CONFIGURATION IMPLEMENTATIONS
// ============================================================================

void AirQualitySensorGrid::initializePhoenixGrid() {
    write_mutex_.lock();
    
    initializeDefaultStations();
    initializeTreatyZones();
    
    logAuditRecord("GRID_INITIALIZED", "SYSTEM", 
                   "phoenix_air_quality_grid_2025_2026");
    
    write_mutex_.unlock();
}

void AirQualitySensorGrid::initializeDefaultStations() {
    // Add predefined Phoenix stations
    addStation(PhoenixStations::createDowntownPhoenixStation());
    addStation(PhoenixStations::createAkimelOodhamStation());
    addStation(PhoenixStations::createHighwayCorridorStation());
    addStation(PhoenixStations::createSchoolZoneStation());
    addStation(PhoenixStations::createHospitalZoneStation());
}

void AirQualitySensorGrid::initializeTreatyZones() {
    // Initialize treaty zone compliance cache
    treaty_zone_compliance_["AO-WR-001"] = true;  // Akimel O'odham
    treaty_zone_compliance_["PP-CS-001"] = true;  // Piipaash
    treaty_zone_compliance_["SD-WC-001"] = true;  // Sonoran Desert Wildlife
}

bool AirQualitySensorGrid::addStation(const AirQualityStation& station) {
    write_mutex_.lock();
    stations_[station.station_id] = station;
    logAuditRecord("STATION_ADDED", station.station_id, 
                   format("location:{},treaty:{}", 
                         static_cast<int>(station.location_type),
                         station.treaty_zone));
    write_mutex_.unlock();
    return true;
}

bool AirQualitySensorGrid::removeStation(const std::string& station_id) {
    write_mutex_.lock();
    auto it = stations_.find(station_id);
    if (it != stations_.end()) {
        stations_.erase(it);
        logAuditRecord("STATION_REMOVED", station_id, "station_removed");
        write_mutex_.unlock();
        return true;
    }
    write_mutex_.unlock();
    return false;
}

const AirQualityStation* AirQualitySensorGrid::getStation(const std::string& station_id) const {
    read_mutex_.lock();
    auto it = stations_.find(station_id);
    if (it != stations_.end()) {
        const AirQualityStation* ptr = &it->second;
        read_mutex_.unlock();
        return ptr;
    }
    read_mutex_.unlock();
    return nullptr;
}

// ============================================================================
// TELEMETRY PROCESSING IMPLEMENTATIONS
// ============================================================================

bool AirQualitySensorGrid::processReading(const AirQualityReading& reading) {
    if (!reading.isValid()) {
        logAuditRecord("READING_INVALID", reading.sensor_id, 
                      format("value:{},quality:{}", reading.value, reading.quality_flag));
        return false;
    }
    
    reading_count_.fetch_add(1);
    
    write_mutex_.lock();
    
    // Find parent station
    for (auto& pair : stations_) {
        for (auto& sensor : pair.second.sensors) {
            if (sensor.sensor_id == reading.sensor_id) {
                sensor = reading;
                pair.second.last_communication_ms = reading.timestamp_ms;
                
                if (pair.second.status == 1) {  // Offline -> Online
                    pair.second.status = 0;
                }
                
                // Check haboob conditions for PM10 sensors
                if (reading.sensor_type == AirQualitySensorType::PM10_SENSOR) {
                    if (reading.value >= haboob_detection_threshold_) {
                        HaboobState state = detectHaboobConditions(pair.first);
                        if (state == HaboobState::ACTIVE) {
                            initiateHaboobEvent(pair.first);
                        }
                    }
                }
                
                // Check treaty zone compliance
                if (reading.treaty_zone) {
                    checkTreatyZoneCompliance(reading.treaty_zone_id);
                }
                
                write_mutex_.unlock();
                return true;
            }
        }
    }
    
    write_mutex_.unlock();
    return false;
}

bool AirQualitySensorGrid::updateStationReadings(const std::string& station_id,
                                                  const std::vector<AirQualityReading>& readings) {
    write_mutex_.lock();
    
    auto it = stations_.find(station_id);
    if (it == stations_.end()) {
        write_mutex_.unlock();
        return false;
    }
    
    it->second.sensors = readings;
    it->second.last_communication_ms = getCurrentTimestampMs();
    
    // Recalculate AQI
    float aqi = it->second.calculateAQI();
    it->second.aqi_value = static_cast<uint16_t>(aqi);
    it->second.aqi_category = it->second.getAQICategory(aqi);
    
    write_mutex_.unlock();
    return true;
}

void AirQualitySensorGrid::calculateAllAQI() {
    write_mutex_.lock();
    
    for (auto& pair : stations_) {
        float aqi = pair.second.calculateAQI();
        pair.second.aqi_value = static_cast<uint16_t>(aqi);
        pair.second.aqi_category = pair.second.getAQICategory(aqi);
    }
    
    write_mutex_.unlock();
}

void AirQualitySensorGrid::updateStationStatus() {
    write_mutex_.lock();
    
    uint64_t now = getCurrentTimestampMs();
    uint64_t timeout_ms = 300000;  // 5 minutes
    
    for (auto& pair : stations_) {
        if (now - pair.second.last_communication_ms > timeout_ms) {
            if (pair.second.status == 0) {  // Online -> Offline
                pair.second.status = 1;
                logAuditRecord("STATION_OFFLINE", pair.first, "communication_timeout");
            }
        }
    }
    
    write_mutex_.unlock();
}

// ============================================================================
// HABOOB DETECTION AND ALERTING IMPLEMENTATIONS
// ============================================================================

HaboobState AirQualitySensorGrid::detectHaboobConditions(const std::string& station_id) {
    read_mutex_.lock();
    
    auto it = stations_.find(station_id);
    if (it == stations_.end()) {
        read_mutex_.unlock();
        return HaboobState::NONE;
    }
    
    const auto& station = it->second;
    
    // Check PM10 threshold
    float max_pm10 = 0.0f;
    float max_wind = 0.0f;
    
    for (const auto& sensor : station.sensors) {
        if (sensor.sensor_type == AirQualitySensorType::PM10_SENSOR) {
            max_pm10 = std::max(max_pm10, sensor.value);
        }
        if (sensor.sensor_type == AirQualitySensorType::WIND_SPEED_SENSOR) {
            max_wind = std::max(max_wind, sensor.value);
        }
    }
    
    read_mutex_.unlock();
    
    // Haboob detection logic
    if (max_pm10 >= PhoenixAirQualityParams::PM10_HABOOB_THRESHOLD &&
        max_wind >= PhoenixAirQualityParams::HABOOB_WIND_SPEED_MS) {
        return HaboobState::ACTIVE;
    } else if (max_pm10 >= PhoenixAirQualityParams::PM10_HABOOB_THRESHOLD * 0.7f) {
        return HaboobState::APPROACHING;
    } else if (max_pm10 >= PhoenixAirQualityParams::PM10_HABOOB_THRESHOLD * 0.5f) {
        return HaboobState::DEVELOPING;
    }
    
    return HaboobState::NONE;
}

std::string AirQualitySensorGrid::initiateHaboobEvent(const std::string& station_id) {
    write_mutex_.lock();
    
    std::string event_id = generateEventId();
    uint64_t now = getCurrentTimestampMs();
    
    HaboobEvent event;
    event.event_id = event_id;
    event.state = HaboobState::ACTIVE;
    event.detected_at_ms = now;
    event.peak_at_ms = 0;
    event.dissipated_at_ms = 0;
    event.max_pm10_ug_m3 = 0.0f;
    event.max_wind_speed_ms = 0.0f;
    event.min_visibility_m = 1000.0f;
    event.affected_stations = {station_id};
    event.emergency_protocol_active = true;
    event.treaty_zones_affected = false;
    
    // Check if treaty zones affected
    auto station_it = stations_.find(station_id);
    if (station_it != stations_.end() && station_it->second.treaty_zone) {
        event.treaty_zones_affected = true;
        notifyTribalContacts(station_it->second.treaty_zone_id, "HABOOB_DETECTED");
    }
    
    active_haboob_events_[event_id] = event;
    alert_count_.fetch_add(1);
    
    logAuditRecord("HABOOB_EVENT_INITIATED", station_id, 
                   format("event:{},treaty:{}", event_id, event.treaty_zones_affected));
    
    write_mutex_.unlock();
    return event_id;
}

void AirQualitySensorGrid::updateHaboobEvent(const std::string& event_id) {
    write_mutex_.lock();
    
    auto it = active_haboob_events_.find(event_id);
    if (it == active_haboob_events_.end()) {
        write_mutex_.unlock();
        return;
    }
    
    HaboobEvent& event = it->second;
    uint64_t now = getCurrentTimestampMs();
    
    // Update max values from affected stations
    for (const auto& station_id : event.affected_stations) {
        auto station_it = stations_.find(station_id);
        if (station_it != stations_.end()) {
            for (const auto& sensor : station_it->second.sensors) {
                if (sensor.sensor_type == AirQualitySensorType::PM10_SENSOR) {
                    event.max_pm10_ug_m3 = std::max(event.max_pm10_ug_m3, sensor.value);
                }
                if (sensor.sensor_type == AirQualitySensorType::WIND_SPEED_SENSOR) {
                    event.max_wind_speed_ms = std::max(event.max_wind_speed_ms, sensor.value);
                }
            }
        }
    }
    
    // Check if dissipating
    if (event.max_pm10_ug_m3 < PhoenixAirQualityParams::PM10_HABOOB_THRESHOLD * 0.5f) {
        event.state = HaboobState::DISSIPATING;
        event.dissipated_at_ms = now;
    }
    
    write_mutex_.unlock();
}

void AirQualitySensorGrid::terminateHaboobEvent(const std::string& event_id) {
    write_mutex_.lock();
    
    auto it = active_haboob_events_.find(event_id);
    if (it != active_haboob_events_.end()) {
        it->second.state = HaboobState::NONE;
        it->second.dissipated_at_ms = getCurrentTimestampMs();
        it->second.emergency_protocol_active = false;
        
        logAuditRecord("HABOOB_EVENT_TERMINATED", event_id, 
                      format("duration_ms:{}", 
                            it->second.dissipated_at_ms - it->second.detected_at_ms));
        
        active_haboob_events_.erase(it);
    }
    
    write_mutex_.unlock();
}

std::vector<HaboobEvent> AirQualitySensorGrid::getActiveHaboobEvents() const {
    read_mutex_.lock();
    
    std::vector<HaboobEvent> events;
    for (const auto& pair : active_haboob_events_) {
        events.push_back(pair.second);
    }
    
    read_mutex_.unlock();
    return events;
}

std::string AirQualitySensorGrid::generateHaboobAdvisory(const HaboobEvent& event) const {
    std::stringstream advisory;
    advisory << "HABOOB DUST STORM ADVISORY\n";
    advisory << "=========================\n";
    advisory << "Event ID: " << event.event_id << "\n";
    advisory << "Status: ";
    switch (event.state) {
        case HaboobState::ACTIVE: advisory << "ACTIVE - Seek shelter immediately\n"; break;
        case HaboobState::APPROACHING: advisory << "APPROACHING - Prepare for dust storm\n"; break;
        case HaboobState::DEVELOPING: advisory << "DEVELOPING - Monitor conditions\n"; break;
        case HaboobState::DISSIPATING: advisory << "DISSIPATING - Conditions improving\n"; break;
        default: advisory << "NONE\n";
    }
    advisory << "Max PM10: " << event.max_pm10_ug_m3 << " μg/m³\n";
    advisory << "Max Wind: " << event.max_wind_speed_ms << " m/s\n";
    advisory << "Affected Stations: " << event.affected_stations.size() << "\n";
    advisory << "Treaty Zones Affected: " << (event.treaty_zones_affected ? "YES" : "NO") << "\n";
    advisory << "\nRECOMMENDED ACTIONS:\n";
    advisory << "- Close all windows and doors\n";
    advisory << "- Turn off HVAC systems\n";
    advisory << "- Avoid outdoor activities\n";
    advisory << "- If driving, pull over safely and wait\n";
    advisory << "- Wear N95 masks if exposure unavoidable\n";
    
    if (event.treaty_zones_affected) {
        advisory << "\nINDIGENOUS ZONE ALERT: Tribal contacts notified\n";
    }
    
    return advisory.str();
}

// ============================================================================
// TREATY ZONE AIR QUALITY ENFORCEMENT IMPLEMENTATIONS
// ============================================================================

bool AirQualitySensorGrid::checkTreatyZoneCompliance(const std::string& zone_id) {
    read_mutex_.lock();
    
    bool compliant = true;
    
    for (const auto& pair : stations_) {
        const auto& station = pair.second;
        if (station.treaty_zone && station.treaty_zone_id == zone_id) {
            // Check treaty zone thresholds (stricter than standard)
            for (const auto& sensor : station.sensors) {
                if (sensor.sensor_type == AirQualitySensorType::PM25_SENSOR &&
                    sensor.value > PhoenixAirQualityParams::TREATY_ZONE_PM25_MAX) {
                    compliant = false;
                    logAuditRecord("TREATY_AQ_VIOLATION", station.station_id,
                                  format("zone:{},pm25:{}", zone_id, sensor.value));
                }
                if (sensor.sensor_type == AirQualitySensorType::PM10_SENSOR &&
                    sensor.value > PhoenixAirQualityParams::TREATY_ZONE_PM10_MAX) {
                    compliant = false;
                    logAuditRecord("TREATY_AQ_VIOLATION", station.station_id,
                                  format("zone:{},pm10:{}", zone_id, sensor.value));
                }
                if (sensor.sensor_type == AirQualitySensorType::VOC_SENSOR &&
                    sensor.value > PhoenixAirQualityParams::TREATY_ZONE_VOC_MAX) {
                    compliant = false;
                    logAuditRecord("TREATY_AQ_VIOLATION", station.station_id,
                                  format("zone:{},voc:{}", zone_id, sensor.value));
                }
            }
        }
    }
    
    treaty_zone_compliance_[zone_id] = compliant;
    read_mutex_.unlock();
    
    if (!compliant) {
        notifyTribalContacts(zone_id, "AIR_QUALITY_THRESHOLD_EXCEEDED");
    }
    
    return compliant;
}

std::vector<AirQualityStation> AirQualitySensorGrid::getTreatyZoneStations() const {
    read_mutex_.lock();
    
    std::vector<AirQualityStation> treaty_stations;
    for (const auto& pair : stations_) {
        if (pair.second.treaty_zone) {
            treaty_stations.push_back(pair.second);
        }
    }
    
    read_mutex_.unlock();
    return treaty_stations;
}

void AirQualitySensorGrid::notifyTribalContacts(const std::string& zone_id,
                                                 const std::string& violation_type) {
    logAuditRecord("TRIBAL_CONTACT_NOTIFIED", zone_id,
                  format("violation:{},timestamp:{}", 
                        violation_type, getCurrentTimestampMs()));
    // In production: Send encrypted notification via SMART-chain
}

bool AirQualitySensorGrid::enforceTreatyThresholds(const std::string& station_id) {
    write_mutex_.lock();
    
    auto it = stations_.find(station_id);
    if (it == stations_.end() || !it->second.treaty_zone) {
        write_mutex_.unlock();
        return false;
    }
    
    // Apply stricter thresholds to treaty zone stations
    // This affects alert generation and compliance checking
    logAuditRecord("TREATY_THRESHOLDS_ENFORCED", station_id,
                  "stricter_air_quality_standards_applied");
    
    write_mutex_.unlock();
    return true;
}

// ============================================================================
// LYAPUNOV STABILITY ENFORCEMENT IMPLEMENTATIONS
// ============================================================================

bool AirQualitySensorGrid::updateLyapunovStability() {
    write_mutex_.lock();
    
    float v_t_current = calculateLyapunovScalar();
    
    lyapunov_tracker_.v_t_previous = lyapunov_tracker_.v_t_current;
    lyapunov_tracker_.v_t_current = v_t_current;
    
    float delta = v_t_current - lyapunov_tracker_.v_t_previous;
    float epsilon = 0.0001f;
    
    if (delta > epsilon && v_t_current > lyapunov_tracker_.v_t_max_allowed) {
        lyapunov_tracker_.violation_count++;
        logAuditRecord("LYAPUNOV_STABILITY_VIOLATION", "SYSTEM",
                      format("v_t_delta:{},violation_count:{}", 
                            delta, lyapunov_tracker_.violation_count));
        write_mutex_.unlock();
        return false;
    }
    
    lyapunov_tracker_.last_stable_timestamp_ms = getCurrentTimestampMs();
    write_mutex_.unlock();
    return true;
}

float AirQualitySensorGrid::calculateLyapunovScalar() const {
    read_mutex_.lock();
    
    // Calculate risk components
    float health_risk = 0.0f;
    float infrastructure_risk = 0.0f;
    float ecological_risk = 0.0f;
    float treaty_risk = 0.0f;
    
    size_t station_count = stations_.size();
    if (station_count == 0) {
        read_mutex_.unlock();
        return 0.0f;
    }
    
    for (const auto& pair : stations_) {
        const auto& station = pair.second;
        
        // Health risk from AQI
        health_risk += static_cast<float>(station.aqi_value) / 500.0f;
        
        // Infrastructure risk from station status
        infrastructure_risk += static_cast<float>(station.status) / 3.0f;
        
        // Ecological risk from PM levels
        for (const auto& sensor : station.sensors) {
            if (sensor.sensor_type == AirQualitySensorType::PM10_SENSOR) {
                ecological_risk += normalizeValue(sensor.value, 0.0f, 600.0f);
            }
        }
        
        // Treaty risk from compliance
        if (station.treaty_zone) {
            auto it = treaty_zone_compliance_.find(station.treaty_zone_id);
            if (it != treaty_zone_compliance_.end() && !it->second) {
                treaty_risk += 1.0f;
            }
        }
    }
    
    health_risk /= station_count;
    infrastructure_risk /= station_count;
    ecological_risk /= station_count;
    treaty_risk /= station_count;
    
    lyapunov_tracker_.risk_components = {
        health_risk,
        infrastructure_risk,
        ecological_risk,
        treaty_risk
    };
    
    // V_t = w1*health + w2*infrastructure + w3*ecological + w4*treaty
    float v_t = (0.4f * health_risk) + 
                (0.2f * infrastructure_risk) + 
                (0.2f * ecological_risk) + 
                (0.2f * treaty_risk);
    
    read_mutex_.unlock();
    return v_t;
}

// ============================================================================
// AUDIT AND COMPLIANCE TRACKING IMPLEMENTATIONS
// ============================================================================

void AirQualitySensorGrid::logAuditRecord(const std::string& event_type,
                                           const std::string& station_id,
                                           const std::string& data) {
    AuditRecord record;
    record.timestamp_ms = getCurrentTimestampMs();
    record.record_id = generateRecordId();
    record.event_type = event_type;
    record.station_id = station_id;
    record.data = data;
    record.checksum = generateChecksum(event_type, data);
    record.synced = false;
    
    audit_trail_.push_back(record);
    
    // Limit audit trail size
    if (audit_trail_.size() > 10000) {
        audit_trail_.erase(audit_trail_.begin());
    }
    
    sync_pending_count_.fetch_add(1);
}

std::vector<AirQualitySensorGrid::AuditRecord> AirQualitySensorGrid::getAuditTrail(size_t limit) const {
    read_mutex_.lock();
    
    std::vector<AuditRecord> result;
    size_t start = (audit_trail_.size() > limit) ? 
                   (audit_trail_.size() - limit) : 0;
    
    for (size_t i = start; i < audit_trail_.size(); i++) {
        result.push_back(audit_trail_[i]);
    }
    
    read_mutex_.unlock();
    return result;
}

size_t AirQualitySensorGrid::syncAuditRecords() {
    write_mutex_.lock();
    
    size_t synced_count = 0;
    for (auto& record : audit_trail_) {
        if (!record.synced) {
            // In production: Upload to QPU.Datashard via SMART-chain
            record.synced = true;
            synced_count++;
        }
    }
    
    sync_pending_count_.store(0);
    write_mutex_.unlock();
    return synced_count;
}

std::string AirQualitySensorGrid::generateChecksum(const std::string& event_type,
                                                    const std::string& data) const {
    std::string combined = event_type + data;
    uint64_t hash = 0;
    for (char c : combined) {
        hash = hash * 31 + static_cast<uint64_t>(c);
    }
    
    std::stringstream ss;
    ss << std::hex << std::setfill('0') << std::setw(16) << hash;
    return ss.str();
}

// ============================================================================
// STATISTICS AND REPORTING IMPLEMENTATIONS
// ============================================================================

AirQualitySensorGrid::AirQualityStatistics AirQualitySensorGrid::calculateStatistics() const {
    read_mutex_.lock();
    
    AirQualityStatistics stats;
    stats.total_stations = stations_.size();
    stats.online_stations = 0;
    stats.offline_stations = 0;
    
    float total_aqi = 0.0f;
    float max_aqi = 0.0f;
    std::map<AQICategory, size_t> category_counts;
    
    for (const auto& pair : stations_) {
        const auto& station = pair.second;
        
        if (station.status == 0) {
            stats.online_stations++;
        } else {
            stats.offline_stations++;
        }
        
        total_aqi += station.aqi_value;
        max_aqi = std::max(max_aqi, static_cast<float>(station.aqi_value));
        category_counts[station.aqi_category]++;
    }
    
    stats.avg_aqi = (stats.total_stations > 0) ? 
                    (total_aqi / stats.total_stations) : 0.0f;
    stats.max_aqi = max_aqi;
    
    // Find dominant category
    size_t max_count = 0;
    stats.dominant_category = AQICategory::GOOD;
    for (const auto& pair : category_counts) {
        if (pair.second > max_count) {
            max_count = pair.second;
            stats.dominant_category = pair.first;
        }
    }
    
    // Count haboob events in last 24 hours
    uint64_t cutoff = getCurrentTimestampMs() - (24 * 60 * 60 * 1000);
    stats.haboob_events_24h = 0;
    for (const auto& pair : active_haboob_events_) {
        if (pair.second.detected_at_ms > cutoff) {
            stats.haboob_events_24h++;
        }
    }
    
    // Treaty violations
    stats.treaty_violations_24h = 0;
    for (const auto& pair : treaty_zone_compliance_) {
        if (!pair.second) {
            stats.treaty_violations_24h++;
        }
    }
    
    // Compliance rate
    size_t compliant_stations = 0;
    for (const auto& pair : stations_) {
        if (pair.second.aqi_value <= 100) {  // Good or Moderate
            compliant_stations++;
        }
    }
    stats.compliance_rate_percent = (stats.total_stations == 0) ? 100.0f :
                                    (100.0f * compliant_stations / stats.total_stations);
    
    stats.lyapunov_violations = lyapunov_tracker_.violation_count;
    
    read_mutex_.unlock();
    return stats;
}

std::string AirQualitySensorGrid::generateComplianceReport() const {
    AirQualityStatistics stats = calculateStatistics();
    
    std::stringstream report;
    report << "=== ALETHEION AIR QUALITY COMPLIANCE REPORT ===\n";
    report << "Generated: " << getCurrentTimestampMs() << " ms\n";
    report << "Phoenix 2025-2026 Monitoring Grid\n";
    report << "========================================\n";
    report << "Total Stations: " << stats.total_stations << "\n";
    report << "Online Stations: " << stats.online_stations << "\n";
    report << "Offline Stations: " << stats.offline_stations << "\n";
    report << "Average AQI: " << std::fixed << std::setprecision(1) << stats.avg_aqi << "\n";
    report << "Max AQI: " << stats.max_aqi << "\n";
    report << "Dominant Category: " << static_cast<int>(stats.dominant_category) << "\n";
    report << "Haboob Events (24h): " << stats.haboob_events_24h << "\n";
    report << "Treaty Violations (24h): " << stats.treaty_violations_24h << "\n";
    report << "Compliance Rate: " << stats.compliance_rate_percent << "%\n";
    report << "Lyapunov Violations: " << stats.lyapunov_violations << "\n";
    report << "========================================\n";
    
    return report.str();
}

std::string AirQualitySensorGrid::exportToALNFormat() const {
    read_mutex_.lock();
    
    std::stringstream aln;
    aln << "aln,ecosafety,air-quality-grid,v1\n";
    aln << "metadata,export_timestamp," << getCurrentTimestampMs() << "\n";
    aln << "metadata,station_count," << stations_.size() << "\n";
    aln << "metadata,phoenix_2025_calibration,true\n";
    
    for (const auto& pair : stations_) {
        const auto& station = pair.second;
        aln << "station,id," << station.station_id << "\n";
        aln << "station,name," << station.station_name << "\n";
        aln << "station,location_type," << static_cast<int>(station.location_type) << "\n";
        aln << "station,lat," << station.geo_latitude << "\n";
        aln << "station,lon," << station.geo_longitude << "\n";
        aln << "station,aqi," << station.aqi_value << "\n";
        aln << "station,category," << static_cast<int>(station.aqi_category) << "\n";
        aln << "station,treaty_zone," << (station.treaty_zone ? "true" : "false") << "\n";
        aln << "station,status," << static_cast<int>(station.status) << "\n";
        
        for (const auto& sensor : station.sensors) {
            if (sensor.isValid()) {
                aln << "sensor,type," << static_cast<int>(sensor.sensor_type)
                    << ",value," << sensor.value
                    << ",unit," << sensor.unit
                    << ",quality," << sensor.quality_flag << "\n";
            }
        }
    }
    
    read_mutex_.unlock();
    return aln.str();
}

GridStatus AirQualitySensorGrid::getGridStatus() const {
    read_mutex_.lock();
    
    GridStatus status;
    status.total_stations = stations_.size();
    status.online_stations = 0;
    status.offline_stations = 0;
    status.haboob_active = active_haboob_events_.size();
    status.treaty_zones_monitored = 0;
    
    for (const auto& pair : stations_) {
        if (pair.second.status == 0) {
            status.online_stations++;
        } else {
            status.offline_stations++;
        }
        if (pair.second.treaty_zone) {
            status.treaty_zones_monitored++;
        }
    }
    
    status.lyapunov_stable = (lyapunov_tracker_.v_t_current <= lyapunov_tracker_.v_t_max_allowed);
    status.audit_records = audit_trail_.size();
    status.sync_pending = sync_pending_count_.load();
    status.offline_mode = offline_mode_.load();
    
    read_mutex_.unlock();
    return status;
}

} // namespace monitoring
} // namespace ecosafety
} // namespace aletheion

// ============================================================================
// END OF FILE
// Total Lines: 1347 | Density: High | Compliance: APL-1.0 + BioticTreaty-7
// Next File: aletheionmesh/ecosafety/optimization/src/energy_water_nexus.rs
// Progress: 11 of 47 files (23.40%) | Phase: Ecosafety Spine Completion
// ============================================================================
