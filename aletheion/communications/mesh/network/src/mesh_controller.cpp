#ifndef MESH_CONTROLLER_HPP
#define MESH_CONTROLLER_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t MESH_CONTROLLER_VERSION = 20260310;
constexpr size_t MAX_MESH_NODES = 16384;
constexpr size_t MAX_CONNECTIONS_PER_NODE = 64;
constexpr size_t MAX_MESSAGE_QUEUES = 8192;
constexpr size_t MAX_NETWORK_SEGMENTS = 512;
constexpr double PHOENIX_METRO_AREA_KM2 = 1608.0;
constexpr uint32_t MESH_FREQUENCY_MHZ = 915;
constexpr uint32_t MAX_TRANSMIT_POWER_MW = 1000;

enum class NodeType : uint8_t {
    GATEWAY = 0, ROUTER = 1, END_DEVICE = 2, REPEATER = 3,
    SENSOR = 4, ACTUATOR = 5, MOBILE = 6, FIXED = 7
};

enum class ConnectionQuality : uint8_t {
    EXCELLENT = 0, GOOD = 1, FAIR = 2, POOR = 3, LOST = 4
};

enum class EncryptionStandard : uint8_t {
    AES_256_GCM = 0, CHACHA20_POLY1305 = 1, X25519_ED25519 = 2,
    POST_QUANTUM_KYBER = 3, POST_QUANTUM_DILITHIUM = 4
};

struct MeshNode {
    uint64_t node_id;
    NodeType node_type;
    double latitude;
    double longitude;
    double elevation_m;
    uint32_t transmit_power_mw;
    uint32_t frequency_mhz;
    EncryptionStandard encryption;
    uint64_t connections[MAX_CONNECTIONS_PER_NODE];
    size_t connection_count;
    ConnectionQuality uplink_quality;
    ConnectionQuality downlink_quality;
    double battery_pct;
    uint64_t uptime_s;
    uint64_t last_heartbeat_ns;
    uint64_t messages_sent;
    uint64_t messages_received;
    uint64_t messages_forwarded;
    bool operational;
    bool gateway_capable;
    bool solar_powered;
};

struct NetworkSegment {
    uint64_t segment_id;
    uint64_t gateway_node_id;
    uint64_t node_ids[256];
    size_t node_count;
    double coverage_area_km2;
    double avg_signal_strength_dbm;
    double packet_delivery_ratio;
    double latency_ms;
    uint64_t total_messages;
    uint64_t dropped_messages;
    bool operational;
    bool internet_connected;
};

struct MessageQueue {
    uint64_t queue_id;
    uint64_t source_node_id;
    uint64_t destination_node_id;
    uint32_t message_size_bytes;
    uint64_t created_ns;
    uint64_t expires_ns;
    uint8_t priority;
    uint8_t retry_count;
    bool encrypted;
    bool delivered;
    bool acknowledged;
};

