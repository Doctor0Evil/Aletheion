// Aletheion Cool Pavement Urban Heat Mitigation Controller v20260310
// License: BioticTreaty_v3
// Compliance: Neurorights_v1, Phoenix_Heat_Resilience_Protocol_2026

const HEAT_MITIGATION_VERSION = 20260310;
const MAX_PAVEMENT_SEGMENTS = 2048;
const MAX_COOLING_ZONES = 256;
const TARGET_SURFACE_TEMP_REDUCTION_C = 10.5;
const MISTING_ACTIVATION_TEMP_C = 38.0;
const ALBEDO_ENHANCEMENT_INTERVAL_MS = 86400000;

class PavementSegment {
    constructor(segmentId, lat, lon, lengthM, widthM) {
        this.segmentId = segmentId;
        this.latitude = lat;
        this.longitude = lon;
        this.lengthM = lengthM;
        this.widthM = widthM;
        this.areaM2 = lengthM * widthM;
        this.surfaceTemperatureC = 0.0;
        this.subsurfaceTemperatureC = 0.0;
        this.albedoCoefficient = 0.15;
        this.targetAlbedo = 0.35;
        this.mistingSystemActive = false;
        this.mistingFlowRateLMin = 0.0;
        this.waterConsumptionL = 0.0;
        this.energyConsumptionKWh = 0.0;
        this.lastMaintenanceNs = 0;
        this.operational = true;
        this.degradationIndex = 0.0;
    }
    computeTemperatureReduction(ambientTempC, solarRadiationWm2) {
        const baselineTemp = ambientTempC + (solarRadiationWm2 * 0.015 * (1 - this.albedoCoefficient));
        const currentReduction = baselineTemp - this.surfaceTemperatureC;
        return { baselineTemp, currentReduction, targetReduction: TARGET_SURFACE_TEMP_REDUCTION_C };
    }
    activateMistingSystem(flowRateLMin, nowNs) {
        if (!this.operational) return false;
        this.mistingSystemActive = true;
        this.mistingFlowRateLMin = Math.min(flowRateLMin, 50.0);
        this.lastMistingActivationNs = nowNs;
        return true;
    }
    deactivateMistingSystem() {
        this.mistingSystemActive = false;
        this.mistingFlowRateLMin = 0.0;
    }
    updateAlbedoCoating(newAlbedo, nowNs) {
        if (newAlbedo < 0.2 || newAlbedo > 0.5) return false;
        this.albedoCoefficient = newAlbedo;
        this.lastMaintenanceNs = nowNs;
        this.degradationIndex = 0.0;
        return true;
    }
    computeDegradation(nowNs) {
        const elapsedDays = (nowNs - this.lastMaintenanceNs) / 86400000000000;
        this.degradationIndex = Math.min(elapsedDays * 0.002, 1.0);
        this.albedoCoefficient = this.targetAlbedo * (1 - this.degradationIndex * 0.3);
        return this.degradationIndex;
    }
    isMaintenanceRequired(nowNs) {
        const elapsedDays = (nowNs - this.lastMaintenanceNs) / 86400000000000;
        return elapsedDays > 180 || this.degradationIndex > 0.5;
    }
}

