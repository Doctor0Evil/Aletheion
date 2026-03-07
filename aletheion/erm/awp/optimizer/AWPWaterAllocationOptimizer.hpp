#pragma once
// Aletheion :: ERM :: AWP :: AWPWaterAllocationOptimizer
// Path: aletheion/erm/awp/optimizer/AWPWaterAllocationOptimizer.hpp
// Role: Stateless optimizer for allocating Advanced Purified Water (AWP)
//       from Phoenix facilities (Cave Creek, North Gateway, 91st Ave)
//       to demand zones including Shade/Cool Corridor thermal sinks.

#include <string>
#include <vector>
#include <unordered_map>
#include <cstdint>
#include <limits>
#include <algorithm>
#include <cmath>

namespace aletheion {
namespace erm {
namespace awp {

struct PlantId {
    std::string value;
};

struct DemandZoneId {
    std::string value;
};

struct Date {
    int year;
    int month;
    int day;
};

struct DailyCapacityMGD {
    double mgd; // million gallons per day
};

struct DailyDemandMGD {
    double mgd;
};

struct ReliabilityWeight {
    double weight; // [0,1], higher = more critical
};

struct DistanceKm {
    double km;
};

struct ThermalPriority {
    double weight; // for shade / cool corridor zones
};

struct Plant {
    PlantId id;
    std::string name;
    DailyCapacityMGD nominal_capacity_mgd;
    double reliability_score;      // [0,1] based on infrastructure maturity, grid resilience
    bool is_online;                // current operability flag
    bool is_advanced_purified;     // true for AWP (Cave Creek, North Gateway, 91st Ave)[web:56][web:137][web:148]
};

struct DemandZone {
    DemandZoneId id;
    std::string name;
    DailyDemandMGD base_potable_demand_mgd;   // indoor + outdoor baseline
    DailyDemandMGD shade_irrigation_demand_mgd; // trees, shade structures, cool corridors[web:149][web:158][web:84][web:141]
    ReliabilityWeight criticality;            // hospitals, cooling centers, etc.
    ThermalPriority thermal_priority;         // high in heat-vulnerable neighborhoods[web:149][web:158][web:142]
};

struct NetworkEdge {
    PlantId plant_id;
    DemandZoneId zone_id;
    DistanceKm distance_km;
    double conveyance_loss_ratio; // 0.0..1.0 fraction lost in transit (leakage, evaporation)
};

struct AllocationDecision {
    PlantId plant_id;
    DemandZoneId zone_id;
    double allocated_mgd; // at plant outlet
};

struct AllocationResult {
    Date date;
    std::vector<AllocationDecision> decisions;
    std::unordered_map<std::string, double> unmet_demand_mgd_by_zone;
    std::unordered_map<std::string, double> used_capacity_mgd_by_plant;
    double portfolio_reuse_fraction; // AWP fraction of total supplied water
};

struct PortfolioTargets {
    double min_reuse_fraction;     // e.g., target share of supply from AWP
    double max_groundwater_fraction;
    double max_colorado_river_fraction;
};

struct NonAWPSupply {
    // aggregate non-AWP contributions (surface water, groundwater, exchanges)[web:53][web:142]
    double groundwater_mgd;
    double colorado_river_mgd;
    double other_surface_mgd;
};

class AWPWaterAllocationOptimizer {
public:
    static AllocationResult optimize(
        const Date& date,
        const std::vector<Plant>& plants,
        const std::vector<DemandZone>& zones,
        const std::vector<NetworkEdge>& network,
        const PortfolioTargets& portfolio_targets,
        const NonAWPSupply& non_awp_supply
    )
    {
        AllocationResult result;
        result.date = date;
        result.decisions.clear();
        result.unmet_demand_mgd_by_zone.clear();
        result.used_capacity_mgd_by_plant.clear();
        result.portfolio_reuse_fraction = 0.0;

        // index helper maps
        std::unordered_map<std::string, const Plant*> plant_by_id;
        for (const auto& p : plants) {
            plant_by_id[p.id.value] = &p;
            result.used_capacity_mgd_by_plant[p.id.value] = 0.0;
        }

        std::unordered_map<std::string, std::vector<const NetworkEdge*>> edges_from_plant;
        std::unordered_map<std::string, std::vector<const NetworkEdge*>> edges_to_zone;
        for (const auto& e : network) {
            edges_from_plant[e.plant_id.value].push_back(&e);
            edges_to_zone[e.zone_id.value].push_back(&e);
        }

        // compute effective demand per zone (potable + weighted thermal irrigation)
        std::unordered_map<std::string, double> total_demand_mgd_by_zone;
        for (const auto& z : zones) {
            double base = z.base_potable_demand_mgd.mgd;
            double thermal = z.shade_irrigation_demand_mgd.mgd * (0.5 + 0.5 * z.thermal_priority.weight);
            total_demand_mgd_by_zone[z.id.value] = base + thermal;
            result.unmet_demand_mgd_by_zone[z.id.value] = base + thermal;
        }

        // compute an "AWP attractiveness score" for each plant
        std::unordered_map<std::string, double> plant_score;
        for (const auto& p : plants) {
            if (!p.is_online || !p.is_advanced_purified) {
                plant_score[p.id.value] = 0.0;
                continue;
            }
            // leverage AWP plants preferentially as reuse backbone[web:56][web:137][web:148][web:157]
            plant_score[p.id.value] = 0.5 + 0.5 * p.reliability_score;
        }

        // compute "priority score" for each zone
        std::unordered_map<std::string, double> zone_score;
        for (const auto& z : zones) {
            double s = 0.4 * z.criticality.weight + 0.4 * z.thermal_priority.weight + 0.2;
            zone_score[z.id.value] = s;
        }

        // approximate available AWP capacity
        double total_awp_capacity = 0.0;
        for (const auto& p : plants) {
            if (p.is_online && p.is_advanced_purified) {
                total_awp_capacity += p.nominal_capacity_mgd.mgd;
            }
        }

        // approximate total demand
        double total_demand = 0.0;
        for (const auto& kv : total_demand_mgd_by_zone) {
            total_demand += kv.second;
        }

        // compute non-AWP supply contribution
        double total_non_awp_supply = non_awp_supply.groundwater_mgd
                                    + non_awp_supply.colorado_river_mgd
                                    + non_awp_supply.other_surface_mgd;

        // initial target AWP contribution
        double desired_awp_supply = std::min(total_demand,
                                             std::max(0.0, portfolio_targets.min_reuse_fraction * (total_demand)));

        desired_awp_supply = std::min(desired_awp_supply, total_awp_capacity);

        // distribute AWP plant capacities proportionally to plant_score
        std::unordered_map<std::string, double> awp_capacity_budget;
        double score_sum = 0.0;
        for (const auto& kv : plant_score) {
            score_sum += kv.second;
        }
        if (score_sum <= 0.0) {
            score_sum = 1.0;
        }
        for (const auto& p : plants) {
            if (!p.is_online || !p.is_advanced_purified) {
                awp_capacity_budget[p.id.value] = 0.0;
            } else {
                awp_capacity_budget[p.id.value] = desired_awp_supply * (plant_score[p.id.value] / score_sum);
                awp_capacity_budget[p.id.value] = std::min(awp_capacity_budget[p.id.value], p.nominal_capacity_mgd.mgd);
            }
        }

        // order zones by priority
        std::vector<const DemandZone*> sorted_zones;
        sorted_zones.reserve(zones.size());
        for (const auto& z : zones) {
            sorted_zones.push_back(&z);
        }
        std::sort(sorted_zones.begin(), sorted_zones.end(),
                  [&zone_score](const DemandZone* a, const DemandZone* b) {
                      return zone_score[a->id.value] > zone_score[b->id.value];
                  });

        // greedy allocation loop
        for (const auto* zptr : sorted_zones) {
            auto& zone = *zptr;
            double remaining_demand = result.unmet_demand_mgd_by_zone[zone.id.value];
            if (remaining_demand <= 0.0) continue;

            auto itEdges = edges_to_zone.find(zone.id.value);
            if (itEdges == edges_to_zone.end()) {
                continue;
            }

            // sort incoming edges by (distance, plant_score) preference
            auto incoming = itEdges->second;
            std::sort(incoming.begin(), incoming.end(),
                      [&plant_score](const NetworkEdge* a, const NetworkEdge* b) {
                          double score_a = plant_score[a->plant_id.value];
                          double score_b = plant_score[b->plant_id.value];
                          if (std::fabs(score_a - score_b) > 1e-9) {
                              return score_a > score_b; // prefer higher-scored plants
                          }
                          return a->distance_km.km < b->distance_km.km;
                      });

            for (const auto* edge : incoming) {
                if (remaining_demand <= 1e-9) break;
                auto pit = awp_capacity_budget.find(edge->plant_id.value);
                if (pit == awp_capacity_budget.end()) continue;
                double available_at_plant = pit->second;
                if (available_at_plant <= 1e-9) continue;

                double loss_factor = 1.0 - std::max(0.0, std::min(1.0, edge->conveyance_loss_ratio));
                if (loss_factor <= 0.0) continue;

                double needed_at_plant = remaining_demand / loss_factor;
                double allocated_at_plant = std::min(available_at_plant, needed_at_plant);
                double delivered = allocated_at_plant * loss_factor;

                if (delivered <= 1e-9) continue;

                AllocationDecision d;
                d.plant_id = edge->plant_id;
                d.zone_id = zone.id;
                d.allocated_mgd = allocated_at_plant;
                result.decisions.push_back(d);

                pit->second -= allocated_at_plant;
                result.used_capacity_mgd_by_plant[edge->plant_id.value] += allocated_at_plant;
                remaining_demand -= delivered;
                result.unmet_demand_mgd_by_zone[zone.id.value] = std::max(0.0, remaining_demand);
            }
        }

        // compute portfolio reuse fraction using approximate total supply
        double supplied_from_awp = 0.0;
        for (const auto& kv : result.used_capacity_mgd_by_plant) {
            const auto& pid = kv.first;
            auto pIt = plant_by_id.find(pid);
            if (pIt != plant_by_id.end() && pIt->second->is_advanced_purified) {
                supplied_from_awp += kv.second;
            }
        }

        double total_supplied = supplied_from_awp + total_non_awp_supply;
        if (total_supplied > 0.0) {
            result.portfolio_reuse_fraction = supplied_from_awp / total_supplied;
        } else {
            result.portfolio_reuse_fraction = 0.0;
        }

        return result;
    }
};

} // namespace awp
} // namespace erm
} // namespace aletheion