class MeshNetworkController {
private:
    uint64_t controller_id_;
    char city_code_[8];
    MeshNode nodes_[MAX_MESH_NODES];
    size_t node_count_;
    NetworkSegment segments_[MAX_NETWORK_SEGMENTS];
    size_t segment_count_;
    MessageQueue queues_[MAX_MESSAGE_QUEUES];
    size_t queue_head_;
    size_t queue_tail_;
    size_t queue_size_;
    uint64_t total_messages_processed_;
    uint64_t total_messages_dropped_;
    uint64_t total_bytes_transmitted_;
    double network_coverage_pct_;
    double avg_latency_ms_;
    double packet_delivery_ratio_;
    uint64_t security_alerts_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= node_count_ * segment_count_ * queue_size_;
        sum ^= total_messages_processed_;
        sum ^= total_messages_dropped_;
        sum ^= security_alerts_;
        for (size_t i = 0; i < node_count_; ++i) {
            sum ^= nodes_[i].node_id * static_cast<uint64_t>(nodes_[i].operational);
        }
        audit_checksum_ = sum;
    }
    
    double ComputeDistance(double lat1, double lon1, double lat2, double lon2) {
        const double earth_radius_km = 6371.0;
        const double d_lat = (lat2 - lat1) * M_PI / 180.0;
        const double d_lon = (lon2 - lon1) * M_PI / 180.0;
        const double a = std::sin(d_lat / 2) * std::sin(d_lat / 2) +
                        std::cos(lat1 * M_PI / 180.0) * std::cos(lat2 * M_PI / 180.0) *
                        std::sin(d_lon / 2) * std::sin(d_lon / 2);
        const double c = 2 * std::atan2(std::sqrt(a), std::sqrt(1 - a));
        return earth_radius_km * c;
    }
    
    ConnectionQuality ComputeConnectionQuality(double signal_strength_dbm) {
        if (signal_strength_dbm >= -50) return ConnectionQuality::EXCELLENT;
        if (signal_strength_dbm >= -70) return ConnectionQuality::GOOD;
        if (signal_strength_dbm >= -85) return ConnectionQuality::FAIR;
        if (signal_strength_dbm >= -95) return ConnectionQuality::POOR;
        return ConnectionQuality::LOST;
    }
    
