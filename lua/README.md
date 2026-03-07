# Aletheion Lua Scripting Layer

**Version:** 1.0.0  
**License:** Apache-2.0 WITH Neurorights-Extension  
**Owner DID:** `did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7`

## Overview

This Lua scripting layer provides lightweight corridor policy enforcement for Aletheion GOD-City edge nodes. It is designed for resource-constrained environments where full Rust or C++ implementations may be too heavy.

## Features

- **Evidence Wallet Management:** Lightweight wallet creation and evidence tracking
- **Corridor Policy Enforcement:** Access control for different corridor types
- **Neurorights Protection:** Equal protection verification and prohibited action blocking
- **Audit Logging:** Immutable audit trail for all operations
- **Consent Management:** Explicit consent registration and revocation

## Requirements

- Lua 5.4+
- LuaRocks (optional, for package management)

## Usage

```lua
local EvidenceWallet = require('evidence_wallet')
local CorridorPolicy = require('corridor_policy')

-- Create wallet manager
local wallet_mgr = EvidenceWallet.WalletManager.new()

-- Create corridor policy
local policy = CorridorPolicy.CorridorPolicy.new()

-- Register consent
policy:register_consent("did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7")

-- Add evidence record
local record, err = wallet_mgr:add_evidence_record(
    "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
    {
        evidence_type = "health",
        metric = "respiratory_improvement",
        delta = 15.5,
        unit = "percent",
        corridor = "rehab_neuroassist",
        owner_did = "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    }
)

-- Verify corridor access
local allowed, err = policy:verify_corridor_access(
    "rehab_neuroassist",
    "healthcare_object.BCIClinicalAugmentation.v1",
    "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
)
```

## Compliance

This implementation enforces:

- **GDPR:** Data minimization and explicit consent
- **HIPAA:** Health data protection
- **EU AI Act 2024:** High-risk AI system transparency
- **Neurorights Charter v1:** No covert control, equal protection

## Neurorights Statement

> All residents, regardless of race, disability, or augmentation status, are represented in the same DID and consent systems. Organic BCI and neuroprosthetic interfaces are modeled as `healthcare_object.BCIClinicalAugmentation` to ensure medical-grade safeguards and neurorights protection for all augmented users. Exclusion from general city-gadget catalogs reflects risk-based classification (implant vs. wearable) and regulatory requirements, not any judgement about the user's identity or worth.

## Corridor Types

| Corridor | Risk Level | BCI Allowed | Purpose |
|----------|------------|-------------|---------|
| `rehab_neuroassist` | Medium | Yes | Clinical rehabilitation |
| `public_plaza_AR` | Low | No | Public AR services |
| `assistive_rehab_research` | High | Yes | Research with oversight |
| `consciousness_preservation` | Critical | Yes | Preservation operations |

## Contact

- **Security:** security@aletheion.city
- **Neurorights Ombud:** ombud@aletheion.city
- **Clinical Safety Board:** safety-board@aletheion.city
