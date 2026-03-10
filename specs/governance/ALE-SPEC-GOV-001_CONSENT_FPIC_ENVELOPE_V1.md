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

```aln
// ALN Schema Definition: ConsentEnvelope v1
// Destination: /src/aln/schemas/consent_envelope_v1.aln

namespace Aletheion.Governance.Consent;

struct ConsentEnvelope {
    // Immutable Identity Headers
    grantor_did: DID_URI,                  // Decentralized Identifier (W3C Standard)
    grantor_type: Enum { Individual, Tribal_Council, Biotic_Steward, Augmented_Citizen },
    biometric_hash: PQC_Secure_Hash,       // Post-Quantum Secure Hash of Biosignal Signature
    
    // Scope & Permissions
    scope_vector: Vec<PermissionScope>,    // Granular permissions (Sensor, Feature, Purpose)
    jurisdiction: Enum { Phoenix_Muni, Akimel_Oodham, Piipaash, State_AZ, Federal_US },
    treaty_bindings: Vec<TreatyID>,        // Links to BioticTreaty, UNDRIP clauses
    
    // Temporal Bounds
    issued_at: UnixTimestamp_Nano,
    valid_from: UnixTimestamp_Nano,
    expires_at: UnixTimestamp_Nano,        // Null for indefinite (requires renewal check)
    revocation_status: Enum { Active, Revoked, Suspended, Expired },
    
    // Security & Integrity
    signature_scheme: PQC_Signature_Suite, // NIST-Standardized Post-Quantum Algorithm
    signature_bytes: Byte_Array,           // Cryptographic proof of consent
    nonce: UUID_v7,                        // Prevents replay attacks
};

struct PermissionScope {
    resource_id: String,                   // e.g., "Sensor_Thermal_NorthPhoenix_004"
    action_type: Enum { Read, Write, Optimize, Infer, Share },
    purpose_code: String,                  // e.g., "Heat_Mitigation", "Medical_Emergency"
    data_retention_days: u32,              // Max storage duration (0 = Ephemeral)
    inference_allowed: bool,               // FALSE for Neurorights-Protected Data
};
