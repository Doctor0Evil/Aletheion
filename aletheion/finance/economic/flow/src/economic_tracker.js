// Aletheion Local Economic Flow Tracker v20260310
// License: BioticTreaty_v3
// Compliance: Arizona_Financial_Laws_2026_BioticTreaty_v3_Indigenous_Economic_Rights

const ECONOMIC_TRACKER_VERSION = 20260310;
const MAX_BUSINESS_ENTITIES = 65536;
const MAX_ECONOMIC_TRANSACTIONS = 2097152;
const MAX_SUPPLY_CHAINS = 8192;
const PHOENIX_LIVING_WAGE_USD_HOUR = 22.50;
const LOCAL_MULTIPLIER_TARGET = 0.75;

class BusinessEntity {
    constructor(entityId, entityName, entityType, sector, locationLat, locationLon) {
        this.entityId = entityId;
        this.entityName = entityName;
        this.entityType = entityType;
        this.sector = sector;
        this.locationLat = locationLat;
        this.locationLon = locationLon;
        this.employeeCount = 0;
        this.averageWageUsd = 0.0;
        this.localOwnershipPct = 100.0;
        this.indigenousOwned = false;
        this.certifiedB Corp = false;
        this.cooperative = false;
        this.revenueUsd = 0.0;
        this.localSpendPct = 0.0;
        this.taxContributionUsd = 0.0;
        this.communityInvestmentUsd = 0.0;
        this.createdAtNs = Date.now() * 1000000;
        this.lastReportingNs = Date.now() * 1000000;
        this.complianceScore = 100.0;
        this.operational = true;
    }
    computeLivingWageCompliance() {
        return this.averageWageUsd >= PHOENIX_LIVING_WAGE_USD_HOUR ? 1.0 : this.averageWageUsd / PHOENIX_LIVING_WAGE_USD_HOUR;
    }
    computeLocalMultiplier() {
        return this.localSpendPct / 100.0;
    }
    computeSocialImpactScore() {
        let score = 0.0;
        score += this.computeLivingWageCompliance() * 0.25;
        score += this.localOwnershipPct / 100.0 * 0.20;
        score += this.indigenousOwned ? 0.15 : 0.0;
        score += this.certifiedBCorp ? 0.15 : 0.0;
        score += this.cooperative ? 0.15 : 0.0;
        score += (this.communityInvestmentUsd / this.revenueUsd.max(1)) * 0.10;
        return Math.min(1.0, score);
    }
}

class EconomicTransaction {
    constructor(txId, fromEntityId, toEntityId, amountUsd, transactionType, timestampNs) {
        this.txId = txId;
        this.fromEntityId = fromEntityId;
        this.toEntityId = toEntityId;
        this.amountUsd = amountUsd;
        this.transactionType = transactionType;
        this.timestampNs = timestampNs;
        this.localTransaction = false;
        this.importTransaction = false;
        this.exportTransaction = false;
        this.sectorFrom = '';
        this.sectorTo = '';
        this.valueAddedUsd = 0.0;
        this.taxGeneratedUsd = 0.0;
        this.communityBenefitUsd = 0.0;
        this.verified = false;
        this.finalized = false;
    }
    classifyTransaction(entities) {
        const fromEntity = entities.get(this.fromEntityId);
        const toEntity = entities.get(this.toEntityId);
        if (fromEntity && toEntity) {
            this.localTransaction = true;
            this.sectorFrom = fromEntity.sector;
            this.sectorTo = toEntity.sector;
            this.valueAddedUsd = this.amountUsd * 0.3;
            this.taxGeneratedUsd = this.amountUsd * 0.08;
            this.communityBenefitUsd = this.amountUsd * 0.05;
        } else {
            this.importTransaction = true;
        }
        this.verified = true;
        this.finalized = true;
    }
}

