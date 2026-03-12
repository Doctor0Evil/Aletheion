#include <cstdint>
#include <cstring>

struct NeuromorphicHint {
    uint8_t tsn_enabled;
    uint8_t snn_backend;
    uint16_t power_budget_mw;
    uint16_t reserved;
};

struct EdgeSensorFrame {
    uint64_t ts;
    float value;
    uint8_t channel;
    uint8_t flags;
};

struct SnSpikingPacket {
    uint64_t ts;
    uint16_t neuron_id;
    uint16_t weight_idx;
    float amplitude;
};

class TsnSnnNodeAdapter {
public:
    explicit TsnSnnNodeAdapter(const NeuromorphicHint& hint)
        : hint_(hint) {}

    bool isTsnEnabled() const {
        return hint_.tsn_enabled != 0;
    }

    SnSpikingPacket toSpikingPacket(const EdgeSensorFrame& frame) const {
        SnSpikingPacket pkt{};
        pkt.ts = frame.ts;
        pkt.neuron_id = static_cast<uint16_t>(frame.channel);
        pkt.weight_idx = 0;
        pkt.amplitude = frame.value;
        return pkt;
    }

private:
    NeuromorphicHint hint_;
};
