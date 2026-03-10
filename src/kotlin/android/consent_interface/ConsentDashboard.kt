// Profile: Offline-First, Accessibility (WCAG 2.2 AAA), Somatic-Safe

package com.aletheion.citizen.consent

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import aletheion.security.DIDWallet
import aletheion.governance.ConsentEnvelope

// Somatic Load Budget: Max 3 notifications/day regarding consent changes
const val MAX_DAILY_CONSENT_ALERTS = 3 

@Composable
fun ConsentDashboard(envelopes: List<ConsentEnvelope>, wallet: DIDWallet) {
    // Visual Hierarchy: Active Consents (Green), Revoked (Red), Expiring (Amber)
    // No Dark Patterns: "Revoke" button must be same size/color prominence as "Grant"
    
    val activeCount = envelopes.count { it.revocation_status == Active }
    val riskCount = envelopes.count { it.inference_allowed == true } // Warning Indicator
    
    // Accessibility: ContentDescription for Screen Readers
    // Language: Dynamic (English, Spanish, O'odham)
    
    ConsentList(
        items = envelopes,
        onRevoke = { envelope -> 
            wallet.signRevocation(envelope) 
            // Haptic Feedback: Gentle Pulse (No Alarm)
        },
        somaticLoadMonitor = { 
            if (dailyAlertCount > MAX_DAILY_CONSENT_ALERTS) queueNotification() 
        }
    )
}

// Security: Biometric Auth Required for Revocation
// Offline: Changes sync when node connectivity restored (Conflict Resolution: Latest-Wins-Unless-Veto)
