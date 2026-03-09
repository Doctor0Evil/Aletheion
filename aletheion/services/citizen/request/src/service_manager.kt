package aletheion.services.citizen.request

const val SERVICE_MANAGER_VERSION = 20260310L
const val MAX_SERVICE_REQUESTS = 524288
const val MAX_SERVICE_TYPES = 512
const val MAX_SERVICE_PROVIDERS = 8192
const val TARGET_RESOLUTION_TIME_H = 24.0
const val CITIZEN_SATISFACTION_TARGET = 0.85

enum class ServiceType {
    UTILITIES, TRANSPORTATION, HOUSING, HEALTHCARE, SAFETY,
    WASTE_MANAGEMENT, PARKS_RECREATION, ADMINISTRATION, EDUCATION,
    EMERGENCY, ENVIRONMENTAL, SOCIAL_SERVICES, LEGAL, EMPLOYMENT
}

enum class RequestStatus {
    SUBMITTED, ACKNOWLEDGED, IN_PROGRESS, PENDING_INFO,
    RESOLVED, CLOSED, ESCALATED, CANCELLED, REJECTED
}

enum class Priority {
    LOW(1), NORMAL(2), HIGH(3), URGENT(4), CRITICAL(5)
    val level: Int
    constructor(level: Int) { this.level = level }
}

data class ServiceRequest(
    val requestId: ULong,
    val citizenDid: String,
    val serviceType: ServiceType,
    val priority: Priority,
    val title: String,
    val description: String,
    val locationLat: Double,
    val locationLon: Double,
    val submittedAtNs: Long,
    val acknowledgedAtNs: Long?,
    val resolvedAtNs: Long?,
    val closedAtNs: Long?,
    val status: RequestStatus,
    val assignedProviderId: ULong?,
    val estimatedResolutionNs: Long?,
    val actualResolutionNs: Long?,
    val satisfactionRating: Double?,
    val accessibilityNeeds: Boolean,
    val languagePreference: String,
    val indigenousCommunity: Boolean
) {
    fun isActive(nowNs: Long): Boolean = 
        status !in listOf(RequestStatus.RESOLVED, RequestStatus.CLOSED, RequestStatus.CANCELLED)
    fun timeToResolve(nowNs: Long): Long? = 
        resolvedAtNs?.let { it - submittedAtNs }
    fun isOverdue(nowNs: Long): Boolean = 
        estimatedResolutionNs?.let { nowNs > it } ?: false
    fun requiresEscalation(nowNs: Long): Boolean = 
        isOverdue(nowNs) && status == RequestStatus.IN_PROGRESS
}

data class ServiceProvider(
    val providerId: ULong,
    val providerName: String,
    val serviceTypes: List<ServiceType>,
    val capacity: UInt,
    val currentLoad: UInt,
    val averageResolutionTimeH: Double,
    val satisfactionScore: Double,
    val accessibilityCompliant: Boolean,
    val multilingualSupport: List<String>,
    val indigenousCulturalCompetency: Boolean,
    val operational: Boolean,
    val lastUpdatedNs: Long
) {
    fun canAcceptRequest(): Boolean = 
        operational && currentLoad < capacity && satisfactionScore >= 0.7
    fun utilizationRate(): Double = 
        currentLoad.toDouble() / capacity.max(1U).toDouble()
}

data class ServiceCategory(
    val categoryId: ULong,
    val categoryName: String,
    val serviceType: ServiceType,
    val description: String,
    val averageResolutionTimeH: Double,
    val targetResolutionTimeH: Double,
    val escalationThresholdH: Double,
    val autoAssignEnabled: Boolean,
    val citizenFeedbackRequired: Boolean,
    val indigenousConsultationRequired: Boolean
)

