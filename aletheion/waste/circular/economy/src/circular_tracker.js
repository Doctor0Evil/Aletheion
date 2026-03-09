// Aletheion Zero-Waste Circular Economy Tracker v20260310
// License: BioticTreaty_v3
// Compliance: Phoenix_ZeroWaste_2026_Arizona_Environmental_Laws_BioticTreaty_v3

const CIRCULAR_TRACKER_VERSION = 20260310;
const MAX_MATERIAL_FLOWS = 4096;
const MAX_CIRCULAR_LOOPS = 1024;
const MAX_BUSINESS_ENTITIES = 2048;
const TARGET_CIRCULARITY_RATE_PCT = 95.0;
const TARGET_WASTE_DIVERSION_PCT = 99.0;

class MaterialFlow {
    constructor(flowId, materialType, massKg, sourceId, destinationId, timestampNs) {
        this.flowId = flowId;
        this.materialType = materialType;
        this.massKg = massKg;
        this.sourceId = sourceId;
        this.destinationId = destinationId;
        this.timestampNs = timestampNs;
        this.flowType = 'linear';
        this.circularityScore = 0.0;
        this.carbonFootprintKg = 0.0;
        this.waterFootprintL = 0.0;
        this.recycled = false;
        this.upcycled = false;
        this.downcycled = false;
        this.landfilled = false;
        this.incinerated = false;
    }
    computeCircularityScore() {
        if (this.recycled || this.upcycled) {
            this.circularityScore = 0.9;
            this.flowType = 'circular';
        } else if (this.downcycled) {
            this.circularityScore = 0.5;
            this.flowType = 'partial_circular';
        } else if (this.landfilled || this.incinerated) {
            this.circularityScore = 0.0;
            this.flowType = 'linear';
        } else {
            this.circularityScore = 0.3;
            this.flowType = 'unknown';
        }
        return this.circularityScore;
    }
    computeCarbonFootprint() {
        const emissionFactors = {
            'plastic': 3.5, 'metal': 2.0, 'glass': 1.5, 'paper': 1.0,
            'organic': 0.5, 'textile': 4.0, 'electronic': 5.0, 'construction': 1.8
        };
        const factor = emissionFactors[this.materialType] || 2.0;
        this.carbonFootprintKg = this.massKg * factor * (1.0 - this.circularityScore);
        return this.carbonFootprintKg;
    }
    computeWaterFootprint() {
        const waterFactors = {
            'plastic': 100, 'metal': 150, 'glass': 50, 'paper': 200,
            'organic': 20, 'textile': 500, 'electronic': 300, 'construction': 80
        };
        const factor = waterFactors[this.materialType] || 100;
        this.waterFootprintL = this.massKg * factor * (1.0 - this.circularityScore * 0.8);
        return this.waterFootprintL;
    }
}

class CircularLoop {
    constructor(loopId, loopName, loopType, participants) {
        this.loopId = loopId;
        this.loopName = loopName;
        this.loopType = loopType;
        this.participants = participants;
        this.materialInputs = [];
        this.materialOutputs = [];
        this.totalMassInKg = 0.0;
        this.totalMassOutKg = 0.0;
        this.lossesKg = 0.0;
        this.efficiency = 0.0;
        this.createdAtNs = Date.now() * 1000000;
        this.lastOptimizedNs = Date.now() * 1000000;
        this.operational = true;
    }
    addMaterialInput(flow) {
        this.materialInputs.push(flow);
        this.totalMassInKg += flow.massKg;
    }
    addMaterialOutput(flow) {
        this.materialOutputs.push(flow);
        this.totalMassOutKg += flow.massKg;
    }
    computeEfficiency() {
        this.lossesKg = this.totalMassInKg - this.totalMassOutKg;
        if (this.totalMassInKg === 0) {
            this.efficiency = 0.0;
        } else {
            this.efficiency = this.totalMassOutKg / this.totalMassInKg;
        }
        return this.efficiency;
    }
    computeLoopCircularity() {
        const circularOutputs = this.materialOutputs.filter(f => f.circularityScore > 0.5).length;
        if (this.materialOutputs.length === 0) return 0.0;
        return circularOutputs / this.materialOutputs.length;
    }
}

