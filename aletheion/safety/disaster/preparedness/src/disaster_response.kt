package aletheion.safety.disaster.preparedness

const val DISASTER_RESPONSE_VERSION = 20260310L
const val MAX_DISASTER_TYPES = 64
const val MAX_EMERGENCY_SHELTERS = 512
const val MAX_EVACUATION_ROUTES = 2048
const val MAX_RESOURCE_INVENTORIES = 4096
const val PHOENIX_HEAT_EMERGENCY_TEMP_F = 115.0
const val FLOOD_WARNING_THRESHOLD_IN = 2.0

enum class DisasterType {
    EXTREME_HEAT, FLASH_FLOOD, DUST_STORM, EARTHQUAKE, WILDFIRE,
    HAZMAT_RELEASE, POWER_OUTAGE, WATER_CONTAMINATION, MEDICAL_EMERGENCY,
    TERRORIST_THREAT, CIVIL_UNREST, CYBER_ATTACK
}

enum class AlertLevel {
    NORMAL(1), ADVISORY(2), WATCH(3), WARNING(4), EMERGENCY(5)
    val level: Int
    constructor(level: Int) { this.level = level }
}

enum class ShelterStatus {
    AVAILABLE, OPENING, OPERATIONAL, AT_CAPACITY, CLOSED, EVACUATED
}

data class EmergencyShelter(
    val shelterId: ULong,
    val shelterName: String,
    val latitude: Double,
    val longitude: Double,
    val capacity: UInt,
    val currentOccupancy: UInt,
    val status: ShelterStatus,
    val shelterType: String,
    val accessibilityCompliant: Boolean,
    val petFriendly: Boolean,
    val medicalSupportAvailable: Boolean,
    val powerBackupAvailable: Boolean,
    val waterSupplyDays: UInt,
    val foodSupplyDays: UInt,
    val lastInspectionNs: Long,
    val operational: Boolean
) {
    fun availabilityRatio(): Double = 
        (capacity - currentOccupancy).toDouble() / capacity.max(1U).toDouble()
    fun canAcceptOccupants(): Boolean = 
        operational && status != ShelterStatus.AT_CAPACITY && availabilityRatio() > 0.1
    fun daysOfSupplies(): UInt = waterSupplyDays.coerceAtMost(foodSupplyDays)
}

data class EvacuationRoute(
    val routeId: ULong,
    val routeName: String,
    val startLat: Double,
    val startLon: Double,
    val endLat: Double,
    val endLon: Double,
    val distanceMiles: Double,
    val estimatedTravelTimeMin: UInt,
    val capacityVehiclesPerHour: UInt,
    val currentStatus: String,
    val hazards: List<String>,
    val lastUpdatedNs: Long,
    val operational: Boolean
) {
    fun isPassable(): Boolean = operational && hazards.isEmpty()
    fun congestionLevel(): Double = 1.0 - (capacityVehiclesPerHour.toDouble() / 1000.0).coerceAtMost(1.0)
}

data class ResourceInventory(
    val inventoryId: ULong,
    val resourceType: String,
    val quantity: Double,
    val unit: String,
    val locationLat: Double,
    val locationLon: Double,
    val expirationDateNs: Long?,
    val reserved: Double,
    val available: Double,
    val lastUpdatedNs: Long,
    val supplier: String
) {
    fun availabilityRatio(): Double = available / quantity.max(0.01)
    fun isExpired(nowNs: Long): Boolean = expirationDateNs != null && nowNs > expirationDateNs!!
}

data class DisasterEvent(
    val eventId: ULong,
    val disasterType: DisasterType,
    val alertLevel: AlertLevel,
    val affectedAreaKm2: Double,
    val estimatedPopulationAffected: UInt,
    val startTimeNs: Long,
    val predictedEndTimeNs: Long?,
    val actualEndTimeNs: Long?,
    val casualtiesReported: UInt,
    val injuriesReported: UInt,
    val structuresDamaged: UInt,
    val evacuationOrdered: Boolean,
    val sheltersActivated: UInt,
    val status: String
) {
    fun isActive(nowNs: Long): Boolean = 
        status == "ACTIVE" && (predictedEndTimeNs == null || nowNs < predictedEndTimeNs)
    fun durationHours(nowNs: Long): Double = 
        (nowNs - startTimeNs).toDouble() / 3600000000000.0
}

