#pragma once

#include <cstdint>
#include <array>

namespace alethion::agro {

enum class TribalScope : std::uint8_t {
    None = 0,
    JointStewardship = 1,
    IndependentStewardship = 2
};

struct GeoCellId {
    std::uint16_t township;
    std::uint16_t range;
    std::uint8_t section;
    std::uint8_t quarter;
};

struct FpicConsentTag {
    std::array<char, 4> tribal_code;
    std::uint64_t consent_epoch_utc;
    std::uint16_t consent_version;
    bool on_device_only;
};

struct PesticideEvent {
    std::uint64_t event_epoch_utc;
    GeoCellId geo_cell;
    std::array<char, 32> phoenix_soil;
    std::array<char, 64> active_ingredient;
    std::array<char, 16> product_epa_reg_no;
    std::uint32_t rate_value;
    std::uint32_t total_area_sq_m;
    std::array<char, 16> applicator_epa_establishment_id;
    std::array<char, 16> operator_local_id;
    TribalScope tribal_scope;
    bool has_fpic;
    FpicConsentTag fpic;
};

enum class PesticideEventError {
    None = 0,
    MissingRegNumber,
    ZeroArea,
    InvalidGeoCell,
    MissingApplicator,
    MissingFpicForTribalScope
};

inline bool is_zero_str(const std::array<char, 16>& buf) {
    for (char c : buf) {
        if (c != 0) {
            return false;
        }
    }
    return true;
}

inline PesticideEventError validate(const PesticideEvent& ev) {
    if (is_zero_str(ev.product_epa_reg_no)) {
        return PesticideEventError::MissingRegNumber;
    }
    if (ev.total_area_sq_m == 0) {
        return PesticideEventError::ZeroArea;
    }
    if (ev.geo_cell.section == 0 || ev.geo_cell.section > 36) {
        return PesticideEventError::InvalidGeoCell;
    }
    if (is_zero_str(ev.applicator_epa_establishment_id)) {
        return PesticideEventError::MissingApplicator;
    }
    if (ev.tribal_scope != TribalScope::None) {
        if (!ev.has_fpic || ev.fpic.consent_epoch_utc == 0) {
            return PesticideEventError::MissingFpicForTribalScope;
        }
    }
    return PesticideEventError::None;
}

} // namespace alethion::agro
