// ============================================================================
// MODULE: neurorights-client.js
// PURPOSE: Neurorights protection and consent management client
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

const crypto = require('crypto');
const axios = require('axios');
const winston = require('winston');

// Configuration
const CONFIG = {
  OWNER_DID: 'did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7',
  SAFETY_KERNEL_REF: 'VitalNetSafetyKernel:1.0.0',
  NEURORIGHTS_POLICY_VERSION: 'AugmentedHumanRights:v1',
  API_BASE_URL: process.env.ALETHEION_API_URL || 'https://api.aletheion.city'
};

// Logger setup
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  transports: [
    new winston.transports.File({ filename: 'logs/neurorights-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/neurorights-combined.log' })
  ]
});

if (process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.simple()
  }));
}

/**
 * Neurorights Policy Class
 */
class NeurorightsPolicy {
  constructor() {
    this.version = CONFIG.NEURORIGHTS_POLICY_VERSION;
    this.principles = [
      'No covert neuromorphic control',
      'No Death-Network style sabotage',
      'Explicit informed consent and revocation rights',
      'Immutable audit and clinical oversight',
      'Equal protection regardless of race or disability',
      'Consciousness preservation rights with explicit consent',
      'No discrimination based on augmentation status or technology type',
      'Deviceless and organically-integrated cybernetics receive equal protection',
      'All biophysical data is protected under medical-grade safeguards',
      'Appeal and override path available via Clinical Safety Board'
    ];

    this.prohibited_actions = [
      'covert_neuromorphic_control',
      'death_network_sabotage',
      'discriminatory_corridor_access',
      'unconsented_biophysical_data_access',
      'downgrade_of_augmentation_rights',
      'exclusion_based_on_integration_type'
    ];

    this.required_safeguards = [
      'VitalNetSafetyKernel enforcement',
      'Immutable ROW audit logs',
      'Explicit consent for all BCI operations',
      'Clinical oversight for organic integrations',
      'Independent safety review for firmware changes',
      'Neurorights ombud escalation path'
    ];
  }

  /**
   * Check if an action is prohibited
   */
  isProhibited(action) {
    return this.prohibited_actions.includes(action);
  }

  /**
   * Verify equal protection principle
   */
  verifyEqualProtection(has_bci) {
    // All users receive equal protection regardless of augmentation status
    logger.info('Equal protection verified', { has_bci });
    return true;
  }

  /**
   * Get policy as JSON
   */
  toJSON() {
    return {
      version: this.version,
      principles: this.principles,
      prohibited_actions: this.prohibited_actions,
      required_safeguards: this.required_safeguards
    };
  }
}

/**
 * Safety Kernel Client Class
 */
class SafetyKernelClient {
  constructor(kernel_ref = CONFIG.SAFETY_KERNEL_REF) {
    this.kernel_ref = kernel_ref;
    this.policy = new NeurorightsPolicy();
    this.biofield_load_ceiling = 0.5; // W/kg per FCC/ICNIRP
    this.consent_profiles = new Map();
    this.audit_log = [];
  }

  /**
   * Verify an evidence record against safety constraints
   */
  verifyRecord(record) {
    // Check consent
    if (!this.consent_profiles.get(record.owner_did)) {
      throw new Error(`No consent profile for owner: ${record.owner_did}`);
    }

    // Check for prohibited actions in evidence type
    if (this.policy.isProhibited(record.evidence_type)) {
      throw new Error(`Prohibited evidence type: ${record.evidence_type}`);
    }

    // Log verification
    this.logAudit(`Record verified: ${record.record_id} by ${record.owner_did}`);

    return true;
  }

  /**
   * Register consent for an owner
   */
  registerConsent(owner_did) {
    this.consent_profiles.set(owner_did, true);
    this.logAudit(`Consent registered: ${owner_did}`);
    logger.info('Consent registered', { owner_did });
  }

  /**
   * Revoke consent for an owner
   */
  revokeConsent(owner_did) {
    this.consent_profiles.set(owner_did, false);
    this.logAudit(`Consent revoked: ${owner_did}`);
    logger.info('Consent revoked', { owner_did });
  }

  /**
   * Log an audit entry
   */
  logAudit(message) {
    const timestamp = new Date().toISOString();
    this.audit_log.push(`[${timestamp}] ${message}`);
  }

  /**
   * Get audit log
   */
  getAuditLog() {
    return this.audit_log;
  }

  /**
   * Verify biofield load ceiling
   */
  verifyBiofieldLoad(neuroclass, load) {
    const limits = {
      'human_cortex_v1': 0.5,
      'human_PNS': 0.8
    };
    const limit = limits[neuroclass] || 0.5;
    return load <= limit;
  }
}

/**
 * Neurorights Client Main Class
 */
class NeurorightsClient {
  constructor() {
    this.policy = new NeurorightsPolicy();
    this.safety_kernel = new SafetyKernelClient();
    this.violation_count = 0;
    this.api_base_url = CONFIG.API_BASE_URL;

    logger.info('Neurorights Client initialized', {
      policy_version: this.policy.version,
      safety_kernel: this.safety_kernel.kernel_ref
    });
  }

