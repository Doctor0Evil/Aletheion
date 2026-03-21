// aletheion/climate/citizen/interface/src/ConsentActivity.kt
// Copyright (c) 2026 Aletheion City-OS. All Rights Reserved.
// License: BioticTreaty-Compliant AGPL-3.0-or-later with Indigenous-Rights-Clause
// Purpose: Citizen Interface for Phoenix Desert Grid (Interface → Consent → Log)
// Constraints: No Blacklisted Crypto (SHA/Blake/Argon), No Rollbacks, Offline-First, Neurorights-Hardened
// Status: ACTIVE | VERSION: 1.0.0-E-PHX | TERRITORY: Akimel O'odham & Piipaash Traditional Lands
// Identity: Augmented-Citizen Organically-Integrated (BI-Bound)

package aletheion.climate.citizen.interface

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import aletheion.core.security.PQSecure // Abstracted Post-Quantum Crypto
import aletheion.climate.edge.SensorFusionBridge // Rust FFI Bridge
import aletheion.climate.rights.NeurorightsGuard // BCI/Stress Monitor Bridge
import aletheion.climate.rights.IndigenousTerritory // Land Acknowledgment Bridge

// ============================================================================
// DATA MODELS (High-Density, Immutable)
// ============================================================================
// Per Rule (L): High-density codes, syntax_ladders, cross-language program-ops.
// Per Rule (I): DID-Bound brain-identity (BI) and biosignal-collector respect.

data class EnvironmentalState(
    val sectorId: String,
    val airTempF: Float,
    val pm10: Float,
    val rainfallIn: Float,
    val humidityPct: Float,
    val timestampUtc: Long,
    val territoryHash: Long
)

data class ConsentRequest(
    val requestId: Long,
    val capability: String, // e.g., "CoolPavement", "DustMitigation"
    val impactDescription: String,
    val consentClass: String, // "Visual", "Cognitive", "Haptic"
    val rightsGuards: List<String>,
    val expiryUtc: Long,
    val pqSignature: ByteArray // Abstracted PQ Signature
)

data class CitizenState(
    val envState: EnvironmentalState?,
    val pendingConsent: ConsentRequest?,
    val cognitiveLoadScore: Float, // 0.0 (Low) to 1.0 (Critical)
    val territoryAcknowledged: Boolean,
    val isOffline: Boolean
)

// ============================================================================
// VIEWMODEL (Logic, State Management, Treaty Enforcement)
// ============================================================================
// Per Rule (R): No rollbacks, no digital twins, no fictional content.
// Per Rule (I): Speak on my-behalf... as-an organically-integrated augmented-citizen.

class ConsentViewModel : ViewModel() {
    private val _state = MutableStateFlow(CitizenState(
        envState = null, pendingConsent = null, cognitiveLoadScore = 0.0f,
        territoryAcknowledged = false, isOffline = false
    ))
    val state: StateFlow<CitizenState> = _state

    private val sensorBridge = SensorFusionBridge() // Rust FFI
    private val neuroGuard = NeurorightsGuard() // BCI FFI
    private val territoryBridge = IndigenousTerritory() // Land Acknowledgment FFI
    private val pqSecure = PQSecure() // Abstracted Crypto

    init {
        viewModelScope.launch {
            while (true) {
                val env = sensorBridge.getFusedContext() // Rust Sensor Fusion
                val stress = neuroGuard.getCognitiveLoad() // BCI Stress Check
                val landStatus = territoryBridge.getConsentStatus() // Treaty Check
                _state.value = _state.value.copy(
                    envState = env,
                    cognitiveLoadScore = stress,
                    territoryAcknowledged = landStatus == "VERIFIED_CONSENT",
                    isOffline = !sensorBridge.isNetworkAvailable()
                )
                kotlinx.coroutines.delay(1000) // 1Hz Update
            }
        }
    }

    // Neurorights Protection: Block complex consent if stress is high
    fun canShowConsent(request: ConsentRequest): Boolean {
        if (request.consentClass == "Cognitive" && _state.value.cognitiveLoadScore > 0.7f) {
            logNeurorightsViolation("Cognitive_Overload_Prevented", request.requestId)
            return false
        }
        if (!_state.value.territoryAcknowledged) {
            logTreatyViolation("Land_Acknowledgment_Missing", request.requestId)
            return false
        }
        return true
    }

    fun grantConsent(request: ConsentRequest) {
        val signature = pqSecure.signConsent(request.requestId, "GRANT")
        sensorBridge.submitConsent(request, signature)
        _state.value = _state.value.copy(pendingConsent = null)
    }

    fun denyConsent(request: ConsentRequest) {
        val signature = pqSecure.signConsent(request.requestId, "DENY")
        sensorBridge.submitConsent(request, signature)
        _state.value = _state.value.copy(pendingConsent = null)
    }

    fun acknowledgeTerritory() {
        territoryBridge.submitAcknowledgment("Akimel O'odham", "Piipaash")
        _state.value = _state.value.copy(territoryAcknowledged = true)
    }

    private fun logNeurorightsViolation(reason: String, id: Long) {
        sensorBridge.logAudit("Neurorights_Violation", reason, id)
    }

    private fun logTreatyViolation(reason: String, id: Long) {
        sensorBridge.logAudit("Treaty_Violation", reason, id)
    }
}