class CoolingZone {
    constructor(zoneId, name, neighborhood) {
        this.zoneId = zoneId;
        this.name = name;
        this.neighborhood = neighborhood;
        this.pavementSegments = [];
        this.segmentCount = 0;
        this.averageSurfaceTempC = 0.0;
        this.averageAmbientTempC = 0.0;
        this.heatIndex = 0.0;
        this.vulnerablePopulationCount = 0;
        this.coolingCenters = [];
        this.mistingStations = 0;
        this.totalWaterConsumptionL = 0.0;
        this.totalEnergyConsumptionKWh = 0.0;
        this.alertLevel = 0;
        this.lastAlertNs = 0;
    }
    addPavementSegment(segment) {
        if (this.segmentCount >= 128) return false;
        this.pavementSegments[this.segmentCount] = segment;
        this.segmentCount++;
        return true;
    }
    computeZoneMetrics(nowNs) {
        let totalSurfaceTemp = 0.0;
        let totalAmbientTemp = 0.0;
        let activeMisting = 0;
        this.totalWaterConsumptionL = 0.0;
        this.totalEnergyConsumptionKWh = 0.0;
        for (let i = 0; i < this.segmentCount; i++) {
            const seg = this.pavementSegments[i];
            seg.computeDegradation(nowNs);
            totalSurfaceTemp += seg.surfaceTemperatureC;
            totalAmbientTemp += seg.subsurfaceTemperatureC;
            if (seg.mistingSystemActive) {
                activeMisting++;
                this.totalWaterConsumptionL += seg.mistingFlowRateLMin * 0.06;
                this.totalEnergyConsumptionKWh += 0.5;
            }
        }
        this.averageSurfaceTempC = totalSurfaceTemp / Math.max(this.segmentCount, 1);
        this.averageAmbientTempC = totalAmbientTemp / Math.max(this.segmentCount, 1);
        this.heatIndex = this.computeHeatIndex(this.averageSurfaceTempC, this.averageAmbientTempC);
        this.mistingStations = activeMisting;
    }
    computeHeatIndex(surfaceTempC, ambientTempC) {
        if (surfaceTempC < 27.0) return 0.0;
        const tempComponent = (surfaceTempC - 27.0) * 0.1;
        const humidityComponent = 0.0;
        return Math.min(tempComponent + humidityComponent, 1.0);
    }
    determineAlertLevel() {
        if (this.heatIndex >= 0.8 || this.averageSurfaceTempC >= 50.0) {
            this.alertLevel = 4;
            return "EXTREME_HEAT_EMERGENCY";
        } else if (this.heatIndex >= 0.6 || this.averageSurfaceTempC >= 45.0) {
            this.alertLevel = 3;
            return "SEVERE_HEAT_WARNING";
        } else if (this.heatIndex >= 0.4 || this.averageSurfaceTempC >= 40.0) {
            this.alertLevel = 2;
            return "MODERATE_HEAT_ALERT";
        } else if (this.heatIndex >= 0.2 || this.averageSurfaceTempC >= 35.0) {
            this.alertLevel = 1;
            return "LOW_HEAT_ADVISORY";
        } else {
            this.alertLevel = 0;
            return "NORMAL";
        }
    }
    optimizeMistingStrategy(ambientTempC, solarRadiationWm2, waterBudgetL) {
        let totalFlowNeeded = 0.0;
        for (let i = 0; i < this.segmentCount; i++) {
            const seg = this.pavementSegments[i];
            const tempReduction = seg.computeTemperatureReduction(ambientTempC, solarRadiationWm2);
            if (tempReduction.baselineTemp > MISTING_ACTIVATION_TEMP_C && seg.operational) {
                const requiredFlow = Math.min(30.0, (tempReduction.baselineTemp - MISTING_ACTIVATION_TEMP_C) * 2.0);
                totalFlowNeeded += requiredFlow;
            }
        }
        const allocationFactor = Math.min(1.0, waterBudgetL / Math.max(totalFlowNeeded, 1));
        for (let i = 0; i < this.segmentCount; i++) {
            const seg = this.pavementSegments[i];
            const tempReduction = seg.computeTemperatureReduction(ambientTempC, solarRadiationWm2);
            if (tempReduction.baselineTemp > MISTING_ACTIVATION_TEMP_C && seg.operational) {
                const allocatedFlow = Math.min(30.0, (tempReduction.baselineTemp - MISTING_ACTIVATION_TEMP_C) * 2.0) * allocationFactor;
                seg.activateMistingSystem(allocatedFlow, Date.now() * 1000000);
            } else {
                seg.deactivateMistingSystem();
            }
        }
        return { totalFlowNeeded: totalFlowNeeded * allocationFactor, allocationFactor };
    }
}

