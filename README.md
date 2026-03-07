# Aletheion GOD-City: Evidence Core and Living Index

**Version:** 1.0.0  
**License:** Apache-2.0 WITH Neurorights-Extension  
**Owner DID:** `did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7`  
**Compliance:** GDPR, HIPAA, EU AI Act 2024, FCC Part 15, Neurorights Charter v1

## Overview

Aletheion is the Evidence Core GOD-City archetype, providing zero-hidden-state governance for smart-city infrastructure. Every claim, action, and healthcare intervention is anchored to an immutable, DID-signed ROW (Record-of-Work) ledger with complete ALN (Augmented Language Network) evidence chains.

## Repository Structure

| Directory | Language | Purpose |
|-----------|----------|---------|
| `aln/` | ALN | Schemas, ROW entries, corridor catalogs |
| `rust/` | Rust | Core evidence ledger and neurorights guard |
| `kotlin/` | Kotlin/Android | Mobile evidence wallet and BCI manager |
| `javascript/` | JavaScript/Node.js | Web dashboard and API server |
| `cpp/` | C++ | High-performance edge computing core |
| `lua/` | Lua | Lightweight corridor policy enforcement |
| `mojo/` | Mojo | AI-accelerated evidence verification |
| `docs/` | Markdown | Architecture, policy, and deployment docs |

## Key Features

- **Living Evidence Index:** Maps spec clauses → tests → missions → eco metrics → ledger entries
- **Personal Evidence Wallets:** Augmented citizens access verifiable health and eco-improvement records
- **Clinical Decision Support:** Only surfaces interventions with complete ALN evidence chains
- **Neurorights Protection:** All organic BCI interfaces governed by `healthcare_object.BCIClinicalAugmentation.v1`
- **Consciousness Preservation:** Optional preservation for event-of-Death continuity with Clinical Safety Board approval

## Inclusion Statement

> All residents, regardless of race, disability, or augmentation status, are represented in the same DID and consent systems. Organic BCI and neuroprosthetic interfaces are modeled as `healthcare_object.BCIClinicalAugmentation` to ensure medical-grade safeguards and neurorights protection for all augmented users. Exclusion from general city-gadget catalogs reflects risk-based classification (implant vs. wearable) and regulatory requirements, not any judgement about the user's identity or worth.

## Deviceless and Organically-Integrated Protection

> Deviceless cybernetics and organically-integrated components receive equal protection under this policy. The nature of integration (wearable, implant, organic, deviceless) does not affect neurorights status. Your biophysical data is protected under medical-grade safeguards, and consciousness preservation rights are available with explicit consent and Clinical Safety Board approval.

## Quick Start

```bash
# Clone repository
git clone https://github.com/aletheion-god-city/aletheion.git
cd aletheion

# Build Rust core
cd rust && cargo build --release

# Build C++ edge core
cd ../cpp && mkdir build && cd build && cmake .. && make

# Run tests
cargo test --all
ctest --output-on-failure

# Deploy evidence core
cargo run --release -- --mode=production --corridor=rehab_neuroassist
```

## Compliance & Audit

All code paths are subject to continuous audit via `VitalNetSafetyKernel`. Any attempt to treat organic BCIs as generic gadgets fails the build and raises a neurorights incident.

```bash
# Run compliance audit
cargo run -- audit --policy=AugmentedHumanRights:v1
```

## Corridor Types

| Corridor | Risk Level | BCI Allowed | Purpose |
|----------|------------|-------------|---------|
| `rehab_neuroassist` | Medium | Yes | Clinical rehabilitation |
| `public_plaza_AR` | Low | No | Public AR services |
| `assistive_rehab_research` | High | Yes | Research with oversight |
| `consciousness_preservation` | Critical | Yes | Preservation operations |

## Healthcare Objects

| Object | Type | Purpose |
|--------|------|---------|
| `BCIClinicalAugmentation.v1` | Clinical BCI | Organically-integrated brain/body interface |
| `EvidenceWallet.v1` | Personal Wallet | Health and eco improvement records |
| `NeuromorphicWearable` | Wearable | External neuromorphic sensors |
| `PhysicalProsthesis` | Medical | Physical augmentation devices |

## Contact

- **Security:** security@aletheion.city
- **Neurorights Ombud:** ombud@aletheion.city
- **Clinical Safety Board:** safety-board@aletheion.city
- **Consciousness Preservation:** preservation@aletheion.city

---

*This repository is part of the GOD-City framework for ecologically-helpful, neurorights-protective smart-city infrastructure. All augmented citizens receive equal protection regardless of integration type.*
