// ============================================================================
// ACTIVITY: EvidenceWalletActivity
// PURPOSE: Display personal evidence wallet and health/eco improvements
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

package com.aletheion.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aletheion.viewmodel.EvidenceWalletViewModel
import com.aletheion.model.EvidenceRecord
import dagger.hilt.android.AndroidEntryPoint

@AndroidEntryPoint
class EvidenceWalletActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            MaterialTheme(
                colorScheme = lightColorScheme(
                    primary = Color(0xFF00695C), // Teal for eco/health
                    secondary = Color(0xFF004D40),
                    tertiary = Color(0xFF80CBC4)
                )
            ) {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    EvidenceWalletScreen()
                }
            }
        }
    }
}

@Composable
fun EvidenceWalletScreen(
    viewModel: EvidenceWalletViewModel = hiltViewModel()
) {
    val walletState by viewModel.walletState.collectAsState()
    val evidenceRecords by viewModel.evidenceRecords.collectAsState()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Header
        Text(
            text = "Aletheion Evidence Wallet",
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(bottom = 16.dp)
        )

        // Wallet Status Card
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 16.dp),
            elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "Wallet ID: ${walletState.walletId.take(16)}...",
                    fontSize = 14.sp,
                    color = Color.Gray
                )
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column {
                        Text(
                            text = "Completeness Score",
                            fontSize = 12.sp,
                            color = Color.Gray
                        )
                        Text(
                            text = "${(walletState.completenessScore * 100).toInt()}%",
                            fontSize = 20.sp,
                            fontWeight = FontWeight.Bold,
                            color = if (walletState.completenessScore >= 0.86) Color.Green else Color.Red
                        )
                    }
                    Column {
                        Text(
                            text = "Status",
                            fontSize = 12.sp,
                            color = Color.Gray
                        )
                        Text(
                            text = walletState.status.uppercase(),
                            fontSize = 20.sp,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
                Spacer(modifier = Modifier.height(8.dp))
                if (walletState.consciousnessPreservationEnabled) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(8.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = "⚠ Consciousness Preservation Active",
                            color = Color(0xFF673AB7), // Deep Purple
                            fontWeight = FontWeight.Bold,
                            fontSize = 14.sp
                        )
                    }
                }
            }
        }

        // Equal Protection Notice
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 16.dp),
            colors = CardDefaults.cardColors(containerColor = Color(0xFFE0F2F1))
        ) {
            Column(
                modifier = Modifier.padding(12.dp)
            ) {
                Text(
                    text = "🛡 Neurorights Protection Active",
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp
                )
                Text(
                    text = "Equal protection regardless of augmentation status. " +
                           "Organic BCI interfaces governed by AugmentedHumanRights:v1.",
                    fontSize = 12.sp,
                    color = Color.DarkGray
                )
            }
        }

        // Evidence Records List
        Text(
            text = "Evidence Records",
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier
                .fillMaxWidth()
                .padding(bottom = 8.dp)
        )

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(evidenceRecords) { record ->
                EvidenceRecordCard(record)
            }
        }
    }
}

@Composable
fun EvidenceRecordCard(record: EvidenceRecord) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(
            modifier = Modifier.padding(12.dp)
        ) {
            Row(
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = record.type.uppercase(),
                    fontWeight = FontWeight.Bold,
                    color = if (record.type == "health") Color(0xFF00695C) else Color(0xFF0277BD)
                )
                Text(
                    text = record.timestamp,
                    fontSize = 12.sp,
                    color = Color.Gray
                )
            }
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = record.metric,
                fontSize = 14.sp,
                fontWeight = FontWeight.Medium
            )
            Text(
                text = "Delta: ${record.delta} ${record.unit}",
                fontSize = 12.sp,
                color = Color.DarkGray
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = "Corridor: ${record.corridor}",
                fontSize = 12.sp,
                color = Color.Gray
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = "ROW Ref: ${record.rowRef.take(16)}...",
                fontSize = 10.sp,
                color = Color.LightGray,
                fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
            )
        }
    }
}