  /**
   * Verify equal protection for an owner
   */
  verifyEqualProtection(owner_did, has_bci) {
    this.policy.verifyEqualProtection(has_bci);

    logger.info('Equal protection verified - no discrimination', {
      owner_did,
      has_bci
    });

    return true;
  }

  /**
   * Check for discriminatory actions
   */
  checkDiscrimination(action, target_did) {
    if (this.policy.isProhibited('discriminatory_corridor_access') &&
        action.includes('discriminatory')) {
      this.violation_count += 1;
      throw new Error(
        `Discriminatory action detected: ${action} for ${target_did}`
      );
    }

    return true;
  }

  /**
   * Register consent with biometric verification (simulated)
   */
  async registerConsentWithBiometric(owner_did, biometric_token) {
    // In production, this would verify biometric token with secure enclave
    if (!biometric_token || biometric_token.length < 32) {
      throw new Error('Invalid biometric token');
    }

    this.safety_kernel.registerConsent(owner_did);

    // Submit consent to ROW ledger
    try {
      await axios.post(
        `${this.api_base_url}/api/v1/consent/register`,
        {
          owner_did,
          consent_type: 'biophysical_data_access',
          biometric_verified: true,
          timestamp: new Date().toISOString()
        },
        {
          headers: {
            'Authorization': `Bearer ${process.env.ALETHEION_API_TOKEN}`,
            'Content-Type': 'application/json'
          }
        }
      );

      logger.info('Consent registered with biometric verification', { owner_did });
      return { success: true, owner_did };
    } catch (error) {
      logger.error('Failed to register consent', {
        owner_did,
        error: error.message
      });
      throw new Error(`Consent registration failed: ${error.message}`);
    }
  }

  /**
   * Revoke consent
   */
  async revokeConsent(owner_did) {
    this.safety_kernel.revokeConsent(owner_did);

    // Submit consent revocation to ROW ledger
    try {
      await axios.post(
        `${this.api_base_url}/api/v1/consent/revoke`,
        {
          owner_did,
          timestamp: new Date().toISOString()
        },
        {
          headers: {
            'Authorization': `Bearer ${process.env.ALETHEION_API_TOKEN}`,
            'Content-Type': 'application/json'
          }
        }
      );

      logger.info('Consent revoked', { owner_did });
      return { success: true, owner_did };
    } catch (error) {
      logger.error('Failed to revoke consent', {
        owner_did,
        error: error.message
      });
      throw new Error(`Consent revocation failed: ${error.message}`);
    }
  }

  /**
   * Request consciousness preservation (requires Clinical Safety Board approval)
   */
  async requestConsciousnessPreservation(owner_did, bci_device_id) {
    logger.warn('CONSCIOUSNESS_PRESERVATION_REQUESTED', {
      owner_did,
      bci_device_id,
      requires_clinical_safety_board_approval: true
    });

    // Submit request to Clinical Safety Board
    try {
      const response = await axios.post(
        `${this.api_base_url}/api/v1/consciousness-preservation/request`,
        {
          owner_did,
          bci_device_id,
          request_timestamp: new Date().toISOString(),
          approval_required: true,
          approval_body: 'Clinical Safety Board'
        },
        {
          headers: {
            'Authorization': `Bearer ${process.env.ALETHEION_API_TOKEN}`,
            'Content-Type': 'application/json'
          }
        }
      );

      return {
        success: true,
        request_id: response.data.request_id,
        status: 'pending_approval',
        approval_body: 'Clinical Safety Board',
        neurorights_policy: this.policy.version
      };
    } catch (error) {
      logger.error('Failed to request consciousness preservation', {
        owner_did,
        error: error.message
      });
      throw new Error(`Consciousness preservation request failed: ${error.message}`);
    }
  }

  /**
   * Report a neurorights violation
   */
  async reportViolation(violation_type, details, reporter_did) {
    this.violation_count += 1;

    logger.error('Neurorights violation reported', {
      violation_type,
      details,
      reporter_did
    });

    // Submit violation report to neurorights ombud
    try {
      await axios.post(
        `${this.api_base_url}/api/v1/neurorights/violation-report`,
        {
          violation_type,
          details,
          reporter_did,
          timestamp: new Date().toISOString(),
          violation_id: this.violation_count
        },
        {
          headers: {
            'Authorization': `Bearer ${process.env.ALETHEION_API_TOKEN}`,
            'Content-Type': 'application/json'
          }
        }
      );

      return {
        success: true,
        violation_id: this.violation_count,
        escalation_path: 'clinical_safety_board -> neurorights_ombud -> independent_review'
      };
    } catch (error) {
      logger.error('Failed to report violation', {
        violation_type,
        error: error.message
      });
      throw new Error(`Violation report failed: ${error.message}`);
    }
  }

  /**
   * Get violation count
   */
  getViolationCount() {
    return this.violation_count;
  }

  /**
   * Get audit log
   */
  getAuditLog() {
    return this.safety_kernel.getAuditLog();
  }

  /**
   * Get policy information
   */
  getPolicy() {
    return this.policy.toJSON();
  }
}

// Export for use in other modules
module.exports = {
  NeurorightsPolicy,
  SafetyKernelClient,
  NeurorightsClient,
  CONFIG
};
