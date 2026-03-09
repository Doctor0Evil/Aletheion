package aletheion.waste.organic.composting

const val COMPOSTING_NETWORK_VERSION = 20260310L
const val MAX_COMPOSTING_SITES = 512
const val MAX_FEEDSTOCK_TYPES = 64
const val MAX_BATCH_RECORDS = 4096
const val TARGET_DIVERSION_RATE_PCT = 95.0
const val TARGET_MATURATION_DAYS = 60

enum class FeedstockType {
    FOOD_WASTE, YARD_WASTE, AGRICULTURAL_RESIDUE, MANURE,
    SLUDGE, PAPER, WOOD_CHIPS, LEAVES, GRASS, BRANCHES
}

enum class CompostingMethod {
    WINDROW, AERATED_STATIC_PILE, IN_VESSEL, VERMICOMPOSTING,
    BOKASHI, ANAEROBIC_DIGESTION, HOT_COMPOSTING
}

enum class CompostGrade {
    PREMIUM, GRADE_A, GRADE_B, GRADE_C, SOIL_AMENDMENT, MULCH
}

data class CompostingSite(
    val siteId: ULong,
    val siteName: String,
    val latitude: Double,
    val longitude: Double,
    val areaM2: Double,
    val method: CompostingMethod,
    val capacityTpd: Double,
    val currentThroughputTpd: Double,
    val operational: Boolean,
    val permitted: Boolean,
    val odorComplaints: UInt,
    val lastInspectionNs: Long,
    val communityAccessible: Boolean
) {
    fun utilizationRate(): Double = 
        if (capacityTpd > 0.0) currentThroughputTpd / capacityTpd else 0.0
    fun requiresInspection(nowNs: Long): Boolean = 
        nowNs - lastInspectionNs > 77760000000000L
}

data class FeedstockBatch(
    val batchId: ULong,
    val siteId: ULong,
    val feedstockType: FeedstockType,
    val massKg: Double,
    val carbonNitrogenRatio: Double,
    val moisturePct: Double,
    val contaminationPct: Double,
    val collectionDateNs: Long,
    val processingStartDateNs: Long,
    val expectedCompletionNs: Long,
    val actualCompletionNs: Long?,
    val temperatureProfile: List<Pair<Long, Double>>,
    val turned: Boolean,
    val aerated: Boolean
) {
    fun isMaturing(nowNs: Long): Boolean = 
        actualCompletionNs == null && nowNs < expectedCompletionNs
    fun isComplete(nowNs: Long): Boolean = 
        actualCompletionNs != null || nowNs >= expectedCompletionNs
    fun daysInProcess(nowNs: Long): Long = 
        (nowNs - processingStartDateNs) / 86400000000000L
    fun optimalCNRatio(): Boolean = 
        carbonNitrogenRatio in 25.0..35.0
    fun optimalMoisture(): Boolean = 
        moisturePct in 40.0..60.0
}

data class CompostProduct(
    val productId: ULong,
    val batchId: ULong,
    val siteId: ULong,
    val grade: CompostGrade,
    val massKg: Double,
    val volumeM3: Double,
    val nutrientProfile: Map<String, Double>,
    val contaminantLevels: Map<String, Double>,
    val ph: Double,
    val stabilityIndex: Double,
    val maturityIndex: Double,
    val productionDateNs: Long,
    val expirationDateNs: Long,
    val marketValueUsdTon: Double,
    val carbonSequestrationKg: Double,
    val waterRetentionBenefitL: Double,
    val certified: Boolean
) {
    fun meetsPremiumStandards(): Boolean =
        grade == CompostGrade.PREMIUM &&
        stabilityIndex > 0.9 &&
        maturityIndex > 0.9 &&
        contaminantLevels.all { it.value < 10.0 }
    fun soilHealthBenefit(): Double =
        (nutrientProfile["N"] ?: 0.0) * 0.3 +
        (nutrientProfile["P"] ?: 0.0) * 0.3 +
        (nutrientProfile["K"] ?: 0.0) * 0.2 +
        stabilityIndex * 0.2
}

