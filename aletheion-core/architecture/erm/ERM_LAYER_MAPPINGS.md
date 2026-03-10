# ERM 5-Layer Architecture Specification for Aletheion

**Version:** 1.0.0  
**Status:** Canonical Reference  
**Last Updated:** 2026-03-11  
**Maintainer:** Aletheion Core Architecture Team  
**Compliance:** ALE-COMP-CORE v1.0  

---

## Executive Summary

This document defines the authoritative five-layer Enterprise Reference Model (ERM) architecture for Aletheion. Every module, workflow, and code file in the Aletheion repository MUST be placed within one of these five layers. This specification resolves all ambiguity regarding directory structure, module placement, and architectural responsibility boundaries.

**Critical Constraint:** This architecture operates on state models and operational mirrors only. No simulation layers, no predictive twins, no speculative modeling. All layers reflect actual, measured, verified state from physical sensors, citizen inputs, and treaty-bound operations.

---

## Layer 1: Physical Interface Layer (PIL)

**Directory Prefix:** `aletheion-phy/`  
**Primary Languages:** Rust, C++  
**Responsibility:** Direct hardware interaction, sensor ingestion, actuator control  

### Scope
- Sensor driver implementations (temperature, pressure, flow, air quality, biosignal)
- Actuator command interfaces (valves, pumps, lights, locks, displays)
- Firmware orchestration for edge devices (STM32, ESP32, Raspberry Pi)
- Real-time data acquisition with microsecond timestamping
- Hardware abstraction layers for vendor-agnostic device support

### Mandatory Interfaces
```rust
// All PIL modules must implement this trait
pub trait PhysicalInterface {
    fn read_sensor(&self, sensor_id: &str) -> Result<SensorReading, HardwareError>;
    fn write_actuator(&self, actuator_id: &str, command: &ActuatorCommand) -> Result<(), HardwareError>;
    fn get_calibration_state(&self) -> CalibrationStatus;
}
```

### Compliance Requirements
- All sensor readings must include BirthSignId propagation
- All actuator commands must pass through ALE-COMP-CORE rule-check before execution
- Hardware error states must be logged to S6 (Record) stage immediately
- No caching of physical state beyond 500ms without re-verification

### Example Module Paths
- `/aletheion-phy/water/sensors/flow_meter_driver.rs`
- `/aletheion-phy/thermal/actuators/valve_controller.cpp`
- `/aletheion-phy/air/quality/pm25_sensor_interface.rs`

---

## Layer 2: Data Sovereignty Layer (DSL)

**Directory Prefix:** `aletheion-dsl/`  
**Primary Languages:** Rust, ALN  
**Responsibility:** Identity management, encryption, data residency, consent enforcement  

### Scope
- Decentralized Identity (DID) wallet implementations
- Biometric signature binding (biosignal-collector region-nodes)
- Homomorphic encryption for privacy-preserving analytics
- Data residency enforcement (local-first storage)
- Consent capsule management (neurorights, FPIC, BioticTreaties)
- Zero-knowledge proof generation for compliance verification

### Mandatory Interfaces
```rust
// All DSL modules must implement this trait
pub trait DataSovereignty {
    fn bind_identity(&self, citizen_id: &DID, biosignature: &BioSignature) -> Result<IdentityBinding, SovereigntyError>;
    fn encrypt_payload(&self, data: &Payload, consent_capsule: &ConsentCapsule) -> Result<EncryptedData, EncryptionError>;
    fn verify_residency(&self, data_location: &StorageLocation) -> Result<bool, ResidencyError>;
    fn audit_access(&self, access_request: &AccessRequest) -> Result<AuditLog, AuditError>;
}
```

### Compliance Requirements
- All citizen data must be encrypted at rest and in transit
- Neural data must never leave citizen-controlled storage without explicit consent
- FPIC verification must occur before any Indigenous land data access
- BioticTreaty compliance must be verified before any ecological data processing
- All encryption must use post-quantum secure algorithms (non-blacklisted)

