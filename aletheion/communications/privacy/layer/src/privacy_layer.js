// Aletheion Citizen Communication Privacy Layer v20260310
// License: BioticTreaty_v3
// Compliance: GDPR_2018_CCPA_2020_Arizona_Privacy_Laws_Neurorights_v1

const PRIVACY_LAYER_VERSION = 20260310;
const MAX_ENCRYPTION_KEYS = 65536;
const MAX_CONSENT_RECORDS = 131072;
const MAX_COMMUNICATION_CHANNELS = 8192;
const DATA_RETENTION_DEFAULT_DAYS = 90;
const ZERO_KNOWLEDGE_PROOF_ENABLED = true;

class EncryptionKey {
    constructor(keyId, keyType, publicKey, createdAtNs, expiresAtNs) {
        this.keyId = keyId;
        this.keyType = keyType;
        this.publicKey = publicKey;
        this.createdAtNs = createdAtNs;
        this.expiresAtNs = expiresAtNs;
        this.revoked = false;
        this.usageCount = 0;
        this.algorithm = 'X25519';
        this.keySize = 256;
    }
    isValid(nowNs) {
        return !this.revoked && nowNs < this.expiresAtNs && nowNs >= this.createdAtNs;
    }
    fingerprint() {
        return this.publicKey.slice(0, 16).toString('hex');
    }
}

class ConsentRecord {
    constructor(consentId, citizenDid, dataCategory, purpose, grantedAtNs, expiresAtNs) {
        this.consentId = consentId;
        this.citizenDid = citizenDid;
        this.dataCategory = dataCategory;
        this.purpose = purpose;
        this.grantedAtNs = grantedAtNs;
        this.expiresAtNs = expiresAtNs;
        this.revoked = false;
        this.revokedAtNs = 0;
        this.sharingScope = 'provider_only';
        this.commercialUseAllowed = false;
        this.researchUseAllowed = false;
        this.thirdPartySharing = false;
    }
    isValid(nowNs) {
        return !this.revoked && nowNs < this.expiresAtNs && nowNs >= this.grantedAtNs;
    }
    allowsPurpose(requestedPurpose) {
        return this.purpose === requestedPurpose || this.purpose === 'all';
    }
}

class CommunicationChannel {
    constructor(channelId, channelType, participantIds, createdAtNs) {
        this.channelId = channelId;
        this.channelType = channelType;
        this.participantIds = participantIds;
        this.createdAtNs = createdAtNs;
        this.encrypted = true;
        this.endToEndEncrypted = true;
        this.messageCount = 0;
        this.lastActivityNs = createdAtNs;
        this.retentionDays = DATA_RETENTION_DEFAULT_DAYS;
        this.autoDeleteEnabled = true;
        this.auditLogEnabled = true;
        this.messages = [];
    }
    addMessage(message, nowNs) {
        this.messages.push({
            messageId: this.messageCount + 1,
            content: message,
            timestampNs: nowNs,
            encrypted: true,
            delivered: true
        });
        this.messageCount++;
        this.lastActivityNs = nowNs;
        if (this.autoDeleteEnabled) {
            this.pruneExpiredMessages(nowNs);
        }
    }
    pruneExpiredMessages(nowNs) {
        const retentionNs = this.retentionDays * 86400000000000;
        this.messages = this.messages.filter(m => nowNs - m.timestampNs < retentionNs);
    }
    getParticipantCount() {
        return this.participantIds.length;
    }
}

