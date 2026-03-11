package aletheion.vm.great_functions

import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

data class GreatFunctionId(val value: String)
data class VmInvocationId(val value: Long)
data class VmTenantId(val value: String)
data class VmLocaleId(val value: String)
data class VmNodeId(val value: String)

enum class VmExecutionPhase {
    BOOTSTRAP,
    WARMUP,
    ACTIVE,
    DEGRADED,
    OFFLINE_BUFFERED,
    SHUTDOWN
}

enum class VmRiskLevel {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL
}

enum class VmInterlockState {
    OPEN,
    GUARDED,
    LOCKED
}

data class VmSecurityProfile(
    val risk: VmRiskLevel,
    val maxOfflineMs: Long,
    val maxChainDepth: Int,
    val allowDynamicEval: Boolean
)

data class VmLineQuality(
    val lineNumber: Int,
    val entropyScore: Int,
    val branchingScore: Int,
    val tokenDensityScore: Int,
    val blacklistHits: Int,
    val qualityGrade: Int
)

data class VmPerLineReport(
    val filePath: String,
    val lines: List<VmLineQuality>,
    val rejected: Boolean,
    val rejectionReason: String?
)

data class VmLanguageTag(
    val name: String,
    val versionHint: String,
    val dialect: String
)

data class VmOfflineWindow(
    val allowedMs: Long,
    val softLimitMs: Long,
    val hardLimitMs: Long
)

data class VmLatencyBudget(
    val hardLimitMs: Long,
    val softLimitMs: Long,
    val targetMs: Long
)

data class VmTraceToken(
    val tenantId: VmTenantId,
    val nodeId: VmNodeId,
    val invocationId: VmInvocationId,
    val greatFunctionId: GreatFunctionId,
    val languageTag: VmLanguageTag,
    val issuedAtMs: Long
)

data class CybercoreBrainChannelId(val value: String)

data class CybercoreBrainHandshake(
    val apiVersion: String,
    val tenantId: VmTenantId,
    val nodeId: VmNodeId,
    val deviceSignature: String,
    val localeId: VmLocaleId,
    val phase: VmExecutionPhase,
    val offlineWindow: VmOfflineWindow
)

enum class CybercoreBrainMode {
    STREAM,
    SNAPSHOT,
    COMMAND
}

data class CybercoreBrainRequest(
    val channelId: CybercoreBrainChannelId,
    val mode: CybercoreBrainMode,
    val localeId: VmLocaleId,
    val payloadBytes: ByteArray,
    val traceToken: VmTraceToken,
    val softTimeoutMs: Long,
    val hardTimeoutMs: Long
)

data class CybercoreBrainResponse(
    val channelId: CybercoreBrainChannelId,
    val statusCode: Int,
    val payloadBytes: ByteArray,
    val softDeadlineHit: Boolean,
    val hardDeadlineHit: Boolean
)

data class VmOfflineEnvelope(
    val traceToken: VmTraceToken,
    val request: CybercoreBrainRequest,
    val createdAtMs: Long,
    val expiresAtMs: Long,
    val retryCount: Int
)

interface CybercoreBrainClient {
    fun performHandshake(handshake: CybercoreBrainHandshake): Boolean
    fun invoke(request: CybercoreBrainRequest): CybercoreBrainResponse
}

interface GreatFunctionKotlinBinding {
    val id: GreatFunctionId
    val languageTag: VmLanguageTag
    val latencyBudget: VmLatencyBudget
    val securityProfile: VmSecurityProfile

    fun verifyPerLineQuality(source: String): VmPerLineReport
    fun bindToCybercoreBrain(client: CybercoreBrainClient)
    fun execute(
        traceToken: VmTraceToken,
        mode: CybercoreBrainMode,
        payload: ByteArray
    ): CybercoreBrainResponse
}

object VmTraceSequencer {
    private val counter = AtomicLong(121L)
    fun next(tenantId: VmTenantId, nodeId: VmNodeId, greatFunctionId: GreatFunctionId, languageTag: VmLanguageTag): VmTraceToken {
        val id = VmInvocationId(counter.incrementAndGet())
        val now = System.currentTimeMillis()
        return VmTraceToken(
            tenantId = tenantId,
            nodeId = nodeId,
            invocationId = id,
            greatFunctionId = greatFunctionId,
            languageTag = languageTag,
            issuedAtMs = now
        )
    }
}

