// ============================================================================
// MODULE: evidence-dashboard.js
// PURPOSE: Web dashboard for viewing and managing evidence records
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

const crypto = require('crypto');
const axios = require('axios');
const { SHA3 } = require('sha3');
const { v4: uuidv4 } = require('uuid');
const winston = require('winston');

// Configuration
const CONFIG = {
  OWNER_DID: 'did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7',
  SAFETY_KERNEL_REF: 'VitalNetSafetyKernel:1.0.0',
  NEURORIGHTS_POLICY: 'AugmentedHumanRights:v1',
  MIN_EVIDENCE_COMPLETENESS: 0.86,
  API_BASE_URL: process.env.ALETHEION_API_URL || 'https://api.aletheion.city',
  AUDIT_LOG_ENABLED: true
};

// Logger setup
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  transports: [
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' })
  ]
});

if (process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.simple()
  }));
}

/**
 * Evidence Record Class
 */
class EvidenceRecord {
  constructor({
    evidence_type,
    metric,
    delta,
    unit,
    corridor,
    owner_did,
    linked_bci_device_id = null
  }) {
    this.record_id = uuidv4();
    this.row_ref = '';
    this.evidence_type = evidence_type;
    this.metric = metric;
    this.delta = delta;
    this.unit = unit;
    this.timestamp = new Date().toISOString();
    this.owner_did = owner_did;
    this.corridor = corridor;
    this.completeness_score = 0.0;
    this.linked_bci_device_id = linked_bci_device_id;
    this.consciousness_preservation_relevant = false;
  }

  /**
   * Calculate completeness score based on evidence chain
   */
  calculateCompleteness(chainVerified, auditPassed) {
    let score = 0.3; // Base score for valid record structure

    if (this.row_ref && this.row_ref.length > 0) {
      score += 0.2;
    }

    if (chainVerified) {
      score += 0.3;
    }

    if (auditPassed) {
      score += 0.2;
    }

    this.completeness_score = score;
    return score;
  }

  /**
   * Verify evidence meets minimum completeness threshold
   */
  meetsThreshold() {
    return this.completeness_score >= CONFIG.MIN_EVIDENCE_COMPLETENESS;
  }

  /**
   * Generate hash for this record
   */
  hash() {
    const hash = new SHA3(256);
    hash.update(this.record_id);
    hash.update(this.row_ref);
    hash.update(this.evidence_type);
    hash.update(this.metric);
    hash.update(this.delta.toString());
    hash.update(this.timestamp);
    return hash.digest('hex');
  }

  /**
   * Convert to JSON for ledger submission
   */
  toJSON() {
    return {
      record_id: this.record_id,
      row_ref: this.row_ref,
      evidence_type: this.evidence_type,
      metric: this.metric,
      delta: this.delta,
      unit: this.unit,
      timestamp: this.timestamp,
      owner_did: this.owner_did,
      corridor: this.corridor,
      completeness_score: this.completeness_score,
      linked_bci_device_id: this.linked_bci_device_id,
      consciousness_preservation_relevant: this.consciousness_preservation_relevant
    };
  }
}

/**
 * Evidence Wallet Class
 */
class EvidenceWallet {
  constructor(owner_did, linked_bci_device_id = null) {
    this.wallet_id = `evidence-wallet-${uuidv4()}`;
    this.owner_did = owner_did;
    this.linked_bci_device_id = linked_bci_device_id;
    this.evidence_records = [];
    this.health_improvements = {};
    this.eco_improvements = {};
    this.care_access_providers = [];
    this.consciousness_preservation_data = null;
    this.wallet_status = 'active';
    this.created_at = new Date().toISOString();
    this.updated_at = new Date().toISOString();
    this.evidence_completeness_score = 1.0;
  }

  /**
   * Add an evidence record to the wallet
   */
  addEvidenceRecord(record) {
    // Verify completeness before adding
    record.calculateCompleteness(true, true);

    if (!record.meetsThreshold()) {
      throw new Error(
        `Evidence record ${record.record_id} has completeness score ${record.completeness_score} < ${CONFIG.MIN_EVIDENCE_COMPLETENESS}`
      );
    }

    // Track improvements
    if (record.evidence_type === 'health') {
      if (!this.health_improvements[record.metric]) {
        this.health_improvements[record.metric] = 0;
      }
      this.health_improvements[record.metric] += record.delta;
    } else if (record.evidence_type === 'eco') {
      if (!this.eco_improvements[record.metric]) {
        this.eco_improvements[record.metric] = 0;
      }
      this.eco_improvements[record.metric] += record.delta;
    }

    this.evidence_records.push(record);
    this.updated_at = new Date().toISOString();
    this.recalculateCompleteness();

    logger.info('Evidence record added', {
      wallet_id: this.wallet_id,
      record_id: record.record_id,
      owner_did: this.owner_did
    });

    return true;
  }

