package aletheion.core.treaty.compliance

const val TREATY_VERSION = 20260310L
const val MAX_SPECIES_PROTECTIONS = 128
const val MIN_BUFFER_ZONE_M = 500.0

data class SpeciesProtection(
    val speciesId: UShort,
    val commonName: String,
    val scientificName: String,
    val protectionLevel: Int,
    val bufferZoneM: Double,
    val criticalHabitat: Boolean,
    val seasonalRestrictions: List<Pair<Int, Int>>,
    val populationTrend: Int,
    val lastSurveyNs: Long,
) {
    companion object {
        const val LEVEL_CRITICAL = 5
        const val LEVEL_ENDANGERED = 4
        const val LEVEL_THREATENED = 3
        const val LEVEL_SENSITIVE = 2
        const val LEVEL_MONITORED = 1
    }
    
    fun isActiveRestriction(month: Int): Boolean {
        return seasonalRestrictions.any { (start, end) ->
            if (start <= end) month in start..end
            else month >= start || month <= end
        }
    }
    
    fun requiredBuffer(): Double = when (protectionLevel) {
        LEVEL_CRITICAL -> bufferZoneM * 2.0
        LEVEL_ENDANGERED -> bufferZoneM * 1.5
        LEVEL_THREATENED -> bufferZoneM
        else -> bufferZoneM * 0.5
    }
}

data class TreatyComplianceState(
    val deploymentId: ULong,
    val location: Pair<Double, Double>,
    val timestampNs: Long,
    val nearbySpecies: MutableList<SpeciesProtection>,
    val violationCount: Int,
    val lastCheckNs: Long,
    val complianceScore: Double,
) {
    fun addSpecies(species: SpeciesProtection) {
        if (nearbySpecies.size < MAX_SPECIES_PROTECTIONS) {
            nearbySpecies.add(species)
        }
    }
    
    fun computeComplianceScore(): Double {
        if (nearbySpecies.isEmpty()) return 1.0
        var score = 1.0
        for (species in nearbySpecies) {
            when (species.protectionLevel) {
                SpeciesProtection.LEVEL_CRITICAL -> score -= 0.3
                SpeciesProtection.LEVEL_ENDANGERED -> score -= 0.2
                SpeciesProtection.LEVEL_THREATENED -> score -= 0.1
                else -> score -= 0.05
            }
        }
        return score.coerceIn(0.0, 1.0)
    }
    
    fun hasViolations(): Boolean = violationCount > 0 || complianceScore < 0.7
}

class BioticTreatyChecker(
    private val checkerId: ULong,
    private val initTimestampNs: Long
) {
    private var currentState: TreatyComplianceState? = null
    private val auditLog = mutableListOf<ComplianceAuditEntry>()
    
    data class ComplianceAuditEntry(
        val entryId: ULong,
        val deploymentId: ULong,
        val checkType: String,
        val passed: Boolean,
        val details: String,
        val timestampNs: Long,
    )
    
    fun initializeDeployment(
        deploymentId: ULong,
        lat: Double,
        lon: Double,
        nowNs: Long
    ): TreatyComplianceState {
        val state = TreatyComplianceState(
            deploymentId = deploymentId,
            location = Pair(lat, lon),
            timestampNs = nowNs,
            nearbySpecies = mutableListOf(),
            violationCount = 0,
            lastCheckNs = nowNs,
            complianceScore = 1.0
        )
        currentState = state
        logAudit(deploymentId, "INIT", true, "Deployment initialized", nowNs)
        return state
    }
    
    fun registerSpeciesPresence(
        species: SpeciesProtection,
        distanceM: Double,
        nowNs: Long
    ): Result<Unit> {
        val state = currentState ?: return Result.failure(Error("STATE_NOT_INITIALIZED"))
        if (distanceM < species.requiredBuffer()) {
            state.violationCount++
            logAudit(
                state.deploymentId,
                "BUFFER_VIOLATION",
                false,
                "${species.commonName} buffer violated: ${distanceM}m < ${species.requiredBuffer()}m",
                nowNs
            )
            return Result.failure(Error("BUFFER_ZONE_VIOLATION"))
        }
        state.addSpecies(species)
        state.complianceScore = state.computeComplianceScore()
        logAudit(state.deploymentId, "SPECIES_REGISTERED", true, species.commonName, nowNs)
        return Result.success(Unit)
    }
    
    fun checkSeasonalRestrictions(month: Int, nowNs: Long): Result<Unit> {
        val state = currentState ?: return Result.failure(Error("STATE_NOT_INITIALIZED"))
        for (species in state.nearbySpecies) {
            if (species.isActiveRestriction(month)) {
                if (species.protectionLevel >= SpeciesProtection.LEVEL_THREATENED) {
                    logAudit(
                        state.deploymentId,
                        "SEASONAL_RESTRICTION",
                        false,
                        "${species.commonName} seasonal restriction active",
                        nowNs
                    )
                    return Result.failure(Error("SEASONAL_RESTRICTION_ACTIVE"))
                }
            }
        }
        return Result.success(Unit)
    }
    
    fun preActuationCheck(nowNs: Long): Result<Unit> {
        val state = currentState ?: return Result.failure(Error("STATE_NOT_INITIALIZED"))
        state.lastCheckNs = nowNs
        if (state.hasViolations()) {
            return Result.failure(Error("COMPLIANCE_VIOLATION"))
        }
        if (state.complianceScore < 0.7) {
            return Result.failure(Error("COMPLIANCE_SCORE_LOW"))
        }
        return Result.success(Unit)
    }
    
    private fun logAudit(
        deploymentId: ULong,
        checkType: String,
        passed: Boolean,
        details: String,
        timestampNs: Long
    ) {
        val entry = ComplianceAuditEntry(
            entryId = auditLog.size.toULong(),
            deploymentId = deploymentId,
            checkType = checkType,
            passed = passed,
            details = details,
            timestampNs = timestampNs
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(): List<ComplianceAuditEntry> = auditLog.toList()
    
    fun computeTreatyScore(): Double {
        val state = currentState ?: return 0.0
        val auditPassRate = if (auditLog.isEmpty()) 1.0
            else auditLog.count { it.passed }.toDouble() / auditLog.size
        return (state.complianceScore * 0.7 + auditPassRate * 0.3)
    }
}

fun createHoneybeeProtection(): SpeciesProtection = SpeciesProtection(
    speciesId = 1u,
    commonName = "Western Honey Bee",
    scientificName = "Apis mellifera",
    protectionLevel = SpeciesProtection.LEVEL_THREATENED,
    bufferZoneM = MIN_BUFFER_ZONE_M,
    criticalHabitat = true,
    seasonalRestrictions = listOf(Pair(3, 9)),
    populationTrend = -1,
    lastSurveyNs = 0L
)

fun createSonoranDesertTortoiseProtection(): SpeciesProtection = SpeciesProtection(
    speciesId = 2u,
    commonName = "Desert Tortoise",
    scientificName = "Gopherus morafkai",
    protectionLevel = SpeciesProtection.LEVEL_THREATENED,
    bufferZoneM = 1000.0,
    criticalHabitat = true,
    seasonalRestrictions = listOf(Pair(4, 10)),
    populationTrend = -1,
    lastSurveyNs = 0L
)
