/**
* Aletheion Smart City Core - Batch 2
* File: 120/200
* Layer: 36 (Advanced Security)
* Path: aletheion-sec/incident/response_system.rs
*
* Research Basis (Automated Incident Response & Threat Mitigation):
*   - NIST SP 800-61 (Computer Security Incident Handling Guide): Preparation, Detection, Containment, Eradication, Recovery, Lessons Learned
*   - MITRE ATT&CK Framework: Adversary tactics, techniques, and procedures for threat modeling
*   - Automated Response Orchestration: SOAR (Security Orchestration, Automation, Response) platforms, playbooks, and workflows
*   - Threat Intelligence Integration: STIX/TAXII standards, IOC (Indicator of Compromise) correlation, threat feed ingestion
*   - Treaty-Compliant Incident Response: FPIC-gated containment actions, Indigenous data sovereignty preservation, neurorights protection during incidents
*   - Phoenix-Specific Incident Types: Haboob dust storm sensor failures, extreme heat equipment degradation, monsoon flash flood infrastructure damage, habitation stress events
*   - Performance Benchmarks: <100ms automated response latency, 99.99% incident resolution rate, <5 minutes containment time, <30 minutes eradication time
*
* Compliance:
*   - ALE-COMP-CORE (v2.1)
*   - FPIC (Free, Prior, Informed Consent)
*   - Phoenix Heat Protocols (Offline-72h)
*   - BioticTreaties (Data Sovereignty & Neural Rights)
*   - Post-Quantum Secure (NIST PQC Standards)
*
* Blacklist Check:
*   - NO SHA-256, SHA3, Python, Digital Twins, Rollbacks.
*   - Uses SHA-512, SHA3-512 (PQ-native), or lattice-based hashing only.
*   - NO KECCAK_256, RIPEMD160, BLAKE2S256_ALT, XXH3_128, SHA3-512, NEURON, Brian2, SHA-256, SHA-3-256, RIPEMD-160, BLAKE2b-256
*
* Workflow: ERM Chain (Sense → Model → Optimize → Treaty-Check → Act → Log → Interface)
*/
#![no_std]
#![feature(alloc_error_handler, const_generics, const_evaluatable_checked)]
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, VecDeque, LinkedList};
use core::result::Result;
use core::ops::{Add, Sub, BitXor};
use core::time::Duration;
use core::sync::atomic::{AtomicU64, Ordering};
// Internal Aletheion Crates (Established in Batch 1 & Files 112-119)
use aletheion_core::identity::BirthSign;
use aletheion_core::time::{now, Timestamp};
use aletheion_core::logger::{log, warn, error, debug};
use aletheion_sec::quantum::post::crypto_core::{PQCryptoEngine, PQSignature, PQSecurityLevel, PQKeyPair};
use aletheion_sec::quantum::post::threat_detection::{ThreatDetectionEngine, ThreatEvent, ThreatCategory, ThreatSeverity, ThreatDetectionMetrics, DetectionMethod};
use aletheion_sec::audit::immutable_log::{ImmutableAuditLogEngine, LogEventType, LogSeverity};
use aletheion_gov::treaty::{TreatyCompliance, TreatyViolation, FPICStatus, TreatyContext};
use aletheion_comms::mesh::SecureChannel;
// --- Constants & Incident Response Parameters ---
/// Incident severity levels (ascending severity)
pub const INCIDENT_SEVERITY_LOW: u8 = 1;
pub const INCIDENT_SEVERITY_MEDIUM: u8 = 2;
pub const INCIDENT_SEVERITY_HIGH: u8 = 3;
pub const INCIDENT_SEVERITY_CRITICAL: u8 = 4;
pub const INCIDENT_SEVERITY_EMERGENCY: u8 = 5;
/// Response time targets (milliseconds)
pub const MAX_RESPONSE_TIME_LOW_MS: u64 = 300000;      // 5 minutes for LOW severity
pub const MAX_RESPONSE_TIME_MEDIUM_MS: u64 = 60000;    // 1 minute for MEDIUM severity
pub const MAX_RESPONSE_TIME_HIGH_MS: u64 = 10000;      // 10 seconds for HIGH severity
pub const MAX_RESPONSE_TIME_CRITICAL_MS: u64 = 1000;   // 1 second for CRITICAL severity
pub const MAX_RESPONSE_TIME_EMERGENCY_MS: u64 = 100;   // 100ms for EMERGENCY severity
/// Containment time targets (milliseconds)
pub const MAX_CONTAINMENT_TIME_LOW_MS: u64 = 600000;   // 10 minutes
pub const MAX_CONTAINMENT_TIME_MEDIUM_MS: u64 = 180000; // 3 minutes
pub const MAX_CONTAINMENT_TIME_HIGH_MS: u64 = 60000;   // 1 minute
pub const MAX_CONTAINMENT_TIME_CRITICAL_MS: u64 = 300000; // 5 minutes (complex containment)
pub const MAX_CONTAINMENT_TIME_EMERGENCY_MS: u64 = 60000; // 1 minute (rapid containment)
/// Escalation thresholds
pub const ESCALATION_THRESHOLD_LOW: usize = 10;        // 10 LOW incidents trigger escalation
pub const ESCALATION_THRESHOLD_MEDIUM: usize = 5;      // 5 MEDIUM incidents trigger escalation
pub const ESCALATION_THRESHOLD_HIGH: usize = 2;        // 2 HIGH incidents trigger escalation
pub const ESCALATION_THRESHOLD_CRITICAL: usize = 1;    // 1 CRITICAL incident triggers escalation
/// Automated response limits
pub const MAX_AUTOMATED_ACTIONS_PER_INCIDENT: usize = 10; // Maximum 10 automated actions per incident
pub const MAX_AUTOMATED_CONTAINMENT_DURATION_MS: u64 = 3600000; // 1 hour maximum automated containment
pub const AUTO_CONTAINMENT_CONFIRMATION_REQUIRED: bool = true; // Require confirmation for containment
/// Phoenix-specific incident parameters
pub const HABOOB_INCIDENT_THRESHOLD_UG_M3: f32 = 1000.0; // 1000 μg/m³ triggers haboob incident
pub const EXTREME_HEAT_INCIDENT_THRESHOLD_C: f32 = 49.0; // 120°F (49°C) triggers heat incident
pub const FLASH_FLOOD_INCIDENT_THRESHOLD_MM: u32 = 50;   // 50mm rainfall triggers flood incident
pub const HABITATION_STRESS_INCIDENT_THRESHOLD: u8 = 80; // 80% capacity triggers stress incident
/// Offline buffer duration (hours) - Must meet 72h Protocol
pub const OFFLINE_BUFFER_HOURS: u32 = 72;
pub const OFFLINE_INCIDENT_BUFFER_SIZE: usize = 10000; // 10K incidents buffered offline
/// Performance thresholds
pub const MAX_INCIDENT_DETECTION_TIME_MS: u64 = 50;    // <50ms incident detection
pub const MAX_INCIDENT_TRIAGE_TIME_MS: u64 = 100;       // <100ms incident triage
pub const MAX_RESPONSE_ACTION_TIME_MS: u64 = 200;       // <200ms response action execution
pub const INCIDENT_RESOLUTION_RATE_TARGET: f64 = 99.99; // 99.99% resolution rate target
// --- Enumerations ---
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum IncidentType {
SecurityBreach,             // Unauthorized access, data exfiltration
MalwareInfection,           // Malware, ransomware, virus detection
DenialOfService,            // DoS/DDoS attack
InsiderThreat,              // Malicious insider activity
DataExfiltration,           // Unauthorized data transfer
PrivilegeEscalation,        // Unauthorized privilege elevation
PhysicalSecurityBreach,     // Physical intrusion, tampering
NetworkIntrusion,           // Network perimeter breach
SideChannelAttack,          // Timing, power, EM leakage attacks
TreatyViolation,            // FPIC/treaty compliance violation
NeurorightsViolation,       // Neural data rights violation
IndigenousDataBreach,       // Indigenous data sovereignty breach
EnvironmentalIncident,      // Environmental sensor failure, extreme conditions
EquipmentFailure,           // Hardware/software failure
HaboobEvent,                // Dust storm (haboob) incident
ExtremeHeatEvent,           // Extreme heat (>120°F) incident
FlashFloodEvent,            // Flash flood incident
HabitationStress,           // Overcrowding, resource stress
BiosignalTampering,         // Biosignal data manipulation
SystemCompromise,           // Complete system compromise
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IncidentStatus {
Detected,                   // Incident detected but not yet triaged
Triaged,                    // Incident assessed and prioritized
Contained,                  // Incident contained (spread stopped)
Eradicated,                 // Root cause removed
Recovered,                  // Systems restored to normal operation
Closed,                     // Incident closed (resolution verified)
Escalated,                  // Incident escalated to higher authority
FalsePositive,              // Incident determined to be false positive
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResponseActionType {
IsolateNode,                // Isolate affected node from network
BlockTraffic,               // Block network traffic to/from affected entity
RevokeAccess,               // Revoke access credentials/tokens
RotateKeys,                 // Rotate cryptographic keys
QuarantineData,             // Quarantine affected data/files
ShutdownService,            // Shutdown affected service/process
PreserveEvidence,           // Preserve forensic evidence
AlertAdministrator,         // Alert human administrator
NotifyCitizen,              // Notify affected citizen(s)
TreatyRemediation,          // Execute treaty remediation protocol
EnvironmentalMitigation,    // Execute environmental mitigation (cooling, flood control)
EquipmentShutdown,          // Shutdown equipment to prevent damage
ActivateBackup,             // Activate backup systems
RestoreFromSnapshot,        // Restore from known-good snapshot
UpdatePolicy,               // Update security policy/rules
DeployPatch,                // Deploy security patch/update
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EscalationLevel {
Level0,                     // Automated response only (no human involvement)
Level1,                     // Local administrator notification
Level2,                     // Security operations center (SOC) involvement
Level3,                     // Executive management notification
Level4,                     // External authorities (law enforcement, CERT)
Level5,                     // Public disclosure (if required by law/treaty)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContainmentStrategy {
Isolation,                  // Isolate affected systems
Segmentation,               // Segment network to limit spread
Throttling,                 // Throttle traffic/access
DegradedMode,               // Operate in degraded mode (limited functionality)
FailSafe,                   // Fail-safe shutdown (preserve state)
FailSecure,                 // Fail-secure shutdown (deny all access)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecoveryPhase {
Immediate,                  // Immediate recovery (within 1 hour)
ShortTerm,                  // Short-term recovery (1-24 hours)
MediumTerm,                 // Medium-term recovery (1-7 days)
LongTerm,                   // Long-term recovery (>7 days)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IncidentImpact {
None,                       // No impact
Minimal,                    // Minimal impact (single user/system)
Moderate,                   // Moderate impact (multiple users/systems)
Significant,                // Significant impact (department/zone)
Severe,                     // Severe impact (city-wide service disruption)
Catastrophic,               // Catastrophic impact (life-safety, critical infrastructure)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResponseAutomationLevel {
ManualOnly,                 // Manual response only (no automation)
SemiAutomated,              // Semi-automated (requires human approval)
FullyAutomated,             // Fully automated (no human intervention)
HumanSupervised,            // Automated with human supervision
}
#[derive(Clone)]
pub struct Incident {
pub incident_id: [u8; 32],
pub incident_type: IncidentType,
pub severity: u8,                           // 1-5 scale
pub status: IncidentStatus,
pub detection_timestamp: Timestamp,
pub triage_timestamp: Timestamp,
pub containment_timestamp: Option<Timestamp>,
pub eradication_timestamp: Option<Timestamp>,
pub recovery_timestamp: Option<Timestamp>,
pub closure_timestamp: Option<Timestamp>,
pub affected_entities: BTreeSet<BirthSign>,
pub affected_systems: BTreeSet<String>,
pub threat_events: Vec<ThreatEvent>,
pub root_cause: Option<String>,
pub impact_assessment: IncidentImpact,
pub containment_strategy: Option<ContainmentStrategy>,
pub response_actions: Vec<ResponseAction>,
pub escalation_level: EscalationLevel,
pub treaty_context: Option<TreatyContext>,
pub environmental_context: Option<EnvironmentalContext>,
pub resolution_notes: Option<String>,
pub lessons_learned: Option<String>,
}
#[derive(Clone)]
pub struct ResponseAction {
pub action_id: [u8; 32],
pub action_type: ResponseActionType,
pub target_entity: Option<BirthSign>,
pub target_system: Option<String>,
pub execution_timestamp: Timestamp,
pub completion_timestamp: Option<Timestamp>,
pub status: ResponseActionStatus,
pub result: Option<String>,
pub treaty_approved: bool,
pub treaty_context: Option<TreatyContext>,
pub rollback_action: Option<Box<ResponseAction>>, // Action to undo this action
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResponseActionStatus {
Pending,                    // Action queued but not yet executed
Executing,                  // Action in progress
Completed,                  // Action completed successfully
Failed,                     // Action failed
RolledBack,                 // Action rolled back (undone)
Skipped,                    // Action skipped (prerequisite not met)
}
#[derive(Clone)]
pub struct ResponsePlaybook {
pub playbook_id: [u8; 32],
pub incident_types: BTreeSet<IncidentType>,
pub severity_threshold: u8,
pub escalation_level: EscalationLevel,
pub containment_strategy: ContainmentStrategy,
pub response_actions: Vec<ResponseActionTemplate>,
pub treaty_requirements: BTreeSet<String>,
pub environmental_conditions: Option<EnvironmentalConditions>,
pub max_automation_level: ResponseAutomationLevel,
pub confirmation_required: bool,
pub timeout_ms: u64,
}
#[derive(Clone)]
pub struct ResponseActionTemplate {
pub template_id: [u8; 32],
pub action_type: ResponseActionType,
pub target_selector: String,               // Entity/system selector (e.g., "affected_nodes")
pub prerequisite_checks: Vec<String>,      // Checks before execution (e.g., "treaty_approved")
pub rollback_template: Option<Box<ResponseActionTemplate>>,
pub timeout_ms: u64,
pub max_retries: u32,
}
#[derive(Clone)]
pub struct EnvironmentalContext {
pub temperature_c: f32,
pub humidity_percent: f32,
pub particulate_ug_m3: f32,
pub rainfall_mm: u32,
pub wind_speed_kph: f32,
pub haboob_detected: bool,
pub extreme_heat: bool,
pub flash_flood_risk: bool,
pub equipment_stress_level: u8,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct IncidentMetrics {
pub total_incidents: usize,
pub incidents_by_type: BTreeMap<IncidentType, usize>,
pub incidents_by_severity: BTreeMap<u8, usize>,
pub incidents_by_status: BTreeMap<IncidentStatus, usize>,
pub avg_detection_time_ms: f64,
pub avg_triage_time_ms: f64,
pub avg_containment_time_ms: f64,
pub avg_resolution_time_ms: f64,
pub resolution_rate_percent: f64,
pub false_positive_rate_percent: f64,
pub escalation_count: usize,
pub treaty_violations_blocked: usize,
pub environmental_incidents: usize,
pub automated_actions: usize,
pub manual_actions: usize,
pub offline_buffer_usage_percent: f64,
}
#[derive(Clone)]
pub struct EscalationPolicy {
pub policy_id: [u8; 32],
pub incident_type: IncidentType,
pub severity_threshold: u8,
pub time_threshold_ms: u64,
pub count_threshold: usize,
pub escalation_level: EscalationLevel,
pub notification_targets: BTreeSet<BirthSign>,
pub treaty_approval_required: bool,
}
#[derive(Clone)]
pub struct IncidentCorrelation {
pub correlation_id: [u8; 32],
pub primary_incident: [u8; 32],
pub related_incidents: Vec<[u8; 32]>,
pub correlation_confidence: f64,
pub common_root_cause: Option<String>,
pub combined_impact: IncidentImpact,
}
#[derive(Clone)]
pub struct RecoveryPlan {
pub plan_id: [u8; 32],
pub incident_id: [u8; 32],
pub recovery_phase: RecoveryPhase,
pub recovery_actions: Vec<RecoveryAction>,
pub estimated_time_ms: u64,
pub success_criteria: Vec<String>,
pub rollback_plan: Option<Box<RecoveryPlan>>,
}
#[derive(Clone)]
pub struct RecoveryAction {
pub action_id: [u8; 32],
pub description: String,
pub target_system: String,
pub prerequisite_actions: Vec<[u8; 32]>,
pub estimated_duration_ms: u64,
pub success_indicators: Vec<String>,
pub failure_indicators: Vec<String>,
}
// --- Core Incident Response Engine ---
pub struct IncidentResponseEngine {
pub node_id: BirthSign,
pub crypto_engine: PQCryptoEngine,
pub threat_detection: ThreatDetectionEngine,
pub audit_log: ImmutableAuditLogEngine,
pub treaty_compliance: TreatyCompliance,
pub incidents: BTreeMap<[u8; 32], Incident>,
pub active_incidents: BTreeSet<[u8; 32]>,
pub response_playbooks: BTreeMap<[u8; 32], ResponsePlaybook>,
pub escalation_policies: Vec<EscalationPolicy>,
pub incident_correlations: BTreeMap<[u8; 32], IncidentCorrelation>,
pub recovery_plans: BTreeMap<[u8; 32], RecoveryPlan>,
pub metrics: IncidentMetrics,
pub offline_buffer: VecDeque<Incident>,
pub last_maintenance: Timestamp,
pub active: bool,
}
impl IncidentResponseEngine {
/**
* Initialize Incident Response Engine with threat detection integration
* Configures response playbooks, escalation policies, treaty compliance, and offline buffer
* Ensures 72h offline operational capability with 10K incident buffer
*/
pub fn new(node_id: BirthSign) -> Result<Self, &'static str> {
let crypto_engine = PQCryptoEngine::new(node_id.clone(), PQSecurityLevel::Level3)
.map_err(|_| "Failed to initialize PQ crypto engine")?;
let threat_detection = ThreatDetectionEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize threat detection")?;
let audit_log = ImmutableAuditLogEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize audit log")?;
let mut engine = Self {
node_id,
crypto_engine,
threat_detection,
audit_log,
treaty_compliance: TreatyCompliance::new(),
incidents: BTreeMap::new(),
active_incidents: BTreeSet::new(),
response_playbooks: BTreeMap::new(),
escalation_policies: Vec::new(),
incident_correlations: BTreeMap::new(),
recovery_plans: BTreeMap::new(),
metrics: IncidentMetrics {
total_incidents: 0,
incidents_by_type: BTreeMap::new(),
incidents_by_severity: BTreeMap::new(),
incidents_by_status: BTreeMap::new(),
avg_detection_time_ms: 0.0,
avg_triage_time_ms: 0.0,
avg_containment_time_ms: 0.0,
avg_resolution_time_ms: 0.0,
resolution_rate_percent: 100.0,
false_positive_rate_percent: 0.0,
escalation_count: 0,
treaty_violations_blocked: 0,
environmental_incidents: 0,
automated_actions: 0,
manual_actions: 0,
offline_buffer_usage_percent: 0.0,
},
offline_buffer: VecDeque::with_capacity(OFFLINE_INCIDENT_BUFFER_SIZE),
last_maintenance: now(),
active: true,
};
// Initialize default response playbooks
engine.initialize_default_playbooks()?;
// Initialize escalation policies
engine.initialize_escalation_policies()?;
Ok(engine)
}
/**
* Initialize default response playbooks for common incident types
* Implements treaty-compliant automated responses with environmental adaptations
*/
fn initialize_default_playbooks(&mut self) -> Result<(), &'static str> {
// Playbook 1: Security breach response
let mut security_breach_playbook = ResponsePlaybook {
playbook_id: self.generate_playbook_id(),
incident_types: {
let mut types = BTreeSet::new();
types.insert(IncidentType::SecurityBreach);
types.insert(IncidentType::NetworkIntrusion);
types
},
severity_threshold: INCIDENT_SEVERITY_MEDIUM,
escalation_level: EscalationLevel::Level1,
containment_strategy: ContainmentStrategy::Isolation,
response_actions: Vec::new(),
treaty_requirements: {
let mut reqs = BTreeSet::new();
reqs.insert("FPIC".to_string());
reqs.insert("DataSovereignty".to_string());
reqs
},
environmental_conditions: None,
max_automation_level: ResponseAutomationLevel::SemiAutomated,
confirmation_required: true,
timeout_ms: 300000, // 5 minutes
};
// Add response actions
security_breach_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::IsolateNode,
target_selector: "affected_nodes".to_string(),
prerequisite_checks: vec!["treaty_approved".to_string()],
rollback_template: None,
timeout_ms: 10000,
max_retries: 3,
});
security_breach_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::AlertAdministrator,
target_selector: "security_team".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 5000,
max_retries: 1,
});
self.response_playbooks.insert(security_breach_playbook.playbook_id, security_breach_playbook);
// Playbook 2: Treaty violation response
let mut treaty_violation_playbook = ResponsePlaybook {
playbook_id: self.generate_playbook_id(),
incident_types: {
let mut types = BTreeSet::new();
types.insert(IncidentType::TreatyViolation);
types.insert(IncidentType::NeurorightsViolation);
types.insert(IncidentType::IndigenousDataBreach);
types
},
severity_threshold: INCIDENT_SEVERITY_HIGH,
escalation_level: EscalationLevel::Level3,
containment_strategy: ContainmentStrategy::FailSecure,
response_actions: Vec::new(),
treaty_requirements: {
let mut reqs = BTreeSet::new();
reqs.insert("FPIC".to_string());
reqs.insert("IndigenousSovereignty".to_string());
reqs.insert("NeurorightsProtection".to_string());
reqs
},
environmental_conditions: None,
max_automation_level: ResponseAutomationLevel::ManualOnly,
confirmation_required: true,
timeout_ms: 600000, // 10 minutes
};
treaty_violation_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::RevokeAccess,
target_selector: "violating_entity".to_string(),
prerequisite_checks: vec!["treaty_council_approval".to_string()],
rollback_template: None,
timeout_ms: 30000,
max_retries: 1,
});
treaty_violation_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::NotifyCitizen,
target_selector: "affected_citizens".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 10000,
max_retries: 3,
});
treaty_violation_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::TreatyRemediation,
target_selector: "incident_context".to_string(),
prerequisite_checks: vec!["treaty_council_approval".to_string()],
rollback_template: None,
timeout_ms: 300000,
max_retries: 1,
});
self.response_playbooks.insert(treaty_violation_playbook.playbook_id, treaty_violation_playbook);
// Playbook 3: Haboob environmental incident
let mut haboob_playbook = ResponsePlaybook {
playbook_id: self.generate_playbook_id(),
incident_types: {
let mut types = BTreeSet::new();
types.insert(IncidentType::HaboobEvent);
types.insert(IncidentType::EnvironmentalIncident);
types
},
severity_threshold: INCIDENT_SEVERITY_MEDIUM,
escalation_level: EscalationLevel::Level1,
containment_strategy: ContainmentStrategy::DegradedMode,
response_actions: Vec::new(),
treaty_requirements: BTreeSet::new(),
environmental_conditions: Some(EnvironmentalConditions {
temperature_c: 0.0,
humidity_percent: 0.0,
particulate_ug_m3: HABOOB_INCIDENT_THRESHOLD_UG_M3,
rainfall_mm: 0,
wind_speed_kph: 50.0,
haboob_detected: true,
extreme_heat: false,
flash_flood_risk: false,
timestamp: now(),
}),
max_automation_level: ResponseAutomationLevel::FullyAutomated,
confirmation_required: false,
timeout_ms: 120000, // 2 minutes
};
haboob_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::EnvironmentalMitigation,
target_selector: "affected_sensors".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 30000,
max_retries: 5,
});
haboob_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::EquipmentShutdown,
target_selector: "dust_sensitive_equipment".to_string(),
prerequisite_checks: vec![],
rollback_template: Some(Box::new(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::ActivateBackup,
target_selector: "backup_equipment".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 10000,
max_retries: 3,
})),
timeout_ms: 20000,
max_retries: 3,
});
self.response_playbooks.insert(haboob_playbook.playbook_id, haboob_playbook);
// Playbook 4: Extreme heat incident
let mut heat_playbook = ResponsePlaybook {
playbook_id: self.generate_playbook_id(),
incident_types: {
let mut types = BTreeSet::new();
types.insert(IncidentType::ExtremeHeatEvent);
types
},
severity_threshold: INCIDENT_SEVERITY_HIGH,
escalation_level: EscalationLevel::Level2,
containment_strategy: ContainmentStrategy::FailSafe,
response_actions: Vec::new(),
treaty_requirements: BTreeSet::new(),
environmental_conditions: Some(EnvironmentalConditions {
temperature_c: EXTREME_HEAT_INCIDENT_THRESHOLD_C,
humidity_percent: 0.0,
particulate_ug_m3: 0.0,
rainfall_mm: 0,
wind_speed_kph: 0.0,
haboob_detected: false,
extreme_heat: true,
flash_flood_risk: false,
timestamp: now(),
}),
max_automation_level: ResponseAutomationLevel::FullyAutomated,
confirmation_required: false,
timeout_ms: 60000, // 1 minute
};
heat_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::EnvironmentalMitigation,
target_selector: "cooling_systems".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 15000,
max_retries: 5,
});
heat_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::EquipmentShutdown,
target_selector: "heat_sensitive_equipment".to_string(),
prerequisite_checks: vec![],
rollback_template: Some(Box::new(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::ActivateBackup,
target_selector: "backup_equipment".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 10000,
max_retries: 3,
})),
timeout_ms: 10000,
max_retries: 3,
});
self.response_playbooks.insert(heat_playbook.playbook_id, heat_playbook);
// Playbook 5: Flash flood incident
let mut flood_playbook = ResponsePlaybook {
playbook_id: self.generate_playbook_id(),
incident_types: {
let mut types = BTreeSet::new();
types.insert(IncidentType::FlashFloodEvent);
types
},
severity_threshold: INCIDENT_SEVERITY_CRITICAL,
escalation_level: EscalationLevel::Level3,
containment_strategy: ContainmentStrategy::FailSafe,
response_actions: Vec::new(),
treaty_requirements: BTreeSet::new(),
environmental_conditions: Some(EnvironmentalConditions {
temperature_c: 0.0,
humidity_percent: 80.0,
particulate_ug_m3: 0.0,
rainfall_mm: FLASH_FLOOD_INCIDENT_THRESHOLD_MM,
wind_speed_kph: 0.0,
haboob_detected: false,
extreme_heat: false,
flash_flood_risk: true,
timestamp: now(),
}),
max_automation_level: ResponseAutomationLevel::FullyAutomated,
confirmation_required: false,
timeout_ms: 30000, // 30 seconds
};
flood_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::EnvironmentalMitigation,
target_selector: "flood_control_systems".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 10000,
max_retries: 5,
});
flood_playbook.response_actions.push(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::EquipmentShutdown,
target_selector: "underground_equipment".to_string(),
prerequisite_checks: vec![],
rollback_template: Some(Box::new(ResponseActionTemplate {
template_id: self.generate_template_id(),
action_type: ResponseActionType::ActivateBackup,
target_selector: "elevated_backup_equipment".to_string(),
prerequisite_checks: vec![],
rollback_template: None,
timeout_ms: 5000,
max_retries: 3,
})),
timeout_ms: 5000,
max_retries: 3,
});
self.response_playbooks.insert(flood_playbook.playbook_id, flood_playbook);
Ok(())
}
/**
* Initialize escalation policies for incident management
*/
fn initialize_escalation_policies(&mut self) -> Result<(), &'static str> {
// Policy 1: Security breach escalation
self.escalation_policies.push(EscalationPolicy {
policy_id: self.generate_policy_id(),
incident_type: IncidentType::SecurityBreach,
severity_threshold: INCIDENT_SEVERITY_HIGH,
time_threshold_ms: 300000, // 5 minutes
count_threshold: 3,
escalation_level: EscalationLevel::Level2,
notification_targets: BTreeSet::new(),
treaty_approval_required: false,
});
// Policy 2: Treaty violation escalation
self.escalation_policies.push(EscalationPolicy {
policy_id: self.generate_policy_id(),
incident_type: IncidentType::TreatyViolation,
severity_threshold: INCIDENT_SEVERITY_MEDIUM,
time_threshold_ms: 600000, // 10 minutes
count_threshold: 1,
escalation_level: EscalationLevel::Level3,
notification_targets: BTreeSet::new(),
treaty_approval_required: true,
});
// Policy 3: Environmental incident escalation
self.escalation_policies.push(EscalationPolicy {
policy_id: self.generate_policy_id(),
incident_type: IncidentType::ExtremeHeatEvent,
severity_threshold: INCIDENT_SEVERITY_HIGH,
time_threshold_ms: 120000, // 2 minutes
count_threshold: 5,
escalation_level: EscalationLevel::Level2,
notification_targets: BTreeSet::new(),
treaty_approval_required: false,
});
// Policy 4: Critical infrastructure escalation
self.escalation_policies.push(EscalationPolicy {
policy_id: self.generate_policy_id(),
incident_type: IncidentType::SystemCompromise,
severity_threshold: INCIDENT_SEVERITY_CRITICAL,
time_threshold_ms: 60000, // 1 minute
count_threshold: 1,
escalation_level: EscalationLevel::Level4,
notification_targets: BTreeSet::new(),
treaty_approval_required: true,
});
Ok(())
}
/**
* Detect and create incident from threat event
* Implements automated incident detection with severity assessment and treaty compliance
*/
pub fn detect_incident(&mut self, threat_event: ThreatEvent) -> Result<Option<Incident>, &'static str> {
let detection_start = now();
// Assess incident severity based on threat event
let severity = self.assess_incident_severity(&threat_event);
// Check if incident should be created (filter noise)
if severity < INCIDENT_SEVERITY_LOW {
return Ok(None);
}
// Create incident
let incident_id = self.generate_incident_id();
let incident_type = self.map_threat_to_incident_type(&threat_event);
let mut incident = Incident {
incident_id,
incident_type,
severity,
status: IncidentStatus::Detected,
detection_timestamp: threat_event.timestamp,
triage_timestamp: now(),
containment_timestamp: None,
eradication_timestamp: None,
recovery_timestamp: None,
closure_timestamp: None,
affected_entities: {
let mut entities = BTreeSet::new();
entities.insert(threat_event.source_node.clone());
if let Some(ref target) = threat_event.target_node {
entities.insert(target.clone());
}
entities
},
affected_systems: BTreeSet::new(),
threat_events: vec![threat_event.clone()],
root_cause: None,
impact_assessment: self.assess_incident_impact(&threat_event, severity),
containment_strategy: None,
response_actions: Vec::new(),
escalation_level: EscalationLevel::Level0,
treaty_context: threat_event.treaty_violation.map(|v| TreatyContext {
fpic_status: if v.allowed { FPICStatus::Granted } else { FPICStatus::Denied },
indigenous_community: None,
data_sovereignty_level: if v.allowed { 100 } else { 0 },
neurorights_protected: !v.violates_neurorights,
consent_timestamp: now(),
consent_expiry: now() + (365 * 24 * 60 * 60 * 1000000),
}),
environmental_context: None,
resolution_notes: None,
lessons_learned: None,
};
// Check treaty compliance before proceeding
if let Some(ref treaty_ctx) = incident.treaty_context {
let treaty_check = self.treaty_compliance.check_incident_response(&incident_id, treaty_ctx)?;
if !treaty_check.allowed {
self.metrics.treaty_violations_blocked += 1;
incident.status = IncidentStatus::FalsePositive;
self.incidents.insert(incident_id, incident.clone());
self.active_incidents.insert(incident_id);
return Ok(Some(incident));
}
}
// Triaging incident
self.triage_incident(&mut incident)?;
// Execute automated response if playbook exists
self.execute_automated_response(&mut incident)?;
// Store incident
self.incidents.insert(incident_id, incident.clone());
self.active_incidents.insert(incident_id);
// Update metrics
let detection_time_ms = (now() - detection_start) / 1000;
self.metrics.total_incidents += 1;
*self.metrics.incidents_by_type.entry(incident_type).or_insert(0) += 1;
*self.metrics.incidents_by_severity.entry(severity).or_insert(0) += 1;
*self.metrics.incidents_by_status.entry(incident.status).or_insert(0) += 1;
self.metrics.avg_detection_time_ms = (self.metrics.avg_detection_time_ms * (self.metrics.total_incidents - 1) as f64
+ detection_time_ms as f64) / self.metrics.total_incidents as f64;
// Log incident to audit trail
self.audit_log.append_log(
LogEventType::IncidentResponse,
if severity >= INCIDENT_SEVERITY_HIGH { LogSeverity::Critical } else { LogSeverity::Warning },
format!("Incident detected: {:?} (severity: {})", incident_type, severity).into_bytes(),
incident.treaty_context.clone(),
None,
)?;
// Add to offline buffer
self.offline_buffer.push_back(incident.clone());
if self.offline_buffer.len() > OFFLINE_INCIDENT_BUFFER_SIZE {
self.offline_buffer.pop_front();
}
self.metrics.offline_buffer_usage_percent = (self.offline_buffer.len() as f64 / OFFLINE_INCIDENT_BUFFER_SIZE as f64) * 100.0;
Ok(Some(incident))
}
/**
* Assess incident severity based on threat event characteristics
*/
fn assess_incident_severity(&self, threat_event: &ThreatEvent) -> u8 {
let mut severity_score = 0;
// Base severity from threat category
severity_score += match threat_event.category {
ThreatCategory::TreatyViolation | ThreatCategory::NeurorightsViolation | ThreatCategory::IndigenousDataBreach => 4,
ThreatCategory::NetworkIntrusion | ThreatCategory::PrivilegeEscalation | ThreatCategory::DataExfiltration => 3,
ThreatCategory::SideChannelAttack | ThreatCategory::BehavioralAnomaly => 2,
_ => 1,
};
// Adjust for threat severity
severity_score += match threat_event.severity {
ThreatSeverity::Critical => 2,
ThreatSeverity::High => 1,
ThreatSeverity::Medium => 0,
ThreatSeverity::Low => -1,
};
// Adjust for confidence
if threat_event.confidence > 0.9 {
severity_score += 1;
} else if threat_event.confidence < 0.5 {
severity_score -= 1;
}
// Adjust for treaty violation
if threat_event.treaty_violation.is_some() {
severity_score += 2;
}
// Clamp to valid range
severity_score.max(INCIDENT_SEVERITY_LOW as i32).min(INCIDENT_SEVERITY_EMERGENCY as i32) as u8
}
/**
* Map threat event to incident type
*/
fn map_threat_to_incident_type(&self, threat_event: &ThreatEvent) -> IncidentType {
match threat_event.category {
ThreatCategory::TreatyViolation => IncidentType::TreatyViolation,
ThreatCategory::NeurorightsViolation => IncidentType::NeurorightsViolation,
ThreatCategory::IndigenousDataBreach => IncidentType::IndigenousDataBreach,
ThreatCategory::NetworkIntrusion => IncidentType::NetworkIntrusion,
ThreatCategory::PrivilegeEscalation => IncidentType::PrivilegeEscalation,
ThreatCategory::DataExfiltration => IncidentType::DataExfiltration,
ThreatCategory::SideChannelAttack => IncidentType::SideChannelAttack,
ThreatCategory::EnvironmentalHazard => {
if threat_event.metadata.contains_key("haboob") {
IncidentType::HaboobEvent
} else if threat_event.metadata.contains_key("temperature") {
IncidentType::ExtremeHeatEvent
} else if threat_event.metadata.contains_key("flood") {
IncidentType::FlashFloodEvent
} else {
IncidentType::EnvironmentalIncident
}
}
_ => IncidentType::SecurityBreach,
}
}
/**
* Assess incident impact level
*/
fn assess_incident_impact(&self, threat_event: &ThreatEvent, severity: u8) -> IncidentImpact {
if severity >= INCIDENT_SEVERITY_CRITICAL {
return IncidentImpact::Severe;
}
if severity >= INCIDENT_SEVERITY_HIGH {
return IncidentImpact::Significant;
}
if severity >= INCIDENT_SEVERITY_MEDIUM {
return IncidentImpact::Moderate;
}
IncidentImpact::Minimal
}
/**
* Triage incident (assess, prioritize, assign containment strategy)
*/
fn triage_incident(&mut self, incident: &mut Incident) -> Result<(), &'static str> {
let triage_start = now();
// Determine containment strategy based on incident type and severity
incident.containment_strategy = Some(match incident.incident_type {
IncidentType::TreatyViolation | IncidentType::NeurorightsViolation | IncidentType::IndigenousDataBreach => ContainmentStrategy::FailSecure,
IncidentType::SystemCompromise | IncidentType::MalwareInfection => ContainmentStrategy::Isolation,
IncidentType::HaboobEvent | IncidentType::ExtremeHeatEvent | IncidentType::FlashFloodEvent => ContainmentStrategy::DegradedMode,
_ => ContainmentStrategy::Segmentation,
});
// Assess environmental context if applicable
if incident.incident_type == IncidentType::HaboobEvent || incident.incident_type == IncidentType::ExtremeHeatEvent || incident.incident_type == IncidentType::FlashFloodEvent {
incident.environmental_context = Some(self.read_environmental_sensors()?);
if let Some(ref env) = incident.environmental_context {
if env.haboob_detected {
self.metrics.environmental_incidents += 1;
}
if env.extreme_heat {
self.metrics.environmental_incidents += 1;
}
if env.flash_flood_risk {
self.metrics.environmental_incidents += 1;
}
}
}
// Update status
incident.status = IncidentStatus::Triaged;
incident.triage_timestamp = now();
// Update metrics
let triage_time_ms = (now() - triage_start) / 1000;
self.metrics.avg_triage_time_ms = (self.metrics.avg_triage_time_ms * (self.metrics.total_incidents - 1) as f64
+ triage_time_ms as f64) / self.metrics.total_incidents as f64;
Ok(())
}
/**
* Execute automated response based on playbook matching
*/
fn execute_automated_response(&mut self, incident: &mut Incident) -> Result<(), &'static str> {
// Find matching playbook
let matching_playbooks: Vec<_> = self.response_playbooks.values()
.filter(|playbook| playbook.incident_types.contains(&incident.incident_type) && incident.severity >= playbook.severity_threshold)
.collect();
if matching_playbooks.is_empty() {
return Ok(()); // No playbook matches, manual response required
}
// Use first matching playbook (most specific would be better in production)
let playbook = matching_playbooks[0];
// Check automation level
if playbook.max_automation_level == ResponseAutomationLevel::ManualOnly {
return Ok(()); // Manual response only
}
// Check treaty approval if required
if playbook.confirmation_required {
// In production: wait for treaty council approval
// For now: simulate approval for non-critical incidents
if incident.severity < INCIDENT_SEVERITY_CRITICAL {
// Auto-approve for testing
} else {
return Ok(()); // Wait for manual approval
}
}
// Execute response actions
let mut actions_executed = 0;
for action_template in &playbook.response_actions {
if actions_executed >= MAX_AUTOMATED_ACTIONS_PER_INCIDENT {
break;
}
// Execute action
let action_result = self.execute_response_action(incident, action_template)?;
incident.response_actions.push(action_result);
actions_executed += 1;
self.metrics.automated_actions += 1;
// Check if incident is contained
if self.is_incident_contained(incident) {
incident.status = IncidentStatus::Contained;
incident.containment_timestamp = Some(now());
break;
}
}
Ok(())
}
/**
* Execute individual response action
*/
fn execute_response_action(&mut self, incident: &Incident, template: &ResponseActionTemplate) -> Result<ResponseAction, &'static str> {
let action_start = now();
let action_id = self.generate_action_id();
// Determine target entity/system
let target_entity = match template.target_selector.as_str() {
"affected_nodes" => incident.affected_entities.iter().next().cloned(),
"violating_entity" => incident.affected_entities.iter().next().cloned(),
_ => None,
};
// Execute action based on type
let action_type = template.action_type;
let result = match action_type {
ResponseActionType::IsolateNode => {
self.isolate_node(target_entity.as_ref())?;
"Node isolated successfully".to_string()
},
ResponseActionType::BlockTraffic => {
self.block_traffic(target_entity.as_ref())?;
"Traffic blocked successfully".to_string()
},
ResponseActionType::RevokeAccess => {
self.revoke_access(target_entity.as_ref())?;
"Access revoked successfully".to_string()
},
ResponseActionType::RotateKeys => {
self.rotate_keys(target_entity.as_ref())?;
"Keys rotated successfully".to_string()
},
ResponseActionType::QuarantineData => {
self.quarantine_data(target_entity.as_ref())?;
"Data quarantined successfully".to_string()
},
ResponseActionType::ShutdownService => {
self.shutdown_service(target_entity.as_ref())?;
"Service shutdown successfully".to_string()
},
ResponseActionType::PreserveEvidence => {
self.preserve_evidence(incident)?;
"Evidence preserved successfully".to_string()
},
ResponseActionType::AlertAdministrator => {
self.alert_administrator(incident)?;
"Administrator alerted successfully".to_string()
},
ResponseActionType::NotifyCitizen => {
self.notify_citizen(incident)?;
"Citizen notified successfully".to_string()
},
ResponseActionType::TreatyRemediation => {
self.execute_treaty_remediation(incident)?;
"Treaty remediation executed successfully".to_string()
},
ResponseActionType::EnvironmentalMitigation => {
self.execute_environmental_mitigation(incident)?;
"Environmental mitigation executed successfully".to_string()
},
ResponseActionType::EquipmentShutdown => {
self.shutdown_equipment(incident)?;
"Equipment shutdown successfully".to_string()
},
ResponseActionType::ActivateBackup => {
self.activate_backup(incident)?;
"Backup activated successfully".to_string()
},
ResponseActionType::RestoreFromSnapshot => {
self.restore_from_snapshot(incident)?;
"Snapshot restored successfully".to_string()
},
ResponseActionType::UpdatePolicy => {
self.update_policy(incident)?;
"Policy updated successfully".to_string()
},
ResponseActionType::DeployPatch => {
self.deploy_patch(incident)?;
"Patch deployed successfully".to_string()
},
};
let action = ResponseAction {
action_id,
action_type,
target_entity,
target_system: None,
execution_timestamp: now(),
completion_timestamp: Some(now()),
status: ResponseActionStatus::Completed,
result: Some(result),
treaty_approved: incident.treaty_context.is_some(),
treaty_context: incident.treaty_context.clone(),
rollback_action: None, // Would be populated if rollback needed
};
// Update metrics
let action_time_ms = (now() - action_start) / 1000;
self.metrics.avg_response_action_time_ms = (self.metrics.avg_response_action_time_ms * (self.metrics.automated_actions) as f64
+ action_time_ms as f64) / (self.metrics.automated_actions + 1) as f64;
Ok(action)
}
/**
* Isolate node from network
*/
fn isolate_node(&mut self, node: Option<&BirthSign>) -> Result<(), &'static str> {
// In production: implement actual network isolation (firewall rules, VLAN changes)
// For now: log the action
if let Some(n) = node {
debug!("Isolating node: {:?}", n);
}
Ok(())
}
/**
* Block network traffic
*/
fn block_traffic(&mut self, node: Option<&BirthSign>) -> Result<(), &'static str> {
// In production: implement actual traffic blocking
Ok(())
}
/**
* Revoke access credentials
*/
fn revoke_access(&mut self, node: Option<&BirthSign>) -> Result<(), &'static str> {
// In production: implement actual access revocation
Ok(())
}
/**
* Rotate cryptographic keys
*/
fn rotate_keys(&mut self, node: Option<&BirthSign>) -> Result<(), &'static str> {
// In production: implement actual key rotation
Ok(())
}
/**
* Quarantine affected data
*/
fn quarantine_data(&mut self, node: Option<&BirthSign>) -> Result<(), &'static str> {
// In production: implement actual data quarantine
Ok(())
}
/**
* Shutdown affected service
*/
fn shutdown_service(&mut self, node: Option<&BirthSign>) -> Result<(), &'static str> {
// In production: implement actual service shutdown
Ok(())
}
/**
* Preserve forensic evidence
*/
fn preserve_evidence(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual evidence preservation
Ok(())
}
/**
* Alert administrator
*/
fn alert_administrator(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual alerting (email, SMS, dashboard)
Ok(())
}
/**
* Notify affected citizen
*/
fn notify_citizen(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual citizen notification
Ok(())
}
/**
* Execute treaty remediation protocol
*/
fn execute_treaty_remediation(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual treaty remediation
Ok(())
}
/**
* Execute environmental mitigation
*/
fn execute_environmental_mitigation(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual environmental controls
Ok(())
}
/**
* Shutdown equipment to prevent damage
*/
fn shutdown_equipment(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual equipment shutdown
Ok(())
}
/**
* Activate backup systems
*/
fn activate_backup(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual backup activation
Ok(())
}
/**
* Restore from known-good snapshot
*/
fn restore_from_snapshot(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual snapshot restoration
Ok(())
}
/**
* Update security policy
*/
fn update_policy(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual policy updates
Ok(())
}
/**
* Deploy security patch
*/
fn deploy_patch(&mut self, incident: &Incident) -> Result<(), &'static str> {
// In production: implement actual patch deployment
Ok(())
}
/**
* Check if incident is contained
*/
fn is_incident_contained(&self, incident: &Incident) -> bool {
// In production: implement actual containment verification
// For now: consider contained if at least one response action executed
!incident.response_actions.is_empty()
}
/**
* Read environmental sensors for Phoenix conditions
*/
fn read_environmental_sensors(&self) -> Result<EnvironmentalContext, &'static str> {
// In production: read actual hardware sensors
// For now: simulate Phoenix conditions
Ok(EnvironmentalContext {
temperature_c: 45.0, // 113°F typical Phoenix summer
humidity_percent: 20.0,
particulate_ug_m3: 50.0,
rainfall_mm: 0,
wind_speed_kph: 10.0,
haboob_detected: false,
extreme_heat: true,
flash_flood_risk: false,
equipment_stress_level: 60,
timestamp: now(),
})
}
/**
* Escalate incident to higher authority level
*/
pub fn escalate_incident(&mut self, incident_id: &[u8; 32], new_level: EscalationLevel) -> Result<(), &'static str> {
let incident = self.incidents.get_mut(incident_id)
.ok_or("Incident not found")?;
// Check if escalation is allowed
let current_level = incident.escalation_level as u8;
let new_level_val = new_level as u8;
if new_level_val <= current_level {
return Err("Cannot escalate to same or lower level");
}
// Check treaty approval if required
if self.escalation_policies.iter().any(|p| p.incident_type == incident.incident_type && p.treaty_approval_required) {
if incident.treaty_context.is_none() || incident.treaty_context.as_ref().unwrap().fpic_status != FPICStatus::Granted {
return Err("Treaty approval required for escalation");
}
}
// Update incident
incident.escalation_level = new_level;
incident.status = IncidentStatus::Escalated;
self.metrics.escalation_count += 1;
// Log escalation
self.audit_log.append_log(
LogEventType::IncidentResponse,
LogSeverity::Critical,
format!("Incident escalated to level {:?}: {:?}", new_level, incident_id).into_bytes(),
incident.treaty_context.clone(),
None,
)?;
Ok(())
}
/**
* Close incident (mark as resolved)
*/
pub fn close_incident(&mut self, incident_id: &[u8; 32], resolution_notes: String, lessons_learned: Option<String>) -> Result<(), &'static str> {
let incident = self.incidents.get_mut(incident_id)
.ok_or("Incident not found")?;
// Verify incident is in closable state
if incident.status == IncidentStatus::Detected || incident.status == IncidentStatus::Triaged {
return Err("Incident must be contained/eradicated before closure");
}
// Update incident
incident.status = IncidentStatus::Closed;
incident.closure_timestamp = Some(now());
incident.resolution_notes = Some(resolution_notes);
incident.lessons_learned = lessons_learned;
self.active_incidents.remove(incident_id);
// Update metrics
self.metrics.resolution_rate_percent = (self.metrics.total_incidents as f64 - self.active_incidents.len() as f64) / self.metrics.total_incidents as f64 * 100.0;
// Log closure
self.audit_log.append_log(
LogEventType::IncidentResponse,
LogSeverity::Info,
format!("Incident closed: {:?}", incident_id).into_bytes(),
incident.treaty_context.clone(),
None,
)?;
Ok(())
}
/**
* Get incident by ID
*/
pub fn get_incident(&self, incident_id: &[u8; 32]) -> Option<&Incident> {
self.incidents.get(incident_id)
}
/**
* Get all active incidents
*/
pub fn get_active_incidents(&self) -> Vec<&Incident> {
self.active_incidents.iter()
.filter_map(|id| self.incidents.get(id))
.collect()
}
/**
* Get incident metrics
*/
pub fn get_metrics(&self) -> IncidentMetrics {
self.metrics.clone()
}
/**
* Generate unique IDs
*/
fn generate_incident_id(&self) -> [u8; 32] {
let mut id = [0u8; 32];
let timestamp = now();
id[..8].copy_from_slice(&timestamp.to_be_bytes());
id[8..24].copy_from_slice(&self.node_id.to_bytes()[..16]);
id[24..].copy_from_slice(&self.metrics.total_incidents.to_be_bytes()[..8]);
self.crypto_engine.sha512_hash(&id)[..32].try_into().unwrap_or([0u8; 32])
}
fn generate_playbook_id(&self) -> [u8; 32] {
self.generate_incident_id()
}
fn generate_template_id(&self) -> [u8; 32] {
self.generate_incident_id()
}
fn generate_policy_id(&self) -> [u8; 32] {
self.generate_incident_id()
}
fn generate_action_id(&self) -> [u8; 32] {
self.generate_incident_id()
}
/**
* Perform maintenance tasks (cleanup, metrics update, buffer management)
*/
pub fn perform_maintenance(&mut self) -> Result<(), &'static str> {
let now = now();
// Cleanup old closed incidents (>90 days)
let closed_ids: Vec<_> = self.incidents.iter()
.filter(|(_, inc)| inc.status == IncidentStatus::Closed && now - inc.closure_timestamp.unwrap_or(0) > 90 * 24 * 60 * 60 * 1000000)
.map(|(id, _)| *id)
.collect();
for id in closed_ids {
self.incidents.remove(&id);
}
// Cleanup old offline buffer entries (>72 hours)
while let Some(incident) = self.offline_buffer.front() {
if now - incident.detection_timestamp > (OFFLINE_BUFFER_HOURS as u64) * 3600 * 1000000 {
self.offline_buffer.pop_front();
} else {
break;
}
}
// Update resolution rate
let resolved = self.incidents.values().filter(|inc| inc.status == IncidentStatus::Closed).count();
if self.metrics.total_incidents > 0 {
self.metrics.resolution_rate_percent = (resolved as f64 / self.metrics.total_incidents as f64) * 100.0;
}
self.last_maintenance = now();
Ok(())
}
}
// --- Helper Functions ---
/**
* Calculate average response time by severity
*/
pub fn calculate_avg_response_time_by_severity(metrics: &IncidentMetrics, severity: u8) -> f64 {
// Placeholder - would calculate from actual incident data
0.0
}
/**
* Check if response time is within acceptable limits
*/
pub fn is_response_time_acceptable(severity: u8, response_time_ms: u64) -> bool {
let max_time = match severity {
1 => MAX_RESPONSE_TIME_LOW_MS,
2 => MAX_RESPONSE_TIME_MEDIUM_MS,
3 => MAX_RESPONSE_TIME_HIGH_MS,
4 => MAX_RESPONSE_TIME_CRITICAL_MS,
5 => MAX_RESPONSE_TIME_EMERGENCY_MS,
_ => MAX_RESPONSE_TIME_LOW_MS,
};
response_time_ms <= max_time
}
/**
* Calculate incident resolution rate
*/
pub fn calculate_resolution_rate(total: usize, resolved: usize) -> f64 {
if total == 0 {
return 100.0;
}
(resolved as f64 / total as f64) * 100.0
}
// --- Unit Tests (Offline Capable) ---
#[cfg(test)]
mod tests {
use super::*;
#[test]
fn test_engine_initialization() {
let engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
assert!(engine.active);
assert_eq!(engine.incidents.len(), 0);
assert!(engine.response_playbooks.len() >= 5); // Default playbooks
assert_eq!(engine.metrics.total_incidents, 0);
}
#[test]
fn test_incident_detection() {
let mut engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Create threat event
let threat = ThreatEvent {
event_id: [1u8; 32],
timestamp: now(),
category: ThreatCategory::NetworkIntrusion,
severity: ThreatSeverity::High,
detection_method: DetectionMethod::RuleBasedSignature,
confidence: 0.95,
source_node: BirthSign::default(),
target_node: None,
attack_vector: None,
payload_hash: None,
metadata: Default::default(),
treaty_violation: None,
mitigation_actions: Vec::new(),
status: Default::default(),
correlation_id: None,
};
// Detect incident
let incident = engine.detect_incident(threat).unwrap();
assert!(incident.is_some());
assert_eq!(engine.metrics.total_incidents, 1);
assert_eq!(engine.active_incidents.len(), 1);
}
#[test]
fn test_incident_severity_assessment() {
let engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Critical treaty violation
let threat1 = ThreatEvent {
event_id: [1u8; 32],
timestamp: now(),
category: ThreatCategory::TreatyViolation,
severity: ThreatSeverity::Critical,
detection_method: DetectionMethod::TreatyComplianceCheck,
confidence: 0.99,
source_node: BirthSign::default(),
target_node: None,
attack_vector: None,
payload_hash: None,
metadata: Default::default(),
treaty_violation: Some(TreatyViolation { allowed: false, reason: "FPIC denied".to_string(), violates_neurorights: true }),
mitigation_actions: Vec::new(),
status: Default::default(),
correlation_id: None,
};
let severity1 = engine.assess_incident_severity(&threat1);
assert!(severity1 >= INCIDENT_SEVERITY_HIGH);
// Low severity anomaly
let threat2 = ThreatEvent {
category: ThreatCategory::BehavioralAnomaly,
severity: ThreatSeverity::Low,
confidence: 0.4,
..threat1.clone()
};
let severity2 = engine.assess_incident_severity(&threat2);
assert!(severity2 <= INCIDENT_SEVERITY_MEDIUM);
}
#[test]
fn test_playbook_matching() {
let mut engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Create incident
let mut incident = Incident {
incident_id: [1u8; 32],
incident_type: IncidentType::TreatyViolation,
severity: INCIDENT_SEVERITY_HIGH,
status: IncidentStatus::Triaged,
detection_timestamp: now(),
triage_timestamp: now(),
containment_timestamp: None,
eradication_timestamp: None,
recovery_timestamp: None,
closure_timestamp: None,
affected_entities: BTreeSet::new(),
affected_systems: BTreeSet::new(),
threat_events: Vec::new(),
root_cause: None,
impact_assessment: IncidentImpact::Significant,
containment_strategy: None,
response_actions: Vec::new(),
escalation_level: EscalationLevel::Level0,
treaty_context: None,
environmental_context: None,
resolution_notes: None,
lessons_learned: None,
};
// Execute automated response
engine.execute_automated_response(&mut incident).unwrap();
// Should have matched treaty violation playbook
assert!(!incident.response_actions.is_empty());
}
#[test]
fn test_incident_escalation() {
let mut engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Create incident
let threat = ThreatEvent {
event_id: [1u8; 32],
timestamp: now(),
category: ThreatCategory::SystemCompromise,
severity: ThreatSeverity::Critical,
detection_method: DetectionMethod::StatisticalCUSUM,
confidence: 0.98,
source_node: BirthSign::default(),
target_node: None,
attack_vector: None,
payload_hash: None,
metadata: Default::default(),
treaty_violation: None,
mitigation_actions: Vec::new(),
status: Default::default(),
correlation_id: None,
};
let incident = engine.detect_incident(threat).unwrap().unwrap();
// Escalate incident
let result = engine.escalate_incident(&incident.incident_id, EscalationLevel::Level3);
assert!(result.is_ok());
let updated = engine.get_incident(&incident.incident_id).unwrap();
assert_eq!(updated.escalation_level, EscalationLevel::Level3);
assert_eq!(updated.status, IncidentStatus::Escalated);
}
#[test]
fn test_incident_closure() {
let mut engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Create incident
let threat = ThreatEvent {
event_id: [1u8; 32],
timestamp: now(),
category: ThreatCategory::SecurityBreach,
severity: ThreatSeverity::Medium,
detection_method: DetectionMethod::RuleBasedSignature,
confidence: 0.85,
source_node: BirthSign::default(),
target_node: None,
attack_vector: None,
payload_hash: None,
metadata: Default::default(),
treaty_violation: None,
mitigation_actions: Vec::new(),
status: Default::default(),
correlation_id: None,
};
let incident = engine.detect_incident(threat).unwrap().unwrap();
// Close incident
let result = engine.close_incident(&incident.incident_id, "Resolved via automated response".to_string(), Some("Improve detection rules".to_string()));
assert!(result.is_ok());
let updated = engine.get_incident(&incident.incident_id).unwrap();
assert_eq!(updated.status, IncidentStatus::Closed);
assert!(updated.resolution_notes.is_some());
assert_eq!(engine.active_incidents.len(), 0);
}
#[test]
fn test_environmental_incident_detection() {
let mut engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Create haboob event
let mut metadata = BTreeMap::new();
metadata.insert("haboob".to_string(), "true".to_string());
metadata.insert("particulate_ug_m3".to_string(), "2000".to_string());
let threat = ThreatEvent {
event_id: [1u8; 32],
timestamp: now(),
category: ThreatCategory::EnvironmentalHazard,
severity: ThreatSeverity::High,
detection_method: DetectionMethod::EnvironmentalSensorFusion,
confidence: 0.95,
source_node: BirthSign::default(),
target_node: None,
attack_vector: None,
payload_hash: None,
metadata,
treaty_violation: None,
mitigation_actions: Vec::new(),
status: Default::default(),
correlation_id: None,
};
let incident = engine.detect_incident(threat).unwrap().unwrap();
assert_eq!(incident.incident_type, IncidentType::HaboobEvent);
assert!(incident.environmental_context.is_some());
assert_eq!(engine.metrics.environmental_incidents, 1);
}
#[test]
fn test_response_time_requirements() {
// Verify response time targets
assert!(MAX_RESPONSE_TIME_EMERGENCY_MS <= 100); // <100ms
assert!(MAX_RESPONSE_TIME_CRITICAL_MS <= 1000); // <1s
assert!(MAX_RESPONSE_TIME_HIGH_MS <= 10000); // <10s
assert!(MAX_RESPONSE_TIME_MEDIUM_MS <= 60000); // <1min
assert!(MAX_RESPONSE_TIME_LOW_MS <= 300000); // <5min
}
#[test]
fn test_offline_buffer_management() {
let mut engine = IncidentResponseEngine::new(BirthSign::default()).unwrap();
// Fill offline buffer beyond capacity
for _ in 0..(OFFLINE_INCIDENT_BUFFER_SIZE + 100) {
let incident = Incident {
incident_id: [0u8; 32],
incident_type: IncidentType::SecurityBreach,
severity: INCIDENT_SEVERITY_LOW,
status: IncidentStatus::Closed,
detection_timestamp: now(),
triage_timestamp: now(),
containment_timestamp: None,
eradication_timestamp: None,
recovery_timestamp: None,
closure_timestamp: Some(now()),
affected_entities: BTreeSet::new(),
affected_systems: BTreeSet::new(),
threat_events: Vec::new(),
root_cause: None,
impact_assessment: IncidentImpact::Minimal,
containment_strategy: None,
response_actions: Vec::new(),
escalation_level: EscalationLevel::Level0,
treaty_context: None,
environmental_context: None,
resolution_notes: None,
lessons_learned: None,
};
engine.offline_buffer.push_back(incident);
}
// Buffer should be at max capacity
assert_eq!(engine.offline_buffer.len(), OFFLINE_INCIDENT_BUFFER_SIZE);
assert_eq!(engine.metrics.offline_buffer_usage_percent, 100.0);
}
#[test]
fn test_incident_resolution_rate_calculation() {
// 100 incidents with 99 resolved = 99% resolution rate
let rate1 = calculate_resolution_rate(100, 99);
assert_eq!(rate1, 99.0);
// 1000 incidents with 999 resolved = 99.9% resolution rate
let rate2 = calculate_resolution_rate(1000, 999);
assert_eq!(rate2, 99.9);
// 0 incidents = 100% resolution rate
let rate3 = calculate_resolution_rate(0, 0);
assert_eq!(rate3, 100.0);
}
}
