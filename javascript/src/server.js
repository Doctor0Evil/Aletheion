// ============================================================================
// SERVER: Aletheion Web Dashboard API Server
// PURPOSE: REST API for evidence wallet and neurorights management
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const { body, validationResult } = require('express-validator');
const winston = require('winston');
const { EvidenceDashboard } = require('./evidence-dashboard');
const { NeurorightsClient } = require('./neurorights-client');

// Configuration
const PORT = process.env.PORT || 3000;
const CONFIG = {
  OWNER_DID: 'did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7',
  SAFETY_KERNEL_REF: 'VitalNetSafetyKernel:1.0.0',
  NEURORIGHTS_POLICY: 'AugmentedHumanRights:v1'
};

// Logger setup
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  transports: [
    new winston.transports.File({ filename: 'logs/server-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/server-combined.log' })
  ]
});

if (process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.simple()
  }));
}

// Initialize core services
const evidenceDashboard = new EvidenceDashboard();
const neurorightsClient = new NeurorightsClient();

// Express app setup
const app = express();

// Security middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"]
    }
  }
}));

app.use(cors({
  origin: process.env.ALLOWED_ORIGINS?.split(',') || ['http://localhost:3000'],
  credentials: true
}));

app.use(express.json());

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100 // limit each IP to 100 requests per windowMs
});
app.use('/api/', limiter);

// Request logging middleware
app.use((req, res, next) => {
  logger.info('API Request', {
    method: req.method,
    path: req.path,
    ip: req.ip,
    timestamp: new Date().toISOString()
  });
  next();
});

// ============================================================================
// API ROUTES
// ============================================================================

/**
 * GET /api/v1/health
 * Health check endpoint
 */
app.get('/api/v1/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    owner_did: CONFIG.OWNER_DID,
    compliance: ['GDPR', 'HIPAA', 'EU-AI-Act-2024', 'Neurorights-Charter-v1']
  });
});

/**
 * GET /api/v1/wallet/:owner_did
 * Get evidence wallet for an owner
 */
