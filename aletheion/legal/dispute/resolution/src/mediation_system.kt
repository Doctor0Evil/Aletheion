package aletheion.legal.dispute.resolution

const val MEDIATION_SYSTEM_VERSION = 20260310L
const val MAX_DISPUTE_CASES = 65536
const val MAX_MEDIATORS = 2048
const val MAX_EVIDENCE_ITEMS = 262144
const val MAX_RESOLUTION_AGREEMENTS = 131072

enum class DisputeType {
    CONTRACT, EMPLOYMENT, PROPERTY, CONSUMER, CIVIL, CRIMINAL,
    TRIBAL, ENVIRONMENTAL, LABOR, FAMILY, ADMINISTRATIVE, CONSTITUTIONAL
}

enum class DisputeSeverity {
    LOW(1), MODERATE(2), ELEVATED(3), HIGH(4), CRITICAL(5)
    val level: Int
    constructor(level: Int) { this.level = level }
}

enum class ResolutionMethod {
    NEGOTIATION, MEDIATION, ARBITRATION, TRIBAL_COURT, CIVIL_COURT,
    COMMUNITY_JURY, RESTORATIVE_JUSTICE, PEACE_CIRCLE, ELDER_COUNCIL
}

enum class CaseStatus {
    FILED, UNDER_REVIEW, IN_MEDIATION, IN_ARBITRATION, IN_COURT,
    RESOLVED, DISMISSED, APPEALED, ENFORCED, CLOSED
}

data class DisputeCase(
    val caseId: ULong,
    val disputeType: DisputeType,
    val severity: DisputeSeverity,
    val plaintiffDid: String,
    val defendantDid: String,
    val filedAtNs: Long,
    val description: String,
    val damagesClaimedUsd: Double,
    val resolutionMethod: ResolutionMethod,
    val assignedMediatorId: ULong?,
    val tribalJurisdiction: Boolean,
    val indigenousPartyInvolved: Boolean,
    val status: CaseStatus,
    val resolvedAtNs: Long?,
    val resolutionTerms: String?,
    val complianceVerified: Boolean,
    val appealDeadlineNs: Long?
) {
    fun isActive(nowNs: Long): Boolean = 
        status !in listOf(CaseStatus.RESOLVED, CaseStatus.DISMISSED, CaseStatus.CLOSED)
    fun daysPending(nowNs: Long): Long = 
        (nowNs - filedAtNs) / 86400000000000L
    fun requiresTribalConsent(): Boolean = 
        tribalJurisdiction || indigenousPartyInvolved
}

data class Mediator(
    val mediatorId: ULong,
    val name: String,
    val specialization: List<DisputeType>,
    val certificationLevel: UInt,
    val tribalElder: Boolean,
    val languages: List<String>,
    val casesMediated: UInt,
    val successRatePct: Double,
    val averageResolutionDays: Double,
    val availabilityStatus: String,
    val lastCaseNs: Long,
    val rating: Double,
    val active: Boolean
) {
    fun isQualifiedFor(disputeType: DisputeType): Boolean = 
        active && specialization.contains(disputeType)
    fun canAcceptCase(): Boolean = 
        availabilityStatus == "AVAILABLE" && casesMediated < 100U
}

data class EvidenceItem(
    val evidenceId: ULong,
    val caseId: ULong,
    val evidenceType: String,
    val submittedBy: String,
    val submittedAtNs: Long,
    val verified: Boolean,
    val admissible: Boolean,
    val hash: String,
    val storageLocation: String,
    val accessLevel: UInt,
    val retentionUntilNs: Long
) {
    fun isAccessible(userClearance: UInt): Boolean = accessLevel <= userClearance
}

data class ResolutionAgreement(
    val agreementId: ULong,
    val caseId: ULong,
    val terms: String,
    val agreedAtNs: Long,
    val effectiveDateNs: Long,
    val expirationDateNs: Long?,
    val complianceDeadlineNs: Long?,
    val plaintiffSigned: Boolean,
    val defendantSigned: Boolean,
    val mediatorSigned: Boolean,
    val tribalConsentObtained: Boolean,
    val courtApproved: Boolean,
    val complianceVerified: Boolean,
    val violationsDetected: UInt,
    val enforcementActions: List<String>
) {
    fun isFullyExecuted(): Boolean = 
        plaintiffSigned && defendantSigned && mediatorSigned
    fun isCompliant(nowNs: Long): Boolean = 
        complianceVerified && (complianceDeadlineNs == null || nowNs < complianceDeadlineNs)
}