// ============================================================================
// UI COMPOSABLES (Compose UI, Accessibility, Density)
// ============================================================================
// Per Rule (L): Supported-language set: ALN, Lua, Rust, Javascript, Kotlin/Android, C++
// Per Rule (P): Node-Placement opportunities where civil-disturbance will-not create unrest.

@Composable
fun AletheionClimateInterface(viewModel: ConsentViewModel) {
    val state by viewModel.state.collectAsState()
    Box(modifier = Modifier.fillMaxSize().background(Color(0xFF121212))) {
        LazyColumn(modifier = Modifier.fillMaxSize(), contentPadding = PaddingValues(16.dp)) {
            // Header: Territory Acknowledgment (Hard Constraint)
            item {
                TerritoryBanner(
                    acknowledged = state.territoryAcknowledged,
                    onAcknowledge = { viewModel.acknowledgeTerritory() }
                )
            }
            // Environmental Data (Real-Time from Rust)
            item {
                EnvironmentalDashboard(envState = state.envState, isOffline = state.isOffline)
            }
            // Neurorights Status (BCI Integration)
            item {
                NeurorightsIndicator(loadScore = state.cognitiveLoadScore)
            }
            // Consent Requests (Governed Actions)
            state.pendingConsent?.let { request ->
                item {
                    if (viewModel.canShowConsent(request)) {
                        ConsentDialog(request = request, onGrant = {
                            viewModel.grantConsent(request)
                        }, onDeny = {
                            viewModel.denyConsent(request)
                        })
                    } else {
                        Text(text = "Consent Hidden: Neurorights/Treaty Protection Active", color = Color.Red)
                    }
                }
            }
        }
    }
}

@Composable
fun TerritoryBanner(acknowledged: Boolean, onAcknowledge: () -> Unit) {
    Card(modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp), colors = CardDefaults.cardColors(containerColor = Color(0xFF1E1E1E))) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(text = "Territory Acknowledgment", style = MaterialTheme.typography.titleMedium, color = Color(0xFF00E5FF))
            Text(text = "You are on the traditional lands of the Akimel O'odham and Piipaash nations.", style = MaterialTheme.typography.bodySmall)
            if (!acknowledged) {
                Button(onClick = onAcknowledge, modifier = Modifier.align(Alignment.End)) {
                    Text(text = "Acknowledge & Continue")
                }
            } else {
                Text(text = "Status: Verified Consent", color = Color.Green, style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
fun EnvironmentalDashboard(envState: EnvironmentalState?, isOffline: Boolean) {
    Card(modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp), colors = CardDefaults.cardColors(containerColor = Color(0xFF1E1E1E))) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(text = "Phoenix Desert Grid | Sector: ${envState?.sectorId ?: "UNKNOWN"}", style = MaterialTheme.typography.titleMedium)
            if (isOffline) Text(text = "OFFLINE MODE (Local Cache Active)", color = Color.Yellow)
            Row(horizontalArrangement = Arrangement.SpaceBetween) {
                Column { Text("Temp: ${envState?.airTempF ?: 0}°F"); Text("PM10: ${envState?.pm10 ?: 0} µg/m³") }
                Column { Text("Rain: ${envState?.rainfallIn ?: 0} in"); Text("Humidity: ${envState?.humidityPct ?: 0}%") }
            }
        }
    }
}

@Composable
fun NeurorightsIndicator(loadScore: Float) {
    Card(modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp), colors = CardDefaults.cardColors(containerColor = Color(0xFF1E1E1E))) {
        Row(modifier = Modifier.padding(16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text(text = "Neurorights Status", style = MaterialTheme.typography.titleSmall)
            LinearProgressIndicator(progress = loadScore, modifier = Modifier.width(100.dp), color = if (loadScore > 0.7f) Color.Red else Color.Green)
            Text(text = "${(loadScore * 100).toInt()}% Load", style = MaterialTheme.typography.bodySmall)
        }
    }
}

@Composable
fun ConsentDialog(request: ConsentRequest, onGrant: () -> Unit, onDeny: () -> Unit) {
    Card(modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp), colors = CardDefaults.cardColors(containerColor = Color(0xFF2C2C2C))) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(text = "Action Consent Required", style = MaterialTheme.typography.titleMedium, color = Color(0xFF00E5FF))
            Text(text = "Capability: ${request.capability}", style = MaterialTheme.typography.bodyMedium)
            Text(text = "Impact: ${request.impactDescription}", style = MaterialTheme.typography.bodySmall)
            Text(text = "Rights Guards: ${request.rightsGuards.joinToString()}", style = MaterialTheme.typography.bodySmall)
            Row(horizontalArrangement = Arrangement.End) {
                TextButton(onClick = onDeny) { Text("Deny") }
                Button(onClick = onGrant) { Text("Grant") }
            }
        }
    }
}

// ============================================================================
// ACTIVITY ENTRY POINT (Android Lifecycle)
// ============================================================================
// Per Rule (R): Codes must-be in the supported-languages, contain a filename, and an exact-destination.
// Per Rule (L): Compatibility: Github, and adjustable to any city-builder, or deployment-guide.

class ConsentActivity : ComponentActivity() {
    private val viewModel: ConsentViewModel by viewModels()
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                AletheionClimateInterface(viewModel)
            }
        }
    }
    override fun onDestroy() {
        // Forward-Only Cleanup: No Rollback of State
        super.onDestroy()
    }
}
