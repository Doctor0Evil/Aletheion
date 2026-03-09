#ifndef FLOW_ANALYTICS_HPP
#define FLOW_ANALYTICS_HPP

#include <cstdint>
#include <cstddef>
#include <array>

constexpr uint32_t FLOW_ANALYTICS_VERSION = 20260310;
constexpr size_t MAX_RESOURCE_TYPES = 64;
constexpr size_t MAX_FLOW_PATHS = 2048;
constexpr size_t MAX_ANALYTICS_WINDOWS = 256;
constexpr uint64_t ANALYTICS_INTERVAL_S = 300;

enum class ResourceType : uint8_t {
    WATER = 0, ENERGY = 1, MATERIALS = 2, WASTE = 3,
    FOOD = 4, DATA = 5, TRANSPORT = 6, AIR_QUALITY = 7
};

enum class FlowDirection : uint8_t {
    INBOUND = 0, OUTBOUND = 1, INTERNAL = 2, RECIRCULATED = 3
};

struct ResourceFlow {
    uint64_t flow_id;
    ResourceType resource_type;
    FlowDirection direction;
    double quantity;
    char unit[32];
    uint64_t source_node_id;
    uint64_t destination_node_id;
    uint64_t timestamp_ns;
    uint64_t duration_ns;
    double quality_index;
    bool validated;
};

struct FlowPath {
    uint64_t path_id;
    ResourceType resource_type;
    uint64_t node_ids[16];
    size_t node_count;
    double total_distance_km;
    double total_loss_pct;
    double efficiency_0_1;
    uint64_t last_optimization_ns;
    bool operational;
};

struct AnalyticsWindow {
    uint64_t window_id;
    uint64_t start_ns;
    uint64_t end_ns;
    double total_inbound[MAX_RESOURCE_TYPES];
    double total_outbound[MAX_RESOURCE_TYPES];
    double total_internal[MAX_RESOURCE_TYPES];
    double net_balance[MAX_RESOURCE_TYPES];
    double efficiency_scores[MAX_RESOURCE_TYPES];
    uint64_t flow_count;
    bool finalized;
};

struct ResourceMetrics {
    ResourceType resource_type;
    double total_consumed;
    double total_produced;
    double total_wasted;
    double total_recycled;
    double circularity_index;
    double sustainability_score;
    uint64_t last_measurement_ns;
};

class ResourceFlowAnalyticsEngine {
private:
    uint64_t engine_id_;
    char city_code_[8];
    ResourceFlow flows_[MAX_FLOW_PATHS];
    size_t flow_count_;
    FlowPath paths_[MAX_FLOW_PATHS];
    size_t path_count_;
    AnalyticsWindow windows_[MAX_ANALYTICS_WINDOWS];
    size_t window_count_;
    ResourceMetrics metrics_[MAX_RESOURCE_TYPES];
    uint64_t total_flows_processed_;
    uint64_t total_anomalies_detected_;
    uint64_t security_alerts_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= flow_count_ * path_count_;
        sum ^= total_flows_processed_;
        sum ^= total_anomalies_detected_;
        for (size_t i = 0; i < flow_count_; ++i) {
            sum ^= flows_[i].flow_id * static_cast<uint64_t>(flows_[i].resource_type);
        }
        audit_checksum_ = sum;
    }
    
