
#pragma once
#include <cstdint>
#include <string>

namespace aletheion::citymesh {

struct RiskSnapshot {
    double r_degrade;
    double r_residualmass;
    double r_microplastics;
    double r_tox_acute;
    double r_tox_chronic;
    double r_shear;
    double r_habitatload;
};

struct LyapunovCorridor {
    double v_min;
    double v_max;
};

struct LyapunovWeights {
    double w_risk;
    double w_coverage;
    double w_density;
};

struct CityObjectTelemetry {
    double swarm_coverage;
    double agent_density;
    double agent_density_max;
};

struct LyapunovGateResult {
    bool allow;
    std::string reason;
    double v_obj_t;
};

class CityMeshLyapunovCIGate {
public:
    LyapunovGateResult evaluate(
        const RiskSnapshot& risk,
        const LyapunovWeights& weights,
        const LyapunovCorridor& corridor,
        const CityObjectTelemetry& telem,
        double previous_v_obj
    ) const noexcept {
        double r_scalar = aggregate_risk(risk);
        double v_obj = compute_v_obj(r_scalar, weights, telem);
        if (v_obj < corridor.v_min || v_obj > corridor.v_max) {
            return {false, "Lyapunov corridor violated", v_obj};
        }
        if (v_obj > previous_v_obj) {
            return {false, "Lyapunov residual increased", v_obj};
        }
        return {true, "Lyapunov gate passed", v_obj};
    }

private:
    static double aggregate_risk(const RiskSnapshot& r) noexcept {
        double sum =
            r.r_degrade +
            r.r_residualmass +
            r.r_microplastics +
            r.r_tox_acute +
            r.r_tox_chronic +
            r.r_shear +
            r.r_habitatload;
        return sum / 7.0;
    }

    static double compute_v_obj(
        double r_scalar,
        const LyapunovWeights& w,
        const CityObjectTelemetry& t
    ) noexcept {
        double density_term = t.agent_density - t.agent_density_max;
        if (density_term < 0.0) density_term = 0.0;
        return w.w_risk * r_scalar + w.w_coverage * (1.0 - t.swarm_coverage) + w.w_density * density_term;
    }
};

} // namespace aletheion::citymesh
