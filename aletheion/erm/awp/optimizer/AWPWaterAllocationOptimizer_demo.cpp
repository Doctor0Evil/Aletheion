// Aletheion :: ERM :: AWP :: Demo
// Path: aletheion/erm/awp/optimizer/AWPWaterAllocationOptimizer_demo.cpp
// Role: Example usage of AWPWaterAllocationOptimizer with Phoenix AWP plants
//       and three demand zones (downtown cool corridor, high-vulnerability
//       neighborhood, industrial user).

#include <iostream>
#include "AWPWaterAllocationOptimizer.hpp"

using namespace aletheion::erm::awp;

int main() {
    Date d{2026, 7, 15};

    // --- Define AWP plants (Cave Creek, North Gateway, 91st Ave) ---

    // Cave Creek: being rehabilitated to ~8 MGD AWP capacity by 2026,
    // expandable to higher volumes over time.[web:139][web:164][web:166]
    Plant caveCreek{
        PlantId{"plant_cave_creek"},
        "Cave Creek AWP",
        DailyCapacityMGD{8.0},
        0.82,   // reliability score (planned, but backed by major investment)[web:139][web:164][web:166]
        true,   // online in this scenario
        true    // advanced purified
    };

    // North Gateway: federally funded 8 MGD AWP facility; combined with Cave Creek
    // expected to yield ~14,000 acre-feet/year potable water.[web:148][web:167]
    Plant northGateway{
        PlantId{"plant_north_gateway"},
        "North Gateway AWP",
        DailyCapacityMGD{8.0},
        0.88,
        true,
        true
    };

    // 91st Ave: larger AWP facility under the same portfolio; treated abstractly here.[web:148][web:137][web:56]
    Plant p91st{
        PlantId{"plant_91st_ave"},
        "91st Ave AWP",
        DailyCapacityMGD{12.0},
        0.9,
        true,
        true
    };

    std::vector<Plant> plants{caveCreek, northGateway, p91st};

    // --- Define demand zones ---

    // 1) Downtown Cool Corridor: high thermal priority, moderate base demand,
    // shade-irrigation demand for trees/structures.[web:165][web:168][web:149][web:158]
    DemandZone downtownCool{
        DemandZoneId{"zone_downtown_cool_corridor"},
        "Downtown Cool Corridor",
        DailyDemandMGD{12.0},   // potable + commercial baseline
        DailyDemandMGD{4.0},    // irrigation for shade trees/structures
        ReliabilityWeight{0.7}, // important but not hospital-level
        ThermalPriority{0.95}   // extreme heat vulnerability focus[web:165][web:168][web:149][web:158]
    };

    // 2) High-Vulnerability Neighborhood: residential, high thermal priority,
    // lower base demand but critical for equity-focused shade plan.[web:165][web:168]
    DemandZone vulnNeighborhood{
        DemandZoneId{"zone_high_vulnerability_neighborhood"},
        "Heat-Vulnerable Neighborhood",
        DailyDemandMGD{6.0},
        DailyDemandMGD{3.0},
        ReliabilityWeight{0.85}, // high criticality (cooling centers, clinics)
        ThermalPriority{1.0}     // top shade priority per Shade Phoenix Plan[web:165][web:168]
    };

    // 3) Industrial User: lower thermal priority, high absolute demand but
    // not prioritized over residential/thermal equity loads.[web:53][web:142]
    DemandZone industrialUser{
        DemandZoneId{"zone_industrial_user"},
        "Industrial Water User",
        DailyDemandMGD{10.0},
        DailyDemandMGD{1.0},
        ReliabilityWeight{0.4},
        ThermalPriority{0.2}
    };

    std::vector<DemandZone> zones{downtownCool, vulnNeighborhood, industrialUser};

    // --- Define network edges (simplified distances & losses) ---

    std::vector<NetworkEdge> network{
        // Cave Creek to downtown and neighborhood[web:139][web:163]
        NetworkEdge{caveCreek.id, downtownCool.id, DistanceKm{18.0}, 0.03},
        NetworkEdge{caveCreek.id, vulnNeighborhood.id, DistanceKm{12.0}, 0.02},

        // North Gateway to downtown, neighborhood, industrial[web:148][web:167]
        NetworkEdge{northGateway.id, downtownCool.id, DistanceKm{20.0}, 0.03},
        NetworkEdge{northGateway.id, vulnNeighborhood.id, DistanceKm{10.0}, 0.02},
        NetworkEdge{northGateway.id, industrialUser.id, DistanceKm{15.0}, 0.03},

        // 91st Ave to downtown and industrial (west valley connection)[web:148][web:137]
        NetworkEdge{p91st.id, downtownCool.id, DistanceKm{25.0}, 0.04},
        NetworkEdge{p91st.id, industrialUser.id, DistanceKm{8.0}, 0.02}
    };

    // --- Portfolio targets & non-AWP supply ---

    PortfolioTargets portfolio{
        0.30, // aim for >=30% of supplied water from reuse/AWP[web:53][web:142]
        0.35, // max groundwater fraction (placeholder constraint)
        0.40  // max Colorado River fraction (placeholder constraint)
    };

    // Example non-AWP supply for the day: mixture of groundwater, Colorado River,
    // and other surface sources (Salt/Verde), consistent with diversified portfolio.[web:53][web:142]
    NonAWPSupply non_awp{
        15.0, // groundwater MGD
        12.0, // Colorado River MGD
        8.0   // other surface MGD
    };

    // --- Compute allocation ---

    AllocationResult res = AWPWaterAllocationOptimizer::optimize(
        d, plants, zones, network, portfolio, non_awp
    );

    // --- Report ---

    std::cout << "AWPWaterAllocationOptimizer demo for "
              << res.date.year << "-" << res.date.month << "-" << res.date.day << "\n\n";

    std::cout << "Decisions:\n";
    for (const auto& dec : res.decisions) {
        std::cout << "  Plant " << dec.plant_id.value
                  << " -> Zone " << dec.zone_id.value
                  << " : " << dec.allocated_mgd << " MGD at plant outlet\n";
    }

    std::cout << "\nUsed capacity per plant (MGD):\n";
    for (const auto& kv : res.used_capacity_mgd_by_plant) {
        std::cout << "  " << kv.first << " : " << kv.second << "\n";
    }

    std::cout << "\nUnmet demand per zone (MGD):\n";
    for (const auto& kv : res.unmet_demand_mgd_by_zone) {
        std::cout << "  " << kv.first << " : " << kv.second << "\n";
    }

    std::cout << "\nPortfolio reuse fraction (AWP share of total supplied water): "
              << res.portfolio_reuse_fraction << "\n";

    return 0;
}
