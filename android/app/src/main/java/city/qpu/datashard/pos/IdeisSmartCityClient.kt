package city.qpu.datashard.pos

import android.hardware.biometrics.BiometricPrompt
import androidx.fragment.app.FragmentActivity

data class IdeisSessionToken(
    val sessionId: String,
    val subjectDid: String,
    val deviceId: String,
    val issuedAt: Long,
    val expiresAt: Long,
    val jurisdiction: String,
    val policyProfile: String,
    val authContext: String,
)

data class PosNonFinancialEvent(
    val eventId: String,
    val sku: String,
    val quantity: Int,
    val actorDid: String,
    val deviceId: String,
    val timestamp: Long,
    val policyProfile: String,
)

interface SovereignCoreBridge {
    fun handlePosRequest(
        token: IdeisSessionToken,
        manifestBytes: ByteArray,
        action: String,
        event: PosNonFinancialEvent,
    ): IdeisResponse
}

data class IdeisResponse(
    val code: Int,
    val message: String,
)

class IdeisSmartCityClient(
    private val bridge: SovereignCoreBridge,
) {

    fun authenticateWithBiometrics(
        activity: FragmentActivity,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val prompt = BiometricPrompt.Builder(activity)
            .setTitle("Secure POS")
            .setSubtitle("Verify to continue")
            .setNegativeButton("Cancel", activity.mainExecutor) { _, _ ->
                onError("CANCELLED")
            }
            .build()

        prompt.authenticate(object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult?) {
                onSuccess()
            }

            override fun onAuthenticationError(errorCode: Int, errString: CharSequence?) {
                onError(errString?.toString() ?: "ERROR")
            }

            override fun onAuthenticationFailed() {
                onError("FAILED")
            }
        })
    }

    fun sendPosSale(
        token: IdeisSessionToken,
        manifestBytes: ByteArray,
        event: PosNonFinancialEvent,
    ): IdeisResponse {
        return bridge.handlePosRequest(token, manifestBytes, "POS_SALE", event)
    }

    fun inventorySync(
        token: IdeisSessionToken,
        manifestBytes: ByteArray,
        event: PosNonFinancialEvent,
    ): IdeisResponse {
        return bridge.handlePosRequest(token, manifestBytes, "INVENTORY_SYNC", event)
    }
}
