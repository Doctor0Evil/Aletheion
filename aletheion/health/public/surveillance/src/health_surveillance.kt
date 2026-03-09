package aletheion.health.public.surveillance

const val HEALTH_SURVEILLANCE_VERSION = 20260310L
const val MAX_HEALTH_INDICATORS = 512
const val MAX_DISEASE_CASES = 131072
const val MAX_HEALTH_FACILITIES = 2048
const val MAX_VACCINATION_RECORDS = 524288
const val OUTBREAK_THRESHOLD_CASES = 10
const val REPORTING_COMPLIANCE_TARGET = 0.95

enum class DiseaseCategory {
    INFECTIOUS, CHRONIC, ENVIRONMENTAL, OCCUPATIONAL,
    VECTOR_BORNE, WATER_BORNE, FOOD_BORNE, AIR_BORNE,
    VACCINE_PREVENTABLE, EMERGING, REEMERGING, ZOOTONIC
}

enum class SurveillanceLevel {
    ROUTINE(1), ENHANCED(2), EMERGENCY(3), OUTBREAK(4)
    val level: Int
    constructor(level: Int) { this.level = level }
}

enum class CaseStatus {
    SUSPECTED, PROBABLE, CONFIRMED, RECOVERED, DECEASED,
    UNDER_INVESTIGATION, CLOSED, EXCLUDED
}

data class HealthIndicator(
    val indicatorId: ULong,
    val indicatorName: String,
    val category: DiseaseCategory,
    val targetValue: Double,
    val currentValue: Double,
    val unit: String,
    val dataQualityScore: Double,
    val reportingFrequency: String,
    val lastReportedNs: Long,
    val trend: String,
    val alertThreshold: Double,
    val criticalThreshold: Double,
    val indigenousCommunityRelevant: Boolean,
    val accessibilityCompliant: Boolean
) {
    fun isWithinTarget(): Boolean = currentValue >= targetValue
    fun requiresAlert(): Boolean = currentValue <= alertThreshold
    fun isCritical(): Boolean = currentValue <= criticalThreshold
}

data class DiseaseCase(
    val caseId: ULong,
    val diseaseName: String,
    val category: DiseaseCategory,
    val status: CaseStatus,
    val reportedAtNs: Long,
    val confirmedAtNs: Long?,
    val resolvedAtNs: Long?,
    val patientAge: UInt,
    val patientGender: String,
    val locationLat: Double,
    val locationLon: Double,
    val zipCode: String,
    val hospitalizationRequired: Boolean,
    val outcome: String,
    val riskFactors: List<String>,
    val vaccinationStatus: String,
    val travelHistory: Boolean,
    val contactTracingComplete: Boolean,
    val privacyProtected: Boolean
) {
    fun isActive(nowNs: Long): Boolean = 
        status !in listOf(CaseStatus.RECOVERED, CaseStatus.DECEASED, CaseStatus.CLOSED, CaseStatus.EXCLUDED)
    fun daysToResolution(nowNs: Long): Long? = 
        resolvedAtNs?.let { (it - reportedAtNs) / 86400000000000L }
    fun requiresInvestigation(): Boolean = 
        status == CaseStatus.SUSPECTED || status == CaseStatus.PROBABLE
}

data class HealthFacility(
    val facilityId: ULong,
    val facilityName: String,
    val facilityType: String,
    val latitude: Double,
    val longitude: Double,
    val bedCapacity: UInt,
    val availableBeds: UInt,
    val icuCapacity: UInt,
    val availableIcuBeds: UInt,
    val emergencyDepartment: Boolean,
    val infectiousDiseaseUnit: Boolean,
    val isolationRooms: UInt,
    val availableIsolationRooms: UInt,
    val reportingCompliant: Boolean,
    val lastInspectionNs: Long,
    val operational: Boolean,
    val accessibilityCompliant: Boolean,
    val indigenousCulturalCompetency: Boolean,
    val multilingualStaff: Boolean
) {
    fun bedUtilization(): Double = 
        (bedCapacity - availableBeds).toDouble() / bedCapacity.max(1U).toDouble()
    fun icuUtilization(): Double = 
        (icuCapacity - availableIcuBeds).toDouble() / icuCapacity.max(1U).toDouble()
    fun canAcceptPatients(): Boolean = 
        operational && availableBeds > 0U && reportingCompliant
}

data class VaccinationRecord(
    val recordId: ULong,
    val citizenDid: String,
    val vaccineName: String,
    val diseasePrevented: String,
    val doseNumber: UInt,
    val totalDoses: UInt,
    val administeredAtNs: Long,
    val administeredAtFacilityId: ULong,
    val healthcareProviderId: ULong,
    val lotNumber: String,
    val expirationDateNs: Long,
    val adverseEventReported: Boolean,
    val effectivenessVerified: Boolean,
    val privacyProtected: Boolean,
    val indigenousCommunityProgram: Boolean
) {
    fun isComplete(): Boolean = doseNumber >= totalDoses
    fun isExpired(nowNs: Long): Boolean = nowNs > expirationDateNs
}

