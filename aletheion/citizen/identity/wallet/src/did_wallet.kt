package aletheion.citizen.identity.wallet

import java.security.SecureRandom
import java.nio.ByteBuffer
import java.util.UUID

const val WALLET_VERSION = 20260310L
const val MAX_CREDENTIALS = 256
const val MIN_KEY_LENGTH_BYTES = 32
const val DID_METHOD_ALETHEION = "did:aletheion:"

enum class CredentialType {
    CITIZENSHIP, HEALTH_ACCESS, VOTING_RIGHTS, PROPERTY_DEED, EDUCATION_RECORD,
    EMPLOYMENT_CONTRACT, WATER_RIGHTS, ENERGY_SHARE, TRANSIT_PASS, BCI_CONSENT
}

enum class VerificationLevel {
    SELF_ASSERTED(1), WITNESS_VERIFIED(2), AUTHORITY_SIGNED(3), BIOMETRIC_BOUND(4), QUORUM_CONSENSUS(5)
    val level: Int
    constructor(level: Int) { this.level = level }
    fun meetsMinimum(minimum: VerificationLevel): Boolean = this.level >= minimum.level
}

data class CryptographicKey(
    val keyId: String,
    val publicKey: ByteArray,
    val keyType: String = "X25519",
    val createdAtNs: Long,
    val expiresAtNs: Long,
    val revoked: Boolean = false
) {
    fun isValid(nowNs: Long): Boolean = !revoked && nowNs < expiresAtNs
    fun fingerprint(): String = publicKey.sliceArray(0..7).joinToString("") { "%02x".format(it) }
}

data class VerifiableCredential(
    val credentialId: String,
    val type: CredentialType,
    val issuer: String,
    val subject: String,
    val issuedAtNs: Long,
    val expiresAtNs: Long,
    val verificationLevel: VerificationLevel,
    val claims: Map<String, String>,
    val proof: ByteArray,
    val revoked: Boolean = false
) {
    fun isValid(nowNs: Long): Boolean = !revoked && nowNs < expiresAtNs && nowNs >= issuedAtNs
    fun matchesPurpose(purpose: String): Boolean = claims.containsKey("purpose") && claims["purpose"] == purpose
}

data class PresentationRequest(
    val requestId: String,
    val verifier: String,
    val requestedCredentials: List<CredentialType>,
    val purpose: String,
    const val createdAtNs: Long,
    val expiresAtNs: Long,
    val nonce: ByteArray
) {
    fun isValid(nowNs: Long): Boolean = nowNs < expiresAtNs && nowNs >= createdAtNs
}

data class DIDDocument(
    val did: String,
    val controller: String,
    val verificationMethods: List<CryptographicKey>,
    val authentication: List<String>,
    val assertionMethods: List<String>,
    val keyAgreement: List<String>,
    val serviceEndpoints: List<ServiceEndpoint>,
    val createdAtNs: Long,
    val updatedAtNs: Long,
    val versionId: Long
) {
    data class ServiceEndpoint(
        val id: String,
        val type: String,
        val serviceEndpoint: String,
        val description: String
    )
    fun getActiveKeys(nowNs: Long): List<CryptographicKey> = verificationMethods.filter { it.isValid(nowNs) }
    fun getAuthenticationKeys(nowNs: Long): List<CryptographicKey> {
        return verificationMethods.filter { it.isValid(nowNs) && authentication.contains(it.keyId) }
    }
}