class UrbanHeatMitigationController {
    constructor(controllerId, cityCode) {
        this.controllerId = controllerId;
        this.cityCode = cityCode;
        this.coolingZones = [];
        this.zoneCount = 0;
        this.totalPavementAreaM2 = 0.0;
        this.totalWaterConsumptionL = 0.0;
        this.totalEnergyConsumptionKWh = 0.0;
        this.systemHealthScore = 1.0;
        this.lastSystemCheckNs = 0;
        this.alertHistory = [];
        this.alertCount = 0;
    }
    registerCoolingZone(zone) {
        if (this.zoneCount >= MAX_COOLING_ZONES) return false;
        this.coolingZones[this.zoneCount] = zone;
        this.zoneCount++;
        for (let i = 0; i < zone.segmentCount; i++) {
            this.totalPavementAreaM2 += zone.pavementSegments[i].areaM2;
        }
        return true;
    }
    processAllZones(nowNs, ambientTempC, solarRadiationWm2, waterBudgetL) {
        this.totalWaterConsumptionL = 0.0;
        this.totalEnergyConsumptionKWh = 0.0;
        let criticalZones = [];
        for (let i = 0; i < this.zoneCount; i++) {
            const zone = this.coolingZones[i];
            zone.computeZoneMetrics(nowNs);
            zone.optimizeMistingStrategy(ambientTempC, solarRadiationWm2, waterBudgetL / this.zoneCount);
            const alertType = zone.determineAlertLevel();
            this.totalWaterConsumptionL += zone.totalWaterConsumptionL;
            this.totalEnergyConsumptionKWh += zone.totalEnergyConsumptionKWh;
            if (zone.alertLevel >= 3) {
                criticalZones.push(zone.zoneId);
            }
            if (zone.alertLevel > 0 && (nowNs - zone.lastAlertNs) > 300000000000) {
                this.alertCount++;
                this.alertHistory[this.alertCount] = {
                    zoneId: zone.zoneId,
                    alertLevel: zone.alertLevel,
                    alertType: alertType,
                    heatIndex: zone.heatIndex,
                    timestampNs: nowNs
                };
                zone.lastAlertNs = nowNs;
            }
        }
        this.lastSystemCheckNs = nowNs;
        this.computeSystemHealth(nowNs);
        return criticalZones;
    }
    computeSystemHealth(nowNs) {
        let operationalSegments = 0;
        let totalSegments = 0;
        let maintenanceOverdue = 0;
        for (let i = 0; i < this.zoneCount; i++) {
            const zone = this.coolingZones[i];
            for (let j = 0; j < zone.segmentCount; j++) {
                const seg = zone.pavementSegments[j];
                totalSegments++;
                if (seg.operational) operationalSegments++;
                if (seg.isMaintenanceRequired(nowNs)) maintenanceOverdue++;
            }
        }
        const operationalRatio = operationalSegments / Math.max(totalSegments, 1);
        const maintenanceRatio = 1.0 - (maintenanceOverdue / Math.max(totalSegments, 1));
        this.systemHealthScore = operationalRatio * 0.6 + maintenanceRatio * 0.4;
    }
    generatePublicHeatAdvisory(criticalZones) {
        const advisories = [];
        for (let i = 0; i < criticalZones.length; i++) {
            const zoneId = criticalZones[i];
            for (let j = 0; j < this.zoneCount; j++) {
                const zone = this.coolingZones[j];
                if (zone.zoneId === zoneId) {
                    advisories.push({
                        zoneId: zone.zoneId,
                        zoneName: zone.name,
                        neighborhood: zone.neighborhood,
                        alertLevel: zone.alertLevel,
                        surfaceTempC: zone.averageSurfaceTempC.toFixed(1),
                        heatIndex: zone.heatIndex.toFixed(2),
                        coolingCenters: zone.coolingCenters,
                        recommendation: zone.alertLevel >= 4 ? "SEEK_SHELTER_IMMEDIATELY" : "LIMIT_OUTDOOR_EXPOSURE"
                    });
                    break;
                }
            }
        }
        return advisories;
    }
    getSystemStatus(nowNs) {
        let totalSegments = 0;
        let operationalSegments = 0;
        let mistingActive = 0;
        for (let i = 0; i < this.zoneCount; i++) {
            const zone = this.coolingZones[i];
            for (let j = 0; j < zone.segmentCount; j++) {
                const seg = zone.pavementSegments[j];
                totalSegments++;
                if (seg.operational) operationalSegments++;
                if (seg.mistingSystemActive) mistingActive++;
            }
        }
        const avgTempReduction = this.computeAverageTemperatureReduction();
        return {
            controllerId: this.controllerId,
            cityCode: this.cityCode,
            totalZones: this.zoneCount,
            totalPavementAreaM2: this.totalPavementAreaM2,
            totalSegments: totalSegments,
            operationalSegments: operationalSegments,
            segmentHealthPct: (operationalSegments / Math.max(totalSegments, 1) * 100).toFixed(1),
            mistingStationsActive: mistingActive,
            totalWaterConsumptionL: this.totalWaterConsumptionL.toFixed(1),
            totalEnergyConsumptionKWh: this.totalEnergyConsumptionKWh.toFixed(1),
            averageTempReductionC: avgTempReduction.toFixed(1),
            systemHealthScore: this.systemHealthScore.toFixed(2),
            totalAlertsIssued: this.alertCount,
            lastSystemCheckNs: this.lastSystemCheckNs
        };
    }
    computeAverageTemperatureReduction() {
        let totalReduction = 0.0;
        let count = 0;
        for (let i = 0; i < this.zoneCount; i++) {
            const zone = this.coolingZones[i];
            for (let j = 0; j < zone.segmentCount; j++) {
                const seg = zone.pavementSegments[j];
                const reduction = seg.computeTemperatureReduction(zone.averageAmbientTempC, 800);
                totalReduction += reduction.currentReduction;
                count++;
            }
        }
        return count > 0 ? totalReduction / count : 0.0;
    }
    computeWaterEfficiency() {
        if (this.totalWaterConsumptionL === 0) return 1.0;
        let totalAreaCovered = 0.0;
        for (let i = 0; i < this.zoneCount; i++) {
            const zone = this.coolingZones[i];
            for (let j = 0; j < zone.segmentCount; j++) {
                if (zone.pavementSegments[j].mistingSystemActive) {
                    totalAreaCovered += zone.pavementSegments[j].areaM2;
                }
            }
        }
        const efficiency = totalAreaCovered / Math.max(this.totalWaterConsumptionL, 1);
        return Math.min(efficiency * 10, 1.0);
    }
}

module.exports = {
    PavementSegment,
    CoolingZone,
    UrbanHeatMitigationController,
    VERSION: HEAT_MITIGATION_VERSION,
    TARGET_SURFACE_TEMP_REDUCTION_C,
    MISTING_ACTIVATION_TEMP_C,
};