object VmBlacklistRegistry {
    private val blockedSequences = listOf(
        "SHA3_512",
        "SHA3-512",
        "sha3_512",
        "sha3-512",
        "Sha3_512",
        "Sha3-512"
    )

    private val blockedTokens = listOf(
        "SHA3",
        "sha3",
        "Keccak",
        "KECCAK",
        "ripemd160",
        "RIPEMD160",
        "argon",
        "ARGON"
    )

    fun hasBlacklistedContent(line: String): Boolean {
        val candidate = line.trim()
        if (candidate.isEmpty()) return false
        blockedSequences.forEach { seq ->
            if (candidate.contains(seq)) return true
        }
        val parts = candidate.split(' ', '\t', '(', ')', '{', '}', ',', ';', '.', ':')
        blockedTokens.forEach { token ->
            if (parts.any { it == token }) return true
        }
        return false
    }
}

object VmLineQualityAssessor {
    fun assess(filePath: String, source: String): VmPerLineReport {
        val lines = source.split('\n')
        val results = ArrayList<VmLineQuality>(lines.size)
        var anyRejected = false
        var rejectionReason: String? = null

        lines.forEachIndexed { idx, raw ->
            val lineNumber = idx + 1
            val trimmed = raw.trim()
            val tokens = if (trimmed.isEmpty()) emptyList() else trimmed.split(' ', '\t')
            val tokenCount = tokens.size
            val entropyScore = computeEntropyScore(trimmed)
            val branchingScore = computeBranchingScore(trimmed)
            val tokenDensityScore = computeTokenDensityScore(trimmed, tokenCount)
            val blacklistHits = if (VmBlacklistRegistry.hasBlacklistedContent(trimmed)) 1 else 0
            val qualityGrade = computeQualityGrade(entropyScore, branchingScore, tokenDensityScore, blacklistHits)

            if (blacklistHits > 0 && !anyRejected) {
                anyRejected = true
                rejectionReason = "Blacklisted token in line $lineNumber"
            }

            results.add(
                VmLineQuality(
                    lineNumber = lineNumber,
                    entropyScore = entropyScore,
                    branchingScore = branchingScore,
                    tokenDensityScore = tokenDensityScore,
                    blacklistHits = blacklistHits,
                    qualityGrade = qualityGrade
                )
            )
        }

        return VmPerLineReport(
            filePath = filePath,
            lines = results,
            rejected = anyRejected,
            rejectionReason = rejectionReason
        )
    }

    private fun computeEntropyScore(line: String): Int {
        if (line.isEmpty()) return 0
        val uniqueChars = line.toSet().size
        val length = line.length
        val baseScore = uniqueChars * 3
        val lengthPenalty = if (length > 120) 10 else if (length > 80) 5 else 0
        val score = baseScore - lengthPenalty
        return score.coerceIn(0, 100)
    }

    private fun computeBranchingScore(line: String): Int {
        if (line.isEmpty()) return 0
        var score = 0
        if (line.contains("if ") || line.contains(" if(")) score += 15
        if (line.contains("when ")) score += 12
        if (line.contains("for ") || line.contains("while ")) score += 10
        if (line.contains("&&") || line.contains("||")) score += 8
        return score.coerceIn(0, 100)
    }

    private fun computeTokenDensityScore(line: String, tokenCount: Int): Int {
        if (line.isEmpty()) return 0
        val length = line.length
        if (tokenCount == 0) return 0
        val density = tokenCount.toDouble() / length.toDouble()
        val scaled = (density * 120.0).toInt()
        return scaled.coerceIn(0, 100)
    }

    private fun computeQualityGrade(
        entropyScore: Int,
        branchingScore: Int,
        tokenDensityScore: Int,
        blacklistHits: Int
    ): Int {
        if (blacklistHits > 0) return 0
        val base = (entropyScore * 2 + branchingScore + tokenDensityScore) / 4
        return base.coerceIn(0, 100)
    }
}