public:
    MeshNetworkController(uint64_t controller_id, const char* city_code, uint64_t init_ns)
        : controller_id_(controller_id), node_count_(0), segment_count_(0),
          queue_head_(0), queue_tail_(0), queue_size_(0),
          total_messages_processed_(0), total_messages_dropped_(0),
          total_bytes_transmitted_(0), network_coverage_pct_(0.0),
          avg_latency_ms_(0.0), packet_delivery_ratio_(1.0),
          security_alerts_(0), audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterMeshNode(const MeshNode& node) {
        if (node_count_ >= MAX_MESH_NODES) return false;
        nodes_[node_count_] = node;
        node_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterNetworkSegment(const NetworkSegment& segment) {
        if (segment_count_ >= MAX_NETWORK_SEGMENTS) return false;
        segments_[segment_count_] = segment;
        segment_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool EnqueueMessage(const MessageQueue& message) {
        if (queue_size_ >= MAX_MESSAGE_QUEUES) return false;
        queues_[queue_tail_] = message;
        queue_tail_ = (queue_tail_ + 1) % MAX_MESSAGE_QUEUES;
        queue_size_++;
        return true;
    }
    
    MessageQueue DequeueMessage() {
        if (queue_size_ == 0) return MessageQueue{};
        MessageQueue msg = queues_[queue_head_];
        queues_[queue_head_] = MessageQueue{};
        queue_head_ = (queue_head_ + 1) % MAX_MESSAGE_QUEUES;
        queue_size_--;
        return msg;
    }
    
    void ProcessMessageQueue(uint64_t now_ns) {
        size_t processed = 0;
        size_t initial_size = queue_size_;
        while (queue_size_ > 0 && processed < 100) {
            MessageQueue msg = DequeueMessage();
            if (now_ns > msg.expires_ns) {
                total_messages_dropped_++;
                continue;
            }
            msg.delivered = true;
            msg.acknowledged = true;
            total_messages_processed_++;
            total_bytes_transmitted_ += msg.message_size_bytes;
            processed++;
        }
        if (initial_size > 0) {
            packet_delivery_ratio_ = static_cast<double>(total_messages_processed_) /
                                    (total_messages_processed_ + total_messages_dropped_);
        }
        UpdateAuditChecksum();
    }
    
    void ComputeNetworkCoverage() {
        double covered_area = 0.0;
        size_t operational_gateways = 0;
        for (size_t i = 0; i < segment_count_; ++i) {
            if (segments_[i].operational && segments_[i].internet_connected) {
                covered_area += segments_[i].coverage_area_km2;
                operational_gateways++;
            }
        }
        network_coverage_pct_ = (covered_area / PHOENIX_METRO_AREA_KM2) * 100.0;
    }
    
    void ComputeAverageLatency() {
        double total_latency = 0.0;
        size_t valid_segments = 0;
        for (size_t i = 0; i < segment_count_; ++i) {
            if (segments_[i].operational) {
                total_latency += segments_[i].latency_ms;
                valid_segments++;
            }
        }
        avg_latency_ms_ = valid_segments > 0 ? total_latency / valid_segments : 0.0;
    }
    
    bool DetectNodeFailure(uint64_t node_id, uint64_t now_ns) {
        for (size_t i = 0; i < node_count_; ++i) {
            if (nodes_[i].node_id == node_id) {
                uint64_t elapsed_s = (now_ns - nodes_[i].last_heartbeat_ns) / 1000000000;
                if (elapsed_s > 300 || !nodes_[i].operational) {
                    security_alerts_++;
                    UpdateAuditChecksum();
                    return true;
                }
            }
        }
        return false;
    }
    
    struct ControllerStatus {
        uint64_t controller_id;
        char city_code[8];
        size_t total_nodes;
        size_t operational_nodes;
        size_t total_segments;
        size_t operational_segments;
        size_t queue_size;
        uint64_t total_messages_processed;
        uint64_t total_messages_dropped;
        uint64_t total_bytes_transmitted;
        double network_coverage_pct;
        double avg_latency_ms;
        double packet_delivery_ratio;
        uint64_t security_alerts;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    ControllerStatus GetStatus(uint64_t now_ns) {
        ControllerStatus status;
        status.controller_id = controller_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_nodes = node_count_;
        status.operational_nodes = 0;
        for (size_t i = 0; i < node_count_; ++i) {
            if (nodes_[i].operational) status.operational_nodes++;
        }
        status.total_segments = segment_count_;
        status.operational_segments = 0;
        for (size_t i = 0; i < segment_count_; ++i) {
            if (segments_[i].operational) status.operational_segments++;
        }
        status.queue_size = queue_size_;
        status.total_messages_processed = total_messages_processed_;
        status.total_messages_dropped = total_messages_dropped_;
        status.total_bytes_transmitted = total_bytes_transmitted_;
        ComputeNetworkCoverage();
        status.network_coverage_pct = network_coverage_pct_;
        ComputeAverageLatency();
        status.avg_latency_ms = avg_latency_ms_;
        status.packet_delivery_ratio = packet_delivery_ratio_;
        status.security_alerts = security_alerts_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    double ComputeNetworkHealthScore(uint64_t now_ns) {
        double node_health = static_cast<double>(GetStatus(now_ns).operational_nodes) / 
                            node_count_.max(1);
        double segment_health = static_cast<double>(GetStatus(now_ns).operational_segments) / 
                               segment_count_.max(1);
        double delivery_health = packet_delivery_ratio_;
        double latency_health = avg_latency_ms_ < 50 ? 1.0 : (avg_latency_ms_ < 100 ? 0.8 : 0.6);
        double security_penalty = (security_alerts_ > 10) ? 0.15 : 0.0;
        return (node_health * 0.3 + segment_health * 0.25 + 
                delivery_health * 0.25 + latency_health * 0.2 - security_penalty);
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= node_count_ * segment_count_ * queue_size_;
        sum ^= total_messages_processed_;
        sum ^= total_messages_dropped_;
        sum ^= security_alerts_;
        for (size_t i = 0; i < node_count_; ++i) {
            sum ^= nodes_[i].node_id * static_cast<uint64_t>(nodes_[i].operational);
        }
        return sum == audit_checksum_;
    }
    
    void OptimizeNetwork(uint64_t now_ns) {
        last_optimization_ns_ = now_ns;
        ProcessMessageQueue(now_ns);
        ComputeNetworkCoverage();
        ComputeAverageLatency();
        UpdateAuditChecksum();
    }
};

#endif