class OrganicWasteCompostingNetwork(
    private val networkId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val sites = mutableMapOf<ULong, CompostingSite>()
    private val batches = mutableMapOf<ULong, FeedstockBatch>()
    private val products = mutableMapOf<ULong, CompostProduct>()
    private val auditLog = mutableListOf<CompostingAuditEntry>()
    private var nextBatchId: ULong = 1UL
    private var nextProductId: ULong = 1UL
    private var totalFeedstockProcessedKg: Double = 0.0
    private var totalCompostProducedKg: Double = 0.0
    private var totalContaminationRemovedKg: Double = 0.0
    private var totalCarbonSequesteredKg: Double = 0.0
    private var totalRevenueUsd: Double = 0.0
    private var totalOperatingCostUsd: Double = 0.0
    private var odorComplaintsTotal: UInt = 0U
    private var lastNetworkOptimizationNs: Long = initTimestampNs
    
    data class CompostingAuditEntry(
        val entryId: ULong,
        val action: String,
        val siteId: ULong?,
        val batchId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun registerSite(site: CompostingSite): Result<ULong> {
        if (sites.size >= MAX_COMPOSTING_SITES) {
            logAudit("SITE_REGISTER", null, null, initTimestampNs, false, "Site limit exceeded", 0.3)
            return Result.failure(Error("SITE_LIMIT_EXCEEDED"))
        }
        sites[site.siteId] = site
        logAudit("SITE_REGISTER", site.siteId, null, initTimestampNs, true, "Site registered", 0.02)
        return Result.success(site.siteId)
    }
    
    fun createFeedstockBatch(batch: FeedstockBatch, nowNs: Long): Result<ULong> {
        if (batches.size >= MAX_BATCH_RECORDS) {
            logAudit("BATCH_CREATE", batch.siteId, null, nowNs, false, "Batch limit exceeded", 0.2)
            return Result.failure(Error("BATCH_LIMIT_EXCEEDED"))
        }
        if (!batch.optimalCNRatio()) {
            logAudit("BATCH_CREATE", batch.siteId, batch.batchId, nowNs, false, 
                    "Suboptimal C:N ratio: ${batch.carbonNitrogenRatio}", 0.15)
        }
        if (!batch.optimalMoisture()) {
            logAudit("BATCH_CREATE", batch.siteId, batch.batchId, nowNs, false,
                    "Suboptimal moisture: ${batch.moisturePct}%", 0.1)
        }
        batches[nextBatchId] = batch
        totalFeedstockProcessedKg += batch.massKg
        val batchId = nextBatchId
        nextBatchId++
        logAudit("BATCH_CREATE", batch.siteId, batchId, nowNs, true, 
                "Batch created: ${batch.massKg}kg", 0.02)
        return Result.success(batchId)
    }
    
    fun completeBatch(batchId: ULong, product: CompostProduct, nowNs: Long): Result<ULong> {
        val batch = batches[batchId] ?: return Result.failure(Error("BATCH_NOT_FOUND"))
        val completedBatch = batch.copy(actualCompletionNs = nowNs)
        batches[batchId] = completedBatch
        products[nextProductId] = product
        totalCompostProducedKg += product.massKg
        totalCarbonSequesteredKg += product.carbonSequestrationKg
        totalRevenueUsd += product.massKg / 1000.0 * product.marketValueUsdTon
        val productId = nextProductId
        nextProductId++
        logAudit("BATCH_COMPLETE", batch.siteId, batchId, nowNs, true,
                "Compost produced: ${product.massKg}kg, Grade: ${product.grade}", 0.01)
        return Result.success(productId)
    }
    
    fun recordOdorComplaint(siteId: ULong, nowNs: Long) {
        val site = sites[siteId] ?: return
        val updatedSite = site.copy(
            odorComplaints = site.odorComplaints + 1U,
            lastInspectionNs = nowNs
        )
        sites[siteId] = updatedSite
        odorComplaintsTotal++
        logAudit("ODOR_COMPLAINT", siteId, null, nowNs, false,
                "Odor complaint recorded. Total: ${updatedSite.odorComplaints}", 0.25)
    }
    
    fun computeNetworkDiversionRate(): Double {
        if (totalFeedstockProcessedKg == 0.0) return 0.0
        return totalCompostProducedKg / totalFeedstockProcessedKg * 100.0
    }
    
    fun computeNetworkEfficiency(): Double {
        val diversionScore = (computeNetworkDiversionRate() / TARGET_DIVERSION_RATE_PCT).coerceAtMost(1.0)
        val qualityScore = products.values.count { it.meetsPremiumStandards() }.toDouble() / 
                          products.size.coerceAtLeast(1).toDouble()
        val complaintPenalty = (odorComplaintsTotal.toDouble() * 0.01).coerceAtMost(0.2)
        val carbonScore = (totalCarbonSequesteredKg / totalFeedstockProcessedKg.coerceAtLeast(1.0)).coerceAtMost(1.0)
        return (diversionScore * 0.35 + qualityScore * 0.30 + 
                (1.0 - complaintPenalty) * 0.15 + carbonScore * 0.20).coerceIn(0.0, 1.0)
    }
    
    fun getNetworkStatus(nowNs: Long): NetworkStatus {
        val operationalSites = sites.count { it.value.operational }
        val permittedSites = sites.count { it.value.permitted }
        val activeBatches = batches.count { it.value.isMaturing(nowNs) }
        val completedBatches = batches.count { it.value.isComplete(nowNs) }
        val premiumProducts = products.count { it.value.meetsPremiumStandards() }
        return NetworkStatus(
            networkId = networkId,
            cityCode = cityCode,
            totalSites = sites.size,
            operationalSites = operationalSites,
            permittedSites = permittedSites,
            totalBatches = batches.size,
            activeBatches = activeBatches,
            completedBatches = completedBatches,
            totalProducts = products.size,
            premiumProducts = premiumProducts,
            totalFeedstockProcessedKg = totalFeedstockProcessedKg,
            totalCompostProducedKg = totalCompostProducedKg,
            totalContaminationRemovedKg = totalContaminationRemovedKg,
            totalCarbonSequesteredKg = totalCarbonSequesteredKg,
            diversionRatePct = computeNetworkDiversionRate(),
            networkEfficiencyScore = computeNetworkEfficiency(),
            totalRevenueUsd = totalRevenueUsd,
            totalOperatingCostUsd = totalOperatingCostUsd,
            netProfitUsd = totalRevenueUsd - totalOperatingCostUsd,
            totalOdorComplaints = odorComplaintsTotal,
            lastOptimizationNs = lastNetworkOptimizationNs,
            lastUpdateNs = nowNs
        )
    }
    
    fun findOptimalCompostingSite(feedstockType: FeedstockType, massKg: Double): ULong? {
        return sites.filter { it.value.operational && it.value.permitted }
            .minByOrNull { site ->
                val capacityScore = 1.0 - site.value.utilizationRate()
                val distanceScore = 0.5
                val odorScore = 1.0 - (site.value.odorComplaints.toDouble() / 100.0).coerceAtMost(1.0)
                val methodCompatibility = when (feedstockType) {
                    FeedstockType.FOOD_WASTE, FeedstockType.MANURE -> 
                        if (site.value.method == CompostingMethod.IN_VESSEL) 0.0 else 0.3
                    FeedstockType.YARD_WASTE, FeedstockType.LEAVES, FeedstockType.GRASS ->
                        if (site.value.method == CompostingMethod.WINDROW) 0.0 else 0.2
                    else -> 0.1
                }
                -(capacityScore * 0.4 + distanceScore * 0.2 + odorScore * 0.25 + methodCompatibility * 0.15)
            }?.key
    }
    
    fun scheduleBatchOptimization(batchId: ULong, nowNs: Long): Result<Unit> {
        val batch = batches[batchId] ?: return Result.failure(Error("BATCH_NOT_FOUND"))
        val recommendations = mutableListOf<String>()
        if (!batch.optimalCNRatio()) {
            recommendations.add("Adjust C:N ratio to 25-35:1")
        }
        if (!batch.optimalMoisture()) {
            recommendations.add("Adjust moisture to 40-60%")
        }
        if (batch.daysInProcess(nowNs) > TARGET_MATURATION_DAYS.toLong() && !batch.isComplete(nowNs)) {
            recommendations.add("Consider turning or aeration to accelerate maturation")
        }
        if (recommendations.isNotEmpty()) {
            logAudit("BATCH_OPTIMIZATION", batch.siteId, batchId, nowNs, true,
                    "Optimization recommendations: ${recommendations.joinToString("; ")}", 0.05)
        }
        return Result.success(Unit)
    }
    
    private fun logAudit(action: String, siteId: ULong?, batchId: ULong?, 
                        timestampNs: Long, success: Boolean, details: String, riskScore: Double) {
        val entry = CompostingAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            siteId = siteId,
            batchId = batchId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<CompostingAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
    
    fun computeCarbonBenefit(): Double {
        val sequestrationBenefit = totalCarbonSequesteredKg
        val landfillAvoidanceBenefit = totalCompostProducedKg * 0.5
        val soilHealthBenefit = products.values.sumOf { it.soilHealthBenefit() }
        return sequestrationBenefit + landfillAvoidanceBenefit + soilHealthBenefit * 10.0
    }
}

data class NetworkStatus(
    val networkId: ULong,
    val cityCode: String,
    val totalSites: Int,
    val operationalSites: Int,
    val permittedSites: Int,
    val totalBatches: Int,
    val activeBatches: Int,
    val completedBatches: Int,
    val totalProducts: Int,
    val premiumProducts: Int,
    val totalFeedstockProcessedKg: Double,
    val totalCompostProducedKg: Double,
    val totalContaminationRemovedKg: Double,
    val totalCarbonSequesteredKg: Double,
    val diversionRatePct: Double,
    val networkEfficiencyScore: Double,
    val totalRevenueUsd: Double,
    val totalOperatingCostUsd: Double,
    val netProfitUsd: Double,
    val totalOdorComplaints: UInt,
    val lastOptimizationNs: Long,
    val lastUpdateNs: Long
) {
    fun sustainabilityIndex(): Double {
        val diversionComponent = (diversionRatePct / 100.0).coerceAtMost(1.0)
        val carbonComponent = (totalCarbonSequesteredKg / totalFeedstockProcessedKg.coerceAtLeast(1.0)).coerceAtMost(1.0)
        val qualityComponent = premiumProducts.toDouble() / totalProducts.coerceAtLeast(1).toDouble()
        val communityComponent = 1.0 - (totalOdorComplaints.toDouble() / 100.0).coerceAtMost(1.0)
        return (diversionComponent * 0.35 + carbonComponent * 0.30 + 
                qualityComponent * 0.20 + communityComponent * 0.15).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixCompostingNetwork(networkId: ULong, nowNs: Long): OrganicWasteCompostingNetwork {
    return OrganicWasteCompostingNetwork(networkId, "PHOENIX_AZ", nowNs)
}
