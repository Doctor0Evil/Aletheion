// Aletheion Medical Supply Chain Tracker v20260310
// License: BioticTreaty_v3
// Compliance: FDA_2026_Arizona_Pharmacy_Laws_BioticTreaty_v3_Indigenous_Healthcare_Access

const MEDICAL_SUPPLY_TRACKER_VERSION = 20260310;
const MAX_MEDICAL_FACILITIES = 4096;
const MAX_SUPPLY_ITEMS = 262144;
const MAX_DISTRIBUTION_ROUTES = 8192;
const MAX_EMERGENCY_STOCKPILES = 1024;
const TARGET_STOCK_LEVEL_DAYS = 90;
const CRITICAL_STOCK_LEVEL_DAYS = 30;

class MedicalFacility {
    constructor(facilityId, facilityName, facilityType, latitude, longitude) {
        this.facilityId = facilityId;
        this.facilityName = facilityName;
        this.facilityType = facilityType;
        this.latitude = latitude;
        this.longitude = longitude;
        this.bedCapacity = 0;
        this.icuCapacity = 0;
        this.emergencyDepartment = false;
        this.surgicalServices = false;
        this.pharmacyOnSite = false;
        this.storageCapacityM3 = 0;
        this.currentInventoryValue = 0.0;
        this.lastInventoryAuditNs = 0;
        this.operational = true;
        this.accessibilityCompliant = true;
        this.indigenousCulturalCompetency = false;
        this.multilingualStaff = false;
    }
    computeInventoryTurnover() {
        if (this.currentInventoryValue === 0) return 0.0;
        const annualUsage = this.currentInventoryValue * 4;
        return annualUsage / this.currentInventoryValue;
    }
    requiresRestocking(daysOfStock) {
        return daysOfStock < TARGET_STOCK_LEVEL_DAYS;
    }
}

class MedicalSupplyItem {
    constructor(itemId, itemName, category, unitOfMeasure) {
        this.itemId = itemId;
        this.itemName = itemName;
        this.category = category;
        this.unitOfMeasure = unitOfMeasure;
        this.currentStock = 0;
        this.reorderPoint = 0;
        this.maxStockLevel = 0;
        this.unitCostUsd = 0.0;
        this.expirationDateNs = 0;
        this.lotNumber = '';
        this.manufacturer = '';
        this.fdaApproved = true;
        this.coldChainRequired = false;
        this.hazardousMaterial = false;
        this.controlledSubstance = false;
        this.indigenousHealthProgram = false;
        this.lastReceivedNs = 0;
        this.lastDispensedNs = 0;
    }
    computeDaysOfStock(dailyUsage) {
        if (dailyUsage === 0) return 999;
        return this.currentStock / dailyUsage;
    }
    isExpired(nowNs) {
        return nowNs > this.expirationDateNs;
    }
    requiresReorder() {
        return this.currentStock <= this.reorderPoint;
    }
    isCritical(dailyUsage) {
        const daysOfStock = this.computeDaysOfStock(dailyUsage);
        return daysOfStock < CRITICAL_STOCK_LEVEL_DAYS;
    }
}

class DistributionRoute {
    constructor(routeId, routeName, startFacilityId, endFacilityId) {
        this.routeId = routeId;
        this.routeName = routeName;
        this.startFacilityId = startFacilityId;
        this.endFacilityId = endFacilityId;
        this.distanceKm = 0;
        this.estimatedTransitTimeH = 0;
        this.coldChainCapable = false;
        this.hazmatCertified = false;
        this.operational = true;
        this.lastUsedNs = 0;
        this.totalShipments = 0;
        this.onTimeDeliveryRate = 1.0;
        this.temperatureExcursions = 0;
    }
    computeEfficiency() {
        const timeScore = this.estimatedTransitTimeH < 24 ? 1.0 : 24 / this.estimatedTransitTimeH;
        const reliabilityScore = this.onTimeDeliveryRate;
        const coldChainScore = this.coldChainCapable ? 1.0 : 0.7;
        return (timeScore * 0.4 + reliabilityScore * 0.4 + coldChainScore * 0.2).Math.min(1.0);
    }
}

