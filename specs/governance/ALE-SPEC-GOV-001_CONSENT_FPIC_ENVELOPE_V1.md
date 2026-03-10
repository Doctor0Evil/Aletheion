# ALE-SPEC-GOV-001: Consent & FPIC Envelope Specification v1.0
**Repository:** `github.com/Doctor0Evil/Aletheion`  
**Path:** `/specs/governance/ALE-SPEC-GOV-001_CONSENT_FPIC_ENVELOPE_V1.md`  
**Status:** `ACTIVE` | **Version:** `1.0.0` | **ERM-Layers:** `L1-L5`  
**Treaties:** `UNDRIP`, `BioticTreaty-Phoenix`, `Neurorights-Constitutional-Base`, `Arizona-Tribal-Compacts`  
**Languages:** `ALN`, `Rust(2024)`, `Lua`, `Kotlin`  
**Security:** `PQC-Compliant`, `Offline-First`, `Zero-Knowledge-Proof-Enabled`  

## 1.0 Purpose & Scope
This specification defines the machine-readable envelope for Free, Prior, and Informed Consent (FPIC) and Individual Biosignal Consent within Aletheion. It governs Workflows 18–25 (Governance Keystone) and acts as the primary constraint for all data ingestion (Workflows 1–17). No sensor data shall be processed without a valid `ConsentEnvelope` attached to the `DataProvenance` header. This document binds legal obligations (Phoenix City Code, Tribal Law) to executable logic.

## 2.0 ALN Schema: ConsentEnvelope
The `ConsentEnvelope` is the atomic unit of sovereignty. It must be signed by a DID-Bound Brain Identity (BI) or Tribal Authority Key.