### Example Module Paths
- `/aletheion-dsl/identity/did_wallet_manager.rs`
- `/aletheion-dsl/consent/neurorights_capsule_validator.aln`
- `/aletheion-dsl/encryption/pq_crypto_wrapper.cpp`
- `/aletheion-dsl/audit/access_log_generator.rs`

---

## Layer 3: Workflow Orchestration Layer (WOL)

**Directory Prefix:** `aletheion-wol/`  
**Primary Languages:** Rust, Lua, JavaScript  
**Responsibility:** Seven-stage workflow spine execution, SMART-Chain routing, stage contract enforcement  

### Scope
- S1-S7 stage implementation for all city workflows (water, thermal, waste, mobility, governance)
- Decision envelope routing between stages
- BirthSignId propagation through workflow chains
- Workflow state machine management
- Parallel workflow coordination
- Error recovery and escalation protocols

### Mandatory Interfaces
```rust
// All WOL modules must implement the seven-stage trait chain
pub trait SenseStage { fn sense(&self, context: &WorkflowContext) -> Result<DataPacket, SenseError>; }
pub trait ModelStage { fn model(&self, data: &DataPacket) -> Result<ModelOutput, ModelError>; }
pub trait AllocateStage { fn allocate(&self, output: &ModelOutput) -> Result<AllocationDecision, AllocationError>; }
pub trait RuleCheckStage { fn check(&self, decision: &AllocationDecision) -> Result<RuleResult, ComplianceError>; }
pub trait ActuateStage { fn actuate(&self, result: &RuleResult) -> Result<ActuationStatus, ActuationError>; }
pub trait RecordStage { fn record(&self, status: &ActuationStatus) -> Result<RecordId, RecordError>; }
pub trait TalkBackStage { fn notify(&self, record_id: &RecordId) -> Result<NotificationStatus, NotificationError>; }
```

### Compliance Requirements
- Every workflow must implement all seven stages (no shortcuts)
- BirthSignId must be present in every decision envelope
- ALE-COMP-CORE rule-check (S4) is mandatory and cannot be bypassed
- All stage transitions must be logged with cryptographic timestamps
- Workflow manifests must declare all dependencies and stage contracts

### Example Module Paths
- `/aletheion-wol/water-thermal/s1_sense/mod.rs`
- `/aletheion-wol/water-thermal/s2_model/mod.rs`
- `/aletheion-wol/water-thermal/s3_allocate/mod.lua`
- `/aletheion-wol/water-thermal/s4_rulecheck/mod.aln`
- `/aletheion-wol/water-thermal/s5_actuate/mod.rs`
- `/aletheion-wol/water-thermal/s6_record/mod.js`
- `/aletheion-wol/water-thermal/s7_talkback/mod.kt`
- `/aletheion-wol/water-thermal/manifest.yaml`

---

## Layer 4: Governance & Treaty Layer (GTL)

**Directory Prefix:** `aletheion-gtl/`  
**Primary Languages:** ALN, Rust  
**Responsibility:** Rights grammars, treaty enforcement, Indigenous FPIC, BioticTreaties, neurorights  