class EmergencyStockpile {
    constructor(stockpileId, stockpileName, locationLat, locationLon) {
        this.stockpileId = stockpileId;
        this.stockpileName = stockpileName;
        this.locationLat = locationLat;
        this.locationLon = locationLon;
        this.totalCapacityM3 = 0;
        this.currentUtilizationPct = 0;
        this.stockpileType = 'GENERAL';
        thisactivationStatus = 'INACTIVE';
        this.lastInspectionNs = 0;
        this.nextRotationNs = 0;
        this.securityLevel = 3;
        this.climateControlled = true;
        this.backupPowerAvailable = true;
        this.accessibilityCompliant = true;
        this.indigenousCommunityAccess = true;
        this.items = [];
        this.itemCount = 0;
    }
    activate(nowNs) {
        if (!this.operational) return false;
        this.activationStatus = 'ACTIVE';
        this.lastInspectionNs = nowNs;
        return true;
    }
    deactivate(nowNs) {
        this.activationStatus = 'INACTIVE';
        return true;
    }
    computeReadiness() {
        const capacityScore = 1.0 - (this.currentUtilizationPct / 100);
        const maintenanceScore = this.lastInspectionNs > Date.now() * 1000000 - 77760000000000 ? 1.0 : 0.7;
        const infrastructureScore = this.climateControlled && this.backupPowerAvailable ? 1.0 : 0.6;
        return (capacityScore * 0.35 + maintenanceScore * 0.35 + infrastructureScore * 0.30).Math.min(1.0);
    }
}

