# Aletheion Web Dashboard

**Version:** 1.0.0  
**License:** Apache-2.0 WITH Neurorights-Extension  
**Owner DID:** `did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7`

## Overview

This JavaScript/Node.js web dashboard provides augmented citizens, city operators, and clinical safety boards with secure access to the Aletheion evidence ledger, neurorights management, and system audit tools.

## Features

- **Evidence Dashboard:** View and manage evidence records with ROW references
- **Neurorights Client:** Consent management, violation reporting, and safety kernel enforcement
- **Living Index:** Audit tool for undocumented behaviors and evidence completeness
- **Consciousness Preservation:** Request workflow with Clinical Safety Board approval
- **REST API:** Full API for integration with other systems

## Installation

```bash
# Install dependencies
npm install

# Set environment variables
cp .env.example .env
# Edit .env with your configuration

# Start development server
npm run dev

# Start production server
npm start

# Run tests
npm test

# Run audit tool
npm run audit
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/health` | Health check |
| GET | `/api/v1/wallet/:owner_did` | Get evidence wallet |
| POST | `/api/v1/evidence` | Add evidence record |
| GET | `/api/v1/living-index` | Get living index status |
| POST | `/api/v1/audit` | Run evidence audit |
| GET | `/api/v1/neurorights/policy` | Get neurorights policy |
| POST | `/api/v1/neurorights/consent/register` | Register consent |
| POST | `/api/v1/neurorights/consent/revoke` | Revoke consent |
| POST | `/api/v1/neurorights/violation-report` | Report violation |
| POST | `/api/v1/consciousness-preservation/request` | Request consciousness preservation |

## Compliance

This dashboard enforces:

- **GDPR:** Data minimization, explicit consent, right to erasure
- **HIPAA:** Encrypted health data, audit logging
- **EU AI Act 2024:** High-risk AI system transparency
- **Neurorights Charter v1:** No covert control, equal protection

## Neurorights Statement

> All residents, regardless of race, disability, or augmentation status, are represented in the same DID and consent systems. Organic BCI and neuroprosthetic interfaces are modeled as `healthcare_object.BCIClinicalAugmentation` to ensure medical-grade safeguards and neurorights protection for all augmented users. Exclusion from general city-gadget catalogs reflects risk-based classification (implant vs. wearable) and regulatory requirements, not any judgement about the user's identity or worth.

## Security

- **Rate Limiting:** 100 requests per 15 minutes per IP
- **Helmet.js:** Security headers (CSP, X-Frame-Options, etc.)
- **Audit Logging:** All actions logged to immutable ROW ledger
- **Biometric Verification:** Required for consent changes

## Contact

- **Security:** security@aletheion.city
- **Neurorights Ombud:** ombud@aletheion.city
- **Clinical Safety Board:** safety-board@aletheion.city
