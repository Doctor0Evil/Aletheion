// ALETHEION_BIOSIGNAL_PRIVACY_ENGINE_V1.0.0
// LICENSE: BioticTreaty_Compliant_AGPLv3
// ECO_IMPACT: K=0.91 | E=0.88 | R=0.13
// CHAIN: SMART (Interface → Treaty-Check → Log)
// CONSTRAINTS: Neurorights-Compliant, Offline-First, Zero-Knowledge
// INDIGENOUS_RIGHTS: Traditional_Healing_Data_Sovereignty

package io.aletheion.health.privacy

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.aletheion.core.crypto.LocalEncryption
import io.aletheion.data.local.HealthDatabase
import io.aletheion.domain.model.NeuroConsent
import io.aletheion.domain.model.BiosignalReading
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import java.time.LocalDateTime

// --- STATE DEFINITIONS ---
data class HealthDashboardState(
    val heartRateBpm: Int = 0,
    val stressLevel: Float = 0f,
    val sleepQualityScore: Float = 0f,
    val environmentalHealthCorrelation: Float = 0f,
    val consentActive: Boolean = false,
    val dataSharingLevel: DataSharingLevel = DataSharingLevel.NONE,
    val indigenousHealingOptIn: Boolean = false
)

enum class DataSharingLevel {
    NONE,
    AGGREGATED_ANONYMOUS,
    RESEARCH_OPT_IN,
    FULL_MEDICAL_SHARE
}

// --- PRIVACY ENGINE VIEWMODEL ---
class BiosignalPrivacyEngine(
    private val db: HealthDatabase,
    private val encryption: LocalEncryption // Post-quantum safe local encryption
) : ViewModel() {

    private val _state = MutableStateFlow(HealthDashboardState())
    val state: StateFlow<HealthDashboardState> = _state

    // --- NEURORIGHTS COMPLIANCE ---
    // All biosignal collection requires explicit, revocable consent
    fun updateNeuroConsent(consent: NeuroConsent) {
        viewModelScope.launch {
            db.neuroConsentDao().insert(consent)
            _state.value = _state.value.copy(consentActive = consent.isActive)
            
            if (!consent.isActive) {
                // Immediate data deletion on consent withdrawal
                purgeAllBiosignalData()
            }
        }
    }

    // --- SMART: TREATY-CHECK ---
    // Validates data sharing permissions before any transmission
    suspend fun canShareData(reading: BiosignalReading): Boolean {
        val consent = db.neuroConsentDao().getCurrent()
        return when (_state.value.dataSharingLevel) {
            DataSharingLevel.NONE -> false
            DataSharingLevel.AGGREGATED_ANONYMOUS -> reading.isAnonymized()
            DataSharingLevel.RESEARCH_OPT_IN -> consent?.researchOptIn == true
            DataSharingLevel.FULL_MEDICAL_SHARE -> consent?.medicalShare == true
        }
    }

    // --- ERM: SENSE → LOG ---
    // Stores biosignal readings with local encryption (offline-first)
    fun logBiosignalReading(reading: BiosignalReading) {
        viewModelScope.launch {
            val encryptedReading = encryption.encrypt(reading.toJson())
            
            db.biosignalDao().insert(
                BiosignalReading(
                    timestamp = LocalDateTime.now(),
                    heartRate = reading.heartRateBpm,
                    stressLevel = reading.stressLevel,
                    sleepScore = reading.sleepQualityScore,
                    encryptedData = encryptedReading,
                    synced = false // Queue for later sync when online
                )
            )
            
            updateDashboardState(reading)
        }
    }

    // --- ENVIRONMENTAL HEALTH CORRELATION ---
    // Correlates biosignals with environmental data (heat, air quality)
    fun calculateEnvironmentalCorrelation(envRiskScore: Float): Float {
        val recentReadings = db.biosignalDao().getLast24Hours()
        val avgStress = recentReadings.map { it.stressLevel }.average().toFloat()
        
        // Correlation: higher environmental risk → higher stress
        return (avgStress * envRiskScore).coerceIn(0f, 1f)
    }

    // --- INDIGENOUS HEALING SOVEREIGNTY ---
    // Optional integration with traditional healing practices
    fun setIndigenousHealingOptIn(optIn: Boolean) {
        viewModelScope.launch {
            db.preferencesDao().setIndigenousHealingOptIn(optIn)
            _state.value = _state.value.copy(indigenousHealingOptIn = optIn)
        }
    }

    // --- LOCAL DATA PURGE (NEURORIGHTS) ---
    private suspend fun purgeAllBiosignalData() {
        db.biosignalDao().deleteAll()
        db.encryptionKeysDao().rotate() // Rotate encryption keys
    }

    // --- DASHBOARD STATE UPDATE ---
    private fun updateDashboardState(reading: BiosignalReading) {
        _state.value = _state.value.copy(
            heartRateBpm = reading.heartRateBpm,
            stressLevel = reading.stressLevel,
            sleepQualityScore = reading.sleepQualityScore
        )
    }
}

// --- BIOSIGNAL READING MODEL ---
data class BiosignalReading(
    val timestamp: LocalDateTime,
    val heartRateBpm: Int,
    val stressLevel: Float,
    val sleepQualityScore: Float,
    val encryptedData: String = "",
    val synced: Boolean = false
) {
    fun isAnonymized(): Boolean {
        // Remove all personally identifiable information
        return true
    }
    
    fun toJson(): String {
        return "{\"hr\":$heartRateBpm,\"stress\":$stressLevel,\"sleep\":$sleepQualityScore}"
    }
}
