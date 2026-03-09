package aletheion.emergency.dust.storm

const val DUST_WARNING_VERSION = 20260310L
const val MAX_VISIBILITY_SENSORS = 2048
const val MAX_DUST_ALERTS = 16384
const val MAX_AIR_QUALITY_STATIONS = 1024
const val CRITICAL_VISIBILITY_MI = 0.25
const val WARNING_VISIBILITY_MI = 0.5
const val PM10_CRITICAL_UGM3 = 500.0
const val PM2_5_CRITICAL_UGM3 = 300.0

enum class DustAlertLevel {
    NORMAL(0), ADVISORY(1), WATCH(2), WARNING(3), EMERGENCY(4), CRITICAL(5)
    val level: Int
    constructor(level: Int) { this.level = level }
}

enum class SensorLocationType {
    HIGHWAY, URBAN, RURAL, AIRPORT, INDUSTRIAL, RESIDENTIAL, SCHOOL, HOSPITAL
}

data class VisibilitySensor(
    val sensorId: ULong,
    val locationType: SensorLocationType,
    val latitude: Double,
    val longitude: Double,
    val elevationFt: Double,
    val currentVisibilityMi: Double,
    val pm10Ugm3: Double,
    val pm2_5Ugm3: Double,
    val windSpeedMph: Double,
    val windDirectionDeg: Double,
    val humidityPct: Double,
    val temperatureF: Double,
    val lastReadingNs: Long,
    val operational: Boolean,
    val calibrationValid: Boolean,
    val maintenanceRequired: Boolean,
    val watershedZoneId: UInt
) {
    fun computeAlertLevel(): DustAlertLevel {
        return when {
            currentVisibilityMi < 0.1 || pm10Ugm3 > PM10_CRITICAL_UGM3 -> DustAlertLevel.CRITICAL
            currentVisibilityMi < 0.25 || pm10Ugm3 > 400.0 -> DustAlertLevel.EMERGENCY
            currentVisibilityMi < 0.5 || pm10Ugm3 > 300.0 -> DustAlertLevel.WARNING
            currentVisibilityMi < 1.0 || pm10Ugm3 > 200.0 -> DustAlertLevel.WATCH
            currentVisibilityMi < 2.0 || pm10Ugm3 > 100.0 -> DustAlertLevel.ADVISORY
            else -> DustAlertLevel.NORMAL
        }
    }
    fun isOperational(nowNs: Long): Boolean = 
        operational && calibrationValid && (nowNs - lastReadingNs) < 3600000000000L
}

data class DustAlert(
    val alertId: ULong,
    val alertLevel: DustAlertLevel,
    val affectedZones: List<UInt>,
    val issuedAtNs: Long,
    val expiresAtNs: Long,
    val minVisibilityMi: Double,
    val maxPm10Ugm3: Double,
    val maxPm2_5Ugm3: Double,
    val windSpeedMph: Double,
    val stormDirectionDeg: Double,
    val stormSpeedMph: Double,
    val publicNotificationSent: Boolean,
    val transportationAlertsSent: Boolean,
    val schoolClosuresRecommended: Boolean,
    val outdoorWorkSuspension: Boolean,
    val casualtiesReported: UInt,
    val accidentsReported: UInt,
    val flightCancellations: UInt
) {
    fun isActive(nowNs: Long): Boolean = nowNs in issuedAtNs until expiresAtNs
    fun requiresEmergencyResponse(): Boolean = alertLevel.level >= 4
    fun durationMinutes(nowNs: Long): Long = 
        ((if (nowNs < expiresAtNs) nowNs else expiresAtNs) - issuedAtNs) / 60000000000L
}

data class AirQualityStation(
    val stationId: ULong,
    val stationName: String,
    val latitude: Double,
    val longitude: Double,
    val pm10Ugm3: Double,
    val pm2_5Ugm3: Double,
    val o3Ugm3: Double,
    val no2Ugm3: Double,
    val coUgm3: Double,
    val so2Ugm3: Double,
    val aqiValue: UInt,
    val aqiCategory: String,
    val lastReadingNs: Long,
    val operational: Boolean,
    val publicDisplayEnabled: Boolean
) {
    fun isHealthy(): Boolean = aqiValue < 100U && operational
    fun requiresHealthWarning(): Boolean = aqiValue >= 150U
}