class CitizenServiceRequestManager(
    private val managerId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val requests = mutableMapOf<ULong, ServiceRequest>()
    private val providers = mutableMapOf<ULong, ServiceProvider>()
    private val categories = mutableMapOf<ULong, ServiceCategory>()
    private val auditLog = mutableListOf<ServiceAuditEntry>()
    private var nextRequestId: ULong = 1UL
    private var totalRequestsSubmitted: ULong = 0UL
    private var totalRequestsResolved: ULong = 0UL
    private var averageResolutionTimeH: Double = 0.0
    private var citizenSatisfactionScore: Double = 0.0
    private var escalationCount: ULong = 0UL
    private var indigenousConsultations: ULong = 0UL
    
    data class ServiceAuditEntry(
        val entryId: ULong,
        val action: String,
        val requestId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun submitServiceRequest(request: ServiceRequest, nowNs: Long): Result<ULong> {
        if (requests.size >= MAX_SERVICE_REQUESTS) {
            logAudit("REQUEST_SUBMIT", null, nowNs, false, "Request limit exceeded", 0.3)
            return Result.failure(Error("REQUEST_LIMIT_EXCEEDED"))
        }
        requests[nextRequestId] = request
        totalRequestsSubmitted++
        if (request.indigenousCommunity) {
            indigenousConsultations++
        }
        logAudit("REQUEST_SUBMIT", nextRequestId, nowNs, true, "Request submitted: ${request.serviceType}", 0.05)
        val requestId = nextRequestId
        nextRequestId++
        return Result.success(requestId)
    }
    
    fun registerServiceProvider(provider: ServiceProvider): Result<ULong> {
        if (providers.size >= MAX_SERVICE_PROVIDERS) {
            return Result.failure(Error("PROVIDER_LIMIT_EXCEEDED"))
        }
        if (!provider.accessibilityCompliant) {
            return Result.failure(Error("ACCESSIBILITY_COMPLIANCE_REQUIRED"))
        }
        providers[provider.providerId] = provider
        logAudit("PROVIDER_REGISTER", null, initTimestampNs, true, "Provider registered: ${provider.providerName}", 0.02)
        return Result.success(provider.providerId)
    }
    
    fun assignProvider(requestId: ULong, providerId: ULong, nowNs: Long): Result<Unit> {
        val request = requests[requestId] ?: return Result.failure(Error("REQUEST_NOT_FOUND"))
        val provider = providers[providerId] ?: return Result.failure(Error("PROVIDER_NOT_FOUND"))
        if (!provider.canAcceptRequest()) {
            return Result.failure(Error("PROVIDER_UNAVAILABLE"))
        }
        val updatedRequest = request.copy(
            status = RequestStatus.IN_PROGRESS,
            assignedProviderId = providerId,
            acknowledgedAtNs = nowNs
        )
        requests[requestId] = updatedRequest
        val updatedProvider = provider.copy(currentLoad = provider.currentLoad + 1U, lastUpdatedNs = nowNs)
        providers[providerId] = updatedProvider
        logAudit("PROVIDER_ASSIGN", requestId, nowNs, true, "Provider $providerId assigned", 0.05)
        return Result.success(Unit)
    }
    
    fun resolveRequest(requestId: ULong, satisfactionRating: Double?, nowNs: Long): Result<Unit> {
        val request = requests[requestId] ?: return Result.failure(Error("REQUEST_NOT_FOUND"))
        val updatedRequest = request.copy(
            status = RequestStatus.RESOLVED,
            resolvedAtNs = nowNs,
            satisfactionRating = satisfactionRating
        )
        requests[requestId] = updatedRequest
        totalRequestsResolved++
        if (satisfactionRating != null) {
            updateSatisfactionScore(satisfactionRating)
        }
        val resolutionTime = (nowNs - request.submittedAtNs) / 3600000000000
        updateAverageResolutionTime(resolutionTime.toDouble())
        logAudit("REQUEST_RESOLVE", requestId, nowNs, true, "Request resolved", 0.02)
        return Result.success(Unit)
    }
    
    fun closeRequest(requestId: ULong, nowNs: Long): Result<Unit> {
        val request = requests[requestId] ?: return Result.failure(Error("REQUEST_NOT_FOUND"))
        val updatedRequest = request.copy(
            status = RequestStatus.CLOSED,
            closedAtNs = nowNs
        )
        requests[requestId] = updatedRequest
        val providerId = request.assignedProviderId
        if (providerId != null) {
            val provider = providers[providerId]
            if (provider != null) {
                providers[providerId] = provider.copy(
                    currentLoad = provider.currentLoad - 1U,
                    lastUpdatedNs = nowNs
                )
            }
        }
        logAudit("REQUEST_CLOSE", requestId, nowNs, true, "Request closed", 0.02)
        return Result.success(Unit)
    }
    
    fun escalateRequest(requestId: ULong, reason: String, nowNs: Long): Result<Unit> {
        val request = requests[requestId] ?: return Result.failure(Error("REQUEST_NOT_FOUND"))
        val updatedRequest = request.copy(status = RequestStatus.ESCALATED)
        requests[requestId] = updatedRequest
        escalationCount++
        logAudit("REQUEST_ESCALATE", requestId, nowNs, true, "Escalated: $reason", 0.15)
        return Result.success(Unit)
    }
    
    private fun updateSatisfactionScore(newRating: Double) {
        val totalRatings = requests.count { it.value.satisfactionRating != null }
        val sumRatings = requests.values.filter { it.satisfactionRating != null }
            .sumOf { it.satisfactionRating!! }
        citizenSatisfactionScore = sumRatings / totalRatings.max(1).toDouble()
    }
    
    private fun updateAverageResolutionTime(newTimeH: Double) {
        averageResolutionTimeH = (averageResolutionTimeH * (totalRequestsResolved - 1).toDouble() + newTimeH) / 
            totalRequestsResolved.toDouble().max(1.0)
    }
    
    fun findBestProvider(serviceType: ServiceType, language: String, accessibilityNeeds: Boolean): ULong? {
        return providers.values.filter { 
            it.canAcceptRequest() &&
            it.serviceTypes.contains(serviceType) &&
            (!accessibilityNeeds || it.accessibilityCompliant) &&
            (language in it.multilingualSupport || it.multilingualSupport.isEmpty())
        }.maxByOrNull { it.satisfactionScore * (1.0 - it.utilizationRate()) }?.providerId
    }
    
    fun getManagerStatus(nowNs: Long): ManagerStatus {
        val activeRequests = requests.count { it.value.isActive(nowNs) }
        val overdueRequests = requests.count { it.value.isOverdue(nowNs) }
        val operationalProviders = providers.count { it.value.operational }
        return ManagerStatus(
            managerId = managerId,
            cityCode = cityCode,
            totalRequests = requests.size,
            activeRequests = activeRequests,
            resolvedRequests = totalRequestsResolved,
            overdueRequests = overdueRequests,
            totalProviders = providers.size,
            operationalProviders = operationalProviders,
            averageResolutionTimeH = averageResolutionTimeH,
            citizenSatisfactionScore = citizenSatisfactionScore,
            escalationCount = escalationCount,
            indigenousConsultations = indigenousConsultations,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeServiceQualityIndex(): Double {
        val resolutionScore = if (averageResolutionTimeH <= TARGET_RESOLUTION_TIME_H) 1.0 
            else (TARGET_RESOLUTION_TIME_H / averageResolutionTimeH).coerceAtMost(1.0)
        val satisfactionScore = citizenSatisfactionScore
        val availabilityScore = providers.count { it.value.canAcceptRequest() }.toDouble() / 
            providers.size.coerceAtLeast(1).toDouble()
        val escalationPenalty = (escalationCount.toDouble() / totalRequestsSubmitted.max(1UL).toDouble()) * 0.2
        return (resolutionScore * 0.35 + satisfactionScore * 0.35 + availabilityScore * 0.30 - escalationPenalty).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, requestId: ULong?, timestampNs: Long, 
                        success: Boolean, details: String, riskScore: Double) {
        val entry = ServiceAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            requestId = requestId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<ServiceAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class ManagerStatus(
    val managerId: ULong,
    val cityCode: String,
    val totalRequests: Int,
    val activeRequests: Int,
    val resolvedRequests: ULong,
    val overdueRequests: Int,
    val totalProviders: Int,
    val operationalProviders: Int,
    val averageResolutionTimeH: Double,
    val citizenSatisfactionScore: Double,
    val escalationCount: ULong,
    val indigenousConsultations: ULong,
    val lastUpdateNs: Long
) {
    fun serviceEfficiency(): Double {
        val resolutionEfficiency = resolvedRequests.toDouble() / totalRequests.max(1).toDouble()
        val providerUtilization = (totalProviders - operationalProviders).toDouble() / totalProviders.max(1).toDouble()
        val satisfactionFactor = citizenSatisfactionScore
        return (resolutionEfficiency * 0.4 + (1.0 - providerUtilization) * 0.3 + satisfactionFactor * 0.3).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixServiceManager(managerId: ULong, nowNs: Long): CitizenServiceRequestManager {
    return CitizenServiceRequestManager(managerId, "PHOENIX_AZ", nowNs)
}
