package aletheion.transportation.fleet.management

const val FLEET_MANAGER_VERSION = 20260310L
const val MAX_VEHICLES = 8192
const val MAX_ROUTES = 4096
const val MAX_CHARGING_STATIONS = 512
const val VEHICLE_HEALTH_CHECK_INTERVAL_S = 60L

enum class VehicleType {
    PASSENGER_SEDAN, PASSENGER_SUV, TRANSIT_BUS, DELIVERY_VAN,
    FREIGHT_TRUCK, EMERGENCY_VEHICLE, MAINTENANCE_VEHICLE, AUTONOMOUS_SHUTTLE
}

enum class VehicleStatus {
    AVAILABLE, IN_SERVICE, CHARGING, MAINTENANCE, OFFLINE,
    EMERGENCY_STOP, LOW_BATTERY, ROUTE_ASSIGNED, PICKUP_PENDING, DROPOFF_COMPLETE
}

enum class PowertrainType {
    BATTERY_ELECTRIC, HYDROGEN_FUEL_CELL, HYBRID_ELECTRIC, SOLAR_ELECTRIC
}

data class VehicleIdentity(
    val vehicleId: ULong,
    val vin: String,
    val vehicleType: VehicleType,
    val powertrain: PowertrainType,
    val manufacturer: String,
    val modelYear: UInt,
    val capacity: UInt,
    val rangeKm: Double,
    val deploymentDateNs: Long,
    val homeDepot: ULong
)

data class VehicleState(
    val status: VehicleStatus,
    val batterySoc: Double,
    val batteryHealth: Double,
    val locationLat: Double,
    val locationLon: Double,
    val speedKmh: Double,
    val heading: Double,
    val odometerKm: Double,
    val lastCommunicationNs: Long,
    val errorCodes: List<UInt>,
    val temperatureCelsius: Double,
    val tirePressurePsi: List<Double>
) {
    fun isOperational(nowNs: Long): Boolean {
        return status != VehicleStatus.OFFLINE &&
               status != VehicleStatus.MAINTENANCE &&
               status != VehicleStatus.EMERGENCY_STOP &&
               nowNs - lastCommunicationNs < VEHICLE_HEALTH_CHECK_INTERVAL_S * 1000000000 &&
               batterySoc > 15.0 &&
               errorCodes.isEmpty()
    }
    fun requiresCharging(): Boolean = batterySoc < 30.0 || status == VehicleStatus.LOW_BATTERY
    fun requiresMaintenance(): Boolean = batteryHealth < 80.0 || errorCodes.isNotEmpty() || status == VehicleStatus.MAINTENANCE
}

data class RouteAssignment(
    val assignmentId: ULong,
    val vehicleId: ULong,
    val routeId: ULong,
    val pickupLocation: Pair<Double, Double>,
    val dropoffLocation: Pair<Double, Double>,
    val assignedAtNs: Long,
    val estimatedPickupNs: Long,
    val estimatedDropoffNs: Long,
    val completedAtNs: Long?,
    val passengerCount: UInt,
    val priority: UInt
) {
    fun isActive(nowNs: Long): Boolean = completedAtNs == null && nowNs < estimatedDropoffNs + 3600000000000
    fun isOverdue(nowNs: Long): Boolean = completedAtNs == null && nowNs > estimatedDropoffNs
}

data class ChargingStation(
    val stationId: ULong,
    val locationLat: Double,
    val locationLon: Double,
    val connectorType: String,
    val maxPowerKw: Double,
    val availableConnectors: UInt,
    val totalConnectors: UInt,
    val operational: Boolean,
    val energyCostPerKwh: Double,
    val renewableEnergyPct: Double
) {
    fun availabilityRatio(): Double = availableConnectors.toDouble() / totalConnectors.max(1U).toDouble()
}