class DustStormWarningSystem(
    private val systemId: ULong,
    private val cityCode: String,
    private val initTimestampNs: Long
) {
    private val visibilitySensors = mutableMapOf<ULong, VisibilitySensor>()
    private val dustAlerts = mutableMapOf<ULong, DustAlert>()
    private val airQualityStations = mutableMapOf<ULong, AirQualityStation>()
    private val auditLog = mutableListOf<DustAuditEntry>()
    private var nextAlertId: ULong = 1UL
    private var totalAlertsIssued: ULong = 0UL
    private var totalFalseAlarms: ULong = 0UL
    private var totalCasualties: ULong = 0UL
    private var totalAccidents: ULong = 0UL
    private var averageWarningTimeMin: Double = 0.0
    private var detectionAccuracyPct: Double = 100.0
    private var lastMajorStormNs: Long = initTimestampNs
    private var lastSystemOptimizationNs: Long = initTimestampNs
    
    data class DustAuditEntry(
        val entryId: ULong,
        val action: String,
        val alertId: ULong?,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun registerVisibilitySensor(sensor: VisibilitySensor): Result<ULong> {
        if (visibilitySensors.size >= MAX_VISIBILITY_SENSORS.toULong()) {
            logAudit("SENSOR_REGISTER", null, initTimestampNs, false, "Sensor limit exceeded", 0.3)
            return Result.failure(Error("SENSOR_LIMIT_EXCEEDED"))
        }
        if (!sensor.calibrationValid) {
            return Result.failure(Error("CALIBRATION_REQUIRED"))
        }
        visibilitySensors[sensor.sensorId] = sensor
        logAudit("SENSOR_REGISTER", sensor.sensorId, initTimestampNs, true, "Sensor registered", 0.02)
        return Result.success(sensor.sensorId)
    }
    
    fun registerAirQualityStation(station: AirQualityStation): Result<ULong> {
        if (airQualityStations.size >= MAX_AIR_QUALITY_STATIONS.toULong()) {
            return Result.failure(Error("STATION_LIMIT_EXCEEDED"))
        }
        airQualityStations[station.stationId] = station
        return Result.success(station.stationId)
    }
    
    fun issueDustAlert(alert: DustAlert, nowNs: Long): Result<ULong> {
        if (dustAlerts.size >= MAX_DUST_ALERTS.toULong()) {
            return Result.failure(Error("ALERT_LIMIT_EXCEEDED"))
        }
        dustAlerts[nextAlertId] = alert
        totalAlertsIssued++
        if (alert.alertLevel.level >= 4) {
            lastMajorStormNs = nowNs
        }
        logAudit("ALERT_ISSUE", nextAlertId, nowNs, true, "Dust alert issued: ${alert.alertLevel}", 0.1)
        val alertId = nextAlertId
        nextAlertId++
        return Result.success(alertId)
    }
    
    fun processSensorReading(sensorId: ULong, reading: VisibilitySensor, nowNs: Long): Result<Unit> {
        val sensor = visibilitySensors[sensorId] ?: return Result.failure(Error("SENSOR_NOT_FOUND"))
        val updatedSensor = sensor.copy(
            currentVisibilityMi = reading.currentVisibilityMi,
            pm10Ugm3 = reading.pm10Ugm3,
            pm2_5Ugm3 = reading.pm2_5Ugm3,
            windSpeedMph = reading.windSpeedMph,
            lastReadingNs = nowNs
        )
        visibilitySensors[sensorId] = updatedSensor
        val alertLevel = updatedSensor.computeAlertLevel()
        if (alertLevel.level >= 3) {
            triggerAutomaticAlert(alertLevel, listOf(sensor.watershedZoneId), updatedSensor, nowNs)
        }
        return Result.success(Unit)
    }
    
    private fun triggerAutomaticAlert(alertLevel: DustAlertLevel, zoneIds: List<UInt>, 
                                       sensor: VisibilitySensor, nowNs: Long) {
        val alert = DustAlert(
            alertId = nextAlertId,
            alertLevel = alertLevel,
            affectedZones = zoneIds,
            issuedAtNs = nowNs,
            expiresAtNs = nowNs + 7200000000000,
            minVisibilityMi = sensor.currentVisibilityMi,
            maxPm10Ugm3 = sensor.pm10Ugm3,
            maxPm2_5Ugm3 = sensor.pm2_5Ugm3,
            windSpeedMph = sensor.windSpeedMph,
            stormDirectionDeg = sensor.windDirectionDeg,
            stormSpeedMph = sensor.windSpeedMph * 0.5,
            publicNotificationSent = alertLevel.level >= 3,
            transportationAlertsSent = alertLevel.level >= 3,
            schoolClosuresRecommended = alertLevel.level >= 4,
            outdoorWorkSuspension = alertLevel.level >= 4,
            casualtiesReported = 0U,
            accidentsReported = 0U,
            flightCancellations = 0U
        )
        dustAlerts[nextAlertId] = alert
        nextAlertId++
        totalAlertsIssued++
    }
    
    fun recordStormImpact(alertId: ULong, casualties: UInt, accidents: UInt, nowNs: Long) {
        val alert = dustAlerts[alertId] ?: return
        val updatedAlert = alert.copy(
            casualtiesReported = casualties,
            accidentsReported = accidents
        )
        dustAlerts[alertId] = updatedAlert
        totalCasualties += casualties.toULong()
        totalAccidents += accidents.toULong()
        logAudit("IMPACT_RECORD", alertId, nowNs, true, "Casualties: $casualties, Accidents: $accidents", 0.2)
    }
    
    fun computeDetectionAccuracy(): Double {
        if (totalAlertsIssued == 0UL) return 100.0
        val truePositives = totalAlertsIssued - totalFalseAlarms
        detectionAccuracyPct = (truePositives.toDouble() / totalAlertsIssued.toDouble()) * 100.0
        return detectionAccuracyPct
    }
    
    fun findAffectedHighways(alert: DustAlert): List<String> {
        val affectedHighways = mutableListOf<String>()
        for ((_, sensor) in visibilitySensors) {
            if (sensor.locationType == SensorLocationType.HIGHWAY &&
                sensor.currentVisibilityMi < 0.5) {
                affectedHighways.add("I-10, Mile ${(sensor.latitude * 100).toInt()}")
            }
        }
        return affectedHighways.distinct()
    }
    
    fun getSystemStatus(nowNs: Long): SystemStatus {
        val operationalSensors = visibilitySensors.count { it.value.isOperational(nowNs) }
        val activeAlerts = dustAlerts.count { it.value.isActive(nowNs) }
        val healthyStations = airQualityStations.count { it.value.isHealthy() }
        val stationsWithWarnings = airQualityStations.count { it.value.requiresHealthWarning() }
        return SystemStatus(
            systemId = systemId,
            cityCode = cityCode,
            totalVisibilitySensors = visibilitySensors.size,
            operationalSensors = operationalSensors,
            totalAirQualityStations = airQualityStations.size,
            healthyStations = healthyStations,
            stationsWithWarnings = stationsWithWarnings,
            totalDustAlerts = dustAlerts.size,
            activeAlerts = activeAlerts,
            totalAlertsIssued = totalAlertsIssued,
            totalFalseAlarms = totalFalseAlarms,
            totalCasualties = totalCasualties,
            totalAccidents = totalAccidents,
            detectionAccuracyPct = computeDetectionAccuracy(),
            averageWarningTimeMin = averageWarningTimeMin,
            lastMajorStormNs = lastMajorStormNs,
            lastOptimizationNs = lastSystemOptimizationNs,
            lastUpdateNs = nowNs
        )
    }
    
    fun computeDustResilienceIndex(nowNs: Long): Double {
        val status = getSystemStatus(nowNs)
        val sensorCoverage = status.operationalSensors.toDouble() / status.totalVisibilitySensors.max(1).toDouble()
        val detectionScore = status.detectionAccuracyPct / 100.0
        val stationHealth = status.healthyStations.toDouble() / status.totalAirQualityStations.max(1).toDouble()
        val casualtyPenalty = if (status.totalCasualties > 0) 0.2 else 0.0
        return (sensorCoverage * 0.35 + detectionScore * 0.35 + 
                stationHealth * 0.30 - casualtyPenalty).coerceIn(0.0, 1.0)
    }
    
    private fun logAudit(action: String, alertId: ULong?, timestampNs: Long, 
                        success: Boolean, details: String, riskScore: Double) {
        val entry = DustAuditEntry(
            entryId = auditLog.size.toULong(),
            action = action,
            alertId = alertId,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<DustAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
}

data class SystemStatus(
    val systemId: ULong,
    val cityCode: String,
    val totalVisibilitySensors: Int,
    val operationalSensors: Int,
    val totalAirQualityStations: Int,
    val healthyStations: Int,
    val stationsWithWarnings: Int,
    val totalDustAlerts: Int,
    val activeAlerts: Int,
    val totalAlertsIssued: ULong,
    val totalFalseAlarms: ULong,
    val totalCasualties: ULong,
    val totalAccidents: ULong,
    val detectionAccuracyPct: Double,
    val averageWarningTimeMin: Double,
    val lastMajorStormNs: Long,
    val lastOptimizationNs: Long,
    val lastUpdateNs: Long
) {
    fun earlyWarningEffectiveness(): Double {
        val detectionFactor = detectionAccuracyPct / 100.0
        val sensorFactor = operationalSensors.toDouble() / totalVisibilitySensors.max(1).toDouble()
        val casualtyFactor = if (totalCasualties == 0UL) 1.0 else 0.7
        return (detectionFactor * 0.5 + sensorFactor * 0.3 + casualtyFactor * 0.2).coerceIn(0.0, 1.0)
    }
}

fun createPhoenixDustWarningSystem(systemId: ULong, nowNs: Long): DustStormWarningSystem {
    return DustStormWarningSystem(systemId, "PHOENIX_AZ", nowNs)
}
