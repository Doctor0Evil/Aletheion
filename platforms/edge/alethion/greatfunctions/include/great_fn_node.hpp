#pragma once

#include <cstdint>
#include <array>

namespace alethion {

struct GreatFnId {
    std::uint16_t city;
    std::uint16_t district;
    std::uint16_t block;
    std::uint16_t lane;
};

struct GreatFnCall {
    GreatFnId target;
    std::uint16_t city_zone;
    std::uint64_t epoch;
    std::array<std::uint32_t, 4> payload_vector;
    std::uint8_t priority;
};

class GreatFnNode {
public:
    GreatFnNode() : ops_budget_(64), processed_(0) {}

    void begin_tick(std::uint64_t epoch) {
        epoch_ = epoch;
        ops_budget_ = 64;
        processed_ = 0;
    }

    bool accept(const GreatFnCall& call) {
        if (processed_ >= 64 || ops_budget_ == 0) {
            return false;
        }
        last_call_ = call;
        ++processed_;
        if (ops_budget_ > 0) {
            --ops_budget_;
        }
        return true;
    }

    std::uint16_t ops_budget() const {
        return ops_budget_;
    }

private:
    std::uint64_t epoch_;
    std::uint16_t ops_budget_;
    std::uint16_t processed_;
    GreatFnCall last_call_;
};

} // namespace alethion