class PublicHealthSurveillanceSystem(
    private val systemId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val healthIndicators = mutableMapOf<ULong, HealthIndicator>()
    private val diseaseCases = mutableMapOf<ULong, DiseaseCase>()
    private val healthFacilities = mutableMapOf<ULong, HealthFacility>()
    private val vaccinationRecords = mutableMapOf<ULong, VaccinationRecord>()
    private val auditLog = mutableListOf<HealthAuditEntry>()
    private var nextCaseId: ULong = 1UL
    private var nextRecordId: ULong = 1UL
    private var totalCasesReported: ULong = 0UL
    private var totalCasesResolved: ULong = 0UL
    private var totalDeaths: ULong = 0UL
    private var outbreakCount: ULong = 0UL
    private var reportingComplianceRate: Double = 1.0
    private var vaccinationCoverageRate: Double = 0.0
    private var lastOutbreakDetectionNs: Long = initTimestampNs
    
    data class HealthAuditEntry(
        val entryId: ULong,
        val action: String,
        val caseId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun registerHealthIndicator(indicator: HealthIndicator): Result<ULong> {
        if (healthIndicators.size >= MAX_HEALTH_INDICATORS.toULong()) {
            logAudit("INDICATOR_REGISTER", null, initTimestampNs, false, "Indicator limit exceeded", 0.3)
            return Result.failure(Error("INDICATOR_LIMIT_EXCEEDED"))
        }
        if (!indicator.accessibilityCompliant) {
            return Result.failure(Error("ACCESSIBILITY_COMPLIANCE_REQUIRED"))
        }
        healthIndicators[indicator.indicatorId] = indicator
        logAudit("INDICATOR_REGISTER", indicator.indicatorId, initTimestampNs, true, "Indicator registered", 0.02)
        return Result.success(indicator.indicatorId)
    }
    
    fun reportDiseaseCase(case_: DiseaseCase, nowNs: Long): Result<ULong> {
        if (diseaseCases.size >= MAX_DISEASE_CASES.toULong()) {
            return Result.failure(Error("CASE_LIMIT_EXCEEDED"))
        }
        if (!case_.privacyProtected) {
            return Result.failure(Error("PRIVACY_PROTECTION_REQUIRED"))
        }
        diseaseCases[nextCaseId] = case_
        totalCasesReported++
        if (case_.status == CaseStatus.DECEASED) {
            totalDeaths++
        }
        checkForOutbreak(case_.diseaseName, case_.locationLat, case_.locationLon, nowNs)
        logAudit("CASE_REPORT", nextCaseId, nowNs, true, "Case reported: ${case_.diseaseName}", 0.1)
        val caseId = nextCaseId
        nextCaseId++
        return Result.success(caseId)
    }
    
    fun resolveCase(caseId: ULong, outcome: String, nowNs: Long): Result<Unit> {
        val case_ = diseaseCases[caseId] ?: return Result.failure(Error("CASE_NOT_FOUND"))
        val status = if (outcome == "DECEASED") CaseStatus.DECEASED else CaseStatus.RECOVERED
        val updatedCase = case_.copy(
            status = status,
            resolvedAtNs = nowNs,
            outcome = outcome
        )
        diseaseCases[caseId] = updatedCase
        if (status == CaseStatus.RECOVERED) {
            totalCasesResolved++
        }
        logAudit("CASE_RESOLVE", caseId, nowNs, true, "Case resolved: $outcome", 0.05)
        return Result.success(Unit)
    }
    
    fun registerHealthFacility(facility: HealthFacility): Result<ULong> {
        if (healthFacilities.size >= MAX_HEALTH_FACILITIES.toULong()) {
            return Result.failure(Error("FACILITY_LIMIT_EXCEEDED"))
        }
        if (!facility.accessibilityCompliant) {
            return Result.failure(Error("ACCESSIBILITY_COMPLIANCE_REQUIRED"))
        }
        healthFacilities[facility.facilityId] = facility
        logAudit("FACILITY_REGISTER", facility.facilityId, initTimestampNs, true, "Facility registered", 0.02)
        return Result.success(facility.facilityId)
    }
    
    fun recordVaccination(record: VaccinationRecord): Result<ULong> {
        if (vaccinationRecords.size >= MAX_VACCINATION_RECORDS.toULong()) {
            return Result.failure(Error("RECORD_LIMIT_EXCEEDED"))
        }
        if (!record.privacyProtected) {
            return Result.failure(Error("PRIVACY_PROTECTION_REQUIRED"))
        }
        vaccinationRecords[nextRecordId] = record
        val recordId = nextRecordId
        nextRecordId++
        updateVaccinationCoverage()
        return Result.success(recordId)
    }
    
    private fun checkForOutbreak(diseaseName: String, lat: Double, lon: Double, nowNs: Long) {
        val nearbyCases = diseaseCases.values.filter { 
            it.diseaseName == diseaseName && 
            it.isActive(nowNs) &&
            computeDistance(lat, lon, it.locationLat, it.locationLon) < 5.0
        }.size
        if (nearbyCases >= OUTBREAK_THRESHOLD_CASES) {
            outbreakCount++
            lastOutbreakDetectionNs = nowNs
            logAudit("OUTBREAK_DETECTED", null, nowNs, true, "Outbreak: $diseaseName, $nearbyCases cases", 0.5)
        }
    }
    
    private fun computeDistance(lat1: Double, lon1: Double, lat2: Double, lon2: Double): Double {
        val earthRadiusKm = 6371.0
        val dLat = Math.toRadians(lat2 - lat1)
        val dLon = Math.toRadians(lon2 - lon1)
        val a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
                Math.cos(Math.toRadians(lat1)) * Math.cos(Math.toRadians(lat2)) *
                Math.sin(dLon / 2) * Math.sin(dLon / 2)
        val c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
        return earthRadiusKm * c
    }
    
    private fun updateVaccinationCoverage() {
        val completeVaccinations = vaccinationRecords.count { it.value.isComplete() }
        val totalPopulation = 1700000.0
        vaccinationCoverageRate = completeVaccinations.toDouble() / totalPopulation
    }
    
    fun computeReportingCompliance(): Double {
        val compliantFacilities = healthFacilities.count { it.value.reportingCompliant }
        reportingComplianceRate = compliantFacilities.toDouble() / healthFacilities.size.max(1).toDouble()
        return reportingComplianceRate
    }
    
    fun getSystemStatus(nowNs: Long): SystemStatus {
        val activeCases = diseaseCases.count { it.value.isActive(nowNs) }
        val suspectedCases = diseaseCases.count { it.value.status == CaseStatus.SUSPECTED }
        val operationalFacilities = healthFacilities.count { it.value.operational }
        val facilitiesAtCapacity = healthFacilities.count { it.value.bedUtilization() > 0.9 }
        val completeVaccinations = vaccinationRecords.count { it.value.isComplete() }
        return SystemStatus(
            systemId = systemId,
            cityCode = cityCode,
            totalHealthIndicators = healthIndicators.size,
            indicatorsWithinTarget = healthIndicators.count { it.value.isWithinTarget() },
            totalDiseaseCases = diseaseCases.size,
            activeCases,
            suspectedCases,
            totalCasesReported,
            totalCasesResolved,
            totalDeaths,
            outbreakCount,
            totalHealthFacilities = healthFacilities.size,
            operationalFacilities,
            facilitiesAtCapacity,
            totalVaccinationRecords = vaccinationRecords.size,
            completeVaccinations,
            reportingComplianceRate = computeReportingCompliance(),
            vaccinationCoverageRate = vaccinationCoverageRate,
            lastOutbreakDetectionNs = lastOutbreakDetectionNs,
            lastUpdateNs = nowNs
        )
    }
    
    fun computePublicHealthIndex(): Double {
        val caseResolutionRate = totalCasesResolved.toDouble() / totalCasesReported.max(1UL).toDouble()
        val facilityAvailability = healthFacilities.count { it.value.canAcceptPatients() }.toDouble() / 
            healthFacilities.size.max(1).toDouble()
        val vaccinationScore = vaccinationCoverageRate
        val complianceScore = reportingComplianceRate
        val outbreakPenalty = if (outbreakCount > 0) 0.1 else 0.0
        return (caseResolutionRate * 0.30 + facilityAvailability * 0.25 + 
                vaccinationScore * 0.25 + complianceScore * 0.20 - outbreakPenalty).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, caseId: ULong?, timestampNs: Long, 
                        success: Boolean, details: String, riskScore: Double) {
        val entry = HealthAuditEntry(
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
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<HealthAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class SystemStatus(
    val systemId: ULong,
    val cityCode: String,
    val totalHealthIndicators: Int,
    val indicatorsWithinTarget: Int,
    val totalDiseaseCases: Int,
    val activeCases: Int,
    val suspectedCases: Int,
    val totalCasesReported: ULong,
    val totalCasesResolved: ULong,
    val totalDeaths: ULong,
    val outbreakCount: ULong,
    val totalHealthFacilities: Int,
    val operationalFacilities: Int,
    val facilitiesAtCapacity: Int,
    val totalVaccinationRecords: Int,
    val completeVaccinations: Int,
    val reportingComplianceRate: Double,
    val vaccinationCoverageRate: Double,
    val lastOutbreakDetectionNs: Long,
    val lastUpdateNs: Long
) {
    fun healthcareSystemCapacity(): Double {
        val bedAvailability = 1.0 - (facilitiesAtCapacity.toDouble() / totalHealthFacilities.max(1).toDouble())
        val resolutionEfficiency = totalCasesResolved.toDouble() / totalCasesReported.max(1UL).toDouble()
        return (bedAvailability * 0.5 + resolutionEfficiency * 0.5).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixHealthSurveillanceSystem(systemId: ULong, nowNs: Long): PublicHealthSurveillanceSystem {
    return PublicHealthSurveillanceSystem(systemId, "PHOENIX_AZ", nowNs)
}
