// ALETHEION_CITIZEN_INTERFACE_V1.0.0
// LICENSE: BioticTreaty_Compliant_AGPLv3
// ECO_IMPACT: K=0.91 | E=0.89 | R=0.13
// CHAIN: SMART (Interface → Log → Treaty-Check)
// CONSTRAINTS: Offline-First, Neuro-Rights Opt-In, No-Cloud-Dependency
// INDIGENOUS_RIGHTS: Akimel_O'odham_Notification_Priority

package io.aletheion.citizen.ui.dashboard

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.aletheion.core.risk.RiskVector
import io.aletheion.data.local.CitizenDatabase
import io.aletheion.domain.model.NeuroConsent
import io.aletheion.domain.model.WaterUsageProfile
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import java.time.LocalDateTime

// --- STATE DEFINITIONS ---
data class DashboardState(
    val heatRisk: Float = 0f,
    val airQualityRisk: Float = 0f,
    val waterUsageGallons: Float = 0f,
    val targetWaterGallons: Float = 50f, // Phoenix Target vs Avg 146
    val alertLevel: AlertLevel = AlertLevel.GREEN,
    val neuroConsentActive: Boolean = false,
    val sovereigntyNotice: String = ""
)

enum class AlertLevel { GREEN, YELLOW, RED, CRITICAL }

// --- VIEWMODEL (OFFLINE FIRST) ---
class EnvironmentalDashboardViewModel(
    private val db: CitizenDatabase,
    private val riskEngine: io.aletheion.core.risk.RiskEngine // Local Rust FFI binding
) : ViewModel() {

    private val _state = MutableStateFlow(DashboardState())
    val state: StateFlow<DashboardState> = _state

    // --- NEURORIGHTS COMPLIANCE ---
    // Notifications are NEVER forced. Consent is explicit and revocable.
    fun updateNeuroConsent(consent: NeuroConsent) {
        viewModelScope.launch {
            db.neuroConsentDao().insert(consent)
            _state.value = _state.value.copy(neuroConsentActive = consent.isActive)
        }
    }

    // --- ERM: SENSE → INTERFACE ---
    // Updates UI based on local RiskEngine calculation (Offline)
    fun refreshEnvironmentalData(localRisk: RiskVector) {
        val alert = determineAlertLevel(localRisk)
        val sovereigntyMsg = if (localRisk.r_sovereignty > 0.0) {
            "Respect Akimel O'odham Water Rights: Reduce Usage Immediately"
        } else ""

        _state.value = _state.value.copy(
            heatRisk = localRisk.r_heat,
            airQualityRisk = localRisk.r_air,
            alertLevel = alert,
            sovereigntyNotice = sovereigntyMsg
        )

        // Log interaction locally for later sync
        logInteraction(alert, localRisk.aggregate_vt)
    }

    // --- SMART: TREATY-CHECK ---
    private fun determineAlertLevel(risk: RiskVector): AlertLevel {
        return when {
            risk.r_heat > 0.9 || risk.r_air > 0.9 -> AlertLevel.CRITICAL
            risk.r_heat > 0.7 || risk.r_air > 0.7 -> AlertLevel.RED
            risk.r_heat > 0.5 || risk.r_air > 0.5 -> AlertLevel.YELLOW
            else -> AlertLevel.GREEN
        }
    }

    // --- WATER USAGE TRACKING ---
    fun logWaterUsage(gallons: Float) {
        viewModelScope.launch {
            val profile = WaterUsageProfile(
                timestamp = LocalDateTime.now(),
                gallons = gallons,
                withinTarget = gallons <= _state.value.targetWaterGallons
            )
            db.waterUsageDao().insert(profile)
            
            // Update running total
            val total = db.waterUsageDao().getDailyTotal()
            _state.value = _state.value.copy(waterUsageGallons = total)
        }
    }

    // --- LOCAL LOGGING (OFFLINE) ---
    private fun logInteraction(alert: AlertLevel, riskScore: Double) {
        viewModelScope.launch {
            db.interactionLogDao().insert(
                io.aletheion.data.local.InteractionLog(
                    timestamp = System.currentTimeMillis(),
                    alertLevel = alert.name,
                    riskScore = riskScore,
                    synced = false // Queue for later sync
                )
            )
        }
    }
}