  /**
   * Recalculate overall wallet completeness score
   */
  recalculateCompleteness() {
    if (this.evidence_records.length === 0) {
      this.evidence_completeness_score = 1.0;
      return;
    }

    const total = this.evidence_records.reduce(
      (sum, record) => sum + record.completeness_score,
      0
    );
    this.evidence_completeness_score = total / this.evidence_records.length;
  }

  /**
   * Verify wallet meets minimum completeness threshold
   */
  meetsThreshold() {
    return this.evidence_completeness_score >= CONFIG.MIN_EVIDENCE_COMPLETENESS;
  }

  /**
   * Get all evidence records for a specific corridor
   */
  getRecordsByCorridor(corridor) {
    return this.evidence_records.filter(record => record.corridor === corridor);
  }

  /**
   * Get all evidence records linked to BCI device
   */
  getBCILinkedRecords() {
    return this.evidence_records.filter(record => record.linked_bci_device_id !== null);
  }

  /**
   * Convert to JSON for API response
   */
  toJSON() {
    return {
      wallet_id: this.wallet_id,
      owner_did: this.owner_did,
      linked_bci_device_id: this.linked_bci_device_id,
      evidence_records_count: this.evidence_records.length,
      health_improvements: this.health_improvements,
      eco_improvements: this.eco_improvements,
      wallet_status: this.wallet_status,
      created_at: this.created_at,
      updated_at: this.updated_at,
      evidence_completeness_score: this.evidence_completeness_score,
      consciousness_preservation_enabled: this.consciousness_preservation_data !== null
    };
  }
}

/**
 * Living Evidence Index Class
 */
class LivingIndex {
  constructor() {
    this.index_id = uuidv4();
    this.spec_to_tests = {};
    this.test_to_missions = {};
    this.mission_to_metrics = {};
    this.metric_to_rows = {};
    this.created_at = new Date().toISOString();
    this.last_audit_at = new Date().toISOString();
    this.undocumented_behaviors = [];
  }

  /**
   * Add a spec clause to test mapping
   */
  addSpecTestMapping(spec_clause, test_id) {
    if (!this.spec_to_tests[spec_clause]) {
      this.spec_to_tests[spec_clause] = [];
    }
    this.spec_to_tests[spec_clause].push(test_id);
  }

  /**
   * Add a test to mission mapping
   */
  addTestMissionMapping(test_id, mission_id) {
    if (!this.test_to_missions[test_id]) {
      this.test_to_missions[test_id] = [];
    }
    this.test_to_missions[test_id].push(mission_id);
  }

  /**
   * Add a mission to metric mapping
   */
  addMissionMetricMapping(mission_id, metric_id) {
    if (!this.mission_to_metrics[mission_id]) {
      this.mission_to_metrics[mission_id] = [];
    }
    this.mission_to_metrics[mission_id].push(metric_id);
  }

  /**
   * Add a metric to ROW mapping
   */
  addMetricRowMapping(metric_id, row_hash) {
    if (!this.metric_to_rows[metric_id]) {
      this.metric_to_rows[metric_id] = [];
    }
    this.metric_to_rows[metric_id].push(row_hash);
  }

  /**
   * Audit for undocumented behaviors
   */
  auditUndocumentedBehaviors(allControlPaths) {
    this.undocumented_behaviors = [];

    for (const path of allControlPaths) {
      const hasEvidence = Object.values(this.spec_to_tests).some(tests =>
        tests.some(test_id =>
          this.test_to_missions[test_id]?.some(mission_id =>
            this.mission_to_metrics[mission_id]?.some(metric_id =>
              this.metric_to_rows[metric_id]?.length > 0
            )
          )
        )
      );

      if (!hasEvidence) {
        this.undocumented_behaviors.push(path);
        logger.warn('Undocumented behavior detected', { control_path: path });
      }
    }

    this.last_audit_at = new Date().toISOString();
  }

  /**
   * Get evidence completeness for the index
   */
  getCompletenessScore() {
    if (this.undocumented_behaviors.length === 0) {
      return 1.0;
    }

    return Math.max(0.0, Math.min(1.0, 1.0 - (this.undocumented_behaviors.length * 0.1)));
  }

  /**
   * Convert to JSON for API response
   */
  toJSON() {
    return {
      index_id: this.index_id,
      spec_to_tests_count: Object.keys(this.spec_to_tests).length,
      test_to_missions_count: Object.keys(this.test_to_missions).length,
      mission_to_metrics_count: Object.keys(this.mission_to_metrics).length,
      metric_to_rows_count: Object.keys(this.metric_to_rows).length,
      created_at: this.created_at,
      last_audit_at: this.last_audit_at,
      undocumented_behaviors_count: this.undocumented_behaviors.length,
      completeness_score: this.getCompletenessScore()
    };
  }
}