### Scope
- ALN rights grammar definitions and parsers
- Birth-Sign model creation and propagation rules
- Googolswarm decision envelope schema validation
- Indigenous land rights verification (Akimel O'odham, Piipaash)
- BioticTreaty compliance checking (bee corridors, tree protection, marine species)
- Neurorights firewall enforcement (no coercive channels, no neural data commercialization)
- ALE-COMP-CORE rule definitions and validators

### Mandatory Interfaces
```rust
// All GTL modules must implement this trait
pub trait GovernanceEnforcement {
    fn validate_treaty(&self, action: &ProposedAction, treaty_id: &TreatyId) -> Result<TreatyCompliance, TreatyError>;
    fn verify_fpic(&self, land_zone: &LandZone, action: &ProposedAction) -> Result<FPICStatus, FPICError>;
    fn check_biotic_corridor(&self, location: &GeoCoordinate, action: &ProposedAction) -> Result<CorridorStatus, CorridorError>;
    fn enforce_neurorights(&self, data_request: &NeuralDataRequest) -> Result<NeurorightsCompliance, NeurorightsError>;
    fn generate_governance_proof(&self, decision: &GovernanceDecision) -> Result<GovernanceProof, ProofError>;
}
```

### Compliance Requirements
- All GTL rules are immutable once deployed (no rollbacks, only forward-compatible upgrades)
- FPIC verification must occur before any action affecting Indigenous territories
- BioticTreaty checks must verify bee-weighted polytopes before any construction or land modification
- Neurorights enforcement must block any subliminal stimuli, manipulative timing, or forced consent
- All governance decisions must generate cryptographic proof for audit

### Example Module Paths
- `/aletheion-gtl/grammar/aln_rights_grammar.ebnf`
- `/aletheion-gtl/birthsign/birth_sign_model.schema.json`
- `/aletheion-gtl/treaties/biotic_treaty_bee_corridor.aln`
- `/aletheion-gtl/fpic/akimel_oodham_land_verification.rs`
- `/aletheion-gtl/neurorights/consent_firewall_validator.rs`
- `/aletheion-gtl/proof/governance_proof_generator.cpp`

---

## Layer 5: Citizen Interface Layer (CIL)

**Directory Prefix:** `aletheion-cil/`  
**Primary Languages:** Kotlin, JavaScript, Rust  
**Responsibility:** User-facing applications, accessibility, grievance interfaces, consent management  

### Scope
- Mobile applications (Android/Kotlin) for citizen interaction
- Web dashboards (JavaScript) for municipal oversight
- Accessibility interfaces (WCAG 2.2 AAA compliance)
- Grievance submission and tracking systems
- Consent management UI (neurorights, data sharing, FPIC)
- Multi-language support (English, Spanish, O'odham)
- Offline-capable operation with sync protocols

### Mandatory Interfaces
```kotlin
// All CIL modules must implement this interface
interface CitizenInterface {
    fun submitGrievance(grievance: GrievanceRequest): Result<GrievanceId, SubmissionError>
    fun manageConsent(consentType: ConsentType, action: ConsentAction): Result<ConsentStatus, ConsentError>
    fun displayAccessibility(content: AccessibleContent): Result<DisplayStatus, DisplayError>
    fun syncOfflineData(localData: LocalDataStore): Result<SyncStatus, SyncError>
    fun verifyIdentity(biosignature: BioSignature): Result<IdentityStatus, IdentityError>
}
```

### Compliance Requirements
- All interfaces must be WCAG 2.2 AAA compliant
- Consent management must be user-initiated and consent-gated (no pre-checked boxes)
- Grievance systems must provide tracking IDs and resolution timelines
- Offline operation must be fully functional for 72+ hours without connectivity
- All citizen data displayed must be encrypted and bound to user DID
- No behavioral prediction for advertising or commercial purposes

### Example Module Paths
- `/aletheion-cil/mobile/android/consent_manager.kt`
- `/aletheion-cil/web/dashboard/grievance_tracker.js`
- `/aletheion-cil/accessibility/screen_reader_optimizer.kt`
- `/aletheion-cil/offline/sync_protocol_handler.rs`
- `/aletheion-cil/language/oodham_interface_translator.js`

---

## Cross-Layer Communication Protocol

### Decision Envelope Structure
All inter-layer communication MUST use the Googolswarm-style decision envelope:

```json
{
  "envelope_version": "1.0.0",
  "decision_id": "uuid-v4-strict",
  "birth_sign_id": "cryptographic-signature-chain",
  "source_layer": "PIL|DSL|WOL|GTL|CIL",
  "target_layer": "PIL|DSL|WOL|GTL|CIL",
  "workflow_stage": "S1|S2|S3|S4|S5|S6|S7",
  "payload": { /* workflow-specific data */ },
  "governance_footprint": {
    "neurorights_compliance": "verified|pending|failed",
    "biotic_treaty_check": "verified|pending|failed",
    "fpic_consent": "verified|pending|not_applicable",
    "ale_comp_core_hash": "cryptographic-proof-hash"
  },
  "timestamp": "ISO8601-microsecond-precision",
  "cryptographic_signature": "post-quantum-secure-signature"
}
```

### Layer Dependency Rules
1. **PIL** may only communicate with **DSL** and **WOL** (never directly to GTL or CIL)
2. **DSL** may communicate with all layers (identity is universal)
3. **WOL** may communicate with **PIL**, **DSL**, **GTL** (orchestration hub)
4. **GTL** may communicate with **WOL** and **DSL** (governance enforcement)
5. **CIL** may only communicate with **WOL** and **DSL** (citizen isolation from hardware)

### BirthSignId Propagation Requirements
- Every decision envelope MUST contain a valid BirthSignId
- BirthSignId cannot be modified, only propagated
- BirthSignId must be cryptographically verifiable at each stage
- BirthSignId loss or corruption triggers immediate S6 (Record) logging and workflow halt

---

## Module Placement Decision Tree

```
Is this module interacting directly with hardware sensors or actuators?
├── YES → Layer 1 (PIL) → /aletheion-phy/
└── NO → Does this module manage identity, encryption, or consent?
    ├── YES → Layer 2 (DSL) → /aletheion-dsl/
    └── NO → Does this module implement S1-S7 workflow stages?
        ├── YES → Layer 3 (WOL) → /aletheion-wol/
        └── NO → Does this module enforce treaties, rights, or FPIC?
            ├── YES → Layer 4 (GTL) → /aletheion-gtl/
            └── NO → Is this module user-facing (mobile, web, accessibility)?
                ├── YES → Layer 5 (CIL) → /aletheion-cil/
                └── NO → REJECT: Module does not fit ERM architecture
```

---

## Compliance Enforcement

### CI/CD Validation Rules
All pull requests MUST pass the following checks before merge:

1. **Directory Structure Validation:** File path must match ERM layer prefix
2. **Language Compliance:** Only Rust, C++, ALN, Lua, JavaScript, Kotlin allowed
3. **Blacklist Scan:** No blacklisted algorithms, libraries, or patterns
4. **Manifest Presence:** WOL modules must include manifest.yaml
5. **BirthSignId Usage:** All workflows must import and use birthsign propagation
6. **ALE-COMP-CORE Integration:** S4 rule-check must be present in all workflows
7. **Documentation Ratio:** Minimum 1:3 code-to-documentation lines

### Automated Rejection Criteria
- Any file containing "digital-twin", "simulation", "predictive-model" (except physics-informed state mirrors)
- Any file using blacklisted cryptographic algorithms
- Any file without proper ERM layer directory prefix
- Any WOL module without all seven stage directories
- Any GTL module without treaty validation tests

---

## Version Control and Evolution

### ERM Architecture Versioning
- This document is versioned using semantic versioning (MAJOR.MINOR.PATCH)
- MAJOR changes require community referendum and Indigenous consultation
- MINOR changes require ALE-COMP-CORE approval and stake multisig
- PATCH changes require core architecture team approval only

### Evolution Proposal Process
All changes to ERM architecture must follow this process:

1. Submit EvolutionProposalRecord to `.evolve.jsonl`
2. sovereigntycore evaluation with stake multisig
3. Hex-stamped proof generation
4. EVOLVE token verification (on-device, repo-local)
5. Community review period (minimum 72 hours)
6. Merge approval with cryptographic signatures

---

## References

- Aletheion Core Repository: https://github.com/Doctor0Evil/Aletheion
- ALE-COMP-CORE Specification: `/aletheion-core/compliance/ALE_COMP_CORE_SPEC.md`
- Birth-Sign Model Schema: `/aletheion-gtl/birthsign/birth_sign_model.schema.json`
- ALN Rights Grammar: `/aletheion-gtl/grammar/aln_rights_grammar.ebnf`
- Googolswarm Envelope Schema: `/aletheion-gtl/envelope/decision_envelope.schema.json`

---

**Document Status:** CANONICAL  
**Next Review:** 2026-06-11  
**Approved By:** Aletheion Core Architecture Team  
**Indigenous Consultation:** Akimel O'odham and Piipaash representatives consulted  
**BioticTreaty Compliance:** Verified for bee corridors and tree protection protocols
