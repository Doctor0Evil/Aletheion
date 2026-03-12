#include <cstdint>
#include <cstring>
#include "tsn_snn_node_adapter.cpp"

struct PosNonFinancialEventFrame {
    char sku[32];
    int32_t quantity;
    uint64_t ts;
    char device_id[32];
};

class TsnSnnPosBridge {
public:
    explicit TsnSnnPosBridge(const NeuromorphicHint& hint)
        : adapter_(hint) {}

    SnSpikingPacket fromPosEvent(const PosNonFinancialEventFrame& ev) const {
        EdgeSensorFrame frame{};
        frame.ts = ev.ts;
        frame.value = static_cast<float>(ev.quantity);
        frame.channel = 0;
        frame.flags = 0;
        return adapter_.toSpikingPacket(frame);
    }

    bool requiresTsn() const {
        return adapter_.isTsnEnabled();
    }

private:
    TsnSnnNodeAdapter adapter_;
};