class DIDWalletManager(
    private val walletId: String,
    private val ownerDid: String,
    private val initTimestampNs: Long
) {
    private var didDocument: DIDDocument? = null
    private val credentials = mutableListOf<VerifiableCredential>()
    private val presentationHistory = mutableListOf<PresentationRecord>()
    private val auditLog = mutableListOf<WalletAuditEntry>()
    private val secureRandom = SecureRandom.getInstance("SHA1PRNG")
    
    data class PresentationRecord(
        val recordId: String,
        val requestId: String,
        val verifier: String,
        val disclosedCredentials: List<String>,
        const val timestampNs: Long,
        val purpose: String,
        val consentGiven: Boolean
    )
    
    data class WalletAuditEntry(
        val entryId: Long,
        val action: String,
        val timestampNs: Long,
        val success: Boolean,
        val details: String,
        val riskScore: Double
    )
    
    fun initializeDIDDocument(controller: String, nowNs: Long): DIDDocument {
        val initialKey = generateCryptographicKey("key-1", nowNs, nowNs + 31536000000000000L)
        val doc = DIDDocument(
            did = ownerDid,
            controller = controller,
            verificationMethods = listOf(initialKey),
            authentication = listOf("key-1"),
            assertionMethods = listOf("key-1"),
            keyAgreement = listOf("key-1"),
            serviceEndpoints = listOf(
                DIDDocument.ServiceEndpoint(
                    id = "${ownerDid}#messaging",
                    type = "DIDMessaging",
                    serviceEndpoint = "https://messaging.aletheion.city",
                    description = "Encrypted citizen messaging endpoint"
                ),
                DIDDocument.ServiceEndpoint(
                    id = "${ownerDid}#biosignal",
                    type = "BiosignalStream",
                    serviceEndpoint = "wss://biosignal.aletheion.city/stream",
                    description = "Neurorights-protected biosignal data stream"
                )
            ),
            createdAtNs = nowNs,
            updatedAtNs = nowNs,
            versionId = 1
        )
        didDocument = doc
        logAudit("DID_INITIALIZE", true, "DID document created for $ownerDid", nowNs, 0.0)
        return doc
    }
    
    fun generateCryptographicKey(keyId: String, createdAtNs: Long, expiresAtNs: Long): CryptographicKey {
        val publicKey = ByteArray(MIN_KEY_LENGTH_BYTES)
        secureRandom.nextBytes(publicKey)
        val key = CryptographicKey(keyId, publicKey, "X25519", createdAtNs, expiresAtNs, false)
        logAudit("KEY_GENERATE", true, "Key $keyId generated", createdAtNs, 0.05)
        return key
    }
    
    fun addCredential(credential: VerifiableCredential, nowNs: Long): Result<Unit> {
        if (credentials.size >= MAX_CREDENTIALS) {
            logAudit("CREDENTIAL_ADD", false, "Wallet credential limit exceeded", nowNs, 0.3)
            return Result.failure(Error("CREDENTIAL_LIMIT_EXCEEDED"))
        }
        if (!credential.isValid(nowNs)) {
            logAudit("CREDENTIAL_ADD", false, "Credential expired or not yet valid", nowNs, 0.2)
            return Result.failure(Error("CREDENTIAL_INVALID_TIME"))
        }
        credentials.add(credential)
        logAudit("CREDENTIAL_ADD", true, "Credential ${credential.credentialId} added", nowNs, 0.02)
        return Result.success(Unit)
    }
    
    fun selectCredentialsForPresentation(
        request: PresentationRequest,
        nowNs: Long,
        requireConsent: Boolean = true
    ): Result<List<VerifiableCredential>> {
        if (!request.isValid(nowNs)) {
            logAudit("PRESENTATION_SELECT", false, "Request expired or invalid", nowNs, 0.15)
            return Result.failure(Error("PRESENTATION_REQUEST_INVALID"))
        }
        val selected = credentials.filter { cred ->
            cred.isValid(nowNs) &&
            request.requestedCredentials.contains(cred.type) &&
            cred.matchesPurpose(request.purpose) &&
            !cred.revoked
        }
        if (selected.isEmpty()) {
            logAudit("PRESENTATION_SELECT", false, "No matching credentials found", nowNs, 0.1)
            return Result.failure(Error("NO_MATCHING_CREDENTIALS"))
        }
        if (requireConsent) {
            logAudit("PRESENTATION_SELECT", true, "${selected.size} credentials selected, consent required", nowNs, 0.05)
        }
        return Result.success(selected)
    }
    
    fun recordPresentation(
        requestId: String,
        verifier: String,
        disclosedCredentialIds: List<String>,
        purpose: String,
        consentGiven: Boolean,
        nowNs: Long
    ) {
        val record = PresentationRecord(
            recordId = UUID.randomUUID().toString(),
            requestId = requestId,
            verifier = verifier,
            disclosedCredentials = disclosedCredentialIds,
            timestampNs = nowNs,
            purpose = purpose,
            consentGiven = consentGiven
        )
        presentationHistory.add(record)
        val riskScore = if (!consentGiven) 0.8 else 0.1
        logAudit("PRESENTATION_RECORD", consentGiven, "Presentation to $verifier for $purpose", nowNs, riskScore)
    }
    
    fun revokeCredential(credentialId: String, nowNs: Long): Result<Unit> {
        val credential = credentials.find { it.credentialId == credentialId }
            ?: return Result.failure(Error("CREDENTIAL_NOT_FOUND"))
        credential.revoked = true
        logAudit("CREDENTIAL_REVOKE", true, "Credential $credentialId revoked", nowNs, 0.1)
        return Result.success(Unit)
    }
    
    fun getWalletStatus(nowNs: Long): WalletStatus {
        val activeCredentials = credentials.count { it.isValid(nowNs) && !it.revoked }
        val expiredCredentials = credentials.count { !it.isValid(nowNs) }
        val revokedCredentials = credentials.count { it.revoked }
        val recentPresentations = presentationHistory.count { nowNs - it.timestampNs < 86400000000000L }
        val avgRiskScore = if (auditLog.isEmpty()) 0.0
            else auditLog.map { it.riskScore }.average()
        return WalletStatus(
            walletId = walletId,
            ownerDid = ownerDid,
            activeCredentials = activeCredentials,
            expiredCredentials = expiredCredentials,
            revokedCredentials = revokedCredentials,
            totalPresentations = presentationHistory.size,
            presentationsLast24h = recentPresentations,
            averageRiskScore = avgRiskScore,
            lastActivityNs = if (auditLog.isEmpty()) initTimestampNs else auditLog.last().timestampNs
        )
    }
    
    private fun logAudit(action: String, success: Boolean, details: String, timestampNs: Long, riskScore: Double) {
        val entry = WalletAuditEntry(
            entryId = auditLog.size.toLong(),
            action = action,
            timestampNs = timestampNs,
            success = success,
            details = details,
            riskScore = riskScore
        )
        auditLog.add(entry)
    }
    
    fun getAuditTrail(fromNs: Long, toNs: Long): List<WalletAuditEntry> {
        return auditLog.filter { it.timestampNs in fromNs..toNs }
    }
    
    fun computePrivacyScore(): Double {
        val consentRate = if (presentationHistory.isEmpty()) 1.0
            else presentationHistory.count { it.consentGiven }.toDouble() / presentationHistory.size
        val credentialHealth = credentials.count { it.isValid(System.nanoTime()) && !it.revoked }.toDouble() /
            credentials.size.coerceAtLeast(1)
        val auditCompliance = if (auditLog.isEmpty()) 1.0
            else auditLog.count { it.success }.toDouble() / auditLog.size
        return (consentRate * 0.5 + credentialHealth * 0.3 + auditCompliance * 0.2).coerceIn(0.0, 1.0)
    }
}

data class WalletStatus(
    val walletId: String,
    val ownerDid: String,
    val activeCredentials: Int,
    val expiredCredentials: Int,
    val revokedCredentials: Int,
    val totalPresentations: Int,
    val presentationsLast24h: Int,
    val averageRiskScore: Double,
    val lastActivityNs: Long
) {
    fun healthIndex(): Double {
        val credentialHealth = activeCredentials.toDouble() / (activeCredentials + expiredCredentials + revokedCredentials).coerceAtLeast(1)
        val riskPenalty = averageRiskScore * 0.5
        return (credentialHealth - riskPenalty).coerceIn(0.0, 1.0)
    }
}

fun createCitizenWallet(walletId: String, citizenDid: String, nowNs: Long): DIDWalletManager {
    val wallet = DIDWalletManager(walletId, citizenDid, nowNs)
    wallet.initializeDIDDocument(citizenDid, nowNs)
    return wallet
}