class SupplyChain {
    constructor(chainId, productName, originEntityId) {
        this.chainId = chainId;
        this.productName = productName;
        this.originEntityId = originEntityId;
        this.participants = [];
        this.totalValueUsd = 0.0;
        this.localValueAddedPct = 0.0;
        this.carbonFootprintKg = 0.0;
        this.waterFootprintL = 0.0;
        this.laborStandardsCompliant = true;
        this.indigenousSourced = false;
        this.createdAtNs = Date.now() * 1000000;
        this.lastUpdatedNs = Date.now() * 1000000;
    }
    addParticipant(entityId, valueAddedUsd, distanceKm) {
        this.participants.push({ entityId, valueAddedUsd, distanceKm });
        this.totalValueUsd += valueAddedUsd;
        this.lastUpdatedNs = Date.now() * 1000000;
    }
    computeLocalValueAdded() {
        if (this.totalValueUsd === 0) return 0.0;
        const localValue = this.participants.filter(p => p.distanceKm < 100)
            .reduce((sum, p) => sum + p.valueAddedUsd, 0);
        this.localValueAddedPct = (localValue / this.totalValueUsd) * 100.0;
        return this.localValueAddedPct;
    }
    computeSupplyChainEfficiency() {
        const participantCount = this.participants.length;
        const avgDistance = this.participants.reduce((sum, p) => sum + p.distanceKm, 0) / participantCount.max(1);
        const distanceScore = 1.0 - (avgDistance / 1000).Math.min(1.0);
        const localScore = this.localValueAddedPct / 100.0;
        return (distanceScore * 0.4 + localScore * 0.6).Math.min(1.0);
    }
}