class DisputeResolutionMediationSystem(
    private val systemId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val cases = mutableMapOf<ULong, DisputeCase>()
    private val mediators = mutableMapOf<ULong, Mediator>()
    private val evidence = mutableMapOf<ULong, EvidenceItem>()
    private val agreements = mutableMapOf<ULong, ResolutionAgreement>()
    private val auditLog = mutableListOf<MediationAuditEntry>()
    private var nextCaseId: ULong = 1UL
    private var nextMediatorId: ULong = 1UL
    private var nextEvidenceId: ULong = 1UL
    private var nextAgreementId: ULong = 1UL
    private var totalCasesFiled: ULong = 0UL
    private var totalCasesResolved: ULong = 0UL
    private var averageResolutionDays: Double = 0.0
    private var complianceRatePct: Double = 100.0
    private var tribalCasesCount: ULong = 0UL
    private var indigenousConsentObtained: ULong = 0UL
    
    data class MediationAuditEntry(
        val entryId: ULong,
        val action: String,
        val caseId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun fileDisputeCase(dispute: DisputeCase, nowNs: Long): Result<ULong> {
        if (cases.size >= MAX_DISPUTE_CASES) {
            logAudit("CASE_FILE", null, nowNs, false, "Case limit exceeded", 0.3)
            return Result.failure(Error("CASE_LIMIT_EXCEEDED"))
        }
        if (dispute.requiresTribalConsent()) {
            tribalCasesCount++
        }
        cases[nextCaseId] = dispute
        totalCasesFiled++
        logAudit("CASE_FILE", nextCaseId, nowNs, true, "Case filed: ${dispute.disputeType}", 0.1)
        val caseId = nextCaseId
        nextCaseId++
        return Result.success(caseId)
    }
    
    fun registerMediator(mediator: Mediator): Result<ULong> {
        if (mediators.size >= MAX_MEDIATORS) {
            return Result.failure(Error("MEDIATOR_LIMIT_EXCEEDED"))
        }
        mediators[nextMediatorId] = mediator
        val mediatorId = nextMediatorId
        nextMediatorId++
        logAudit("MEDIATOR_REGISTER", null, initTimestampNs, true, "Mediator registered: ${mediator.name}", 0.02)
        return Result.success(mediatorId)
    }
    
    fun assignMediator(caseId: ULong, mediatorId: ULong, nowNs: Long): Result<Unit> {
        val dispute = cases[caseId] ?: return Result.failure(Error("CASE_NOT_FOUND"))
        val mediator = mediators[mediatorId] ?: return Result.failure(Error("MEDIATOR_NOT_FOUND"))
        if (!mediator.isQualifiedFor(dispute.disputeType)) {
            return Result.failure(Error("MEDIATOR_NOT_QUALIFIED"))
        }
        if (!mediator.canAcceptCase()) {
            return Result.failure(Error("MEDIATOR_UNAVAILABLE"))
        }
        val updatedCase = dispute.copy(
            assignedMediatorId = mediatorId,
            status = CaseStatus.IN_MEDIATION
        )
        cases[caseId] = updatedCase
        logAudit("MEDIATOR_ASSIGN", caseId, nowNs, true, "Mediator $mediatorId assigned", 0.05)
        return Result.success(Unit)
    }
    
    fun submitEvidence(evidence: EvidenceItem): Result<ULong> {
        if (evidence.size >= MAX_EVIDENCE_ITEMS) {
            return Result.failure(Error("EVIDENCE_LIMIT_EXCEEDED"))
        }
        evidence[nextEvidenceId] = evidence
        val evidenceId = nextEvidenceId
        nextEvidenceId++
        return Result.success(evidenceId)
    }
    
    fun createResolutionAgreement(agreement: ResolutionAgreement, nowNs: Long): Result<ULong> {
        if (agreements.size >= MAX_RESOLUTION_AGREEMENTS) {
            return Result.failure(Error("AGREEMENT_LIMIT_EXCEEDED"))
        }
        val dispute = cases[agreement.caseId] ?: return Result.failure(Error("CASE_NOT_FOUND"))
        if (dispute.requiresTribalConsent() && !agreement.tribalConsentObtained) {
            return Result.failure(Error("TRIBAL_CONSENT_REQUIRED"))
        }
        agreements[nextAgreementId] = agreement
        val updatedCase = dispute.copy(
            status = CaseStatus.RESOLVED,
            resolvedAtNs = nowNs,
            resolutionTerms = agreement.terms
        )
        cases[agreement.caseId] = updatedCase
        totalCasesResolved++
        indigenousConsentObtained++
        logAudit("AGREEMENT_CREATED", agreement.caseId, nowNs, true, "Resolution agreement created", 0.05)
        val agreementId = nextAgreementId
        nextAgreementId++
        return Result.success(agreementId)
    }
    
    fun computeAverageResolutionTime(): Double {
        val resolvedCases = cases.values.filter { it.resolvedAtNs != null }
        if (resolvedCases.isEmpty()) return 0.0
        val totalDays = resolvedCases.sumOf { it.daysPending(it.resolvedAtNs!!) }
        averageResolutionDays = totalDays.toDouble() / resolvedCases.size
        return averageResolutionDays
    }
    
    fun computeComplianceRate(nowNs: Long): Double {
        val activeAgreements = agreements.values.filter { it.expirationDateNs == null || nowNs < it.expirationDateNs }
        if (activeAgreements.isEmpty()) return 100.0
        val compliantCount = activeAgreements.count { it.isCompliant(nowNs) }
        complianceRatePct = (compliantCount.toDouble() / activeAgreements.size) * 100.0
        return complianceRatePct
    }
    
    fun findBestMediator(disputeType: DisputeType, tribalJurisdiction: Boolean, languages: List<String>): ULong? {
        return mediators.values.filter { 
            it.isQualifiedFor(disputeType) && 
            it.canAcceptCase() &&
            (!tribalJurisdiction || it.tribalElder) &&
            languages.all { lang -> it.languages.contains(lang) }
        }.maxByOrNull { it.successRatePct * it.rating }?.mediatorId
    }
    
    fun getSystemStatus(nowNs: Long): SystemStatus {
        val activeCases = cases.count { it.value.isActive(nowNs) }
        val availableMediators = mediators.count { it.value.canAcceptCase() }
        val activeAgreements = agreements.count { it.value.isFullyExecuted() && !it.value.complianceVerified }
        return SystemStatus(
            systemId = systemId,
            cityCode = cityCode,
            totalCasesFiled = totalCasesFiled,
            activeCases = activeCases,
            totalCasesResolved = totalCasesResolved,
            totalMediators = mediators.size,
            availableMediators = availableMediators,
            totalEvidence = evidence.size,
            totalAgreements = agreements.size,
            activeAgreements = activeAgreements,
            averageResolutionDays = averageResolutionDays,
            complianceRatePct = complianceRatePct,
            tribalCasesCount = tribalCasesCount,
            indigenousConsentObtained = indigenousConsentObtained,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeJusticeAccessIndex(): Double {
        val resolutionSpeed = if (averageResolutionDays < 30) 1.0 else (90.0 / averageResolutionDays).coerceAtMost(1.0)
        val complianceScore = complianceRatePct / 100.0
        val tribalAccess = indigenousConsentObtained.toDouble() / tribalCasesCount.max(1UL).toDouble()
        val mediatorAvailability = mediators.count { it.value.canAcceptCase() }.toDouble() / mediators.size.max(1).toDouble()
        return (resolutionSpeed * 0.30 + complianceScore * 0.30 + tribalAccess * 0.25 + mediatorAvailability * 0.15).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, caseId: ULong?, timestampNs: Long, success: Boolean, details: String, riskScore: Double) {
        val entry = MediationAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            caseId = caseId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<MediationAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class SystemStatus(
    val systemId: ULong,
    val cityCode: String,
    val totalCasesFiled: ULong,
    val activeCases: Int,
    val totalCasesResolved: ULong,
    val totalMediators: Int,
    val availableMediators: Int,
    val totalEvidence: Int,
    val totalAgreements: Int,
    val activeAgreements: Int,
    val averageResolutionDays: Double,
    val complianceRatePct: Double,
    val tribalCasesCount: ULong,
    val indigenousConsentObtained: ULong,
    val lastUpdateNs: Long
) {
    fun systemEfficiency(): Double {
        val resolutionEfficiency = totalCasesResolved.toDouble() / totalCasesFiled.max(1UL).toDouble()
        val mediatorUtilization = (totalMediators - availableMediators).toDouble() / totalMediators.max(1).toDouble()
        return (resolutionEfficiency * 0.6 + mediatorUtilization * 0.4).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixMediationSystem(systemId: ULong, nowNs: Long): DisputeResolutionMediationSystem {
    return DisputeResolutionMediationSystem(systemId, "PHOENIX_AZ", nowNs)
}