class BusinessEntity {
    constructor(entityId, entityName, entityType, sector) {
        this.entityId = entityId;
        this.entityName = entityName;
        this.entityType = entityType;
        this.sector = sector;
        this.materialInputs = [];
        this.materialOutputs = [];
        this.wasteGeneratedKg = 0.0;
        this.wasteDivertedKg = 0.0;
        this.recyclingRate = 0.0;
        this.circularityScore = 0.0;
        this.certifications = [];
        this.complianceStatus = 'compliant';
        this.lastAuditNs = 0;
    }
    computeRecyclingRate() {
        if (this.wasteGeneratedKg === 0) {
            this.recyclingRate = 0.0;
        } else {
            this.recyclingRate = this.wasteDivertedKg / this.wasteGeneratedKg;
        }
        return this.recyclingRate;
    }
    computeCircularityScore() {
        const recyclingScore = this.recyclingRate;
        const wasteReductionScore = 1.0 - (this.wasteGeneratedKg / 10000);
        const complianceScore = this.complianceStatus === 'compliant' ? 1.0 : 0.5;
        this.circularityScore = recyclingScore * 0.5 + 
                               Math.max(0, wasteReductionScore) * 0.3 + 
                               complianceScore * 0.2;
        return this.circularityScore;
    }
}