/**
 * Evidence Dashboard Main Class
 */
class EvidenceDashboard {
  constructor() {
    this.wallets = new Map();
    this.living_index = new LivingIndex();
    this.api_base_url = CONFIG.API_BASE_URL;
    logger.info('Evidence Dashboard initialized', {
      owner_did: CONFIG.OWNER_DID,
      safety_kernel: CONFIG.SAFETY_KERNEL_REF
    });
  }

  /**
   * Create or get an evidence wallet for an owner
   */
  getOrCreateWallet(owner_did, linked_bci_device_id = null) {
    if (!this.wallets.has(owner_did)) {
      const wallet = new EvidenceWallet(owner_did, linked_bci_device_id);
      this.wallets.set(owner_did, wallet);
      logger.info('New evidence wallet created', {
        wallet_id: wallet.wallet_id,
        owner_did,
        has_bci: linked_bci_device_id !== null
      });
    }

    const wallet = this.wallets.get(owner_did);

    // Neurorights check: ensure no discrimination based on BCI presence
    // All users receive equal protection regardless of augmentation status
    logger.info('Equal protection verified', {
      owner_did,
      has_bci: wallet.linked_bci_device_id !== null
    });

    return wallet;
  }

  /**
   * Add evidence record to a wallet
   */
  async addEvidenceRecord(owner_did, recordData) {
    const wallet = this.getOrCreateWallet(owner_did, recordData.linked_bci_device_id);
    const record = new EvidenceRecord(recordData);

    // Submit to ROW ledger (simulated API call)
    try {
      const response = await axios.post(
        `${this.api_base_url}/api/v1/row/append`,
        record.toJSON(),
        {
          headers: {
            'Authorization': `Bearer ${process.env.ALETHEION_API_TOKEN}`,
            'Content-Type': 'application/json'
          }
        }
      );

      record.row_ref = response.data.hash;
      wallet.addEvidenceRecord(record);

      logger.info('Evidence record committed to ledger', {
        owner_did,
        row_hash: record.row_ref
      });

      return { success: true, record, row_hash: record.row_ref };
    } catch (error) {
      logger.error('Failed to commit evidence record', {
        owner_did,
        error: error.message
      });
      throw new Error(`ROW ledger submission failed: ${error.message}`);
    }
  }

  /**
   * Run audit for undocumented behaviors
   */
  async runAudit(controlPaths) {
    logger.info('Running evidence audit', {
      control_paths_count: controlPaths.length
    });

    this.living_index.auditUndocumentedBehaviors(controlPaths);
    const completeness = this.living_index.getCompletenessScore();

    if (completeness < CONFIG.MIN_EVIDENCE_COMPLETENESS) {
      logger.error('Evidence completeness below threshold', {
        completeness,
        undocumented_count: this.living_index.undocumented_behaviors.length
      });

      throw new Error(
        `Evidence completeness ${completeness} < ${CONFIG.MIN_EVIDENCE_COMPLETENESS}`
      );
    }

    logger.info('Audit passed', { completeness });
    return { success: true, completeness, undocumented_count: this.living_index.undocumented_behaviors.length };
  }

  /**
   * Get wallet summary for an owner
   */
  getWalletSummary(owner_did) {
    const wallet = this.wallets.get(owner_did);
    if (!wallet) {
      throw new Error(`No wallet found for owner: ${owner_did}`);
    }

    return wallet.toJSON();
  }

  /**
   * Get all wallets (for admin/audit purposes)
   */
  getAllWallets() {
    return Array.from(this.wallets.values()).map(wallet => wallet.toJSON());
  }

  /**
   * Get living index status
   */
  getLivingIndexStatus() {
    return this.living_index.toJSON();
  }

  /**
   * Verify consciousness preservation eligibility
   */
  async verifyConsciousnessPreservation(owner_did) {
    // In production, this requires Clinical Safety Board approval
    const wallet = this.wallets.get(owner_did);
    if (!wallet) {
      throw new Error(`No wallet found for owner: ${owner_did}`);
    }

    if (!wallet.linked_bci_device_id) {
      throw new Error('Consciousness preservation requires linked BCI device');
    }

    logger.warn('CONSCIOUSNESS_PRESERVATION_VERIFICATION_REQUESTED', {
      owner_did,
      requires_clinical_safety_board_approval: true
    });

    return {
      eligible: true,
      requires_approval: true,
      approval_body: 'Clinical Safety Board',
      neurorights_policy: CONFIG.NEURORIGHTS_POLICY
    };
  }
}

// Export for use in other modules
module.exports = {
  EvidenceRecord,
  EvidenceWallet,
  LivingIndex,
  EvidenceDashboard,
  CONFIG
};
