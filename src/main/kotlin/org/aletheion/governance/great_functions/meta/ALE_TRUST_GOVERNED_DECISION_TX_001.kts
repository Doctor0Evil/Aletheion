package org.aletheion.governance.great_functions.meta

import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.security.MessageDigest
import java.util.UUID

@Serializable
data class GovernedDecisionEnvelope(
    val birthSignId: String,
    val workflowId: String,
    val alnNorms: List<String>,
    val inputsHash: String,
    val outputsHash: String,
    val decisionOutcome: DecisionOutcome,
    val reasoning: String? = null,
    val signature: String
)

enum class DecisionOutcome {
    PERMIT,
    DEFER,
    DENY
}

object ALEGovernanceSchemaValidator {

    private val ALLOWED_DECISIONS = setOf("PERMIT", "DEFER", "DENY")

    fun validateEnvelope(envelope: GovernedDecisionEnvelope): Boolean {
        if (!ALLOWED_DECISIONS.contains(envelope.decisionOutcome.name)) return false
        if (!isUUID(envelope.birthSignId) || !isUUID(envelope.workflowId)) return false
        if (envelope.alnNorms.isEmpty()) return false
        if (!isValidHash(envelope.inputsHash) || !isValidHash(envelope.outputsHash)) return false
        if (!isValidSignature(envelope.signature)) return false
        return true
    }

    private fun isUUID(input: String): Boolean =
        runCatching { UUID.fromString(input) }.isSuccess

    private fun isValidHash(hash: String): Boolean =
        hash.matches(Regex("^[a-fA-F0-9]{64,}$"))

    private fun isValidSignature(sig: String): Boolean =
        sig.length in 88..128

    fun serialize(envelope: GovernedDecisionEnvelope): String =
        Json.encodeToString(envelope)
}

object ALNHashingEngine {
    fun computeSHA3Hash(data: String): String {
        val digest = MessageDigest.getInstance("SHA3-512")
        val hashBytes = digest.digest(data.toByteArray(Charsets.UTF_8))
        return hashBytes.joinToString("") { "%02x".format(it) }
    }
}

object GreatFunctionRunner {

    fun performGovernedDecision(
        birthSignId: String,
        workflowId: String,
        alnNorms: List<String>,
        inputs: String,
        outputs: String,
        outcome: DecisionOutcome,
        reasoning: String? = null
    ): GovernedDecisionEnvelope {
        val inputsHash = ALNHashingEngine.computeSHA3Hash(inputs)
        val outputsHash = ALNHashingEngine.computeSHA3Hash(outputs)
        val combined = "$birthSignId|$workflowId|$inputsHash|$outputsHash|${outcome.name}"
        val signature = signEnvelope(combined)

        val envelope = GovernedDecisionEnvelope(
            birthSignId = birthSignId,
            workflowId = workflowId,
            alnNorms = alnNorms,
            inputsHash = inputsHash,
            outputsHash = outputsHash,
            decisionOutcome = outcome,
            reasoning = reasoning,
            signature = signature
        )

        if (!ALEGovernanceSchemaValidator.validateEnvelope(envelope))
            throw IllegalStateException("Non-compliant envelope detected")
        return envelope
    }

    private fun signEnvelope(data: String): String {
        val digest = MessageDigest.getInstance("SHA3-512")
        val signed = digest.digest(data.toByteArray(Charsets.UTF_8))
        return signed.joinToString("") { "%02x".format(it) }
    }
}

fun main() {
    val envelope = GreatFunctionRunner.performGovernedDecision(
        birthSignId = UUID.randomUUID().toString(),
        workflowId = UUID.randomUUID().toString(),
        alnNorms = listOf("RIGHTS_ATOM_LAND_ACCESS", "BIOTIC_TREATY_2026_01"),
        inputs = "{\"sensor\":\"water-level\",\"value\":1.2}",
        outputs = "{\"valveState\":\"PARTIAL_OPEN\"}",
        outcome = DecisionOutcome.PERMIT,
        reasoning = "Preflight cleared all ALN norms."
    )

    println(ALEGovernanceSchemaValidator.serialize(envelope))
}