public:
    ResourceFlowAnalyticsEngine(uint64_t engine_id, const char* city_code, uint64_t init_ns)
        : engine_id_(engine_id), flow_count_(0), path_count_(0), window_count_(0),
          total_flows_processed_(0), total_anomalies_detected_(0), security_alerts_(0),
          audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
        for (size_t i = 0; i < MAX_RESOURCE_TYPES; ++i) {
            metrics_[i].resource_type = static_cast<ResourceType>(i);
            metrics_[i].total_consumed = 0.0;
            metrics_[i].total_produced = 0.0;
            metrics_[i].total_wasted = 0.0;
            metrics_[i].total_recycled = 0.0;
            metrics_[i].circularity_index = 0.0;
            metrics_[i].sustainability_score = 1.0;
            metrics_[i].last_measurement_ns = init_ns;
        }
    }
    
    bool RegisterFlow(const ResourceFlow& flow) {
        if (flow_count_ >= MAX_FLOW_PATHS) return false;
        flows_[flow_count_] = flow;
        flow_count_++;
        total_flows_processed_++;
        UpdateMetrics(flow);
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterPath(const FlowPath& path) {
        if (path_count_ >= MAX_FLOW_PATHS) return false;
        paths_[path_count_] = path;
        path_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    void UpdateMetrics(const ResourceFlow& flow) {
        size_t idx = static_cast<size_t>(flow.resource_type);
        if (idx >= MAX_RESOURCE_TYPES) return;
        switch (flow.direction) {
            case FlowDirection::INBOUND:
                metrics_[idx].total_consumed += flow.quantity;
                break;
            case FlowDirection::OUTBOUND:
                metrics_[idx].total_produced += flow.quantity;
                break;
            case FlowDirection::INTERNAL:
                break;
            case FlowDirection::RECIRCULATED:
                metrics_[idx].total_recycled += flow.quantity;
                break;
        }
        if (flow.quality_index < 0.5) {
            metrics_[idx].total_wasted += flow.quantity * (1.0 - flow.quality_index);
        }
        metrics_[idx].last_measurement_ns = flow.timestamp_ns;
        ComputeCircularityIndex(idx);
    }
    
    void ComputeCircularityIndex(size_t resource_idx) {
        if (resource_idx >= MAX_RESOURCE_TYPES) return;
        double total = metrics_[resource_idx].total_consumed + metrics_[resource_idx].total_recycled;
        if (total > 0.0) {
            metrics_[resource_idx].circularity_index = 
                metrics_[resource_idx].total_recycled / total;
        } else {
            metrics_[resource_idx].circularity_index = 0.0;
        }
        double waste_ratio = 0.0;
        if (metrics_[resource_idx].total_consumed > 0.0) {
            waste_ratio = metrics_[resource_idx].total_wasted / metrics_[resource_idx].total_consumed;
        }
        metrics_[resource_idx].sustainability_score = 
            (metrics_[resource_idx].circularity_index * 0.6 + (1.0 - waste_ratio) * 0.4);
    }
    
    bool DetectAnomaly(const ResourceFlow& flow, uint64_t now_ns) {
        size_t idx = static_cast<size_t>(flow.resource_type);
        if (idx >= MAX_RESOURCE_TYPES) return false;
        double avg_flow = metrics_[idx].total_consumed / (total_flows_processed_ + 1);
        if (flow.quantity > avg_flow * 3.0 || flow.quantity < avg_flow * 0.1) {
            total_anomalies_detected_++;
            if (flow.quantity > avg_flow * 5.0) {
                security_alerts_++;
            }
            UpdateAuditChecksum();
            return true;
        }
        return false;
    }
    
    AnalyticsWindow* CreateAnalyticsWindow(uint64_t start_ns, uint64_t duration_ns) {
        if (window_count_ >= MAX_ANALYTICS_WINDOWS) return nullptr;
        AnalyticsWindow& window = windows_[window_count_];
        window.window_id = window_count_;
        window.start_ns = start_ns;
        window.end_ns = start_ns + duration_ns;
        for (size_t i = 0; i < MAX_RESOURCE_TYPES; ++i) {
            window.total_inbound[i] = 0.0;
            window.total_outbound[i] = 0.0;
            window.total_internal[i] = 0.0;
            window.net_balance[i] = 0.0;
            window.efficiency_scores[i] = 1.0;
        }
        window.flow_count = 0;
        window.finalized = false;
        window_count_++;
        return &window;
    }
    
    void AggregateFlowsToWindow(AnalyticsWindow* window, uint64_t now_ns) {
        if (!window || window->finalized) return;
        for (size_t i = 0; i < flow_count_; ++i) {
            if (flows_[i].timestamp_ns >= window->start_ns && 
                flows_[i].timestamp_ns <= window->end_ns) {
                size_t idx = static_cast<size_t>(flows_[i].resource_type);
                if (idx >= MAX_RESOURCE_TYPES) continue;
                switch (flows_[i].direction) {
                    case FlowDirection::INBOUND:
                        window->total_inbound[idx] += flows_[i].quantity;
                        break;
                    case FlowDirection::OUTBOUND:
                        window->total_outbound[idx] += flows_[i].quantity;
                        break;
                    case FlowDirection::INTERNAL:
                        window->total_internal[idx] += flows_[i].quantity;
                        break;
                    case FlowDirection::RECIRCULATED:
                        window->total_inbound[idx] += flows_[i].quantity;
                        break;
                }
                window->flow_count++;
            }
        }
        for (size_t i = 0; i < MAX_RESOURCE_TYPES; ++i) {
            window->net_balance[i] = window->total_inbound[i] - window->total_outbound[i];
            if (window->total_inbound[i] > 0.0) {
                window->efficiency_scores[i] = 
                    (window->total_outbound[i] + window->total_internal[i]) / window->total_inbound[i];
            }
        }
        window->finalized = true;
    }
    
    double ComputeCitywideSustainabilityScore(uint64_t now_ns) {
        double total_score = 0.0;
        size_t valid_metrics = 0;
        for (size_t i = 0; i < MAX_RESOURCE_TYPES; ++i) {
            if (metrics_[i].last_measurement_ns > now_ns - 86400000000000ULL) {
                total_score += metrics_[i].sustainability_score;
                valid_metrics++;
            }
        }
        if (valid_metrics == 0) return 0.0;
        double base_score = total_score / valid_metrics;
        double anomaly_penalty = (total_anomalies_detected_ > 100) ? 0.1 : 0.0;
        double security_penalty = (security_alerts_ > 10) ? 0.15 : 0.0;
        return (base_score - anomaly_penalty - security_penalty);
    }
    
    struct EngineStatus {
        uint64_t engine_id;
        char city_code[8];
        size_t total_flows;
        size_t total_paths;
        size_t total_windows;
        uint64_t total_flows_processed;
        uint64_t total_anomalies_detected;
        uint64_t security_alerts;
        double citywide_sustainability_score;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    EngineStatus GetStatus(uint64_t now_ns) {
        EngineStatus status;
        status.engine_id = engine_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_flows = flow_count_;
        status.total_paths = path_count_;
        status.total_windows = window_count_;
        status.total_flows_processed = total_flows_processed_;
        status.total_anomalies_detected = total_anomalies_detected_;
        status.security_alerts = security_alerts_;
        status.citywide_sustainability_score = ComputeCitywideSustainabilityScore(now_ns);
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= flow_count_ * path_count_;
        sum ^= total_flows_processed_;
        sum ^= total_anomalies_detected_;
        for (size_t i = 0; i < flow_count_; ++i) {
            sum ^= flows_[i].flow_id * static_cast<uint64_t>(flows_[i].resource_type);
        }
        return sum == audit_checksum_;
    }
    
    uint64_t GetEngineId() const { return engine_id_; }
    size_t GetFlowCount() const { return flow_count_; }
    size_t GetPathCount() const { return path_count_; }
    uint64_t GetSecurityAlerts() const { return security_alerts_; }
    uint64_t GetTotalAnomalies() const { return total_anomalies_detected_; }
};

#endif