app.get('/api/v1/wallet/:owner_did', (req, res) => {
  try {
    const { owner_did } = req.params;

    // Neurorights check: equal protection regardless of BCI status
    neurorightsClient.verifyEqualProtection(owner_did, false);

    const wallet = evidenceDashboard.getWalletSummary(owner_did);
    res.json({ success: true, wallet });
  } catch (error) {
    logger.error('Failed to get wallet', { error: error.message });
    res.status(404).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/v1/evidence
 * Add evidence record to wallet
 */
app.post('/api/v1/evidence', [
  body('owner_did').notEmpty().withMessage('owner_did is required'),
  body('evidence_type').notEmpty().withMessage('evidence_type is required'),
  body('metric').notEmpty().withMessage('metric is required'),
  body('delta').isNumeric().withMessage('delta must be numeric'),
  body('corridor').notEmpty().withMessage('corridor is required')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ success: false, errors: errors.array() });
    }

    const result = await evidenceDashboard.addEvidenceRecord(
      req.body.owner_did,
      req.body
    );

    res.json({ success: true, ...result });
  } catch (error) {
    logger.error('Failed to add evidence record', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * GET /api/v1/living-index
 * Get living evidence index status
 */
app.get('/api/v1/living-index', (req, res) => {
  try {
    const status = evidenceDashboard.getLivingIndexStatus();
    res.json({ success: true, status });
  } catch (error) {
    logger.error('Failed to get living index', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/v1/audit
 * Run evidence audit
 */
app.post('/api/v1/audit', [
  body('control_paths').isArray().withMessage('control_paths must be an array')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ success: false, errors: errors.array() });
    }

    const result = await evidenceDashboard.runAudit(req.body.control_paths);
    res.json({ success: true, ...result });
  } catch (error) {
    logger.error('Audit failed', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * GET /api/v1/neurorights/policy
 * Get neurorights policy
 */
app.get('/api/v1/neurorights/policy', (req, res) => {
  try {
    const policy = neurorightsClient.getPolicy();
    res.json({ success: true, policy });
  } catch (error) {
    logger.error('Failed to get policy', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/v1/neurorights/consent/register
 * Register consent with biometric verification
 */
app.post('/api/v1/neurorights/consent/register', [
  body('owner_did').notEmpty().withMessage('owner_did is required'),
  body('biometric_token').notEmpty().withMessage('biometric_token is required')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ success: false, errors: errors.array() });
    }

    const result = await neurorightsClient.registerConsentWithBiometric(
      req.body.owner_did,
      req.body.biometric_token
    );

    res.json({ success: true, ...result });
  } catch (error) {
    logger.error('Failed to register consent', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/v1/neurorights/consent/revoke
 * Revoke consent
 */
app.post('/api/v1/neurorights/consent/revoke', [
  body('owner_did').notEmpty().withMessage('owner_did is required')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ success: false, errors: errors.array() });
    }

    const result = await neurorightsClient.revokeConsent(req.body.owner_did);
    res.json({ success: true, ...result });
  } catch (error) {
    logger.error('Failed to revoke consent', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/v1/neurorights/violation-report
 * Report a neurorights violation
 */
app.post('/api/v1/neurorights/violation-report', [
  body('violation_type').notEmpty().withMessage('violation_type is required'),
  body('details').notEmpty().withMessage('details is required'),
  body('reporter_did').notEmpty().withMessage('reporter_did is required')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ success: false, errors: errors.array() });
    }

    const result = await neurorightsClient.reportViolation(
      req.body.violation_type,
      req.body.details,
      req.body.reporter_did
    );

    res.json({ success: true, ...result });
  } catch (error) {
    logger.error('Failed to report violation', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * POST /api/v1/consciousness-preservation/request
 * Request consciousness preservation
 */
app.post('/api/v1/consciousness-preservation/request', [
  body('owner_did').notEmpty().withMessage('owner_did is required'),
  body('bci_device_id').notEmpty().withMessage('bci_device_id is required')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ success: false, errors: errors.array() });
    }

    const result = await neurorightsClient.requestConsciousnessPreservation(
      req.body.owner_did,
      req.body.bci_device_id
    );

    res.json({ success: true, ...result });
  } catch (error) {
    logger.error('Failed to request consciousness preservation', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

/**
 * GET /api/v1/neurorights/audit-log
 * Get neurorights audit log
 */
app.get('/api/v1/neurorights/audit-log', (req, res) => {
  try {
    const audit_log = neurorightsClient.getAuditLog();
    res.json({ success: true, audit_log });
  } catch (error) {
    logger.error('Failed to get audit log', { error: error.message });
    res.status(500).json({ success: false, error: error.message });
  }
});

// ============================================================================
// ERROR HANDLING
// ============================================================================

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    success: false,
    error: 'Not Found',
    message: 'The requested resource does not exist'
  });
});

// Global error handler
app.use((err, req, res, next) => {
  logger.error('Unhandled error', {
    error: err.message,
    stack: err.stack,
    path: req.path
  });

  res.status(500).json({
    success: false,
    error: 'Internal Server Error',
    message: process.env.NODE_ENV === 'production'
      ? 'An unexpected error occurred'
      : err.message
  });
});

// ============================================================================
// SERVER STARTUP
// ============================================================================

app.listen(PORT, () => {
  logger.info('Aletheion Web Dashboard Server started', {
    port: PORT,
    owner_did: CONFIG.OWNER_DID,
    safety_kernel: CONFIG.SAFETY_KERNEL_REF,
    neurorights_policy: CONFIG.NEURORIGHTS_POLICY
  });

  console.log(`
╔═══════════════════════════════════════════════════════════════╗
║           ALETHEION GOD-CITY WEB DASHBOARD                    ║
║                                                               ║
║  Version: 1.0.0                                               ║
║  Owner DID: ${CONFIG.OWNER_DID}
║  Safety Kernel: ${CONFIG.SAFETY_KERNEL_REF}
║  Neurorights Policy: ${CONFIG.NEURORIGHTS_POLICY}
║                                                               ║
║  Server running on: http://localhost:${PORT}
║  API Documentation: http://localhost:${PORT}/api/v1/docs
║                                                               ║
║  Compliance: GDPR, HIPAA, EU AI Act 2024, Neurorights v1     ║
╚═══════════════════════════════════════════════════════════════╝
  `);
});

module.exports = app;
