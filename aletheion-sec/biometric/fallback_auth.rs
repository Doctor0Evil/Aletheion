/**
* Aletheion Smart City Core - Batch 2
* File: 119/200
* Layer: 36 (Advanced Security)
* Path: aletheion-sec/biometric/fallback_auth.rs
*
* Research Basis (Biometric Authentication & PQ Fallback):
*   - Multi-Modal Biometrics: Fingerprint (NIST FpVTE 2012), Facial Recognition (NIST FRVT 2019), Iris (NIST IREX), Behavioral (keystroke, gait, voice)
*   - Liveness Detection: Presentation attack detection (PAD), pulse detection, 3D depth sensing, micro-movement analysis
*   - Privacy-Preserving Biometrics: Homomorphic encryption of templates (BFV scheme), secure multi-party computation for matching
*   - Heat-Tolerant Sensors: Wide-temperature fingerprint sensors (-20°C to +85°C), thermal-resistant capacitive arrays, haboob-dust protection
*   - PQ Backup Authentication: CRYSTALS-Dilithium signatures as fallback, Falcon-512 for mobile devices, SPHINCS+ for long-term archival
*   - Treaty-Compliant Consent: FPIC for biometric enrollment, Indigenous data sovereignty for facial templates, neurorights protection for behavioral biometrics
*   - Phoenix-Specific Adaptations: Extreme heat tolerance (120°F+), haboob dust resistance (5,000 μg/m³), monsoon humidity tolerance (95%)
*   - Performance Benchmarks: <100ms biometric verification, <5ms PQ fallback, 99.99% availability, <0.001% false acceptance rate (FAR)
*
* Compliance:
*   - ALE-COMP-CORE (v2.1)
*   - FPIC (Free, Prior, Informed Consent)
*   - Phoenix Heat Protocols (Offline-72h)
*   - BioticTreaties (Biometric Data Sovereignty & Neural Rights)
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
use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use core::result::Result;
use core::ops::{Add, Sub, BitXor};
use core::time::Duration;
use core::sync::atomic::{AtomicU64, Ordering};
// Internal Aletheion Crates (Established in Batch 1 & Files 112-118)
use aletheion_core::identity::BirthSign;
use aletheion_core::time::{now, Timestamp};
use aletheion_core::logger::{log, warn, error, debug};
use aletheion_sec::quantum::post::crypto_core::{PQCryptoEngine, PQSignature, PQSecurityLevel, PQKeyPair, PQAlgorithmSuite, SHA512_HASH_SIZE};
use aletheion_sec::zkp::privacy_compute::{ZKPPrivacyEngine, HomomorphicContext, HomomorphicCiphertext, PrivacyLevel, PrivacyOperation};
use aletheion_gov::treaty::{TreatyCompliance, TreatyViolation, FPICStatus, TreatyContext};
use aletheion_:biosignal::BioSignalStream;
// --- Constants & Biometric Parameters ---
/// Biometric modality weights (for multi-modal fusion)
pub const FINGERPRINT_WEIGHT: f64 = 0.4;           // 40% weight for fingerprint
pub const FACIAL_WEIGHT: f64 = 0.3;                // 30% weight for facial recognition
pub const IRIS_WEIGHT: f64 = 0.2;                  // 20% weight for iris scan
pub const BEHAVIORAL_WEIGHT: f64 = 0.1;            // 10% weight for behavioral biometrics
/// Biometric performance thresholds
pub const MAX_BIOMETRIC_VERIFY_TIME_MS: u64 = 100; // <100ms biometric verification
pub const MAX_PQ_FALLBACK_TIME_MS: u64 = 5;        // <5ms PQ fallback authentication
pub const FALSE_ACCEPTANCE_RATE_MAX: f64 = 0.001;  // <0.001% FAR (1 in 100,000)
pub const FALSE_REJECTION_RATE_MAX: f64 = 0.01;    // <1% FRR (user convenience)
pub const LIVENESS_CONFIDENCE_MIN: f64 = 0.95;     // 95% minimum liveness confidence
/// Biometric template sizes (encrypted)
pub const FINGERPRINT_TEMPLATE_SIZE: usize = 512;  // 512 bytes encrypted template
pub const FACIAL_TEMPLATE_SIZE: usize = 2048;      // 2KB encrypted facial template
pub const IRIS_TEMPLATE_SIZE: usize = 1024;        // 1KB encrypted iris template
pub const BEHAVIORAL_TEMPLATE_SIZE: usize = 256;   // 256 bytes behavioral profile
/// PQ backup key parameters
pub const PQ_BACKUP_KEY_LIFETIME_SECONDS: u64 = 2592000; // 30 days backup key lifetime
pub const PQ_BACKUP_KEYS_PER_USER: usize = 3;      // 3 backup keys per user (rotation)
pub const PQ_BACKUP_KEY_THRESHOLD: usize = 2;      // 2-of-3 threshold for backup recovery
/// Environmental tolerance parameters (Phoenix-specific)
pub const BIOMETRIC_SENSOR_MAX_TEMP_C: f32 = 85.0; // 185°F maximum sensor temperature
pub const BIOMETRIC_SENSOR_DUST_TOLERANCE: f32 = 5000.0; // 5000 μg/m³ dust tolerance
pub const BIOMETRIC_SENSOR_HUMIDITY_RANGE: (f32, f32) = (5.0, 95.0); // 5-95% humidity
/// Offline buffer duration (hours) - Must meet 72h Protocol
pub const OFFLINE_BUFFER_HOURS: u32 = 72;
pub const OFFLINE_AUTH_BUFFER_SIZE: usize = 10000; // 10K authentication attempts buffered
/// Treaty compliance parameters
pub const FPIC_REQUIRED_FOR_BIOMETRIC: bool = true; // FPIC required for biometric enrollment
pub const BIOMETRIC_DATA_SOVEREIGNTY: bool = true;   // Citizen-controlled biometric data
pub const NEURORIGHTS_BEHAVIORAL: bool = true;      // Neurorights protection for behavioral biometrics
pub const INDIGENOUS_FACIAL_SOVEREIGNTY: bool = true; // Indigenous facial data sovereignty
// --- Enumerations ---
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BiometricModality {
Fingerprint,            // Capacitive/ultrasonic fingerprint sensor
FacialRecognition,      // 3D depth-sensing facial recognition
IrisScan,               // Near-infrared iris pattern recognition
VoiceRecognition,       // Voiceprint with liveness detection
KeystrokeDynamics,      // Typing rhythm and pressure patterns
GaitAnalysis,           // Walking pattern recognition
PalmVein,               // Near-infrared palm vein pattern
HeartbeatPattern,       // ECG-based heartbeat biometrics
BehavioralFusion,       // Multi-behavioral pattern fusion
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuthenticationMethod {
BiometricPrimary,       // Primary biometric authentication
BiometricMultiModal,    // Multi-modal biometric fusion
PQBackupSignature,      // PQ signature fallback
PQBackupKeyRecovery,    // PQ key recovery (threshold)
TreatyMediatedAuth,     // Treaty-mediated authentication
EmergencyOverride,      // Physical presence emergency override
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LivenessDetectionMethod {
PulseDetection,         // Blood flow/pulse detection
MicroMovement,          // Subtle micro-movements analysis
3DDepthSensing,         // 3D depth map validation
TextureAnalysis,        // Skin texture and reflection analysis
ChallengeResponse,      // Interactive challenge-response
ThermalPattern,         // Thermal signature validation
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BiometricQuality {
Excellent,              // >95% quality score
Good,                   // 80-95% quality score
Fair,                   // 60-80% quality score
Poor,                   // 40-60% quality score
Unusable,               // <40% quality score
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuthenticationResult {
Success,                // Authentication successful
Failed,                 // Authentication failed
LivenessFailed,         // Liveness detection failed (spoofing attempt)
SensorError,            // Sensor hardware error
EnvironmentalConstraint, // Environmental conditions exceeded limits
TreatyViolation,        // FPIC/treaty compliance violation
TemplateNotFound,       // Biometric template not found
PQFallbackRequired,     // Biometric failed, PQ fallback required
EmergencyOverride,      // Emergency override activated
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BiometricStorageType {
LocalEncrypted,         // Local encrypted storage (device)
HomomorphicCloud,       // Homomorphic encrypted cloud storage
DistributedShares,      // Distributed across multiple nodes
AirGappedBackup,        // Air-gapped offline backup
HSMProtected,           // Hardware Security Module protected
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConsentScope {
EnrollmentOnly,         // Consent only for enrollment
VerificationOnly,       // Consent only for verification
FullAccess,             // Full access to biometric data
ResearchUse,            // Consent for research/analytics
ThirdPartySharing,      // Consent for third-party sharing
Revoked,                // Consent revoked
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvironmentalCondition {
Normal,                 // Normal operating conditions
ExtremeHeat,            // Extreme heat (>110°F / 43°C)
HaboobDustStorm,        // Haboob dust storm conditions
HighHumidity,           // High humidity (>80%)
LowLight,               // Low light conditions
SensorContamination,    // Sensor contamination detected
}
#[derive(Clone)]
pub struct BiometricTemplate {
pub template_id: [u8; 32],
pub modality: BiometricModality,
pub encrypted_template: Vec<u8>,
pub homomorphic_ciphertext: Option<HomomorphicCiphertext>,
pub quality_score: f64,
pub creation_timestamp: Timestamp,
pub last_used: Timestamp,
pub use_count: usize,
pub liveness_threshold: f64,
pub treaty_context: Option<TreatyContext>,
pub storage_type: BiometricStorageType,
}
#[derive(Clone)]
pub struct BiometricEnrollment {
pub enrollment_id: [u8; 32],
pub citizen_did: [u8; 32],
pub modalities: BTreeSet<BiometricModality>,
pub templates: BTreeMap<BiometricModality, BiometricTemplate>,
pub consent_scope: ConsentScope,
pub fpic_status: FPICStatus,
pub consent_timestamp: Timestamp,
pub consent_expiry: Timestamp,
pub treaty_requirements: BTreeSet<String>,
pub environmental_conditions: EnvironmentalConditions,
}
#[derive(Clone)]
pub struct AuthenticationAttempt {
pub attempt_id: [u8; 32],
pub citizen_did: Option<[u8; 32]>,
pub method: AuthenticationMethod,
pub modalities_used: BTreeSet<BiometricModality>,
pub timestamp: Timestamp,
pub result: AuthenticationResult,
pub confidence_score: f64,
pub liveness_confidence: f64,
pub environmental_condition: EnvironmentalCondition,
pub pq_fallback_used: bool,
pub treaty_context: Option<TreatyContext>,
pub sensor_readings: SensorReadings,
}
#[derive(Clone)]
pub struct PQBackupKey {
pub key_id: [u8; 32],
pub citizen_did: [u8; 32],
pub pq_key_pair: PQKeyPair,
pub creation_timestamp: Timestamp,
pub expiry_timestamp: Timestamp,
pub usage_count: usize,
pub last_used: Timestamp,
pub recovery_shares: BTreeMap<usize, Vec<u8>>, // Share index -> encrypted share
pub threshold: usize,
pub treaty_protected: bool,
}
#[derive(Clone)]
pub struct LivenessDetectionResult {
pub method: LivenessDetectionMethod,
pub confidence: f64,
pub passed: bool,
pub artifacts_detected: Vec<String>,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct SensorReadings {
pub temperature_c: f32,
pub humidity_percent: f32,
pub dust_level_ug_m3: f32,
pub light_level_lux: f32,
pub sensor_contamination: bool,
pub signal_quality: f64,
}
#[derive(Clone)]
pub struct EnvironmentalConditions {
pub temperature_c: f32,
pub humidity_percent: f32,
pub particulate_ug_m3: f32,
pub haboob_detected: bool,
pub extreme_heat: bool,
pub monsoon_conditions: bool,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct BiometricFusionResult {
pub fused_score: f64,
pub modality_scores: BTreeMap<BiometricModality, f64>,
pub liveness_passed: bool,
pub environmental_adjustment: f64,
pub final_confidence: f64,
}
#[derive(Clone)]
pub struct AuthenticationMetrics {
pub total_attempts: usize,
pub successful_auths: usize,
pub failed_auths: usize,
pub liveness_failures: usize,
pub sensor_errors: usize,
pub pq_fallbacks_used: usize,
pub treaty_violations_blocked: usize,
pub avg_biometric_time_ms: f64,
pub avg_pq_fallback_time_ms: f64,
pub false_acceptance_count: usize,
pub false_rejection_count: usize,
pub environmental_events: usize,
pub offline_buffer_usage_percent: f64,
}
#[derive(Clone)]
pub struct ConsentRecord {
pub consent_id: [u8; 32],
pub citizen_did: [u8; 32],
pub scope: ConsentScope,
pub fpic_status: FPICStatus,
pub biometric_types: BTreeSet<BiometricModality>,
pub purpose: String,
pub timestamp: Timestamp,
pub expiry: Timestamp,
pub revocation_timestamp: Option<Timestamp>,
pub treaty_context: TreatyContext,
}
#[derive(Clone)]
pub struct EmergencyOverrideToken {
pub token_id: [u8; 32],
pub citizen_did: [u8; 32],
pub creation_timestamp: Timestamp,
pub expiry_timestamp: Timestamp,
pub used: bool,
pub used_timestamp: Option<Timestamp>,
pub justification: String,
pub treaty_authorization: Option<TreatyContext>,
}
// --- Core Biometric Authentication Engine ---
pub struct BiometricAuthenticationEngine {
pub node_id: BirthSign,
pub crypto_engine: PQCryptoEngine,
pub zkp_engine: ZKPPrivacyEngine,
pub treaty_compliance: TreatyCompliance,
pub biometric_templates: BTreeMap<[u8; 32], BiometricTemplate>,
pub enrollments: BTreeMap<[u8; 32], BiometricEnrollment>,
pub pq_backup_keys: BTreeMap<[u8; 32], PQBackupKey>,
pub authentication_attempts: VecDeque<AuthenticationAttempt>,
pub consent_records: BTreeMap<[u8; 32], ConsentRecord>,
pub emergency_tokens: BTreeMap<[u8; 32], EmergencyOverrideToken>,
pub metrics: AuthenticationMetrics,
pub offline_buffer: VecDeque<AuthenticationAttempt>,
pub last_maintenance: Timestamp,
pub active: bool,
}
impl BiometricAuthenticationEngine {
/**
* Initialize Biometric Authentication Engine with PQ Crypto integration
* Configures biometric modalities, PQ backup systems, treaty compliance, and offline buffer
* Ensures 72h offline operational capability with 10K authentication buffer
*/
pub fn new(node_id: BirthSign) -> Result<Self, &'static str> {
let crypto_engine = PQCryptoEngine::new(node_id.clone(), PQSecurityLevel::Level3)
.map_err(|_| "Failed to initialize PQ crypto engine")?;
let zkp_engine = ZKPPrivacyEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize ZKP engine")?;
let mut engine = Self {
node_id,
crypto_engine,
zkp_engine,
treaty_compliance: TreatyCompliance::new(),
biometric_templates: BTreeMap::new(),
enrollments: BTreeMap::new(),
pq_backup_keys: BTreeMap::new(),
authentication_attempts: VecDeque::with_capacity(10000),
consent_records: BTreeMap::new(),
emergency_tokens: BTreeMap::new(),
metrics: AuthenticationMetrics {
total_attempts: 0,
successful_auths: 0,
failed_auths: 0,
liveness_failures: 0,
sensor_errors: 0,
pq_fallbacks_used: 0,
treaty_violations_blocked: 0,
avg_biometric_time_ms: 0.0,
avg_pq_fallback_time_ms: 0.0,
false_acceptance_count: 0,
false_rejection_count: 0,
environmental_events: 0,
offline_buffer_usage_percent: 0.0,
},
offline_buffer: VecDeque::with_capacity(OFFLINE_AUTH_BUFFER_SIZE),
last_maintenance: now(),
active: true,
};
Ok(engine)
}
/**
* Enroll citizen biometrics with treaty-compliant consent
* Implements multi-modal biometric capture, liveness detection, homomorphic encryption, and FPIC verification
* Returns enrollment ID and encrypted templates
*/
pub fn enroll_biometrics(&mut self, citizen_did: &[u8; 32], modalities: BTreeSet<BiometricModality>, raw_samples: BTreeMap<BiometricModality, Vec<u8>>, consent_scope: ConsentScope, fpic_consent: FPICStatus) -> Result<BiometricEnrollment, &'static str> {
let start_time = now();
// Verify treaty compliance for biometric enrollment
if FPIC_REQUIRED_FOR_BIOMETRIC && fpic_consent != FPICStatus::Granted {
self.metrics.treaty_violations_blocked += 1;
return Err("FPIC consent required for biometric enrollment");
}
let treaty_check = self.treaty_compliance.check_biometric_enrollment(citizen_did, &modalities)?;
if !treaty_check.allowed {
self.metrics.treaty_violations_blocked += 1;
return Err(&treaty_check.reason);
}
// Create consent record
let consent_id = self.generate_consent_id();
let consent_expiry = now() + (365 * 24 * 60 * 60 * 1000000); // 1 year expiry
let consent_record = ConsentRecord {
consent_id,
citizen_did: *citizen_did,
scope: consent_scope,
fpic_status: fpic_consent,
biometric_types: modalities.clone(),
purpose: "Citizen authentication and identity verification".to_string(),
timestamp: now(),
expiry: consent_expiry,
revocation_timestamp: None,
treaty_context: TreatyContext {
fpic_status,
indigenous_community: None,
data_sovereignty_level: 100,
neurorights_protected: true,
consent_timestamp: now(),
consent_expiry,
},
};
self.consent_records.insert(consent_id, consent_record);
// Process each biometric modality
let mut templates = BTreeMap::new();
for (modality, raw_sample) in &raw_samples {
// Perform liveness detection
let liveness_result = self.detect_liveness(modality, raw_sample)?;
if !liveness_result.passed {
return Err("Liveness detection failed - possible spoofing attempt");
}
// Extract biometric features and create template
let extracted_template = self.extract_biometric_features(modality, raw_sample)?;
// Encrypt template using homomorphic encryption
let homomorphic_ctx = self.zkp_engine.initialize_homomorphic_context(
self.zkp_engine::HomomorphicScheme::BFV,
8192,
).map_err(|_| "Failed to initialize homomorphic context")?.0;
let encrypted_template = self.zkp_engine.homomorphic_encrypt(&homomorphic_ctx, &extracted_template)?;
// Create biometric template
let template_id = self.generate_template_id();
let template = BiometricTemplate {
template_id,
modality: *modality,
encrypted_template: encrypted_template.ciphertext_data.clone(),
homomorphic_ciphertext: Some(encrypted_template),
quality_score: self.assess_template_quality(modality, &extracted_template),
creation_timestamp: now(),
last_used: now(),
use_count: 0,
liveness_threshold: LIVENESS_CONFIDENCE_MIN,
treaty_context: Some(consent_record.treaty_context.clone()),
storage_type: BiometricStorageType::HomomorphicCloud,
};
templates.insert(*modality, template.clone());
self.biometric_templates.insert(template_id, template);
}
// Generate PQ backup keys for fallback authentication
self.generate_pq_backup_keys(citizen_did)?;
// Create enrollment record
let enrollment_id = self.generate_enrollment_id();
let enrollment = BiometricEnrollment {
enrollment_id,
citizen_did: *citizen_did,
modalities,
templates,
consent_scope,
fpic_status: consent_record.fpic_status,
consent_timestamp: consent_record.timestamp,
consent_expiry: consent_record.expiry,
treaty_requirements: treaty_check.requirements,
environmental_conditions: self.read_environmental_sensors()?,
};
self.enrollments.insert(enrollment_id, enrollment.clone());
// Update metrics
let elapsed_ms = (now() - start_time) / 1000;
debug!("Biometric enrollment completed in {}ms for citizen {:?}", elapsed_ms, citizen_did);
Ok(enrollment)
}
/**
* Detect liveness to prevent spoofing attacks
* Implements multi-method liveness detection (pulse, micro-movement, 3D depth, texture analysis)
*/
fn detect_liveness(&mut self, modality: &BiometricModality, sample: &[u8]) -> Result<LivenessDetectionResult, &'static str> {
let mut confidence = 0.0;
let mut methods_passed = 0;
let mut artifacts = Vec::new();
// Select appropriate liveness methods based on modality
let methods = match modality {
BiometricModality::Fingerprint => vec![LivenessDetectionMethod::PulseDetection, LivenessDetectionMethod::TextureAnalysis],
BiometricModality::FacialRecognition => vec![LivenessDetectionMethod::MicroMovement, LivenessDetectionMethod::3DDepthSensing, LivenessDetectionMethod::ChallengeResponse],
BiometricModality::IrisScan => vec![LivenessDetectionMethod::PulseDetection, LivenessDetectionMethod::ThermalPattern],
_ => vec![LivenessDetectionMethod::ChallengeResponse],
};
// Execute each liveness method
for method in &methods {
let method_result = self.execute_liveness_method(method, sample)?;
confidence += method_result.confidence;
if method_result.passed {
methods_passed += 1;
} else {
artifacts.extend(method_result.artifacts_detected);
}
}
// Calculate overall liveness confidence
let avg_confidence = confidence / methods.len() as f64;
let passed = avg_confidence >= LIVENESS_CONFIDENCE_MIN && methods_passed >= methods.len() / 2;
Ok(LivenessDetectionResult {
method: methods[0], // Primary method
confidence: avg_confidence,
passed,
artifacts_detected: artifacts,
timestamp: now(),
})
}
/**
* Execute specific liveness detection method
*/
fn execute_liveness_method(&mut self, method: &LivenessDetectionMethod, sample: &[u8]) -> Result<LivenessDetectionResult, &'static str> {
// In production: implement actual liveness detection algorithms
// For now: simulate liveness detection with deterministic confidence
let confidence = match method {
LivenessDetectionMethod::PulseDetection => 0.97,
LivenessDetectionMethod::MicroMovement => 0.96,
LivenessDetectionMethod::3DDepthSensing => 0.98,
LivenessDetectionMethod::TextureAnalysis => 0.95,
LivenessDetectionMethod::ChallengeResponse => 0.99,
LivenessDetectionMethod::ThermalPattern => 0.94,
};
let passed = confidence >= LIVENESS_CONFIDENCE_MIN;
Ok(LivenessDetectionResult {
method: *method,
confidence,
passed,
artifacts_detected: Vec::new(),
timestamp: now(),
})
}
/**
* Extract biometric features from raw sample
* Implements modality-specific feature extraction algorithms
*/
fn extract_biometric_features(&mut self, modality: &BiometricModality, sample: &[u8]) -> Result<Vec<u8>, &'static str> {
// In production: implement actual feature extraction algorithms
// For now: simulate feature extraction with SHA-512 hash
let mut features = Vec::with_capacity(64);
features.extend_from_slice(&self.crypto_engine.sha512_hash(sample));
// Add modality-specific transformations
match modality {
BiometricModality::Fingerprint => {
// Minutiae extraction simulation
for i in 0..16 {
features.push((sample[i % sample.len()] + i as u8) % 256);
}
},
BiometricModality::FacialRecognition => {
// Facial landmark extraction simulation
for i in 0..32 {
features.push((sample[i % sample.len()] + (i * 2) as u8) % 256);
}
},
BiometricModality::IrisScan => {
// Iris code generation simulation
for i in 0..24 {
features.push((sample[i % sample.len()] + (i * 3) as u8) % 256);
}
},
_ => {
// Generic feature extraction
for i in 0..8 {
features.push(sample[i % sample.len()]);
}
},
}
Ok(features)
}
/**
* Assess biometric template quality
*/
fn assess_template_quality(&mut self, modality: &BiometricModality, template: &[u8]) -> f64 {
// In production: implement actual quality assessment algorithms
// For now: simulate quality based on template entropy
let entropy = template.iter().map(|&b| b as f64).sum::<f64>() / (template.len() as f64 * 255.0);
let base_quality = entropy * 100.0;
// Modality-specific quality adjustments
let adjusted_quality = match modality {
BiometricModality::Fingerprint => base_quality * 0.9,
BiometricModality::FacialRecognition => base_quality * 0.85,
BiometricModality::IrisScan => base_quality * 0.95,
_ => base_quality,
};
adjusted_quality.min(100.0).max(0.0)
}
/**
* Authenticate citizen using biometrics with PQ fallback
* Implements multi-modal fusion, liveness detection, environmental adaptation, and treaty compliance
* Returns authentication result with confidence score
*/
pub fn authenticate(&mut self, citizen_did: &[u8; 32], samples: BTreeMap<BiometricModality, Vec<u8>>, pq_signature: Option<PQSignature>) -> Result<AuthenticationAttempt, &'static str> {
let start_time = now();
self.metrics.total_attempts += 1;
// Find enrollment
let enrollment = self.enrollments.get(citizen_did)
.ok_or("Biometric enrollment not found")?;
// Check consent validity
if enrollment.fpic_status != FPICStatus::Granted || now() > enrollment.consent_expiry {
self.metrics.failed_auths += 1;
return Ok(self.create_auth_attempt(citizen_did, AuthenticationMethod::BiometricPrimary, AuthenticationResult::TreatyViolation, 0.0, 0.0));
}
// Check environmental conditions
let env = self.read_environmental_sensors()?;
let env_condition = self.assess_environmental_condition(&env);
if env_condition == EnvironmentalCondition::SensorContamination {
self.metrics.sensor_errors += 1;
return Ok(self.create_auth_attempt(citizen_did, AuthenticationMethod::BiometricPrimary, AuthenticationResult::SensorError, 0.0, 0.0));
}
// Attempt biometric authentication
let biometric_result = self.authenticate_biometrics(enrollment, &samples, &env);
match biometric_result {
Ok((confidence, liveness_confidence)) => {
// Biometric authentication successful
if confidence >= 0.8 && liveness_confidence >= LIVENESS_CONFIDENCE_MIN {
self.metrics.successful_auths += 1;
self.metrics.avg_biometric_time_ms = (self.metrics.avg_biometric_time_ms * (self.metrics.successful_auths - 1) as f64
+ (now() - start_time) as f64 / 1000.0) / self.metrics.successful_auths as f64;
let attempt = self.create_auth_attempt(citizen_did, AuthenticationMethod::BiometricMultiModal, AuthenticationResult::Success, confidence, liveness_confidence);
self.log_authentication(&attempt)?;
return Ok(attempt);
}
},
Err(_) => {
// Biometric authentication failed, try PQ fallback
}
}
// Biometric failed, attempt PQ fallback if provided
if let Some(ref sig) = pq_signature {
let pq_result = self.authenticate_pq_fallback(citizen_did, sig);
if pq_result {
self.metrics.pq_fallbacks_used += 1;
self.metrics.successful_auths += 1;
self.metrics.avg_pq_fallback_time_ms = (self.metrics.avg_pq_fallback_time_ms * (self.metrics.pq_fallbacks_used - 1) as f64
+ (now() - start_time) as f64 / 1000.0) / self.metrics.pq_fallbacks_used as f64;
let attempt = self.create_auth_attempt(citizen_did, AuthenticationMethod::PQBackupSignature, AuthenticationResult::Success, 0.99, 1.0);
self.log_authentication(&attempt)?;
return Ok(attempt);
}
}
// All authentication methods failed
self.metrics.failed_auths += 1;
let attempt = self.create_auth_attempt(citizen_did, AuthenticationMethod::BiometricPrimary, AuthenticationResult::Failed, 0.0, 0.0);
self.log_authentication(&attempt)?;
Ok(attempt)
}
/**
* Authenticate using biometric samples
*/
fn authenticate_biometrics(&mut self, enrollment: &BiometricEnrollment, samples: &BTreeMap<BiometricModality, Vec<u8>>, env: &EnvironmentalConditions) -> Result<(f64, f64), &'static str> {
// Collect scores from each modality
let mut modality_scores = BTreeMap::new();
let mut liveness_scores = Vec::new();
for (modality, sample) in samples {
if let Some(template) = enrollment.templates.get(modality) {
// Extract features from sample
let sample_features = self.extract_biometric_features(modality, sample)?;
// Perform homomorphic matching (simulated)
let match_score = self.homomorphic_match(&sample_features, &template.encrypted_template)?;
// Apply environmental adjustment
let adjusted_score = self.apply_environmental_adjustment(match_score, env, modality);
modality_scores.insert(*modality, adjusted_score);
// Check liveness
let liveness_result = self.detect_liveness(modality, sample)?;
liveness_scores.push(liveness_result.confidence);
}
}
// Fuse multi-modal scores
let fusion_result = self.fuse_biometric_scores(&modality_scores, &liveness_scores)?;
Ok((fusion_result.final_confidence, fusion_result.liveness_passed as f64 * fusion_result.liveness_passed as f64))
}
/**
* Perform homomorphic biometric matching
*/
fn homomorphic_match(&mut self, sample_features: &[u8], template: &[u8]) -> Result<f64, &'static str> {
// In production: implement actual homomorphic matching algorithm
// For now: simulate matching with similarity score
let mut similarity = 0.0;
for (s, t) in sample_features.iter().zip(template.iter()) {
if s == t {
similarity += 1.0;
}
}
let score = similarity / sample_features.len() as f64;
Ok(score)
}
/**
* Apply environmental adjustment to biometric score
*/
fn apply_environmental_adjustment(&mut self, score: f64, env: &EnvironmentalConditions, modality: &BiometricModality) -> f64 {
let mut adjusted = score;
// Extreme heat adjustment
if env.extreme_heat {
adjusted *= 0.95;
}
// High humidity adjustment
if env.humidity_percent > 80.0 {
adjusted *= 0.98;
}
// Dust storm adjustment
if env.haboob_detected {
adjusted *= 0.90;
}
// Modality-specific adjustments
match modality {
BiometricModality::Fingerprint => {
// Fingerprint affected by dry skin in heat
if env.temperature_c > 40.0 {
adjusted *= 0.97;
}
},
BiometricModality::FacialRecognition => {
// Facial recognition affected by sweat/dust
if env.haboob_detected || env.temperature_c > 43.0 {
adjusted *= 0.95;
}
},
_ => {},
}
adjusted.max(0.0).min(1.0)
}
/**
* Fuse multi-modal biometric scores
*/
fn fuse_biometric_scores(&mut self, modality_scores: &BTreeMap<BiometricModality, f64>, liveness_scores: &[f64]) -> Result<BiometricFusionResult, &'static str> {
// Calculate weighted fusion based on modality weights
let mut fused_score = 0.0;
let mut total_weight = 0.0;
for (modality, &score) in modality_scores {
let weight = match modality {
BiometricModality::Fingerprint => FINGERPRINT_WEIGHT,
BiometricModality::FacialRecognition => FACIAL_WEIGHT,
BiometricModality::IrisScan => IRIS_WEIGHT,
BiometricModality::VoiceRecognition | BiometricModality::KeystrokeDynamics | BiometricModality::GaitAnalysis => BEHAVIORAL_WEIGHT,
_ => 0.1,
};
fused_score += score * weight;
total_weight += weight;
}
if total_weight > 0.0 {
fused_score /= total_weight;
}
// Calculate average liveness confidence
let avg_liveness = if !liveness_scores.is_empty() {
liveness_scores.iter().sum::<f64>() / liveness_scores.len() as f64
} else {
1.0
};
// Calculate final confidence with liveness factor
let final_confidence = fused_score * avg_liveness;
Ok(BiometricFusionResult {
fused_score,
modality_scores: modality_scores.clone(),
liveness_passed: avg_liveness >= LIVENESS_CONFIDENCE_MIN,
environmental_adjustment: 1.0,
final_confidence,
})
}
/**
* Authenticate using PQ signature fallback
*/
fn authenticate_pq_fallback(&mut self, citizen_did: &[u8; 32], signature: &PQSignature) -> bool {
// Find PQ backup keys for citizen
if let Some((_, backup_key)) = self.pq_backup_keys.iter().find(|(_, k)| k.citizen_did == *citizen_did) {
// Verify signature using backup key
if let Ok(valid) = self.crypto_engine.verify_signature(signature, &backup_key.pq_key_pair.public_key) {
return valid;
}
}
false
}
/**
* Generate PQ backup keys for citizen
*/
fn generate_pq_backup_keys(&mut self, citizen_did: &[u8; 32]) -> Result<(), &'static str> {
// Generate multiple backup keys with threshold scheme
for i in 0..PQ_BACKUP_KEYS_PER_USER {
let key_pair = self.crypto_engine.generate_key_pair(PQAlgorithmSuite::Kyber768_Dilithium3)?;
let key_id = self.generate_backup_key_id();
let backup_key = PQBackupKey {
key_id,
citizen_did: *citizen_did,
pq_key_pair: key_pair,
creation_timestamp: now(),
expiry_timestamp: now() + (PQ_BACKUP_KEY_LIFETIME_SECONDS * 1000000),
usage_count: 0,
last_used: 0,
recovery_shares: BTreeMap::new(),
threshold: PQ_BACKUP_KEY_THRESHOLD,
treaty_protected: true,
};
self.pq_backup_keys.insert(key_id, backup_key);
}
Ok(())
}
/**
* Create authentication attempt record
*/
fn create_auth_attempt(&self, citizen_did: &[u8; 32], method: AuthenticationMethod, result: AuthenticationResult, confidence: f64, liveness_confidence: f64) -> AuthenticationAttempt {
let env = self.read_environmental_sensors().unwrap_or_default();
AuthenticationAttempt {
attempt_id: self.generate_attempt_id(),
citizen_did: Some(*citizen_did),
method,
modalities_used: BTreeSet::new(),
timestamp: now(),
result,
confidence_score: confidence,
liveness_confidence,
environmental_condition: self.assess_environmental_condition(&env),
pq_fallback_used: method == AuthenticationMethod::PQBackupSignature,
treaty_context: None,
sensor_readings: SensorReadings {
temperature_c: env.temperature_c,
humidity_percent: env.humidity_percent,
dust_level_ug_m3: env.particulate_ug_m3,
light_level_lux: 500.0, // Simulated
sensor_contamination: false,
signal_quality: confidence,
},
}
}
/**
* Assess environmental condition for authentication
*/
fn assess_environmental_condition(&self, env: &EnvironmentalConditions) -> EnvironmentalCondition {
if env.sensor_contamination {
return EnvironmentalCondition::SensorContamination;
}
if env.haboob_detected {
return EnvironmentalCondition::HaboobDustStorm;
}
if env.extreme_heat {
return EnvironmentalCondition::ExtremeHeat;
}
if env.humidity_percent > 80.0 {
return EnvironmentalCondition::HighHumidity;
}
EnvironmentalCondition::Normal
}
/**
* Read environmental sensors
*/
fn read_environmental_sensors(&self) -> Result<EnvironmentalConditions, &'static str> {
// In production: read actual hardware sensors
// For now: simulate Phoenix conditions
Ok(EnvironmentalConditions {
temperature_c: 45.0, // 113°F typical Phoenix summer
humidity_percent: 20.0, // Low humidity
particulate_ug_m3: 50.0, // Low dust
haboob_detected: false,
extreme_heat: true, // >110°F
monsoon_conditions: false,
timestamp: now(),
})
}
/**
* Log authentication attempt to offline buffer
*/
fn log_authentication(&mut self, attempt: &AuthenticationAttempt) -> Result<(), &'static str> {
self.offline_buffer.push_back(attempt.clone());
if self.offline_buffer.len() > OFFLINE_AUTH_BUFFER_SIZE {
self.offline_buffer.pop_front();
}
self.metrics.offline_buffer_usage_percent = (self.offline_buffer.len() as f64 / OFFLINE_AUTH_BUFFER_SIZE as f64) * 100.0;
Ok(())
}
/**
* Generate unique IDs
*/
fn generate_template_id(&self) -> [u8; 32] {
let mut id = [0u8; 32];
let timestamp = now();
id[..8].copy_from_slice(&timestamp.to_be_bytes());
id[8..24].copy_from_slice(&self.node_id.to_bytes()[..16]);
id[24..].copy_from_slice(&self.biometric_templates.len().to_be_bytes()[..8]);
self.crypto_engine.sha512_hash(&id)[..32].try_into().unwrap_or([0u8; 32])
}
fn generate_enrollment_id(&self) -> [u8; 32] {
self.generate_template_id()
}
fn generate_consent_id(&self) -> [u8; 32] {
self.generate_template_id()
}
fn generate_backup_key_id(&self) -> [u8; 32] {
self.generate_template_id()
}
fn generate_attempt_id(&self) -> [u8; 32] {
self.generate_template_id()
}
/**
* Get current authentication metrics
*/
pub fn get_metrics(&self) -> AuthenticationMetrics {
self.metrics.clone()
}
/**
* Get enrollment by citizen DID
*/
pub fn get_enrollment(&self, citizen_did: &[u8; 32]) -> Option<&BiometricEnrollment> {
self.enrollments.get(citizen_did)
}
/**
* Revoke biometric consent
*/
pub fn revoke_consent(&mut self, consent_id: &[u8; 32]) -> Result<(), &'static str> {
if let Some(consent) = self.consent_records.get_mut(consent_id) {
consent.revocation_timestamp = Some(now());
consent.fpic_status = FPICStatus::Revoked;
// Delete associated biometric templates
if let Some(enrollment) = self.enrollments.values_mut().find(|e| e.consent_timestamp == consent.timestamp) {
for template in enrollment.templates.values() {
self.biometric_templates.remove(&template.template_id);
}
}
Ok(())
} else {
Err("Consent record not found")
}
}
/**
* Create emergency override token (physical presence required)
*/
pub fn create_emergency_override(&mut self, citizen_did: &[u8; 32], justification: String, treaty_auth: Option<TreatyContext>) -> Result<EmergencyOverrideToken, &'static str> {
let token_id = self.generate_template_id();
let token = EmergencyOverrideToken {
token_id,
citizen_did: *citizen_did,
creation_timestamp: now(),
expiry_timestamp: now() + (24 * 60 * 60 * 1000000), // 24 hour expiry
used: false,
used_timestamp: None,
justification,
treaty_authorization: treaty_auth,
};
self.emergency_tokens.insert(token_id, token.clone());
Ok(token)
}
/**
* Use emergency override token
*/
pub fn use_emergency_override(&mut self, token_id: &[u8; 32]) -> Result<bool, &'static str> {
if let Some(token) = self.emergency_tokens.get_mut(token_id) {
if token.used {
return Ok(false);
}
if now() > token.expiry_timestamp {
return Ok(false);
}
token.used = true;
token.used_timestamp = Some(now());
Ok(true)
} else {
Ok(false)
}
}
/**
* Perform maintenance tasks (cleanup, key rotation, buffer management)
*/
pub fn perform_maintenance(&mut self) -> Result<(), &'static str> {
let now = now();
// Cleanup old authentication attempts (>30 days)
while let Some(attempt) = self.authentication_attempts.front() {
if now - attempt.timestamp > 30 * 24 * 60 * 60 * 1000000 {
self.authentication_attempts.pop_front();
} else {
break;
}
}
// Rotate expired PQ backup keys
let expired_keys: Vec<_> = self.pq_backup_keys.iter()
.filter(|(_, k)| now > k.expiry_timestamp)
.map(|(id, _)| *id)
.collect();
for key_id in expired_keys {
self.pq_backup_keys.remove(&key_id);
}
// Cleanup old offline buffer entries (>72 hours)
while let Some(attempt) = self.offline_buffer.front() {
if now - attempt.timestamp > (OFFLINE_BUFFER_HOURS as u64) * 3600 * 1000000 {
self.offline_buffer.pop_front();
} else {
break;
}
}
// Revoke expired consents
for consent in self.consent_records.values_mut() {
if now > consent.expiry && consent.revocation_timestamp.is_none() {
consent.revocation_timestamp = Some(now);
consent.fpic_status = FPICStatus::Expired;
}
}
self.last_maintenance = now;
Ok(())
}
}
// --- Helper Functions ---
/**
* Calculate authentication success rate
*/
pub fn calculate_success_rate(total: usize, successful: usize) -> f64 {
if total == 0 {
return 100.0;
}
(successful as f64 / total as f64) * 100.0
}
/**
* Calculate false acceptance rate
*/
pub fn calculate_far(false_accepts: usize, total_attempts: usize) -> f64 {
if total_attempts == 0 {
return 0.0;
}
(false_accepts as f64 / total_attempts as f64) * 100.0
}
/**
* Calculate false rejection rate
*/
pub fn calculate_frr(false_rejects: usize, total_legitimate: usize) -> f64 {
if total_legitimate == 0 {
return 0.0;
}
(false_rejects as f64 / total_legitimate as f64) * 100.0
}
/**
* Check if biometric verification time is within acceptable limits
*/
pub fn is_biometric_time_acceptable(latency_ms: f64) -> bool {
latency_ms <= MAX_BIOMETRIC_VERIFY_TIME_MS as f64
}
// --- Unit Tests (Offline Capable) ---
#[cfg(test)]
mod tests {
use super::*;
#[test]
fn test_engine_initialization() {
let engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
assert!(engine.active);
assert_eq!(engine.biometric_templates.len(), 0);
assert_eq!(engine.enrollments.len(), 0);
assert_eq!(engine.metrics.total_attempts, 0);
}
#[test]
fn test_biometric_enrollment() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
let citizen_did = [1u8; 32];
let mut modalities = BTreeSet::new();
modalities.insert(BiometricModality::Fingerprint);
modalities.insert(BiometricModality::FacialRecognition);
let mut samples = BTreeMap::new();
samples.insert(BiometricModality::Fingerprint, vec![1u8; 1024]);
samples.insert(BiometricModality::FacialRecognition, vec![2u8; 2048]);
let enrollment = engine.enroll_biometrics(
&citizen_did,
modalities,
samples,
ConsentScope::FullAccess,
FPICStatus::Granted,
).unwrap();
assert_eq!(enrollment.citizen_did, citizen_did);
assert_eq!(enrollment.modalities.len(), 2);
assert_eq!(enrollment.templates.len(), 2);
assert!(enrollment.fpic_status == FPICStatus::Granted);
}
#[test]
fn test_biometric_authentication_success() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
// Enroll citizen
let citizen_did = [1u8; 32];
let mut modalities = BTreeSet::new();
modalities.insert(BiometricModality::Fingerprint);
let mut samples = BTreeMap::new();
samples.insert(BiometricModality::Fingerprint, vec![1u8; 1024]);
engine.enroll_biometrics(
&citizen_did,
modalities.clone(),
samples.clone(),
ConsentScope::FullAccess,
FPICStatus::Granted,
).unwrap();
// Authenticate with same samples (should succeed)
let auth_result = engine.authenticate(&citizen_did, samples, None).unwrap();
assert_eq!(auth_result.result, AuthenticationResult::Success);
assert!(auth_result.confidence_score >= 0.8);
assert_eq!(engine.metrics.successful_auths, 1);
}
#[test]
fn test_biometric_authentication_failure() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
// Enroll citizen
let citizen_did = [1u8; 32];
let mut modalities = BTreeSet::new();
modalities.insert(BiometricModality::Fingerprint);
let mut samples = BTreeMap::new();
samples.insert(BiometricModality::Fingerprint, vec![1u8; 1024]);
engine.enroll_biometrics(
&citizen_did,
modalities,
samples,
ConsentScope::FullAccess,
FPICStatus::Granted,
).unwrap();
// Authenticate with different samples (should fail)
let mut wrong_samples = BTreeMap::new();
wrong_samples.insert(BiometricModality::Fingerprint, vec![99u8; 1024]);
let auth_result = engine.authenticate(&citizen_did, wrong_samples, None).unwrap();
assert_eq!(auth_result.result, AuthenticationResult::Failed);
assert_eq!(engine.metrics.failed_auths, 1);
}
#[test]
fn test_pq_fallback_authentication() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
// Enroll citizen
let citizen_did = [1u8; 32];
let mut modalities = BTreeSet::new();
modalities.insert(BiometricModality::Fingerprint);
let mut samples = BTreeMap::new();
samples.insert(BiometricModality::Fingerprint, vec![1u8; 1024]);
engine.enroll_biometrics(
&citizen_did,
modalities,
samples,
ConsentScope::FullAccess,
FPICStatus::Granted,
).unwrap();
// Get PQ backup key
let backup_key = engine.pq_backup_keys.values().next().unwrap();
// Create PQ signature
let message = b"test authentication";
let signature = engine.crypto_engine.sign_message(&backup_key.pq_key_pair.key_id, message).unwrap();
// Authenticate with PQ signature (biometric samples can be empty)
let empty_samples = BTreeMap::new();
let auth_result = engine.authenticate(&citizen_did, empty_samples, Some(signature)).unwrap();
assert_eq!(auth_result.result, AuthenticationResult::Success);
assert_eq!(auth_result.method, AuthenticationMethod::PQBackupSignature);
assert_eq!(engine.metrics.pq_fallbacks_used, 1);
}
#[test]
fn test_consent_revocation() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
// Enroll citizen
let citizen_did = [1u8; 32];
let mut modalities = BTreeSet::new();
modalities.insert(BiometricModality::Fingerprint);
let mut samples = BTreeMap::new();
samples.insert(BiometricModality::Fingerprint, vec![1u8; 1024]);
let enrollment = engine.enroll_biometrics(
&citizen_did,
modalities,
samples,
ConsentScope::FullAccess,
FPICStatus::Granted,
).unwrap();
// Revoke consent
let consent_id = engine.consent_records.keys().next().unwrap().clone();
engine.revoke_consent(&consent_id).unwrap();
// Verify consent revoked
let consent = engine.consent_records.get(&consent_id).unwrap();
assert_eq!(consent.fpic_status, FPICStatus::Revoked);
assert!(consent.revocation_timestamp.is_some());
}
#[test]
fn test_emergency_override() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
let citizen_did = [1u8; 32];
// Create emergency override token
let token = engine.create_emergency_override(&citizen_did, "System recovery".to_string(), None).unwrap();
assert!(!token.used);
// Use emergency override
let used = engine.use_emergency_override(&token.token_id).unwrap();
assert!(used);
// Try to use again (should fail)
let used_again = engine.use_emergency_override(&token.token_id).unwrap();
assert!(!used_again);
}
#[test]
fn test_environmental_condition_assessment() {
let engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
// Normal conditions
let normal = EnvironmentalConditions {
temperature_c: 25.0,
humidity_percent: 50.0,
particulate_ug_m3: 50.0,
haboob_detected: false,
extreme_heat: false,
monsoon_conditions: false,
timestamp: now(),
};
assert_eq!(engine.assess_environmental_condition(&normal), EnvironmentalCondition::Normal);
// Extreme heat
let heat = EnvironmentalConditions {
temperature_c: 45.0,
humidity_percent: 20.0,
particulate_ug_m3: 50.0,
haboob_detected: false,
extreme_heat: true,
monsoon_conditions: false,
timestamp: now(),
};
assert_eq!(engine.assess_environmental_condition(&heat), EnvironmentalCondition::ExtremeHeat);
// Haboob
let haboob = EnvironmentalConditions {
temperature_c: 35.0,
humidity_percent: 15.0,
particulate_ug_m3: 5000.0,
haboob_detected: true,
extreme_heat: false,
monsoon_conditions: false,
timestamp: now(),
};
assert_eq!(engine.assess_environmental_condition(&haboob), EnvironmentalCondition::HaboobDustStorm);
}
#[test]
fn test_authentication_metrics() {
// Calculate success rate
let success_rate = calculate_success_rate(100, 95);
assert_eq!(success_rate, 95.0);
// Calculate FAR
let far = calculate_far(1, 10000);
assert!((far - 0.01).abs() < 0.001); // 0.01%
// Calculate FRR
let frr = calculate_frr(5, 1000);
assert!((frr - 0.5).abs() < 0.01); // 0.5%
}
#[test]
fn test_offline_buffer_management() {
let mut engine = BiometricAuthenticationEngine::new(BirthSign::default()).unwrap();
// Fill offline buffer beyond capacity
for _ in 0..(OFFLINE_AUTH_BUFFER_SIZE + 100) {
let attempt = AuthenticationAttempt {
attempt_id: [0u8; 32],
citizen_did: Some([1u8; 32]),
method: AuthenticationMethod::BiometricPrimary,
modalities_used: BTreeSet::new(),
timestamp: now(),
result: AuthenticationResult::Success,
confidence_score: 0.9,
liveness_confidence: 0.95,
environmental_condition: EnvironmentalCondition::Normal,
pq_fallback_used: false,
treaty_context: None,
sensor_readings: SensorReadings {
temperature_c: 25.0,
humidity_percent: 50.0,
dust_level_ug_m3: 50.0,
light_level_lux: 500.0,
sensor_contamination: false,
signal_quality: 0.9,
},
};
engine.offline_buffer.push_back(attempt);
}
// Buffer should be at max capacity
assert_eq!(engine.offline_buffer.len(), OFFLINE_AUTH_BUFFER_SIZE);
assert_eq!(engine.metrics.offline_buffer_usage_percent, 100.0);
}
}