class DisasterPreparednessResponseSystem(
    private val systemId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val disasterEvents = mutableMapOf<ULong, DisasterEvent>()
    private val emergencyShelters = mutableMapOf<ULong, EmergencyShelter>()
    private val evacuationRoutes = mutableMapOf<ULong, EvacuationRoute>()
    private val resourceInventories = mutableMapOf<ULong, ResourceInventory>()
    private val auditLog = mutableListOf<DisasterAuditEntry>()
    private var nextEventId: ULong = 1UL
    private var totalEventsYtd: UInt = 0U
    private var totalEvacuations: UInt = 0U
    private var totalShelterDays: UInt = 0U
    private var averageResponseTimeMin: Double = 0.0
    private var lastDrillNs: Long = initTimestampNs
    private var drillCompliancePct: Double = 100.0
    
    data class DisasterAuditEntry(
        val entryId: ULong,
        val action: String,
        val eventId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun declareDisasterEvent(event: DisasterEvent, nowNs: Long): Result<ULong> {
        if (event.alertLevel.level < 3) {
            logAudit("DISASTER_DECLARE", null, nowNs, false, "Alert level too low", 0.2)
            return Result.failure(Error("ALERT_LEVEL_INSUFFICIENT"))
        }
        disasterEvents[nextEventId] = event
        totalEventsYtd++
        logAudit("DISASTER_DECLARE", nextEventId, nowNs, true, 
                "Disaster declared: ${event.disasterType}", 0.1)
        val eventId = nextEventId
        nextEventId++
        return Result.success(eventId)
    }
    
    fun registerEmergencyShelter(shelter: EmergencyShelter): Result<ULong> {
        if (emergencyShelters.size >= MAX_EMERGENCY_SHELTERS) {
            logAudit("SHELTER_REGISTER", null, initTimestampNs, false, "Shelter limit exceeded", 0.3)
            return Result.failure(Error("SHELTER_LIMIT_EXCEEDED"))
        }
        emergencyShelters[shelter.shelterId] = shelter
        logAudit("SHELTER_REGISTER", shelter.shelterId, initTimestampNs, true, 
                "Shelter registered: ${shelter.shelterName}", 0.02)
        return Result.success(shelter.shelterId)
    }
    
    fun activateShelter(shelterId: ULong, nowNs: Long): Result<Unit> {
        val shelter = emergencyShelters[shelterId] ?: 
            return Result.failure(Error("SHELTER_NOT_FOUND"))
        val updatedShelter = shelter.copy(status = ShelterStatus.OPERATIONAL)
        emergencyShelters[shelterId] = updatedShelter
        logAudit("SHELTER_ACTIVATE", shelterId, nowNs, true, "Shelter activated", 0.05)
        return Result.success(Unit)
    }
    
    fun registerEvacuationRoute(route: EvacuationRoute): Result<ULong> {
        if (evacuationRoutes.size >= MAX_EVACUATION_ROUTES) {
            return Result.failure(Error("ROUTE_LIMIT_EXCEEDED"))
        }
        evacuationRoutes[route.routeId] = route
        return Result.success(route.routeId)
    }
    
    fun registerResourceInventory(inventory: ResourceInventory): Result<ULong> {
        if (resourceInventories.size >= MAX_RESOURCE_INVENTORIES) {
            return Result.failure(Error("INVENTORY_LIMIT_EXCEEDED"))
        }
        resourceInventories[inventory.inventoryId] = inventory
        return Result.success(inventory.inventoryId)
    }
    
    fun findNearestAvailableShelter(lat: Double, lon: Double, needsMedical: Boolean): EmergencyShelter? {
        return emergencyShelters.values.filter { 
            it.canAcceptOccupants() && 
            (!needsMedical || it.medicalSupportAvailable) 
        }.minByOrNull { computeDistance(lat, lon, it.latitude, it.longitude) }
    }
    
    fun findOptimalEvacuationRoute(startLat: Double, startLon: Double, 
                                    endLat: Double, endLon: Double): EvacuationRoute? {
        return evacuationRoutes.values.filter { 
            it.isPassable() && 
            Math.abs(it.startLat - startLat) < 0.1 &&
            Math.abs(it.startLon - startLon) < 0.1
        }.minByOrNull { it.estimatedTravelTimeMin }
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
    
    fun allocateResources(eventId: ULong, resourceType: String, quantity: Double, 
                          nowNs: Long): Result<Unit> {
        val event = disasterEvents[eventId] ?: 
            return Result.failure(Error("EVENT_NOT_FOUND"))
        if (!event.isActive(nowNs)) {
            return Result.failure(Error("EVENT_NOT_ACTIVE"))
        }
        val inventory = resourceInventories.values.find { 
            it.resourceType == resourceType && it.available >= quantity && !it.isExpired(nowNs)
        } ?: return Result.failure(Error("INSUFFICIENT_RESOURCES"))
        val updatedInventory = inventory.copy(
            available = inventory.available - quantity,
            lastUpdatedNs = nowNs
        )
        resourceInventories[inventory.inventoryId] = updatedInventory
        logAudit("RESOURCE_ALLOCATE", eventId, nowNs, true, 
                "Allocated $quantity ${inventory.unit} of $resourceType", 0.05)
        return Result.success(Unit)
    }
    
    fun completeDisasterEvent(eventId: ULong, nowNs: Long): Result<Unit> {
        val event = disasterEvents[eventId] ?: 
            return Result.failure(Error("EVENT_NOT_FOUND"))
        val completedEvent = event.copy(
            status = "CLOSED",
            actualEndTimeNs = nowNs
        )
        disasterEvents[eventId] = completedEvent
        logAudit("EVENT_COMPLETE", eventId, nowNs, true, 
                "Event closed after ${event.durationHours(nowNs)} hours", 0.02)
        return Result.success(Unit)
    }
    
    fun computeSystemReadiness(nowNs: Long): Double {
        val shelterReadiness = emergencyShelters.count { it.value.operational }.toDouble() / 
                              emergencyShelters.size.coerceAtLeast(1).toDouble()
        val routeReadiness = evacuationRoutes.count { it.value.isPassable() }.toDouble() / 
                            evacuationRoutes.size.coerceAtLeast(1).toDouble()
        val resourceReadiness = resourceInventories.count { !it.value.isExpired(nowNs) }.toDouble() / 
                               resourceInventories.size.coerceAtLeast(1).toDouble()
        val drillScore = drillCompliancePct / 100.0
        val daysSinceDrill = (nowNs - lastDrillNs) / 86400000000000
        val drillPenalty = if (daysSinceDrill > 90) 0.2 else 0.0
        return (shelterReadiness * 0.3 + routeReadiness * 0.25 + 
                resourceReadiness * 0.25 + drillScore * 0.2 - drillPenalty).coerceIn(0.0, 1.0)
    }
    
    fun getSystemStatus(nowNs: Long): SystemStatus {
        val activeEvents = disasterEvents.count { it.value.isActive(nowNs) }
        val operationalShelters = emergencyShelters.count { it.value.operational }
        val availableShelters = emergencyShelters.count { it.value.canAcceptOccupants() }
        val passableRoutes = evacuationRoutes.count { it.value.isPassable() }
        val validResources = resourceInventories.count { !it.value.isExpired(nowNs) }
        return SystemStatus(
            systemId = systemId,
            cityCode = cityCode,
            activeDisasterEvents = activeEvents,
            totalEventsYtd = totalEventsYtd,
            totalEvacuations = totalEvacuations,
            totalShelters = emergencyShelters.size,
            operationalShelters = operationalShelters,
            availableShelters = availableShelters,
            totalEvacuationRoutes = evacuationRoutes.size,
            passableRoutes = passableRoutes,
            totalResourceInventories = resourceInventories.size,
            validResources = validResources,
            systemReadinessScore = computeSystemReadiness(nowNs),
            averageResponseTimeMin = averageResponseTimeMin,
            lastDrillNs = lastDrillNs,
            drillCompliancePct = drillCompliancePct,
            lastUpdateNs = nowNs
        )
    }
    
    fun conductEmergencyDrill(drillType: String, nowNs: Long) {
        lastDrillNs = nowNs
        drillCompliancePct = (drillCompliancePct + 5.0).coerceAtMost(100.0)
        logAudit("DRILL_CONDUCTED", null, nowNs, true, "Drill type: $drillType", 0.01)
    }
    
    private fun logAudit(action: String, eventId: ULong?, timestampNs: Long, 
                        success: Boolean, details: String, riskScore: Double) {
        val entry = DisasterAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            eventId = eventId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<DisasterAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class SystemStatus(
    val systemId: ULong,
    val cityCode: String,
    val activeDisasterEvents: Int,
    val totalEventsYtd: UInt,
    val totalEvacuations: UInt,
    val totalShelters: Int,
    val operationalShelters: Int,
    val availableShelters: Int,
    val totalEvacuationRoutes: Int,
    val passableRoutes: Int,
    val totalResourceInventories: Int,
    val validResources: Int,
    val systemReadinessScore: Double,
    val averageResponseTimeMin: Double,
    val lastDrillNs: Long,
    val drillCompliancePct: Double,
    val lastUpdateNs: Long
) {
    fun emergencyPreparednessIndex(): Double {
        val shelterAvailability = availableShelters.toDouble() / totalShelters.coerceAtLeast(1).toDouble()
        val routeAvailability = passableRoutes.toDouble() / totalEvacuationRoutes.coerceAtLeast(1).toDouble()
        val resourceValidity = validResources.toDouble() / totalResourceInventories.coerceAtLeast(1).toDouble()
        return (shelterAvailability * 0.35 + routeAvailability * 0.30 + 
                resourceValidity * 0.20 + systemReadinessScore * 0.15).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixDisasterResponseSystem(systemId: ULong, nowNs: Long): DisasterPreparednessResponseSystem {
    return DisasterPreparednessResponseSystem(systemId, "PHOENIX_AZ", nowNs)
}