class AutonomousFleetManager(
    private val managerId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val vehicles = mutableMapOf<ULong, Pair<VehicleIdentity, VehicleState>>()
    private val routeAssignments = mutableMapOf<ULong, RouteAssignment>()
    private val chargingStations = mutableMapOf<ULong, ChargingStation>()
    private val auditLog = mutableListOf<FleetAuditEntry>()
    private var nextAssignmentId: ULong = 1UL
    private var totalTripsCompleted: ULong = 0UL
    private var totalPassengersServed: ULong = 0UL
    private var totalDistanceKm: Double = 0.0
    private var totalEnergyKwh: Double = 0.0
    private var safetyIncidents: ULong = 0UL
    
    data class FleetAuditEntry(
        val entryId: ULong,
        val action: String,
        val vehicleId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun registerVehicle(identity: VehicleIdentity, initialState: VehicleState, nowNs: Long): Result<ULong> {
        if (vehicles.size >= MAX_VEHICLES) {
            logAudit("VEHICLE_REGISTER", null, nowNs, false, "Fleet size limit exceeded", 0.3)
            return Result.failure(Error("FLEET_SIZE_LIMIT"))
        }
        vehicles[identity.vehicleId] = Pair(identity, initialState)
        logAudit("VEHICLE_REGISTER", identity.vehicleId, nowNs, true, "Vehicle registered", 0.02)
        return Result.success(identity.vehicleId)
    }
    
    fun updateVehicleState(vehicleId: ULong, newState: VehicleState, nowNs: Long): Result<Unit> {
        val vehicle = vehicles[vehicleId] ?: return Result.failure(Error("VEHICLE_NOT_FOUND"))
        vehicles[vehicleId] = Pair(vehicle.first, newState)
        if (newState.errorCodes.isNotEmpty()) {
            logAudit("VEHICLE_ERROR", vehicleId, nowNs, false, "Error codes: ${newState.errorCodes}", 0.2)
        }
        return Result.success(Unit)
    }
    
    fun assignRoute(assignment: RouteAssignment, nowNs: Long): Result<ULong> {
        val vehicle = vehicles[assignment.vehicleId] ?: return Result.failure(Error("VEHICLE_NOT_FOUND"))
        if (!vehicle.second.isOperational(nowNs)) {
            logAudit("ROUTE_ASSIGN", assignment.vehicleId, nowNs, false, "Vehicle not operational", 0.15)
            return Result.failure(Error("VEHICLE_NOT_OPERATIONAL"))
        }
        if (vehicle.second.batterySoc < 20.0) {
            logAudit("ROUTE_ASSIGN", assignment.vehicleId, nowNs, false, "Battery too low", 0.1)
            return Result.failure(Error("INSUFFICIENT_BATTERY"))
        }
        routeAssignments[nextAssignmentId] = assignment
        val assignmentId = nextAssignmentId
        nextAssignmentId++
        logAudit("ROUTE_ASSIGN", assignment.vehicleId, nowNs, true, "Route assigned: $assignmentId", 0.05)
        return Result.success(assignmentId)
    }
    
    fun completeRoute(assignmentId: ULong, nowNs: Long, distanceKm: Double, energyKwh: Double): Result<Unit> {
        val assignment = routeAssignments[assignmentId] ?: return Result.failure(Error("ASSIGNMENT_NOT_FOUND"))
        val completedAssignment = assignment.copy(completedAtNs = nowNs)
        routeAssignments[assignmentId] = completedAssignment
        totalTripsCompleted++
        totalPassengersServed += completedAssignment.passengerCount.toULong()
        totalDistanceKm += distanceKm
        totalEnergyKwh += energyKwh
        logAudit("ROUTE_COMPLETE", assignment.vehicleId, nowNs, true, "Trip completed: ${distanceKm}km", 0.01)
        return Result.success(Unit)
    }
    
    fun registerChargingStation(station: ChargingStation) {
        chargingStations[station.stationId] = station
        logAudit("CHARGING_STATION_REGISTER", null, initTimestampNs, true, "Station ${station.stationId} registered", 0.02)
    }
    
    fun findNearestChargingStation(lat: Double, lon: Double, vehicleId: ULong): ChargingStation? {
        val vehicle = vehicles[vehicleId]?.second ?: return null
        if (!vehicle.requiresCharging()) return null
        return chargingStations.values.filter { it.operational && it.availableConnectors > 0U }
            .minByOrNull { computeDistance(lat, lon, it.locationLat, it.locationLon) }
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
    
    fun getAvailableVehicles(vehicleType: VehicleType? = null, nowNs: Long): List<ULong> {
        return vehicles.filter { (_, pair) ->
            pair.second.status == VehicleStatus.AVAILABLE &&
            pair.second.isOperational(nowNs) &&
            (vehicleType == null || pair.first.vehicleType == vehicleType)
        }.keys.toList()
    }
    
    fun getVehiclesRequiringMaintenance(nowNs: Long): List<ULong> {
        return vehicles.filter { (_, pair) ->
            pair.second.requiresMaintenance()
        }.keys.toList()
    }
    
    fun getVehiclesRequiringCharging(nowNs: Long): List<ULong> {
        return vehicles.filter { (_, pair) ->
            pair.second.requiresCharging() && pair.second.isOperational(nowNs)
        }.keys.toList()
    }
    
    fun computeFleetUtilization(nowNs: Long): Double {
        if (vehicles.isEmpty()) return 0.0
        val operationalCount = vehicles.count { (_, pair) -> pair.second.isOperational(nowNs) }
        val inServiceCount = vehicles.count { (_, pair) -> pair.second.status == VehicleStatus.IN_SERVICE }
        return inServiceCount.toDouble() / operationalCount.max(1).toDouble()
    }
    
    fun computeFleetHealthScore(nowNs: Long): Double {
        if (vehicles.isEmpty()) return 0.0
        val operationalRatio = vehicles.count { (_, pair) -> pair.second.isOperational(nowNs) }.toDouble() / vehicles.size
        val batteryHealthAvg = vehicles.map { (_, pair) -> pair.second.batteryHealth }.average()
        val safetyPenalty = (safetyIncidents.toDouble() * 0.05).coerceAtMost(0.3)
        return (operationalRatio * 0.4 + batteryHealthAvg / 100.0 * 0.4 + (1.0 - safetyPenalty) * 0.2).coerceIn(0.0, 1.0)
    }
    
    fun getFleetStatus(nowNs: Long): FleetStatus {
        val operationalVehicles = vehicles.count { (_, pair) -> pair.second.isOperational(nowNs) }
        val chargingVehicles = getVehiclesRequiringCharging(nowNs).size
        val maintenanceVehicles = getVehiclesRequiringMaintenance(nowNs).size
        val activeAssignments = routeAssignments.count { (_, assignment) -> assignment.isActive(nowNs) }
        return FleetStatus(
            managerId = managerId,
            cityCode = cityCode,
            totalVehicles = vehicles.size,
            operationalVehicles = operationalVehicles,
            availableVehicles = getAvailableVehicles(null, nowNs).size,
            inServiceVehicles = vehicles.count { (_, pair) -> pair.second.status == VehicleStatus.IN_SERVICE },
            chargingVehicles = chargingVehicles,
            maintenanceVehicles = maintenanceVehicles,
            offlineVehicles = vehicles.size - operationalVehicles,
            activeAssignments = activeAssignments,
            totalTripsCompleted = totalTripsCompleted,
            totalPassengersServed = totalPassengersServed,
            totalDistanceKm = totalDistanceKm,
            totalEnergyKwh = totalEnergyKwh,
            safetyIncidents = safetyIncidents,
            fleetUtilization = computeFleetUtilization(nowNs),
            fleetHealthScore = computeFleetHealthScore(nowNs),
            chargingStations = chargingStations.size,
            lastUpdateNs = nowNs
        )
    }
    
    fun recordSafetyIncident(vehicleId: ULong, severity: UInt, nowNs: Long) {
        safetyIncidents++
        logAudit("SAFETY_INCIDENT", vehicleId, nowNs, false, "Safety incident recorded, severity: $severity", 0.5)
    }
    
    private fun logAudit(action: String, vehicleId: ULong?, timestampNs: Long, success: Boolean, details: String, riskScore: Double) {
        val entry = FleetAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            vehicleId = vehicleId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<FleetAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
    
    fun computeOperationalEfficiency(): Double {
        if (totalTripsCompleted == 0UL) return 1.0
        val energyEfficiency = if (totalEnergyKwh > 0) totalDistanceKm / totalEnergyKwh else 0.0
        val normalizedEnergyEff = (energyEfficiency / 5.0).coerceAtMost(1.0)
        val utilizationScore = computeFleetUtilization(System.nanoTime())
        val safetyScore = if (safetyIncidents == 0UL) 1.0 else 0.7
        return (normalizedEnergyEff * 0.4 + utilizationScore * 0.4 + safetyScore * 0.2).coerceIn(0.0, 1.0)
    }
}

data class FleetStatus(
    val managerId: ULong,
    val cityCode: String,
    val totalVehicles: Int,
    val operationalVehicles: Int,
    val availableVehicles: Int,
    val inServiceVehicles: Int,
    val chargingVehicles: Int,
    val maintenanceVehicles: Int,
    val offlineVehicles: Int,
    val activeAssignments: Int,
    val totalTripsCompleted: ULong,
    val totalPassengersServed: ULong,
    val totalDistanceKm: Double,
    val totalEnergyKwh: Double,
    val safetyIncidents: ULong,
    val fleetUtilization: Double,
    val fleetHealthScore: Double,
    val chargingStations: Int,
    val lastUpdateNs: Long
) {
    fun serviceReadiness(): Double {
        val availabilityRatio = availableVehicles.toDouble() / operationalVehicles.max(1).toDouble()
        val healthFactor = fleetHealthScore
        val incidentPenalty = if (safetyIncidents == 0UL) 1.0 else 0.8
        return (availabilityRatio * 0.5 + healthFactor * 0.3 + incidentPenalty * 0.2).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixFleetManager(managerId: ULong, nowNs: Long): AutonomousFleetManager {
    return AutonomousFleetManager(managerId, "PHOENIX_AZ", nowNs)
}