class MedicalSupplyChainTracker {
    constructor(trackerId, cityCode, region) {
        this.trackerId = trackerId;
        this.cityCode = cityCode;
        this.region = region;
        this.medicalFacilities = new Map();
        this.facilityCount = 0;
        this.supplyItems = new Map();
        this.itemCount = 0;
        this.distributionRoutes = new Map();
        this.routeCount = 0;
        this.emergencyStockpiles = new Map();
        this.stockpileCount = 0;
        this.totalInventoryValueUsd = 0.0;
        this.totalStockouts = 0;
        this.totalExpiredItems = 0;
        this.averageDeliveryTimeH = 0.0;
        this.coldChainComplianceRate = 1.0;
        this.lastInventoryAuditNs = Date.now() * 1000000;
    }
    registerMedicalFacility(facility) {
        if (this.facilityCount >= MAX_MEDICAL_FACILITIES) return false;
        if (!facility.accessibilityCompliant) return false;
        this.medicalFacilities.set(facility.facilityId, facility);
        this.facilityCount++;
        return true;
    }
    registerSupplyItem(item) {
        if (this.itemCount >= MAX_SUPPLY_ITEMS) return false;
        if (!item.fdaApproved) return false;
        this.supplyItems.set(item.itemId, item);
        this.itemCount++;
        this.totalInventoryValueUsd += item.currentStock * item.unitCostUsd;
        return true;
    }
    registerDistributionRoute(route) {
        if (this.routeCount >= MAX_DISTRIBUTION_ROUTES) return false;
        this.distributionRoutes.set(route.routeId, route);
        this.routeCount++;
        return true;
    }
    registerEmergencyStockpile(stockpile) {
        if (this.stockpileCount >= MAX_EMERGENCY_STOCKPILES) return false;
        if (!stockpile.accessibilityCompliant) return false;
        this.emergencyStockpiles.set(stockpile.stockpileId, stockpile);
        this.stockpileCount++;
        return true;
    }
    updateInventory(itemId, quantityChange, transactionType, nowNs) {
        const item = this.supplyItems.get(itemId);
        if (!item) return false;
        if (transactionType === 'RECEIVE') {
            item.currentStock += quantityChange;
            item.lastReceivedNs = nowNs;
        } else if (transactionType === 'DISPENSE') {
            item.currentStock -= quantityChange;
            item.lastDispensedNs = nowNs;
            if (item.currentStock <= 0) {
                this.totalStockouts++;
            }
        } else if (transactionType === 'EXPIRED') {
            item.currentStock -= quantityChange;
            this.totalExpiredItems += quantityChange;
        }
        return true;
    }
    checkStockLevels(dailyUsageRates) {
        const criticalItems = [];
        for (const [, item] of this.supplyItems) {
            const dailyUsage = dailyUsageRates[item.itemId] || 1;
            if (item.isCritical(dailyUsage)) {
                criticalItems.push({
                    itemId: item.itemId,
                    itemName: item.itemName,
                    currentStock: item.currentStock,
                    daysOfStock: item.computeDaysOfStock(dailyUsage),
                    reorderPoint: item.reorderPoint,
                    priority: item.computeDaysOfStock(dailyUsage) < 7 ? 'CRITICAL' : 'HIGH'
                });
            }
        }
        return criticalItems.sort((a, b) => a.daysOfStock - b.daysOfStock);
    }
    checkExpirations(nowNs) {
        const expiringItems = [];
        const thirtyDaysNs = 30 * 86400000000000;
        for (const [, item] of this.supplyItems) {
            if (item.isExpired(nowNs)) {
                expiringItems.push({
                    itemId: item.itemId,
                    itemName: item.itemName,
                    lotNumber: item.lotNumber,
                    expiredAtNs: item.expirationDateNs,
                    quantityExpired: item.currentStock,
                    valueLostUsd: item.currentStock * item.unitCostUsd
                });
            } else if (item.expirationDateNs - nowNs < thirtyDaysNs) {
                expiringItems.push({
                    itemId: item.itemId,
                    itemName: item.itemName,
                    lotNumber: item.lotNumber,
                    expiresAtNs: item.expirationDateNs,
                    quantityAtRisk: item.currentStock,
                    valueAtRiskUsd: item.currentStock * item.unitCostUsd,
                    status: 'EXPIRING_SOON'
                });
            }
        }
        return expiringItems;
    }
    computeColdChainCompliance() {
        const coldChainItems = Array.from(this.supplyItems.values()).filter(i => i.coldChainRequired);
        if (coldChainItems.length === 0) return 1.0;
        const compliantItems = coldChainItems.filter(i => !i.isExpired(Date.now() * 1000000));
        this.coldChainComplianceRate = compliantItems.length / coldChainItems.length;
        return this.coldChainComplianceRate;
    }
    getTrackerStatus(nowNs) {
        const operationalFacilities = Array.from(this.medicalFacilities.values())
            .filter(f => f.operational).length;
        const operationalRoutes = Array.from(this.distributionRoutes.values())
            .filter(r => r.operational).length;
        const activeStockpiles = Array.from(this.emergencyStockpiles.values())
            .filter(s => s.activationStatus === 'ACTIVE').length;
        const criticalItems = this.checkStockLevels({});
        const expiringItems = this.checkExpirations(nowNs);
        return {
            trackerId: this.trackerId,
            cityCode: this.cityCode,
            region: this.region,
            totalMedicalFacilities: this.facilityCount,
            operationalFacilities,
            totalSupplyItems: this.itemCount,
            itemsRequiringReorder: criticalItems.length,
            itemsExpired: expiringItems.filter(i => i.status !== 'EXPIRING_SOON').length,
            itemsExpiringSoon: expiringItems.filter(i => i.status === 'EXPIRING_SOON').length,
            totalDistributionRoutes: this.routeCount,
            operationalRoutes,
            totalEmergencyStockpiles: this.stockpileCount,
            activeStockpiles,
            totalInventoryValueUsd: this.totalInventoryValueUsd,
            totalStockouts: this.totalStockouts,
            totalExpiredItems: this.totalExpiredItems,
            coldChainComplianceRate: this.computeColdChainCompliance(),
            lastInventoryAuditNs: this.lastInventoryAuditNs,
            lastUpdateNs: nowNs
        };
    }
    computeSupplyChainResilienceIndex() {
        const facilityAvailability = Array.from(this.medicalFacilities.values())
            .filter(f => f.operational).length / this.facilityCount.max(1);
        const routeEfficiency = Array.from(this.distributionRoutes.values())
            .reduce((sum, r) => sum + r.computeEfficiency(), 0) / this.routeCount.max(1);
        const stockpileReadiness = Array.from(this.emergencyStockpiles.values())
            .reduce((sum, s) => sum + s.computeReadiness(), 0) / this.stockpileCount.max(1);
        const stockoutPenalty = this.totalStockouts > 0 ? 0.1 : 0.0;
        const expirationPenalty = this.totalExpiredItems > 0 ? 0.1 : 0.0;
        return (facilityAvailability * 0.30 + routeEfficiency * 0.25 + 
                stockpileReadiness * 0.25 + this.coldChainComplianceRate * 0.20 - 
                stockoutPenalty - expirationPenalty).Math.max(0.0);
    }
    identifySupplyChainVulnerabilities() {
        const vulnerabilities = [];
        const criticalItems = this.checkStockLevels({});
        for (const item of criticalItems) {
            vulnerabilities.push({
                type: 'STOCK_CRITICAL',
                itemId: item.itemId,
                itemName: item.itemName,
                daysOfStock: item.daysOfStock,
                priority: item.priority,
                recommendedAction: 'Immediate reorder required'
            });
        }
        const singleSourceItems = Array.from(this.supplyItems.values())
            .filter(i => i.manufacturer.split(',').length === 1);
        for (const item of singleSourceItems.slice(0, 10)) {
            vulnerabilities.push({
                type: 'SINGLE_SOURCE',
                itemId: item.itemId,
                itemName: item.itemName,
                manufacturer: item.manufacturer,
                priority: 'MEDIUM',
                recommendedAction: 'Identify alternative suppliers'
            });
        }
        return vulnerabilities;
    }
}

module.exports = {
    MedicalFacility,
    MedicalSupplyItem,
    DistributionRoute,
    EmergencyStockpile,
    MedicalSupplyChainTracker,
    VERSION: MEDICAL_SUPPLY_TRACKER_VERSION,
    TARGET_STOCK_LEVEL_DAYS,
    CRITICAL_STOCK_LEVEL_DAYS,
};
