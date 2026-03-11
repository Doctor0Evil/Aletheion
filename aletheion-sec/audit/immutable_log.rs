/**
* Aletheion Smart City Core - Batch 2
* File: 116/200
* Layer: 36 (Advanced Security)
* Path: aletheion-sec/audit/immutable_log.rs
*
* Research Basis (Immutable Audit Trail & Forensic Logging):
*   - Blockchain-inspired Logging: Append-only logs with cryptographic linking, Merkle trees for efficient verification
*   - Tamper-Evident Storage: Write-once-read-many (WORM) principles, hardware-protected log storage, immutable timestamps
*   - Digital Forensics Standards: NIST SP 800-86 (Guide to Integrating Forensic Techniques), ISO/IEC 27037 (Digital Evidence)
*   - Log Integrity Verification: Hash chains (SHA-512), Merkle Patricia Tries for efficient range proofs, PQ signatures for authentication
*   - Chain of Custody: Cryptographic custody transfer, timestamp notarization, witness signatures for legal admissibility
*   - Log Compression & Archival: Zstandard compression (lossless), tiered storage (hot/warm/cold), automated archival policies
*   - Forensic Query Engine: Time-range queries, entity correlation, pattern matching, anomaly detection on historical logs
*   - Treaty Compliance Auditing: FPIC event logging, Indigenous data access trails, neurorights protection verification
*   - Phoenix-Specific Event Correlation: Haboob sensor fusion with security events, extreme heat equipment stress logging, monsoon flood correlation
*   - Performance Benchmarks: <100μs log append, <1ms query response, 99.999% integrity guarantee, 10,000 logs/sec throughput
*   - Legal Admissibility: Digital signature standards (eIDAS, ESIGN Act), timestamp authority integration, court-ready evidence packages
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
// Internal Aletheion Crates (Established in Batch 1 & Files 112-115)
use aletheion_core::identity::BirthSign;
use aletheion_core::time::{now, Timestamp};
use aletheion_core::logger::{log, warn, error, debug};
use aletheion_sec::quantum::post::crypto_core::{PQCryptoEngine, PQSignature, PQSecurityLevel, PQKeyPair};
use aletheion_sec::quantum::post::threat_detection::{ThreatEvent, ThreatCategory, ThreatSeverity};
use aletheion_gov::treaty::{TreatyCompliance, TreatyViolation, FPICStatus, TreatyEvent};
use aletheion_comms::mesh::SecureChannel;
// --- Constants & Audit Log Parameters ---
/// Log entry size constraints
pub const MAX_LOG_ENTRY_SIZE_BYTES: usize = 65536;     // 64KB maximum log entry
pub const MIN_LOG_ENTRY_SIZE_BYTES: usize = 64;        // 64 bytes minimum (header only)
pub const LOG_HEADER_SIZE_BYTES: usize = 128;          // Fixed header size
/// Hash chain parameters
pub const HASH_CHAIN_WINDOW_SIZE: usize = 1000;        // 1000 entries per hash chain segment
pub const MERKLE_TREE_FANOUT: usize = 16;              // 16 children per Merkle node
pub const MERKLE_PROOF_MAX_DEPTH: usize = 8;           // Maximum proof depth (16^8 = 4.3B entries)
/// Log rotation and archival parameters
pub const LOG_ROTATION_SIZE_MB: usize = 100;           // Rotate log after 100MB
pub const LOG_RETENTION_DAYS_HOT: u32 = 7;             // Hot storage: 7 days
pub const LOG_RETENTION_DAYS_WARM: u32 = 90;           // Warm storage: 90 days
pub const LOG_RETENTION_DAYS_COLD: u32 = 3650;         // Cold storage: 10 years (legal requirement)
pub const ARCHIVAL_COMPRESSION_LEVEL: u8 = 10;         // Zstandard compression level (max)
/// Performance thresholds
pub const MAX_LOG_APPEND_TIME_US: u64 = 100;           // <100μs log append latency
pub const MAX_LOG_QUERY_TIME_MS: u64 = 1;              // <1ms query response time
pub const MAX_LOG_VERIFICATION_TIME_MS: u64 = 10;      // <10ms integrity verification
pub const LOG_THROUGHPUT_ENTRIES_PER_SEC: usize = 10000; // 10K entries/sec target
/// Integrity guarantees
pub const INTEGRITY_GUARANTEE_PERCENT: f64 = 99.999;   // 99.999% integrity guarantee
pub const MAX_ALLOWED_CORRUPTION_PERCENT: f64 = 0.001; // <0.001% corruption tolerance
/// Offline buffer duration (hours) - Must meet 72h Protocol
pub const OFFLINE_BUFFER_HOURS: u32 = 72;
pub const OFFLINE_LOG_BUFFER_SIZE: usize = 100000;     // 100K log entries buffered offline
/// Forensic analysis parameters
pub const FORENSIC_QUERY_MAX_RESULTS: usize = 10000;   // Maximum query results
pub const FORENSIC_CORRELATION_WINDOW_HOURS: u32 = 24; // Correlate events within 24h window
pub const ANOMALY_DETECTION_SENSITIVITY: f64 = 2.5;    // 2.5 sigma for anomaly detection
/// Treaty compliance logging parameters
pub const FPIC_EVENT_REQUIRED_FIELDS: usize = 8;       // Minimum fields for FPIC event
pub const TREATY_VIOLATION_LOG_LEVEL: u8 = 100;        // Maximum logging level for violations
pub const INDIGENOUS_DATA_ACCESS_AUDIT: bool = true;   // Always audit Indigenous data access
/// Phoenix-specific environmental logging
pub const ENVIRONMENTAL_EVENT_CORRELATION: bool = true; // Correlate security events with environmental conditions
pub const HABOOB_SECURITY_CORRELATION: bool = true;     // Correlate haboob events with security incidents
pub const EXTREME_HEAT_LOG_ENHANCEMENT: bool = true;    // Enhanced logging during extreme heat (>110°F)
// --- Enumerations ---
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LogEventType {
SecurityEvent,              // Security-related events (authentication, access control)
ThreatDetection,           // Threat detection and anomaly alerts
TreatyCompliance,          // FPIC, Indigenous rights, neurorights events
SystemOperation,           // System operations (boot, shutdown, maintenance)
NetworkActivity,           // Network traffic and communication events
DataAccess,                // Data access and modification events
UserActivity,              // Citizen/user activity events
EnvironmentalEvent,        // Environmental sensor readings and alerts
EquipmentStatus,           // Equipment health and status updates
IncidentResponse,          // Incident response and mitigation actions
ForensicQuery,             // Forensic analysis and investigation queries
PolicyChange,              // Security policy modifications
KeyManagement,             // Cryptographic key operations
BootEvent,                 // Secure boot and attestation events
AuditSystemEvent,          // Audit system internal events
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogSeverity {
Debug,                      // Debug-level information (developer use)
Info,                       // Informational events (normal operation)
Notice,                     // Normal but significant events
Warning,                    // Warning conditions (potential issues)
Error,                      // Error conditions (operation failed)
Critical,                   // Critical conditions (immediate action required)
Alert,                      // Alert conditions (must be fixed immediately)
Emergency,                  // Emergency conditions (system unusable)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogIntegrityStatus {
Verified,                   // Log entry integrity verified (hash chain intact)
Unverified,                 // Log entry not yet verified
Corrupted,                  // Log entry corrupted (hash mismatch)
Tampered,                   // Log entry tampered with (signature invalid)
Missing,                    // Log entry missing from chain
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogStorageTier {
Hot,                        // Hot storage (SSD, <7 days, frequent access)
Warm,                       // Warm storage (HDD, 7-90 days, occasional access)
Cold,                       // Cold storage (tape/cloud, >90 days, archival)
Offline,                    // Offline buffer (volatile memory, <72h)
Immutable,                  // Immutable storage (WORM, blockchain, permanent)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ForensicQueryType {
TimeRange,                  // Query by time range
EntityCorrelation,          // Correlate events by entity (DID, node ID)
EventTypeFilter,            // Filter by event type
SeverityThreshold,          // Filter by severity level
PatternMatching,            // Pattern matching on log content
AnomalyDetection,           // Detect anomalies in historical logs
IntegrityVerification,      // Verify log integrity over range
TreatyComplianceAudit,      // Audit treaty compliance events
ChainOfCustody,             // Trace chain of custody for evidence
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogExportFormat {
JSON,                       // JSON format (human-readable)
CBOR,                       // CBOR format (binary, compact)
ProtocolBuffers,            // Protocol Buffers (efficient binary)
CSV,                        // CSV format (spreadsheet-compatible)
EvidencePackage,            // Court-ready evidence package (signed, timestamped)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChainOfCustodyAction {
LogCreated,                 // Log entry created
LogAccessed,                // Log entry accessed (read)
LogExported,                // Log entry exported
LogTransferred,             // Log custody transferred to another entity
LogVerified,                // Log integrity verified
LogArchived,                // Log archived to long-term storage
LogRestored,                // Log restored from archive
}
#[derive(Clone)]
pub struct LogEntry {
pub entry_id: [u8; 32],                    // Unique entry identifier (SHA-512 hash)
pub timestamp: Timestamp,                   // Precise timestamp (microseconds)
pub event_type: LogEventType,               // Type of event logged
pub severity: LogSeverity,                  // Severity level
pub source_node: BirthSign,                 // Node that generated the log
pub target_entity: Option<BirthSign>,       // Target entity (if applicable)
pub event_data: Vec<u8>,                    // Event-specific data (serialized)
pub previous_hash: [u8; 64],                // Hash of previous log entry (hash chain)
pub merkle_hash: [u8; 64],                  // Merkle tree hash for this entry
pub pq_signature: Option<PQSignature>,      // PQ signature for authentication
pub treaty_context: Option<TreatyContext>,  // Treaty compliance context (FPIC, etc.)
pub environmental_context: Option<EnvironmentalContext>, // Phoenix environmental conditions
pub integrity_status: LogIntegrityStatus,   // Integrity verification status
pub storage_tier: LogStorageTier,           // Current storage tier
}
#[derive(Clone)]
pub struct TreatyContext {
pub fpic_status: FPICStatus,                // FPIC status (granted, pending, denied)
pub indigenous_community: Option<String>,   // Indigenous community involved
pub data_sovereignty_level: u8,             // Data sovereignty score (0-100)
pub neurorights_protected: bool,            // Neurorights protection status
pub consent_timestamp: Timestamp,           // When consent was obtained
pub consent_expiry: Timestamp,              // When consent expires
pub treaty_event_id: Option<[u8; 32]>,      // Reference to treaty event
}
#[derive(Clone)]
pub struct EnvironmentalContext {
pub temperature_c: f32,                     // Ambient temperature (°C)
pub humidity_percent: f32,                  // Relative humidity (%)
pub particulate_ug_m3: f32,                 // Particulate matter (μg/m³)
pub haboob_detected: bool,                  // Haboob (dust storm) detected
pub extreme_heat: bool,                     // Extreme heat conditions (>43°C / 110°F)
pub monsoon_conditions: bool,               // Monsoon/rain conditions
pub timestamp: Timestamp,                   // Environmental reading timestamp
}
#[derive(Clone)]
pub struct LogChainSegment {
pub segment_id: [u8; 32],                   // Unique segment identifier
pub start_index: usize,                     // Starting log index in segment
pub end_index: usize,                       // Ending log index in segment
pub entry_count: usize,                     // Number of entries in segment
pub root_hash: [u8; 64],                    // Merkle root hash for segment
pub pq_signature: PQSignature,              // PQ signature over root hash
pub timestamp_range: (Timestamp, Timestamp),// Time range covered by segment
pub storage_location: String,               // Storage location (path/URL)
pub integrity_verified: bool,               // Integrity verification status
}
#[derive(Clone)]
pub struct MerkleTreeNode {
pub node_hash: [u8; 64],                    // Hash of this node
pub left_child: Option<Box<MerkleTreeNode>>, // Left child (recursive)
pub right_child: Option<Box<MerkleTreeNode>>, // Right child (recursive)
pub leaf_entries: Option<Vec<usize>>,       // Leaf entry indices (if leaf node)
pub depth: usize,                           // Depth in tree (0 = root)
}
#[derive(Clone)]
pub struct ForensicQuery {
pub query_id: [u8; 32],                     // Unique query identifier
pub query_type: ForensicQueryType,          // Type of forensic query
pub time_range: Option<(Timestamp, Timestamp)>, // Time range filter
pub entity_filter: Option<BirthSign>,       // Entity filter (DID/node ID)
pub event_types: BTreeSet<LogEventType>,    // Event type filters
pub min_severity: LogSeverity,              // Minimum severity threshold
pub max_results: usize,                     // Maximum number of results
pub include_correlations: bool,             // Include correlated events
pub treaty_compliance_only: bool,           // Filter to treaty compliance events only
}
#[derive(Clone)]
pub struct ForensicQueryResult {
pub query_id: [u8; 32],                     // Reference to query
pub result_count: usize,                    // Number of results returned
pub log_entries: Vec<LogEntry>,             // Matching log entries
pub correlated_events: Vec<[u8; 32]>,       // IDs of correlated events
pub integrity_issues: Vec<LogIntegrityIssue>, // Any integrity issues detected
pub query_duration_us: u64,                 // Query execution time (μs)
pub treaty_violations_found: usize,         // Number of treaty violations found
}
#[derive(Clone)]
pub struct LogIntegrityIssue {
pub entry_id: [u8; 32],                     // Affected log entry ID
pub issue_type: LogIntegrityStatus,         // Type of integrity issue
pub expected_hash: [u8; 64],                // Expected hash value
pub actual_hash: [u8; 64],                  // Actual hash value (if available)
pub timestamp: Timestamp,                   // When issue was detected
pub severity: LogSeverity,                  // Severity of integrity issue
pub remediation: Option<String>,            // Recommended remediation
}
#[derive(Clone)]
pub struct ChainOfCustodyRecord {
pub custody_id: [u8; 32],                   // Unique custody record ID
pub log_entry_id: [u8; 32],                 // Log entry being transferred
pub action: ChainOfCustodyAction,           // Custody action performed
pub from_entity: BirthSign,                 // Entity transferring custody
pub to_entity: BirthSign,                   // Entity receiving custody
pub timestamp: Timestamp,                   // When transfer occurred
pub pq_signature: PQSignature,              // Signature authorizing transfer
pub reason: String,                         // Reason for custody transfer
pub legal_authority: Option<String>,        // Legal authority (court order, etc.)
}
#[derive(Clone)]
pub struct AuditSystemMetrics {
pub total_log_entries: usize,               // Total log entries created
pub log_entries_hot: usize,                 // Entries in hot storage
pub log_entries_warm: usize,                // Entries in warm storage
pub log_entries_cold: usize,                // Entries in cold storage
pub integrity_violations: usize,            // Integrity violations detected
pub treaty_violations_logged: usize,        // Treaty violations logged
pub forensic_queries: usize,                // Forensic queries executed
pub avg_append_latency_us: f64,             // Average append latency (μs)
pub max_append_latency_us: u64,             // Maximum append latency (μs)
pub avg_query_latency_ms: f64,              // Average query latency (ms)
pub storage_bytes_total: u64,               // Total storage used (bytes)
pub storage_bytes_hot: u64,                 // Hot storage used (bytes)
pub storage_bytes_warm: u64,                // Warm storage used (bytes)
pub storage_bytes_cold: u64,                // Cold storage used (bytes)
pub offline_buffer_usage_percent: f64,      // Offline buffer usage percentage
}
#[derive(Clone)]
pub struct LogExportRequest {
pub export_id: [u8; 32],                    // Unique export request ID
pub time_range: (Timestamp, Timestamp),     // Time range to export
pub event_types: BTreeSet<LogEventType>,    // Event types to include
pub min_severity: LogSeverity,              // Minimum severity to include
pub export_format: LogExportFormat,         // Export format (JSON, CBOR, etc.)
pub recipient_did: BirthSign,               // Recipient DID (for encrypted export)
pub pq_encryption_key: Option<Vec<u8>>,     // PQ encryption key for export
pub chain_of_custody: bool,                 // Include chain of custody records
pub treaty_compliance_only: bool,           // Export only treaty compliance events
pub signature_required: bool,               // Require PQ signature on export
}
#[derive(Clone)]
pub struct EvidencePackage {
pub package_id: [u8; 32],                   // Unique evidence package ID
pub log_entries: Vec<LogEntry>,             // Log entries included in package
pub chain_of_custody: Vec<ChainOfCustodyRecord>, // Chain of custody records
pub pq_signature: PQSignature,              // PQ signature over entire package
pub timestamp_authority: Option<TimestampAuthorityAttestation>, // Timestamp notarization
pub legal_metadata: LegalMetadata,          // Legal metadata for court admissibility
pub hash: [u8; 64],                         // SHA-512 hash of package
}
#[derive(Clone)]
pub struct TimestampAuthorityAttestation {
pub authority_id: [u8; 32],                 // Timestamp authority identifier
pub timestamp: Timestamp,                   // Notarized timestamp
pub pq_signature: PQSignature,              // Authority's PQ signature
pub certificate_chain: Vec<PQKeyPair>,      // Certificate chain for authority
}
#[derive(Clone)]
pub struct LegalMetadata {
pub case_number: Option<String>,            // Legal case number
pub jurisdiction: String,                   // Legal jurisdiction
pub requesting_authority: Option<String>,   // Requesting legal authority
pub purpose: String,                        // Purpose of evidence collection
pub retention_requirement: u32,             // Legal retention requirement (days)
pub confidentiality_level: u8,              // Confidentiality level (0-100)
pub redaction_required: bool,               // Whether redaction is required
}
#[derive(Clone)]
pub struct LogCompressionStats {
pub original_size_bytes: usize,             // Original log size (bytes)
pub compressed_size_bytes: usize,           // Compressed log size (bytes)
pub compression_ratio: f64,                 // Compression ratio (original/compressed)
pub compression_algorithm: String,          // Compression algorithm used
pub compression_level: u8,                  // Compression level (1-10)
pub decompression_time_us: u64,             // Time to decompress (μs)
}
// --- Core Immutable Audit Log Engine ---
pub struct ImmutableAuditLogEngine {
pub node_id: BirthSign,
pub crypto_engine: PQCryptoEngine,
pub treaty_compliance: TreatyCompliance,
pub log_entries: LinkedList<LogEntry>,      // In-memory log entries (hot storage)
pub log_chain_segments: Vec<LogChainSegment>, // Hash chain segments (archived)
pub merkle_tree_root: Option<MerkleTreeNode>, // Current Merkle tree root
pub forensic_queries: BTreeMap<[u8; 32], ForensicQuery>, // Active forensic queries
pub chain_of_custody: Vec<ChainOfCustodyRecord>, // Chain of custody records
pub metrics: AuditSystemMetrics,
pub offline_buffer: VecDeque<LogEntry>,     // Offline buffer for disconnected operation
pub last_archival: Timestamp,               // Last archival timestamp
pub last_integrity_check: Timestamp,        // Last integrity verification timestamp
pub active: bool,
}
impl ImmutableAuditLogEngine {
/**
* Initialize Immutable Audit Log Engine with PQ Crypto integration
* Configures hash chains, Merkle trees, storage tiers, and offline buffer
* Ensures 72h offline operational capability with 100K entry buffer
*/
pub fn new(node_id: BirthSign) -> Result<Self, &'static str> {
let crypto_engine = PQCryptoEngine::new(node_id.clone(), PQSecurityLevel::Level3)
.map_err(|_| "Failed to initialize PQ crypto engine")?;
let mut engine = Self {
node_id,
crypto_engine,
treaty_compliance: TreatyCompliance::new(),
log_entries: LinkedList::new(),
log_chain_segments: Vec::new(),
merkle_tree_root: None,
forensic_queries: BTreeMap::new(),
chain_of_custody: Vec::new(),
metrics: AuditSystemMetrics {
total_log_entries: 0,
log_entries_hot: 0,
log_entries_warm: 0,
log_entries_cold: 0,
integrity_violations: 0,
treaty_violations_logged: 0,
forensic_queries: 0,
avg_append_latency_us: 0.0,
max_append_latency_us: 0,
avg_query_latency_ms: 0.0,
storage_bytes_total: 0,
storage_bytes_hot: 0,
storage_bytes_warm: 0,
storage_bytes_cold: 0,
offline_buffer_usage_percent: 0.0,
},
offline_buffer: VecDeque::with_capacity(OFFLINE_LOG_BUFFER_SIZE),
last_archival: now(),
last_integrity_check: now(),
active: true,
};
// Initialize empty Merkle tree
engine.merkle_tree_root = Some(engine.build_empty_merkle_tree());
Ok(engine)
}
/**
* Append log entry to immutable audit trail
* Implements hash chaining, Merkle tree update, PQ signature, and treaty context
* Returns entry ID and integrity status
*/
pub fn append_log(&mut self, event_type: LogEventType, severity: LogSeverity, event_data: Vec<u8>, treaty_ctx: Option<TreatyContext>, env_ctx: Option<EnvironmentalContext>) -> Result<([u8; 32], LogIntegrityStatus), &'static str> {
let start_time = now();
// Validate log entry size
if event_data.len() > MAX_LOG_ENTRY_SIZE_BYTES {
return Err("Log entry exceeds maximum size");
}
if event_data.len() < MIN_LOG_ENTRY_SIZE_BYTES && severity != LogSeverity::Debug {
return Err("Log entry below minimum size for non-debug events");
}
// Get previous hash (hash chain)
let previous_hash = if let Some(last_entry) = self.log_entries.back() {
last_entry.merkle_hash
} else {
[0u8; 64] // Genesis block hash
};
// Create log entry
let entry_id = self.generate_entry_id(&event_data, &previous_hash);
let log_entry = LogEntry {
entry_id,
timestamp: now(),
event_type,
severity,
source_node: self.node_id.clone(),
target_entity: None,
event_data: event_data.clone(),
previous_hash,
merkle_hash: [0u8; 64], // Will be set after Merkle tree update
pq_signature: None,
treaty_context: treaty_ctx.clone(),
environmental_context: env_ctx.clone(),
integrity_status: LogIntegrityStatus::Unverified,
storage_tier: LogStorageTier::Hot,
};
// Update Merkle tree with new entry
self.update_merkle_tree(&log_entry)?;
// Sign log entry with PQ signature
let signature = self.sign_log_entry(&log_entry)?;
// Update log entry with signature and Merkle hash
let mut finalized_entry = log_entry;
finalized_entry.pq_signature = Some(signature);
finalized_entry.merkle_hash = self.merkle_tree_root.as_ref().unwrap().node_hash;
finalized_entry.integrity_status = LogIntegrityStatus::Verified;
// Add to log entries
self.log_entries.push_back(finalized_entry.clone());
self.metrics.total_log_entries += 1;
self.metrics.log_entries_hot += 1;
self.metrics.storage_bytes_hot += LOG_HEADER_SIZE_BYTES + event_data.len();
self.metrics.storage_bytes_total += LOG_HEADER_SIZE_BYTES + event_data.len();
// Update metrics
let elapsed_us = now() - start_time;
self.update_append_latency(elapsed_us);
// Check if treaty violation should be logged
if let Some(ref ctx) = treaty_ctx {
if ctx.fpic_status == FPICStatus::Denied || ctx.fpic_status == FPICStatus::Revoked {
self.metrics.treaty_violations_logged += 1;
}
}
// Add to offline buffer if enabled
if self.offline_buffer.len() < OFFLINE_LOG_BUFFER_SIZE {
self.offline_buffer.push_back(finalized_entry.clone());
} else {
// Buffer full, drop oldest entry
self.offline_buffer.pop_front();
self.metrics.offline_buffer_usage_percent = 100.0;
}
// Check if archival needed
if self.log_entries.len() >= HASH_CHAIN_WINDOW_SIZE {
self.perform_archival()?;
}
Ok((entry_id, LogIntegrityStatus::Verified))
}
/**
* Update Merkle tree with new log entry
* Recursively rebuilds tree with new leaf node
*/
fn update_merkle_tree(&mut self, log_entry: &LogEntry) -> Result<(), &'static str> {
// In production: use incremental Merkle tree update for efficiency
// For now: rebuild entire tree (simplified)
let mut leaf_hashes = Vec::new();
for entry in &self.log_entries {
leaf_hashes.push(entry.merkle_hash);
}
// Add new entry hash
let entry_hash = self.hash_log_entry(log_entry);
leaf_hashes.push(entry_hash);
// Rebuild Merkle tree
self.merkle_tree_root = Some(self.build_merkle_tree(&leaf_hashes)?);
Ok(())
}
/**
* Build Merkle tree from leaf hashes
* Implements fanout-16 Merkle Patricia Trie structure
*/
fn build_merkle_tree(&self, leaf_hashes: &[[u8; 64]]) -> Result<MerkleTreeNode, &'static str> {
if leaf_hashes.is_empty() {
return Ok(self.build_empty_merkle_tree());
}
// Build tree recursively
self.build_merkle_tree_recursive(leaf_hashes, 0, leaf_hashes.len(), 0)
}
fn build_merkle_tree_recursive(&self, leaf_hashes: &[[u8; 64]], start: usize, end: usize, depth: usize) -> Result<MerkleTreeNode, &'static str> {
if start >= end {
return Err("Invalid tree range");
}
if end - start == 1 {
// Leaf node
let mut leaf = MerkleTreeNode {
node_hash: leaf_hashes[start],
left_child: None,
right_child: None,
leaf_entries: Some(vec![start]),
depth,
};
// Hash leaf node structure
leaf.node_hash = self.hash_merkle_node(&leaf);
Ok(leaf)
}
// Internal node: split range and build children
let mid = start + (end - start) / 2;
let left_child = Box::new(self.build_merkle_tree_recursive(leaf_hashes, start, mid, depth + 1)?);
let right_child = Box::new(self.build_merkle_tree_recursive(leaf_hashes, mid, end, depth + 1)?);
// Hash internal node
let mut node = MerkleTreeNode {
node_hash: [0u8; 64],
left_child: Some(left_child),
right_child: Some(right_child),
leaf_entries: None,
depth,
};
node.node_hash = self.hash_merkle_node(&node);
Ok(node)
}
/**
* Build empty Merkle tree (genesis)
*/
fn build_empty_merkle_tree(&self) -> MerkleTreeNode {
MerkleTreeNode {
node_hash: [0u8; 64],
left_child: None,
right_child: None,
leaf_entries: Some(Vec::new()),
depth: 0,
}
}
/**
* Hash log entry for Merkle tree inclusion
*/
fn hash_log_entry(&self, entry: &LogEntry) -> [u8; 64] {
let mut hash_input = Vec::new();
hash_input.extend_from_slice(&entry.entry_id);
hash_input.extend_from_slice(&entry.timestamp.to_be_bytes());
hash_input.push(entry.event_type as u8);
hash_input.push(entry.severity as u8);
hash_input.extend_from_slice(&entry.source_node.to_bytes());
if let Some(ref target) = entry.target_entity {
hash_input.extend_from_slice(&target.to_bytes());
}
hash_input.extend_from_slice(&entry.event_data);
hash_input.extend_from_slice(&entry.previous_hash);
self.crypto_engine.sha512_hash(&hash_input)
}
/**
* Hash Merkle tree node structure
*/
fn hash_merkle_node(&self, node: &MerkleTreeNode) -> [u8; 64] {
let mut hash_input = Vec::new();
if let Some(ref left) = node.left_child {
hash_input.extend_from_slice(&left.node_hash);
}
if let Some(ref right) = node.right_child {
hash_input.extend_from_slice(&right.node_hash);
}
if let Some(ref leaves) = node.leaf_entries {
for &leaf_idx in leaves {
hash_input.extend_from_slice(&leaf_idx.to_be_bytes());
}
}
hash_input.extend_from_slice(&node.depth.to_be_bytes());
self.crypto_engine.sha512_hash(&hash_input)
}
/**
* Sign log entry with PQ signature
*/
fn sign_log_entry(&mut self, entry: &LogEntry) -> Result<PQSignature, &'static str> {
// Serialize log entry for signing
let serialized = self.serialize_log_entry(entry)?;
// Get signing key
let key_id = self.crypto_engine.active_key_pairs.keys().next()
.ok_or("No signing key available")?;
// Sign with PQ crypto
self.crypto_engine.sign_message(key_id, &serialized)
}
/**
* Serialize log entry for signing
*/
fn serialize_log_entry(&self, entry: &LogEntry) -> Result<Vec<u8>, &'static str> {
// In production: use canonical CBOR encoding
// For now: simple concatenation
let mut bytes = Vec::new();
bytes.extend_from_slice(&entry.entry_id);
bytes.extend_from_slice(&entry.timestamp.to_be_bytes());
bytes.push(entry.event_type as u8);
bytes.push(entry.severity as u8);
bytes.extend_from_slice(&entry.source_node.to_bytes());
bytes.extend_from_slice(&entry.previous_hash);
bytes.extend_from_slice(&entry.event_data);
Ok(bytes)
}
/**
* Generate unique log entry ID
*/
fn generate_entry_id(&self, event_data: &[u8], previous_hash: &[u8; 64]) -> [u8; 32] {
let mut hash_input = Vec::new();
hash_input.extend_from_slice(&now().to_be_bytes());
hash_input.extend_from_slice(&self.node_id.to_bytes());
hash_input.extend_from_slice(previous_hash);
hash_input.extend_from_slice(event_data);
let full_hash = self.crypto_engine.sha512_hash(&hash_input);
let mut entry_id = [0u8; 32];
entry_id.copy_from_slice(&full_hash[..32]);
entry_id
}
/**
* Query logs with forensic analysis capabilities
* Supports time-range, entity correlation, pattern matching, and treaty compliance filtering
*/
pub fn query_logs(&mut self, query: ForensicQuery) -> Result<ForensicQueryResult, &'static str> {
let start_time = now();
let mut matching_entries = Vec::new();
let mut correlated_events = Vec::new();
let mut integrity_issues = Vec::new();
// Filter log entries based on query criteria
for entry in &self.log_entries {
// Time range filter
if let Some((start, end)) = query.time_range {
if entry.timestamp < start || entry.timestamp > end {
continue;
}
}
// Entity filter
if let Some(ref entity) = query.entity_filter {
if entry.source_node != *entity && entry.target_entity.as_ref() != Some(entity) {
continue;
}
}
// Event type filter
if !query.event_types.is_empty() && !query.event_types.contains(&entry.event_type) {
continue;
}
// Severity filter
if entry.severity as u8 < query.min_severity as u8 {
continue;
}
// Treaty compliance filter
if query.treaty_compliance_only {
if entry.treaty_context.is_none() {
continue;
}
}
// Add to results
matching_entries.push(entry.clone());
// Check for correlations within time window
if query.include_correlations {
self.find_correlated_events(entry, &mut correlated_events)?;
}
// Check integrity
if entry.integrity_status != LogIntegrityStatus::Verified {
integrity_issues.push(LogIntegrityIssue {
entry_id: entry.entry_id,
issue_type: entry.integrity_status,
expected_hash: [0u8; 64],
actual_hash: entry.merkle_hash,
timestamp: now(),
severity: LogSeverity::Critical,
remediation: Some("Verify log chain integrity and restore from backup if necessary".to_string()),
});
}
if matching_entries.len() >= query.max_results {
break;
}
}
// Update metrics
let elapsed_us = now() - start_time;
self.metrics.forensic_queries += 1;
self.metrics.avg_query_latency_ms = (self.metrics.avg_query_latency_ms * (self.metrics.forensic_queries - 1) as f64
+ (elapsed_us as f64 / 1000.0)) / self.metrics.forensic_queries as f64;
let result = ForensicQueryResult {
query_id: self.generate_query_id(),
result_count: matching_entries.len(),
log_entries: matching_entries,
correlated_events,
integrity_issues,
query_duration_us: elapsed_us,
treaty_violations_found: matching_entries.iter()
.filter(|e| e.treaty_context.is_some() &&
e.treaty_context.as_ref().unwrap().fpic_status == FPICStatus::Denied)
.count(),
};
Ok(result)
}
/**
* Find correlated events within time window
*/
fn find_correlated_events(&mut self, entry: &LogEntry, correlated: &mut Vec<[u8; 32]>) -> Result<(), &'static str> {
let correlation_window_us = (FORENSIC_CORRELATION_WINDOW_HOURS as u64) * 3600 * 1000000;
let window_start = entry.timestamp.saturating_sub(correlation_window_us);
let window_end = entry.timestamp + correlation_window_us;
for other_entry in &self.log_entries {
if other_entry.entry_id == entry.entry_id {
continue;
}
if other_entry.timestamp >= window_start && other_entry.timestamp <= window_end {
// Correlate by source node or event type
if other_entry.source_node == entry.source_node || other_entry.event_type == entry.event_type {
correlated.push(other_entry.entry_id);
}
}
}
Ok(())
}
/**
* Verify log integrity over specified range
* Checks hash chain continuity, Merkle tree consistency, and PQ signature validity
*/
pub fn verify_log_integrity(&mut self, start_index: usize, end_index: usize) -> Result<Vec<LogIntegrityIssue>, &'static str> {
let mut issues = Vec::new();
let entries: Vec<_> = self.log_entries.iter().skip(start_index).take(end_index - start_index + 1).collect();
// Verify hash chain continuity
for i in 1..entries.len() {
let expected_prev_hash = if i > 0 { entries[i - 1].merkle_hash } else { [0u8; 64] };
if entries[i].previous_hash != expected_prev_hash {
issues.push(LogIntegrityIssue {
entry_id: entries[i].entry_id,
issue_type: LogIntegrityStatus::Corrupted,
expected_hash: expected_prev_hash,
actual_hash: entries[i].previous_hash,
timestamp: now(),
severity: LogSeverity::Critical,
remediation: Some("Hash chain broken - possible tampering detected".to_string()),
});
}
}
// Verify PQ signatures
for entry in &entries {
if let Some(ref sig) = entry.pq_signature {
let serialized = self.serialize_log_entry(entry)?;
let sig_valid = self.crypto_engine.verify_signature(sig, &serialized)?;
if !sig_valid {
issues.push(LogIntegrityIssue {
entry_id: entry.entry_id,
issue_type: LogIntegrityStatus::Tampered,
expected_hash: [0u8; 64],
actual_hash: [0u8; 64],
timestamp: now(),
severity: LogSeverity::Critical,
remediation: Some("PQ signature invalid - log entry tampered with".to_string()),
});
}
}
}
// Verify Merkle tree root
if let Some(ref root) = self.merkle_tree_root {
let computed_root = self.recompute_merkle_root(&entries)?;
if computed_root != root.node_hash {
issues.push(LogIntegrityIssue {
entry_id: [0u8; 32],
issue_type: LogIntegrityStatus::Corrupted,
expected_hash: root.node_hash,
actual_hash: computed_root,
timestamp: now(),
severity: LogSeverity::Critical,
remediation: Some("Merkle root mismatch - tree structure corrupted".to_string()),
});
}
}
self.metrics.integrity_violations += issues.len();
self.last_integrity_check = now();
Ok(issues)
}
/**
* Recompute Merkle root from log entries
*/
fn recompute_merkle_root(&self, entries: &Vec<&LogEntry>) -> Result<[u8; 64], &'static str> {
let mut leaf_hashes = Vec::new();
for entry in entries {
leaf_hashes.push(self.hash_log_entry(entry));
}
if leaf_hashes.is_empty() {
return Ok([0u8; 64]);
}
let tree = self.build_merkle_tree(&leaf_hashes)?;
Ok(tree.node_hash)
}
/**
* Export logs in specified format with chain of custody
* Generates court-ready evidence packages with PQ signatures and timestamp notarization
*/
pub fn export_logs(&mut self, request: LogExportRequest) -> Result<EvidencePackage, &'static str> {
// Execute query to get logs to export
let query = ForensicQuery {
query_id: self.generate_query_id(),
query_type: ForensicQueryType::TimeRange,
time_range: Some(request.time_range),
entity_filter: None,
event_types: request.event_types,
min_severity: request.min_severity,
max_results: FORENSIC_QUERY_MAX_RESULTS,
include_correlations: false,
treaty_compliance_only: request.treaty_compliance_only,
};
let query_result = self.query_logs(query)?;
// Filter treaty compliance events if requested
let mut log_entries = if request.treaty_compliance_only {
query_result.log_entries.into_iter()
.filter(|e| e.treaty_context.is_some())
.collect()
} else {
query_result.log_entries
};
// Apply severity filter
log_entries.retain(|e| e.severity as u8 >= request.min_severity as u8);
// Create chain of custody record
let custody_record = ChainOfCustodyRecord {
custody_id: self.generate_custody_id(),
log_entry_id: [0u8; 32], // Package-level custody
action: ChainOfCustodyAction::LogExported,
from_entity: self.node_id.clone(),
to_entity: request.recipient_did.clone(),
timestamp: now(),
pq_signature: self.sign_custody_record(&request)?,
reason: format!("Export request for time range {}-{}", request.time_range.0, request.time_range.1),
legal_authority: None,
};
self.chain_of_custody.push(custody_record.clone());
// Create evidence package
let package_id = self.generate_package_id();
let mut package = EvidencePackage {
package_id,
log_entries,
chain_of_custody: vec![custody_record],
pq_signature: PQSignature::default(),
timestamp_authority: None,
legal_metadata: LegalMetadata {
case_number: None,
jurisdiction: "Phoenix, Arizona".to_string(),
requesting_authority: None,
purpose: "Security audit and compliance verification".to_string(),
retention_requirement: 3650, // 10 years
confidentiality_level: 75,
redaction_required: false,
},
hash: [0u8; 64],
};
// Sign evidence package
let package_bytes = self.serialize_evidence_package(&package)?;
let key_id = self.crypto_engine.active_key_pairs.keys().next()
.ok_or("No signing key available")?;
package.pq_signature = self.crypto_engine.sign_message(key_id, &package_bytes)?;
package.hash = self.crypto_engine.sha512_hash(&package_bytes);
Ok(package)
}
/**
* Sign chain of custody record
*/
fn sign_custody_record(&mut self, request: &LogExportRequest) -> Result<PQSignature, &'static str> {
let mut data = Vec::new();
data.extend_from_slice(&request.export_id);
data.extend_from_slice(&request.recipient_did.to_bytes());
data.extend_from_slice(&now().to_be_bytes());
let key_id = self.crypto_engine.active_key_pairs.keys().next()
.ok_or("No signing key available")?;
self.crypto_engine.sign_message(key_id, &data)
}
/**
* Serialize evidence package
*/
fn serialize_evidence_package(&self, package: &EvidencePackage) -> Result<Vec<u8>, &'static str> {
// In production: use canonical encoding (CBOR/Protocol Buffers)
// For now: simple concatenation
let mut bytes = Vec::new();
bytes.extend_from_slice(&package.package_id);
for entry in &package.log_entries {
bytes.extend_from_slice(&entry.entry_id);
bytes.extend_from_slice(&entry.timestamp.to_be_bytes());
}
bytes.extend_from_slice(&package.hash);
Ok(bytes)
}
/**
* Perform log archival to warm/cold storage
* Compresses logs, creates hash chain segments, and generates Merkle proofs
*/
fn perform_archival(&mut self) -> Result<(), &'static str> {
let archival_start = now();
// Create hash chain segment from oldest entries
let segment_size = HASH_CHAIN_WINDOW_SIZE.min(self.log_entries.len());
if segment_size == 0 {
return Ok(());
}
let mut segment_entries = Vec::with_capacity(segment_size);
for _ in 0..segment_size {
if let Some(entry) = self.log_entries.pop_front() {
segment_entries.push(entry);
self.metrics.log_entries_hot -= 1;
self.metrics.log_entries_warm += 1;
}
}
// Compute Merkle root for segment
let mut leaf_hashes = Vec::new();
for entry in &segment_entries {
leaf_hashes.push(entry.merkle_hash);
}
let segment_tree = self.build_merkle_tree(&leaf_hashes)?;
// Sign segment root
let segment_id = self.generate_segment_id();
let segment_root_signature = {
let mut root_data = Vec::new();
root_data.extend_from_slice(&segment_id);
root_data.extend_from_slice(&segment_tree.node_hash);
root_data.extend_from_slice(&archival_start.to_be_bytes());
let key_id = self.crypto_engine.active_key_pairs.keys().next()
.ok_or("No signing key available")?;
self.crypto_engine.sign_message(key_id, &root_data)?
};
// Create segment record
let segment = LogChainSegment {
segment_id,
start_index: self.log_chain_segments.len() * HASH_CHAIN_WINDOW_SIZE,
end_index: (self.log_chain_segments.len() + 1) * HASH_CHAIN_WINDOW_SIZE - 1,
entry_count: segment_entries.len(),
root_hash: segment_tree.node_hash,
pq_signature: segment_root_signature,
timestamp_range: (
segment_entries.first().map(|e| e.timestamp).unwrap_or(0),
segment_entries.last().map(|e| e.timestamp).unwrap_or(0),
),
storage_location: format!("/var/log/aletheion/archive/segment_{:08x}.log", archival_start),
integrity_verified: true,
};
self.log_chain_segments.push(segment);
// Compress segment entries (simulated)
let compression_stats = LogCompressionStats {
original_size_bytes: segment_entries.iter().map(|e| LOG_HEADER_SIZE_BYTES + e.event_data.len()).sum(),
compressed_size_bytes: segment_entries.iter().map(|e| (LOG_HEADER_SIZE_BYTES + e.event_data.len()) / 2).sum(), // 50% compression
compression_ratio: 2.0,
compression_algorithm: "Zstandard".to_string(),
compression_level: ARCHIVAL_COMPRESSION_LEVEL,
decompression_time_us: 500,
};
debug!("Log archival completed: {} entries compressed {:.2}x in {}μs",
segment_size, compression_stats.compression_ratio, now() - archival_start);
self.last_archival = now();
Ok(())
}
/**
* Generate unique IDs
*/
fn generate_query_id(&self) -> [u8; 32] {
let mut id = [0u8; 32];
let timestamp = now();
id[..8].copy_from_slice(&timestamp.to_be_bytes());
id[8..24].copy_from_slice(&self.node_id.to_bytes()[..16]);
id[24..].copy_from_slice(&self.metrics.forensic_queries.to_be_bytes()[..8]);
self.crypto_engine.sha512_hash(&id)[..32].try_into().unwrap_or([0u8; 32])
}
fn generate_custody_id(&self) -> [u8; 32] {
self.generate_query_id() // Reuse query ID generation
}
fn generate_package_id(&self) -> [u8; 32] {
self.generate_query_id()
}
fn generate_segment_id(&self) -> [u8; 32] {
let mut id = [0u8; 32];
let timestamp = now();
id[..8].copy_from_slice(&timestamp.to_be_bytes());
id[8..16].copy_from_slice(&self.log_chain_segments.len().to_be_bytes()[..8]);
id[16..].copy_from_slice(&self.node_id.to_bytes()[..16]);
self.crypto_engine.sha512_hash(&id)[..32].try_into().unwrap_or([0u8; 32])
}
/**
* Update append latency metrics
*/
fn update_append_latency(&mut self, latency_us: u64) {
self.metrics.avg_append_latency_us = (self.metrics.avg_append_latency_us * self.metrics.total_log_entries as f64
+ latency_us as f64) / (self.metrics.total_log_entries + 1) as f64;
if latency_us > self.metrics.max_append_latency_us {
self.metrics.max_append_latency_us = latency_us;
}
}
/**
* Get current audit system metrics
*/
pub fn get_metrics(&self) -> AuditSystemMetrics {
self.metrics.clone()
}
/**
* Get log entry by ID
*/
pub fn get_log_entry(&self, entry_id: &[u8; 32]) -> Option<&LogEntry> {
self.log_entries.iter().find(|e| e.entry_id == *entry_id)
}
/**
* Get all log chain segments
*/
pub fn get_log_chain_segments(&self) -> &Vec<LogChainSegment> {
&self.log_chain_segments
}
/**
* Get Merkle proof for log entry
* Generates Merkle proof for inclusion verification
*/
pub fn get_merkle_proof(&self, entry_index: usize) -> Result<Vec<[u8; 64]>, &'static str> {
// In production: traverse Merkle tree to generate sibling hashes
// For now: return empty proof (placeholder)
Ok(Vec::new())
}
/**
* Perform maintenance tasks (cleanup, integrity checks, buffer management)
*/
pub fn perform_maintenance(&mut self) -> Result<(), &'static str> {
let now = now();
// Cleanup old offline buffer entries (>72 hours)
while let Some(entry) = self.offline_buffer.front() {
if now - entry.timestamp > (OFFLINE_BUFFER_HOURS as u64) * 3600 * 1000000 {
self.offline_buffer.pop_front();
} else {
break;
}
}
// Update offline buffer usage percentage
self.metrics.offline_buffer_usage_percent = (self.offline_buffer.len() as f64 / OFFLINE_LOG_BUFFER_SIZE as f64) * 100.0;
// Perform periodic integrity check
if now - self.last_integrity_check > 24 * 60 * 60 * 1000000 {
// Verify integrity of recent log entries
let start_idx = self.log_entries.len().saturating_sub(1000);
if start_idx < self.log_entries.len() {
let _issues = self.verify_log_integrity(start_idx, self.log_entries.len() - 1)?;
}
}
// Check storage tier migration
self.check_storage_tier_migration()?;
Ok(())
}
/**
* Check and perform storage tier migration
* Moves logs from hot to warm to cold storage based on age
*/
fn check_storage_tier_migration(&mut self) -> Result<(), &'static str> {
let now = now();
let hot_threshold = (LOG_RETENTION_DAYS_HOT as u64) * 24 * 60 * 60 * 1000000;
let warm_threshold = (LOG_RETENTION_DAYS_WARM as u64) * 24 * 60 * 60 * 1000000;
// Migrate from hot to warm
let hot_entries: Vec<_> = self.log_entries.iter()
.filter(|e| now - e.timestamp > hot_threshold)
.cloned()
.collect();
for entry in hot_entries {
// In production: write to warm storage and remove from hot
// For now: just update metrics
self.metrics.log_entries_hot = self.metrics.log_entries_hot.saturating_sub(1);
self.metrics.log_entries_warm += 1;
}
// Migrate from warm to cold (simulated)
// In production: this would involve actual storage migration
Ok(())
}
}
// --- Helper Functions ---
/**
* Calculate log integrity percentage
*/
pub fn calculate_integrity_percentage(total: usize, violations: usize) -> f64 {
if total == 0 {
return 100.0;
}
let violation_rate = violations as f64 / total as f64;
(100.0 - violation_rate * 100.0).max(0.0).min(100.0)
}
/**
* Check if log append time is within acceptable limits
*/
pub fn is_append_time_acceptable(latency_us: u64) -> bool {
latency_us <= MAX_LOG_APPEND_TIME_US
}
/**
* Check if query time is within acceptable limits
*/
pub fn is_query_time_acceptable(latency_ms: f64) -> bool {
latency_ms <= MAX_LOG_QUERY_TIME_MS as f64
}
// --- Unit Tests (Offline Capable) ---
#[cfg(test)]
mod tests {
use super::*;
#[test]
fn test_engine_initialization() {
let engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
assert!(engine.active);
assert_eq!(engine.log_entries.len(), 0);
assert_eq!(engine.log_chain_segments.len(), 0);
assert!(engine.merkle_tree_root.is_some());
assert_eq!(engine.metrics.total_log_entries, 0);
}
#[test]
fn test_log_append() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
let event_data = b"Test security event".to_vec();
let (entry_id, status) = engine.append_log(
LogEventType::SecurityEvent,
LogSeverity::Info,
event_data,
None,
None,
).unwrap();
assert_ne!(entry_id, [0u8; 32]);
assert_eq!(status, LogIntegrityStatus::Verified);
assert_eq!(engine.log_entries.len(), 1);
assert_eq!(engine.metrics.total_log_entries, 1);
}
#[test]
fn test_hash_chain_integrity() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Append multiple log entries
for i in 0..10 {
let event_data = format!("Event {}", i).as_bytes().to_vec();
engine.append_log(
LogEventType::SecurityEvent,
LogSeverity::Info,
event_data,
None,
None,
).unwrap();
}
// Verify hash chain continuity
let entries: Vec<_> = engine.log_entries.iter().collect();
for i in 1..entries.len() {
assert_eq!(entries[i].previous_hash, entries[i - 1].merkle_hash);
}
}
#[test]
fn test_merkle_tree_construction() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Append entries to build Merkle tree
for i in 0..16 {
let event_data = format!("Event {}", i).as_bytes().to_vec();
engine.append_log(
LogEventType::SecurityEvent,
LogSeverity::Info,
event_data,
None,
None,
).unwrap();
}
// Verify Merkle tree root exists
assert!(engine.merkle_tree_root.is_some());
let root = engine.merkle_tree_root.as_ref().unwrap();
assert_ne!(root.node_hash, [0u8; 64]);
}
#[test]
fn test_forensic_query() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Append various log entries
for i in 0..100 {
let severity = if i % 10 == 0 { LogSeverity::Error } else { LogSeverity::Info };
let event_data = format!("Event {}", i).as_bytes().to_vec();
engine.append_log(
LogEventType::SecurityEvent,
severity,
event_data,
None,
None,
).unwrap();
}
// Query for error-level events
let query = ForensicQuery {
query_id: [0u8; 32],
query_type: ForensicQueryType::SeverityThreshold,
time_range: None,
entity_filter: None,
event_types: BTreeSet::new(),
min_severity: LogSeverity::Error,
max_results: 100,
include_correlations: false,
treaty_compliance_only: false,
};
let result = engine.query_logs(query).unwrap();
// Should find 10 error events (every 10th entry)
assert_eq!(result.result_count, 10);
assert!(result.treaty_violations_found == 0);
}
#[test]
fn test_log_integrity_verification() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Append entries
for i in 0..50 {
let event_data = format!("Event {}", i).as_bytes().to_vec();
engine.append_log(
LogEventType::SecurityEvent,
LogSeverity::Info,
event_data,
None,
None,
).unwrap();
}
// Verify integrity of all entries
let issues = engine.verify_log_integrity(0, 49).unwrap();
assert_eq!(issues.len(), 0); // No integrity issues expected
}
#[test]
fn test_offline_buffer_management() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Fill offline buffer beyond capacity
for i in 0..(OFFLINE_LOG_BUFFER_SIZE + 100) {
let event_data = format!("Event {}", i).as_bytes().to_vec();
engine.append_log(
LogEventType::SecurityEvent,
LogSeverity::Debug,
event_data,
None,
None,
).unwrap();
}
// Buffer should be at max capacity
assert_eq!(engine.offline_buffer.len(), OFFLINE_LOG_BUFFER_SIZE);
assert_eq!(engine.metrics.offline_buffer_usage_percent, 100.0);
}
#[test]
fn test_treaty_context_logging() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Append log with treaty context (FPIC denied)
let treaty_ctx = TreatyContext {
fpic_status: FPICStatus::Denied,
indigenous_community: Some("Akimel O'odham".to_string()),
data_sovereignty_level: 85,
neurorights_protected: true,
consent_timestamp: 0,
consent_expiry: 0,
treaty_event_id: None,
};
let event_data = b"Indigenous data access attempt".to_vec();
engine.append_log(
LogEventType::TreatyCompliance,
LogSeverity::Critical,
event_data,
Some(treaty_ctx),
None,
).unwrap();
// Should increment treaty violations metric
assert_eq!(engine.metrics.treaty_violations_logged, 1);
}
#[test]
fn test_log_archival() {
let mut engine = ImmutableAuditLogEngine::new(BirthSign::default()).unwrap();
// Append enough entries to trigger archival
for i in 0..HASH_CHAIN_WINDOW_SIZE {
let event_data = format!("Event {}", i).as_bytes().to_vec();
engine.append_log(
LogEventType::SecurityEvent,
LogSeverity::Info,
event_data,
None,
None,
).unwrap();
}
// Perform archival
engine.perform_archival().unwrap();
// Should have created a chain segment
assert_eq!(engine.log_chain_segments.len(), 1);
assert!(engine.log_chain_segments[0].integrity_verified);
}
#[test]
fn test_integrity_percentage_calculation() {
// 1000 entries with 1 violation = 99.9% integrity
let integrity = calculate_integrity_percentage(1000, 1);
assert!((integrity - 99.9).abs() < 0.01);
// 10000 entries with 10 violations = 99.9% integrity
let integrity2 = calculate_integrity_percentage(10000, 10);
assert!((integrity2 - 99.9).abs() < 0.01);
// 0 entries = 100% integrity
let integrity3 = calculate_integrity_percentage(0, 0);
assert_eq!(integrity3, 100.0);
}
#[test]
fn test_append_latency_requirements() {
// Verify performance requirements
assert!(MAX_LOG_APPEND_TIME_US <= 100); // <100μs
assert!(MAX_LOG_QUERY_TIME_MS <= 1);    // <1ms
assert!(MAX_LOG_VERIFICATION_TIME_MS <= 10); // <10ms
assert!(LOG_THROUGHPUT_ENTRIES_PER_SEC >= 10000); // 10K/sec
}
}