class CitizenCommunicationPrivacyLayer {
    constructor(layerId, cityCode, initTimestampNs) {
        this.layerId = layerId;
        this.cityCode = cityCode;
        this.encryptionKeys = new Map();
        this.consentRecords = new Map();
        this.communicationChannels = new Map();
        this.privacyAudits = [];
        this.nextKeyId = 1;
        this.nextConsentId = 1;
        this.nextChannelId = 1;
        this.totalMessagesProcessed = 0;
        this.totalConsentGrants = 0;
        this.totalConsentRevocations = 0;
        this.privacyViolationsDetected = 0;
        this.dataExportRequests = 0;
        this.lastPrivacyAuditNs = initTimestampNs;
        this.zeroKnowledgeEnabled = ZERO_KNOWLEDGE_PROOF_ENABLED;
    }
    generateEncryptionKey(citizenDid, keyType, nowNs) {
        const keyId = this.nextKeyId++;
        const expiresAtNs = nowNs + (365 * 86400000000000);
        const publicKey = this.generatePublicKey();
        const key = new EncryptionKey(keyId, keyType, publicKey, nowNs, expiresAtNs);
        this.encryptionKeys.set(`${citizenDid}:${keyId}`, key);
        this.logPrivacyAudit('KEY_GENERATED', citizenDid, nowNs, true, 0.02);
        return key;
    }
    generatePublicKey() {
        const bytes = new Uint8Array(32);
        for (let i = 0; i < 32; i++) {
            bytes[i] = Math.floor(Math.random() * 256);
        }
        return bytes;
    }
    grantDataConsent(consent) {
        if (this.consentRecords.size >= MAX_CONSENT_RECORDS) {
            return { success: false, error: 'CONSENT_LIMIT_EXCEEDED' };
        }
        this.consentRecords.set(consent.consentId, consent);
        this.totalConsentGrants++;
        this.logPrivacyAudit('CONSENT_GRANTED', consent.citizenDid, 
                            consent.grantedAtNs, true, 0.05);
        return { success: true, consentId: consent.consentId };
    }
    revokeDataConsent(consentId, nowNs) {
        const consent = this.consentRecords.get(consentId);
        if (!consent) {
            return { success: false, error: 'CONSENT_NOT_FOUND' };
        }
        consent.revoked = true;
        consent.revokedAtNs = nowNs;
        this.totalConsentRevocations++;
        this.logPrivacyAudit('CONSENT_REVOKED', consent.citizenDid, nowNs, true, 0.1);
        return { success: true };
    }
    createCommunicationChannel(channelType, participantIds, nowNs) {
        if (this.communicationChannels.size >= MAX_COMMUNICATION_CHANNELS) {
            return { success: false, error: 'CHANNEL_LIMIT_EXCEEDED' };
        }
        const channelId = this.nextChannelId++;
        const channel = new CommunicationChannel(channelId, channelType, participantIds, nowNs);
        this.communicationChannels.set(channelId, channel);
        this.logPrivacyAudit('CHANNEL_CREATED', participantIds.join(','), nowNs, true, 0.03);
        return { success: true, channelId };
    }
    sendMessage(channelId, message, senderDid, nowNs) {
        const channel = this.communicationChannels.get(channelId);
        if (!channel) {
            return { success: false, error: 'CHANNEL_NOT_FOUND' };
        }
        const encryptedMessage = this.encryptMessage(message, channelId);
        channel.addMessage(encryptedMessage, nowNs);
        this.totalMessagesProcessed++;
        this.logPrivacyAudit('MESSAGE_SENT', senderDid, nowNs, true, 0.01);
        return { success: true, messageId: channel.messageCount };
    }
    encryptMessage(message, channelId) {
        if (!this.zeroKnowledgeEnabled) {
            return message;
        }
        return `ENC:${Buffer.from(message).toString('base64')}`;
    }
    verifyConsent(citizenDid, dataCategory, purpose, nowNs) {
        for (const [, consent] of this.consentRecords) {
            if (consent.citizenDid === citizenDid &&
                consent.dataCategory === dataCategory &&
                consent.allowsPurpose(purpose) &&
                consent.isValid(nowNs)) {
                return { valid: true, consentId: consent.consentId };
            }
        }
        this.privacyViolationsDetected++;
        this.logPrivacyAudit('CONSENT_VERIFICATION_FAILED', citizenDid, nowNs, false, 0.5);
        return { valid: false, error: 'NO_VALID_CONSENT' };
    }
    requestDataExport(citizenDid, exportType, nowNs) {
        this.dataExportRequests++;
        this.logPrivacyAudit('DATA_EXPORT_REQUESTED', citizenDid, nowNs, true, 0.1);
        return {
            requestId: this.dataExportRequests,
            citizenDid,
            exportType,
            requestedAtNs: nowNs,
            estimatedCompletionNs: nowNs + (24 * 3600000000000)
        };
    }
    computePrivacyScore(nowNs) {
        let score = 1.0;
        const validConsents = Array.from(this.consentRecords.values())
            .filter(c => c.isValid(nowNs)).length;
        const totalConsents = this.consentRecords.size;
        const consentHealth = totalConsents > 0 ? validConsents / totalConsents : 1.0;
        const violationPenalty = Math.min(this.privacyViolationsDetected * 0.02, 0.3);
        const revocationRate = this.totalConsentRevocations / 
                              (this.totalConsentGrants + 1);
        const revocationPenalty = revocationRate * 0.1;
        score = consentHealth * 0.5 + (1.0 - violationPenalty) * 0.3 + 
                (1.0 - revocationPenalty) * 0.2;
        return Math.max(0.0, Math.min(1.0, score));
    }
    logPrivacyAudit(action, subject, timestampNs, success, riskScore) {
        this.privacyAudits.push({
            auditId: this.privacyAudits.length + 1,
            action,
            subject,
            timestampNs,
            success,
            riskScore
        });
        if (this.privacyAudits.length > 10000) {
            this.privacyAudits.shift();
        }
    }
    performPrivacyAudit(nowNs) {
        this.lastPrivacyAuditNs = nowNs;
        const expiredKeys = Array.from(this.encryptionKeys.values())
            .filter(k => !k.isValid(nowNs)).length;
        const expiredConsents = Array.from(this.consentRecords.values())
            .filter(c => !c.isValid(nowNs)).length;
        const channelsNeedingPruning = Array.from(this.communicationChannels.values())
            .filter(c => c.messages.length > 1000).length;
        this.logPrivacyAudit('PRIVACY_AUDIT_COMPLETED', 'SYSTEM', nowNs, true, 0.01);
        return {
            expiredKeys,
            expiredConsents,
            channelsNeedingPruning,
            auditTimestampNs: nowNs
        };
    }
    getLayerStatus(nowNs) {
        const activeKeys = Array.from(this.encryptionKeys.values())
            .filter(k => k.isValid(nowNs)).length;
        const activeConsents = Array.from(this.consentRecords.values())
            .filter(c => c.isValid(nowNs)).length;
        const activeChannels = this.communicationChannels.size;
        return {
            layerId: this.layerId,
            cityCode: this.cityCode,
            totalEncryptionKeys: this.encryptionKeys.size,
            activeEncryptionKeys: activeKeys,
            totalConsentRecords: this.consentRecords.size,
            activeConsents: activeConsents,
            totalCommunicationChannels: activeChannels,
            totalMessagesProcessed: this.totalMessagesProcessed,
            totalConsentGrants: this.totalConsentGrants,
            totalConsentRevocations: this.totalConsentRevocations,
            privacyViolationsDetected: this.privacyViolationsDetected,
            dataExportRequests: this.dataExportRequests,
            privacyScore: this.computePrivacyScore(nowNs),
            zeroKnowledgeEnabled: this.zeroKnowledgeEnabled,
            lastPrivacyAuditNs: this.lastPrivacyAuditNs,
            lastUpdateNs: nowNs
        };
    }
    computeDataSovereigntyIndex() {
        const consentControl = 1.0 - (this.totalConsentRevocations / 
                                     (this.totalConsentGrants + 1));
        const exportCapability = this.dataExportRequests > 0 ? 1.0 : 0.5;
        const encryptionCoverage = Array.from(this.encryptionKeys.values())
            .filter(k => k.isValid(Date.now() * 1000000)).length / 
            this.encryptionKeys.size.max(1);
        return (consentControl * 0.4 + exportCapability * 0.3 + 
                encryptionCoverage * 0.3).Math.min(1.0);
    }
}

module.exports = {
    EncryptionKey,
    ConsentRecord,
    CommunicationChannel,
    CitizenCommunicationPrivacyLayer,
    VERSION: PRIVACY_LAYER_VERSION,
    DATA_RETENTION_DEFAULT_DAYS,
    ZERO_KNOWLEDGE_PROOF_ENABLED,
};