class ZeroWasteCircularEconomyTracker {
    constructor(trackerId, cityCode, region) {
        this.trackerId = trackerId;
        this.cityCode = cityCode;
        this.region = region;
        this.materialFlows = [];
        this.flowCount = 0;
        this.circularLoops = [];
        this.loopCount = 0;
        this.businessEntities = [];
        this.entityCount = 0;
        this.totalMaterialThroughputKg = 0.0;
        this.totalCircularMassKg = 0.0;
        this.totalLinearMassKg = 0.0;
        this.totalCarbonSavedKg = 0.0;
        this.totalWaterSavedL = 0.0;
        this.totalWasteDivertedKg = 0.0;
        this.totalWasteLandfilledKg = 0.0;
        this.citywideCircularityRate = 0.0;
        this.lastOptimizationNs = Date.now() * 1000000;
    }
    registerMaterialFlow(flow) {
        if (this.flowCount >= MAX_MATERIAL_FLOWS) return false;
        flow.computeCircularityScore();
        flow.computeCarbonFootprint();
        flow.computeWaterFootprint();
        this.materialFlows.push(flow);
        this.flowCount++;
        this.totalMaterialThroughputKg += flow.massKg;
        if (flow.circularityScore > 0.5) {
            this.totalCircularMassKg += flow.massKg;
            this.totalCarbonSavedKg += flow.carbonFootprintKg * 0.7;
            this.totalWaterSavedL += flow.waterFootprintL * 0.6;
        } else {
            this.totalLinearMassKg += flow.massKg;
        }
        if (!flow.landfilled) {
            this.totalWasteDivertedKg += flow.massKg;
        } else {
            this.totalWasteLandfilledKg += flow.massKg;
        }
        return true;
    }
    registerCircularLoop(loop) {
        if (this.loopCount >= MAX_CIRCULAR_LOOPS) return false;
        this.circularLoops.push(loop);
        this.loopCount++;
        return true;
    }
    registerBusinessEntity(entity) {
        if (this.entityCount >= MAX_BUSINESS_ENTITIES) return false;
        this.businessEntities.push(entity);
        this.entityCount++;
        return true;
    }
    computeCitywideCircularityRate() {
        if (this.totalMaterialThroughputKg === 0) {
            this.citywideCircularityRate = 0.0;
        } else {
            this.citywideCircularityRate = this.totalCircularMassKg / 
                                          this.totalMaterialThroughputKg * 100.0;
        }
        return this.citywideCircularityRate;
    }
    computeWasteDiversionRate() {
        const totalWaste = this.totalWasteDivertedKg + this.totalWasteLandfilledKg;
        if (totalWaste === 0) return 0.0;
        return this.totalWasteDivertedKg / totalWaste * 100.0;
    }
    identifyCircularityOpportunities() {
        const opportunities = [];
        const linearFlows = this.materialFlows.filter(f => f.flowType === 'linear');
        for (const flow of linearFlows) {
            opportunities.push({
                flowId: flow.flowId,
                materialType: flow.materialType,
                massKg: flow.massKg,
                currentDestination: flow.destinationId,
                suggestedAction: 'Find circular destination',
                potentialCarbonSavings: flow.carbonFootprintKg * 0.7,
                potentialWaterSavings: flow.waterFootprintL * 0.6,
                priority: flow.massKg > 1000 ? 'HIGH' : 'MEDIUM'
            });
        }
        return opportunities.sort((a, b) => b.massKg - a.massKg);
    }
    optimizeCircularLoops(nowNs) {
        for (const loop of this.circularLoops) {
            if (loop.operational) {
                loop.computeEfficiency();
                loop.lastOptimizedNs = nowNs;
                if (loop.efficiency < 0.8) {
                    loop.operational = false;
                }
            }
        }
        this.lastOptimizationNs = nowNs;
    }
    generateCircularEconomyReport(nowNs) {
        this.computeCitywideCircularityRate();
        const diversionRate = this.computeWasteDiversionRate();
        const operationalLoops = this.circularLoops.filter(l => l.operational).length;
        const compliantEntities = this.businessEntities.filter(
            e => e.complianceStatus === 'compliant'
        ).length;
        return {
            trackerId: this.trackerId,
            cityCode: this.cityCode,
            region: this.region,
            reportTimestampNs: nowNs,
            totalMaterialFlows: this.flowCount,
            totalCircularLoops: this.loopCount,
            operationalLoops: operationalLoops,
            totalBusinessEntities: this.entityCount,
            compliantEntities: compliantEntities,
            totalMaterialThroughputKg: this.totalMaterialThroughputKg,
            totalCircularMassKg: this.totalCircularMassKg,
            totalLinearMassKg: this.totalLinearMassKg,
            citywideCircularityRate: this.citywideCircularityRate,
            wasteDiversionRate: diversionRate,
            totalCarbonSavedKg: this.totalCarbonSavedKg,
            totalWaterSavedL: this.totalWaterSavedL,
            totalWasteDivertedKg: this.totalWasteDivertedKg,
            totalWasteLandfilledKg: this.totalWasteLandfilledKg,
            circularityTargetProgress: this.citywideCircularityRate / TARGET_CIRCULARITY_RATE_PCT,
            diversionTargetProgress: diversionRate / TARGET_WASTE_DIVERSION_PCT,
            lastOptimizationNs: this.lastOptimizationNs
        };
    }
    computeSystemEfficiency() {
        const circularityScore = this.citywideCircularityRate / 100.0;
        const diversionScore = this.computeWasteDiversionRate() / 100.0;
        const loopEfficiency = this.circularLoops.filter(l => l.operational).length / 
                              this.circularLoops.length.max(1);
        const entityCompliance = this.businessEntities.filter(
            e => e.complianceStatus === 'compliant'
        ).length / this.businessEntities.length.max(1);
        return (circularityScore * 0.35 + diversionScore * 0.30 + 
                loopEfficiency * 0.20 + entityCompliance * 0.15).Math.min(1.0);
    }
}

module.exports = {
    MaterialFlow,
    CircularLoop,
    BusinessEntity,
    ZeroWasteCircularEconomyTracker,
    VERSION: CIRCULAR_TRACKER_VERSION,
    TARGET_CIRCULARITY_RATE_PCT,
    TARGET_WASTE_DIVERSION_PCT,
};