class CybercoreBrainGreatFunction121(
    override val id: GreatFunctionId = GreatFunctionId("GF_0121_KOTLIN_CYBERCORE_BRAIN"),
    override val languageTag: VmLanguageTag = VmLanguageTag(
        name = "Kotlin",
        versionHint = "1.9",
        dialect = "Android"
    ),
    override val latencyBudget: VmLatencyBudget = VmLatencyBudget(
        hardLimitMs = 50L,
        softLimitMs = 24L,
        targetMs = 12L
    ),
    override val securityProfile: VmSecurityProfile = VmSecurityProfile(
        risk = VmRiskLevel.LOW,
        maxOfflineMs = 120000L,
        maxChainDepth = 5,
        allowDynamicEval = false
    )
) : GreatFunctionKotlinBinding {

    @Volatile
    private var cyberClient: CybercoreBrainClient? = null

    private val offlineBuffer = ConcurrentHashMap<Long, VmOfflineEnvelope>()

    override fun verifyPerLineQuality(source: String): VmPerLineReport {
        return VmLineQualityAssessor.assess(
            filePath = "core/vm/interops/kotlin/android/src/main/kotlin/aletheion/vm/great_functions/CybercoreBrainInterop121.kt",
            source = source
        )
    }

    override fun bindToCybercoreBrain(client: CybercoreBrainClient) {
        val handshake = CybercoreBrainHandshake(
            apiVersion = "1.0.0",
            tenantId = VmTenantId("default"),
            nodeId = VmNodeId("android-node"),
            deviceSignature = "KOTLIN_ANDROID_DEVICE",
            localeId = VmLocaleId("en_US"),
            phase = VmExecutionPhase.ACTIVE,
            offlineWindow = VmOfflineWindow(
                allowedMs = securityProfile.maxOfflineMs,
                softLimitMs = securityProfile.maxOfflineMs / 2,
                hardLimitMs = securityProfile.maxOfflineMs
            )
        )
        if (client.performHandshake(handshake)) {
            cyberClient = client
        }
    }

    override fun execute(
        traceToken: VmTraceToken,
        mode: CybercoreBrainMode,
        payload: ByteArray
    ): CybercoreBrainResponse {
        val clientRef = cyberClient
        val softTimeout = latencyBudget.softLimitMs
        val hardTimeout = latencyBudget.hardLimitMs

        if (clientRef == null) {
            bufferOffline(traceToken, mode, payload, softTimeout, hardTimeout)
            return CybercoreBrainResponse(
                channelId = CybercoreBrainChannelId("offline-buffered"),
                statusCode = 202,
                payloadBytes = ByteArray(0),
                softDeadlineHit = false,
                hardDeadlineHit = false
            )
        }

        val request = CybercoreBrainRequest(
            channelId = CybercoreBrainChannelId("primary-cybercore"),
            mode = mode,
            localeId = traceToken.languageTag.versionHint.let { VmLocaleId("en_US") },
            payloadBytes = payload,
            traceToken = traceToken,
            softTimeoutMs = softTimeout,
            hardTimeoutMs = hardTimeout
        )

        val start = System.nanoTime()
        val response = clientRef.invoke(request)
        val elapsedMs = (System.nanoTime() - start) / 1_000_000L

        val softHit = elapsedMs > softTimeout
        val hardHit = elapsedMs > hardTimeout

        return CybercoreBrainResponse(
            channelId = response.channelId,
            statusCode = response.statusCode,
            payloadBytes = response.payloadBytes,
            softDeadlineHit = softHit || response.softDeadlineHit,
            hardDeadlineHit = hardHit || response.hardDeadlineHit
        )
    }

    private fun bufferOffline(
        traceToken: VmTraceToken,
        mode: CybercoreBrainMode,
        payload: ByteArray,
        softTimeout: Long,
        hardTimeout: Long
    ) {
        val now = System.currentTimeMillis()
        val envelope = VmOfflineEnvelope(
            traceToken = traceToken,
            request = CybercoreBrainRequest(
                channelId = CybercoreBrainChannelId("offline-buffered"),
                mode = mode,
                localeId = VmLocaleId("en_US"),
                payloadBytes = payload,
                traceToken = traceToken,
                softTimeoutMs = softTimeout,
                hardTimeoutMs = hardTimeout
            ),
            createdAtMs = now,
            expiresAtMs = now + securityProfile.maxOfflineMs,
            retryCount = 0
        )
        offlineBuffer[traceToken.invocationId.value] = envelope
    }

    fun flushOfflineQueue() {
        val clientRef = cyberClient ?: return
        val now = System.currentTimeMillis()
        val iterator = offlineBuffer.entries.iterator()
        while (iterator.hasNext()) {
            val entry = iterator.next()
            val envelope = entry.value
            if (envelope.expiresAtMs <= now) {
                iterator.remove()
                continue
            }
            val response = clientRef.invoke(envelope.request)
            if (response.statusCode in 200..299) {
                iterator.remove()
            } else {
                val updated = envelope.copy(
                    retryCount = envelope.retryCount + 1
                )
                offlineBuffer[entry.key] = updated
            }
        }
    }
}
