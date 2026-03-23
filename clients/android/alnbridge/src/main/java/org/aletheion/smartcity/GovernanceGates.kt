
package org.aletheion.smartcity

import org.aletheion.aln.NeuroRightInference
import org.aletheion.aln.NeurorightsPolicy
import org.aletheion.aln.TreatyRegionBinding
import org.aletheion.aln.MustPassGateResult
import org.aletheion.aln.MustPassGateStatus

data class HighImpactDecisionContext(
    val objectId: String,
    val geoRegionId: String,
    val telemetryFreshSeconds: Long,
    val requiresFpic: Boolean,
    val neurorightsQueryFeatures: Set<String>,
    val neurorightsPlannedInferences: Set<String>
)

class GovernanceGates(
    private val treatyResolver: (String) -> TreatyRegionBinding?,
    private val neurorightsResolver: () -> NeurorightsPolicy?
) {
    fun evaluate(context: HighImpactDecisionContext): MustPassGateResult {
        val treaty = treatyResolver.invoke(context.geoRegionId)
            ?: return MustPassGateResult(MustPassGateStatus.BLOCKED_NO_TREATY, "No treaty binding for region")

        if (context.requiresFpic && !treaty.hasValidFpicFor(context.objectId)) {
            return MustPassGateResult(MustPassGateStatus.BLOCKED_FPIC, "FPIC not satisfied")
        }

        if (!treaty.isWithinProtectedReach(context.objectId)) {
            return MustPassGateResult(MustPassGateStatus.BLOCKED_PROTECTED_REACH, "Protected reach violation")
        }

        val policy = neurorightsResolver.invoke()
            ?: return MustPassGateResult(MustPassGateStatus.BLOCKED_NEURORIGHTS_POLICY, "Neurorights policy unavailable")

        if (!policy.allowedFeatureSpace.containsAll(context.neurorightsQueryFeatures)) {
            return MustPassGateResult(MustPassGateStatus.BLOCKED_NEURORIGHTS_FEATURES, "Requested features exceed allowed neurorights space")
        }

        if (context.neurorightsPlannedInferences.any { disallowed ->
                policy.disallowedInferenceSpace.contains(NeuroRightInference(disallowed))
            }) {
            return MustPassGateResult(MustPassGateStatus.BLOCKED_NEURORIGHTS_INFERENCES, "Planned inferences violate neurorights policy")
        }

        return MustPassGateResult(MustPassGateStatus.ALLOW, "All governance gates passed")
    }
}
