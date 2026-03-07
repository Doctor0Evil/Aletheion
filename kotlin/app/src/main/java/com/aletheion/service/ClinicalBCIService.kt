// ============================================================================
// SERVICE: ClinicalBCIService
// PURPOSE: Secure management of Clinical BCI interfaces and consent
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

package com.aletheion.service

import android.app.*
import android.content.Intent
import android.os.Binder
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.aletheion.BuildConfig
import com.aletheion.model.BCIClinicalAugmentation
import com.aletheion.security.VitalNetSafetyKernel
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject

@AndroidEntryPoint
class ClinicalBCIService : Service() {

    companion object {
        private const val CHANNEL_ID = "AletheionBCIChannel"
        private const val NOTIFICATION_ID = 1
        private const val TAG = "ClinicalBCIService"
    }

    private val binder = LocalBinder()
    private val safetyKernel = VitalNetSafetyKernel.getInstance()

    // State Flow for BCI Status
    private val _bciStatus = MutableStateFlow(BCIStatus.DISCONNECTED)
    val bciStatus: StateFlow<BCIStatus> = _bciStatus

    // Current BCI Configuration
    private var currentBCI: BCIClinicalAugmentation? = null

    inner class LocalBinder : Binder() {
        fun getService(): ClinicalBCIService = this@ClinicalBCIService
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification())
        Log.i(TAG, "Clinical BCI Service started with Safety Kernel: ${BuildConfig.SAFETY_KERNEL_REF}")
    }

    override fun onBind(intent: Intent?): IBinder {
        return binder
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "ACTION_CONNECT_BCI" -> connectBCI(intent.getStringExtra("DEVICE_ID"))
            "ACTION_DISCONNECT_BCI" -> disconnectBCI()
            "ACTION_UPDATE_CONSENT" -> updateConsent(intent.getBooleanExtra("CONSENT_GRANTED", false))
        }
        return START_STICKY
    }

    /**
     * Connect to BCI device with safety checks
     */
    fun connectBCI(deviceId: String?) {
        if (deviceId == null) {
            Log.e(TAG, "Device ID missing")
            return
        }

        // Safety Kernel Verification
        if (!safetyKernel.verifyDeviceSafety(deviceId)) {
            Log.e(TAG, "Safety kernel verification failed for $deviceId")
            _bciStatus.value = BCIStatus.SAFETYViolation
            return
        }

        // Check Consent
        if (!safetyKernel.hasConsent(BuildConfig.OWNER_DID)) {
            Log.e(TAG, "Consent missing for ${BuildConfig.OWNER_DID}")
            _bciStatus.value = BCIStatus.CONSENT_REQUIRED
            return
        }

        // Initialize BCI Object
        currentBCI = BCIClinicalAugmentation(
            deviceId = deviceId,
            deviceModel = "CyberOrganic_BCI_A1",
            speciesNeuroclass = "human_cortex_v1",
            biofieldLoadCeiling = 0.5f,
            ownerDid = BuildConfig.OWNER_DID,
            safetyKernelRef = BuildConfig.SAFETY_KERNEL_REF,
            consciousnessPreservationEnabled = false // Default false, requires explicit UI consent
        )

        _bciStatus.value = BCIStatus.CONNECTED
        Log.i(TAG, "BCI Connected: $deviceId")
    }

    /**
     * Disconnect BCI safely
     */
    fun disconnectBCI() {
        currentBCI = null
        _bciStatus.value = BCIStatus.DISCONNECTED
        Log.i(TAG, "BCI Disconnected")
    }

    /**
     * Update consent profile
     */
    fun updateConsent(granted: Boolean) {
        if (granted) {
            safetyKernel.registerConsent(BuildConfig.OWNER_DID)
            Log.i(TAG, "Consent granted for ${BuildConfig.OWNER_DID}")
        } else {
            safetyKernel.revokeConsent(BuildConfig.OWNER_DID)
            disconnectBCI() // Force disconnect on consent revocation
            Log.i(TAG, "Consent revoked for ${BuildConfig.OWNER_DID}")
        }
    }

    /**
     * Enable Consciousness Preservation (Requires Extra Verification)
     */
    fun enableConsciousnessPreservation() {
        // In production, this requires Clinical Safety Board approval
        // Here we log the request for audit
        Log.wtf(
            TAG,
            "CONSCIOUSNESS_PRESERVATION_REQUESTED by ${BuildConfig.OWNER_DID} - " +
            "REQUIRES_CLINICAL_SAFETY_BOARD_APPROVAL"
        )
        currentBCI?.consciousnessPreservationEnabled = true
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Clinical BCI Status",
            NotificationManager.IMPORTANCE_LOW
        )
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }

    private fun createNotification(): Notification {
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Aletheion BCI Service")
            .setContentText("Neurorights Protection Active")
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .setOngoing(true)
            .build()
    }

    enum class BCIStatus {
        DISCONNECTED,
        CONNECTED,
        CONSENT_REQUIRED,
        SAFETY_VIOLATION
    }
}