class LocalEconomicFlowTracker {
    constructor(trackerId, cityCode, region) {
        this.trackerId = trackerId;
        this.cityCode = cityCode;
        this.region = region;
        this.businessEntities = new Map();
        this.entityCount = 0;
        this.transactions = [];
        this.txCount = 0;
        this.supplyChains = [];
        this.chainCount = 0;
        this.totalGdpUsd = 0.0;
        this.localGdpPct = 0.0;
        this.totalEmployment = 0;
        this.livingWageJobsPct = 0.0;
        this.localMultiplier = 0.0;
        this.importLeakageUsd = 0.0;
        this.exportRevenueUsd = 0.0;
        this.indigenousEconomyUsd = 0.0;
        this.cooperativeEconomyUsd = 0.0;
        this.lastAnalysisNs = Date.now() * 1000000;
    }
    registerBusinessEntity(entity) {
        if (this.entityCount >= MAX_BUSINESS_ENTITIES) return false;
        this.businessEntities.set(entity.entityId, entity);
        this.entityCount++;
        this.totalEmployment += entity.employeeCount;
        if (entity.indigenousOwned) {
            this.indigenousEconomyUsd += entity.revenueUsd;
        }
        if (entity.cooperative) {
            this.cooperativeEconomyUsd += entity.revenueUsd;
        }
        return true;
    }
    recordTransaction(transaction) {
        if (this.txCount >= MAX_ECONOMIC_TRANSACTIONS) return false;
        transaction.classifyTransaction(this.businessEntities);
        this.transactions.push(transaction);
        this.txCount++;
        if (transaction.localTransaction) {
            this.totalGdpUsd += transaction.valueAddedUsd;
        }
        if (transaction.importTransaction) {
            this.importLeakageUsd += transaction.amountUsd;
        }
        return true;
    }
    registerSupplyChain(chain) {
        if (this.chainCount >= MAX_SUPPLY_CHAINS) return false;
        this.supplyChains.push(chain);
        this.chainCount++;
        return true;
    }
    computeLocalMultiplier() {
        const localTx = this.transactions.filter(t => t.localTransaction).length;
        if (this.txCount === 0) return 0.0;
        this.localMultiplier = localTx / this.txCount;
        return this.localMultiplier;
    }
    computeLivingWageJobsPct() {
        let livingWageJobs = 0;
        for (const [, entity] of this.businessEntities) {
            if (entity.averageWageUsd >= PHOENIX_LIVING_WAGE_USD_HOUR) {
                livingWageJobs += entity.employeeCount;
            }
        }
        if (this.totalEmployment === 0) return 0.0;
        this.livingWageJobsPct = (livingWageJobs / this.totalEmployment) * 100.0;
        return this.livingWageJobsPct;
    }
    computeEconomicResilienceIndex() {
        const localMultiplierScore = this.computeLocalMultiplier();
        const livingWageScore = this.computeLivingWageJobsPct() / 100.0;
        const diversityScore = this.computeSectorDiversity();
        const indigenousScore = this.indigenousEconomyUsd / this.totalGdpUsd.max(1);
        const cooperativeScore = this.cooperativeEconomyUsd / this.totalGdpUsd.max(1);
        const importLeakagePenalty = this.importLeakageUsd / (this.totalGdpUsd + this.importLeakageUsd).max(1);
        return (localMultiplierScore * 0.25 + livingWageScore * 0.20 + diversityScore * 0.20 +
                indigenousScore * 0.15 + cooperativeScore * 0.10 + (1.0 - importLeakagePenalty) * 0.10).Math.min(1.0);
    }
    computeSectorDiversity() {
        const sectors = new Map();
        for (const [, entity] of this.businessEntities) {
            sectors.set(entity.sector, (sectors.get(entity.sector) || 0) + 1);
        }
        if (sectors.size === 0) return 0.0;
        const maxPossible = this.entityCount;
        return sectors.size / Math.min(maxPossible, 20);
    }
    identifyEconomicOpportunities() {
        const opportunities = [];
        const sectorGaps = this.analyzeSectorGaps();
        for (const gap of sectorGaps) {
            opportunities.push({
                sector: gap.sector,
                gapSeverity: gap.severity,
                importDependency: gap.importPct,
                localCapacity: gap.localCapacity,
                recommendedAction: 'Develop local production capacity',
                estimatedImpactUsd: gap.importDependency * 1000000,
                priority: gap.severity > 0.7 ? 'HIGH' : 'MEDIUM'
            });
        }
        return opportunities.sort((a, b) => b.estimatedImpactUsd - a.estimatedImpactUsd);
    }
    analyzeSectorGaps() {
        const sectors = {};
        const imports = {};
        for (const tx of this.transactions) {
            if (tx.importTransaction) {
                imports[tx.sectorTo] = (imports[tx.sectorTo] || 0) + tx.amountUsd;
            }
        }
        const gaps = [];
        for (const [sector, importValue] of Object.entries(imports)) {
            const localValue = this.transactions.filter(t => t.sectorTo === sector && t.localTransaction)
                .reduce((sum, t) => sum + t.amountUsd, 0);
            const total = importValue + localValue;
            if (total > 0) {
                gaps.push({
                    sector,
                    importPct: importValue / total,
                    localCapacity: localValue / total,
                    severity: importValue / total
                });
            }
        }
        return gaps;
    }
    generateEconomicReport(nowNs) {
        this.computeLocalMultiplier();
        this.computeLivingWageJobsPct();
        const resilienceIndex = this.computeEconomicResilienceIndex();
        const activeEntities = Array.from(this.businessEntities.values()).filter(e => e.operational).length;
        const avgSocialImpact = Array.from(this.businessEntities.values())
            .reduce((sum, e) => sum + e.computeSocialImpactScore(), 0) / this.entityCount.max(1);
        return {
            trackerId: this.trackerId,
            cityCode: this.cityCode,
            region: this.region,
            reportTimestampNs: nowNs,
            totalBusinessEntities: this.entityCount,
            activeEntities,
            totalTransactions: this.txCount,
            totalSupplyChains: this.chainCount,
            totalGdpUsd: this.totalGdpUsd,
            localGdpPct: this.localMultiplier * 100.0,
            totalEmployment: this.totalEmployment,
            livingWageJobsPct: this.livingWageJobsPct,
            localMultiplier: this.localMultiplier,
            importLeakageUsd: this.importLeakageUsd,
            exportRevenueUsd: this.exportRevenueUsd,
            indigenousEconomyUsd: this.indigenousEconomyUsd,
            cooperativeEconomyUsd: this.cooperativeEconomyUsd,
            economicResilienceIndex: resilienceIndex,
            averageSocialImpactScore: avgSocialImpact,
            lastAnalysisNs: this.lastAnalysisNs
        };
    }
}

module.exports = {
    BusinessEntity,
    EconomicTransaction,
    SupplyChain,
    LocalEconomicFlowTracker,
    VERSION: ECONOMIC_TRACKER_VERSION,
    PHOENIX_LIVING_WAGE_USD_HOUR,
    LOCAL_MULTIPLIER_TARGET,
};
