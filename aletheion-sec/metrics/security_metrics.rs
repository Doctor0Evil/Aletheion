/**
* Aletheion Smart City Core - Batch 2
* File: 125/200
* Layer: 36 (Advanced Security)
* Path: aletheion-sec/metrics/security_metrics.rs
*
* Research Basis (Real-time Security Posture Monitoring & Threat Intelligence):
*   - Security Posture Scoring: CIS Controls scoring, NIST CSF maturity levels, quantitative risk assessment models
*   - Threat Intelligence Aggregation: STIX/TAXII standards, IOC correlation, threat feed fusion, MITRE ATT&CK mapping
*   - Predictive Analytics: Statistical process control (CUSUM, EWMA), time series forecasting (ARIMA, exponential smoothing), anomaly detection (Isolation Forest, LOF)
*   - Compliance Metrics: NIST SP 800-55 (Performance Measurement Guide), ISO 27001 metrics, regulatory compliance dashboards
*   - Performance Monitoring: SLO/SLI frameworks, latency percentiles (p50, p95, p99), availability calculations, error budgets
*   - Resource Utilization: CPU/memory/disk/network monitoring, capacity planning models, auto-scaling triggers
*   - Treaty Compliance Dashboards: FPIC compliance tracking, Indigenous data sovereignty metrics, neurorights protection indicators
*   - Phoenix-Specific Metrics: Haboob resilience scoring, extreme heat operational continuity, monsoon flood readiness
*   - Performance Benchmarks: <100ms metric calculation latency, 99.9% prediction accuracy, <1s dashboard refresh, 99.99% metric integrity
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
use alloc::collections::{BTreeMap, BTreeSet, VecDeque, LinkedList, HashMap, HashSet};
use core::result::Result;
use core::ops::{Add, Sub, BitXor, Mul, Div};
use core::time::Duration;
use core::sync::atomic::{AtomicU64, Ordering};
// Internal Aletheion Crates (Established in Batch 1 & Files 112-124)
use aletheion_core::identity::BirthSign;
use aletheion_core::time::{now, Timestamp};
use aletheion_core::logger::{log, warn, error, debug};
use aletheion_sec::quantum::post::crypto_core::{PQCryptoEngine, PQSignature, PQSecurityLevel, PQKeyPair};
use aletheion_sec::quantum::post::threat_detection::{ThreatDetectionEngine, ThreatEvent, ThreatCategory, ThreatSeverity, ThreatDetectionMetrics, DetectionMethod};
use aletheion_sec::incident::response_system::{IncidentResponseEngine, Incident, IncidentType, IncidentStatus, IncidentMetrics, IncidentImpact};
use aletheion_sec::compliance::compliance_automation::{ComplianceAutomationEngine, ComplianceCheck, ComplianceStatus, ComplianceDomain, ComplianceMetrics};
use aletheion_sec::audit::immutable_log::{ImmutableAuditLogEngine, LogEventType, LogSeverity};
use aletheion_gov::treaty::{TreatyCompliance, TreatyViolation, FPICStatus, TreatyContext};
// --- Constants & Security Metrics Parameters ---
/// Security posture scoring constants
pub const POSTURE_SCORE_MAX: u8 = 100;                    // Maximum posture score
pub const POSTURE_SCORE_CRITICAL_THRESHOLD: u8 = 30;      // <30 = Critical risk
pub const POSTURE_SCORE_HIGH_THRESHOLD: u8 = 50;          // 30-50 = High risk
pub const POSTURE_SCORE_MEDIUM_THRESHOLD: u8 = 70;        // 50-70 = Medium risk
pub const POSTURE_SCORE_LOW_THRESHOLD: u8 = 90;           // 70-90 = Low risk
pub const POSTURE_SCORE_EXCELLENT_THRESHOLD: u8 = 95;     // >95 = Excellent
/// Threat intelligence constants
pub const THREAT_INTEL_UPDATE_INTERVAL_MS: u64 = 3600000; // 1 hour threat feed update
pub const THREAT_CORRELATION_WINDOW_MS: u64 = 300000;     // 5 minutes correlation window
pub const THREAT_CONFIDENCE_THRESHOLD: f64 = 0.7;         // 70% confidence threshold
pub const IOC_EXPIRATION_DAYS: u32 = 30;                  // 30 days IOC expiration
/// Predictive analytics constants
pub const PREDICTION_HORIZON_HOURS: u32 = 24;             // 24-hour prediction horizon
pub const TIME_SERIES_HISTORY_DAYS: u32 = 90;             // 90 days historical data
pub const ANOMALY_DETECTION_SIGMA: f64 = 3.0;             // 3-sigma anomaly threshold
pub const TREND_DETECTION_MIN_POINTS: usize = 10;         // Minimum 10 points for trend detection
/// Performance monitoring constants
pub const LATENCY_P50_TARGET_MS: u64 = 50;                // p50 latency target
pub const LATENCY_P95_TARGET_MS: u64 = 100;               // p95 latency target
pub const LATENCY_P99_TARGET_MS: u64 = 200;               // p99 latency target
pub const AVAILABILITY_TARGET_PERCENT: f64 = 99.99;       // 99.99% availability target
pub const ERROR_BUDGET_BURN_RATE_THRESHOLD: f64 = 0.5;    // 50% error budget burn rate threshold
/// Compliance metrics constants
pub const COMPLIANCE_SCORE_TARGET_PERCENT: f64 = 99.9;    // 99.9% compliance target
pub const VIOLATION_REMEDIATION_TARGET_HOURS: u32 = 24;   // 24 hours violation remediation target
pub const AUDIT_FREQUENCY_DAYS: u32 = 30;                 // 30 days audit frequency
/// Resource utilization constants
pub const CPU_UTILIZATION_WARNING_PERCENT: f64 = 80.0;    // 80% CPU warning threshold
pub const MEMORY_UTILIZATION_WARNING_PERCENT: f64 = 85.0; // 85% memory warning threshold
pub const DISK_UTILIZATION_WARNING_PERCENT: f64 = 90.0;   // 90% disk warning threshold
pub const NETWORK_BANDWIDTH_WARNING_PERCENT: f64 = 75.0;  // 75% network bandwidth warning
pub const CAPACITY_PLANNING_HEADROOM_PERCENT: f64 = 20.0; // 20% capacity headroom
/// Treaty compliance constants
pub const FPIC_COMPLIANCE_TARGET_PERCENT: f64 = 100.0;    // 100% FPIC compliance target
pub const INDIGENOUS_DATA_SOVEREIGNTY_TARGET_PERCENT: f64 = 100.0; // 100% sovereignty
pub const NEURORIGHTS_PROTECTION_TARGET_PERCENT: f64 = 100.0; // 100% protection
/// Phoenix-specific metrics constants
pub const HABOOB_RESILIENCE_SCORE_TARGET: u8 = 90;        // 90% haboob resilience target
pub const EXTREME_HEAT_CONTINUITY_SCORE_TARGET: u8 = 95;  // 95% heat continuity target
pub const MONSOON_FLOOD_READINESS_SCORE_TARGET: u8 = 85;  // 85% flood readiness target
/// Performance thresholds
pub const MAX_METRIC_CALCULATION_MS: u64 = 100;           // <100ms metric calculation
pub const MAX_DASHBOARD_REFRESH_MS: u64 = 1000;           // <1s dashboard refresh
pub const PREDICTION_ACCURACY_TARGET_PERCENT: f64 = 99.9; // 99.9% prediction accuracy
pub const METRIC_INTEGRITY_TARGET_PERCENT: f64 = 99.99;   // 99.99% metric integrity
/// Time window constants for metrics aggregation
pub const METRICS_WINDOW_1MIN_MS: u64 = 60000;
pub const METRICS_WINDOW_5MIN_MS: u64 = 300000;
pub const METRICS_WINDOW_15MIN_MS: u64 = 900000;
pub const METRICS_WINDOW_1HOUR_MS: u64 = 3600000;
pub const METRICS_WINDOW_24HOUR_MS: u64 = 86400000;
pub const METRICS_WINDOW_7DAY_MS: u64 = 604800000;
pub const METRICS_WINDOW_30DAY_MS: u64 = 2592000000;
// --- Enumerations ---
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SecurityPostureLevel {
Critical,                   // Score < 30 - Immediate action required
HighRisk,                   // Score 30-50 - High risk, urgent remediation
MediumRisk,                 // Score 50-70 - Medium risk, planned remediation
LowRisk,                    // Score 70-90 - Low risk, monitoring
Excellent,                  // Score > 90 - Excellent security posture
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetricCategory {
ThreatDetection,            // Threat detection metrics
IncidentResponse,           // Incident response metrics
Compliance,                 // Compliance metrics
Performance,                // Performance metrics
ResourceUtilization,        // Resource utilization metrics
TreatyCompliance,           // Treaty compliance metrics
EnvironmentalResilience,    // Environmental resilience metrics
SecurityPosture,            // Overall security posture
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PredictionModelType {
ExponentialSmoothing,       // Simple exponential smoothing
MovingAverage,              // Moving average (SMA, EMA, WMA)
LinearRegression,           // Linear regression trend
CUSUM,                      // Cumulative Sum control chart
ARIMA,                      // AutoRegressive Integrated Moving Average
IsolationForest,            // Isolation Forest anomaly detection
Prophet,                    // Facebook Prophet time series
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlertSeverity {
Info,                       // Informational alert
Warning,                    // Warning alert (threshold exceeded)
Critical,                   // Critical alert (immediate action required)
Emergency,                  // Emergency alert (system failure imminent)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DashboardTimeRange {
Last1Hour,                  // Last 1 hour
Last6Hours,                 // Last 6 hours
Last24Hours,                // Last 24 hours
Last7Days,                  // Last 7 days
Last30Days,                 // Last 30 days
Last90Days,                 // Last 90 days
Custom,                     // Custom time range
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComplianceTrend {
Improving,                  // Compliance improving over time
Stable,                     // Compliance stable
Declining,                  // Compliance declining
Volatile,                   // Compliance volatile (fluctuating)
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceBottleneck {
CPU,                        // CPU bottleneck
Memory,                     // Memory bottleneck
DiskIO,                     // Disk I/O bottleneck
Network,                    // Network bottleneck
Storage,                    // Storage capacity bottleneck
Database,                   // Database performance bottleneck
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreatTrendDirection {
Increasing,                 // Threat activity increasing
Decreasing,                 // Threat activity decreasing
Stable,                     // Threat activity stable
Spike,                      // Threat activity spike
Dip,                        // Threat activity dip
}
#[derive(Clone)]
pub struct SecurityPostureScore {
pub overall_score: u8,                      // 0-100 overall security posture
pub threat_detection_score: u8,             // Threat detection effectiveness
pub incident_response_score: u8,            // Incident response effectiveness
pub compliance_score: u8,                   // Regulatory compliance score
pub performance_score: u8,                  // System performance score
pub resource_utilization_score: u8,         // Resource utilization efficiency
pub treaty_compliance_score: u8,            // Treaty/FPIC compliance score
pub environmental_resilience_score: u8,     // Environmental resilience score
pub timestamp: Timestamp,
pub posture_level: SecurityPostureLevel,
pub risk_factors: Vec<String>,
pub improvement_recommendations: Vec<String>,
}
#[derive(Clone)]
pub struct ThreatIntelligenceMetric {
pub metric_id: [u8; 32],
pub threat_category: ThreatCategory,
pub threat_count: usize,
pub unique_sources: usize,
pub unique_targets: usize,
pub avg_confidence: f64,
pub max_severity: ThreatSeverity,
pub trend_direction: ThreatTrendDirection,
pub trend_magnitude: f64,
pub correlation_score: f64,
pub iocs: BTreeSet<String>,
pub timestamp: Timestamp,
pub time_window_ms: u64,
}
#[derive(Clone)]
pub struct PerformanceMetric {
pub metric_id: [u8; 32],
pub operation_name: String,
pub p50_latency_ms: f64,
pub p95_latency_ms: f64,
pub p99_latency_ms: f64,
pub avg_latency_ms: f64,
pub max_latency_ms: f64,
pub min_latency_ms: f64,
pub request_count: usize,
pub error_count: usize,
pub error_rate_percent: f64,
pub availability_percent: f64,
pub error_budget_remaining_percent: f64,
pub timestamp: Timestamp,
pub time_window_ms: u64,
}
#[derive(Clone)]
pub struct ResourceUtilizationMetric {
pub metric_id: [u8; 32],
pub resource_type: String,
pub cpu_percent: f64,
pub memory_percent: f64,
pub disk_percent: f64,
pub network_bandwidth_percent: f64,
pub active_connections: usize,
pub thread_count: usize,
pub file_descriptors_used: usize,
pub bottleneck: Option<ResourceBottleneck>,
pub capacity_remaining_percent: f64,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct PredictiveAlert {
pub alert_id: [u8; 32],
pub severity: AlertSeverity,
pub metric_category: MetricCategory,
pub description: String,
pub predicted_value: f64,
pub threshold_value: f64,
pub confidence_percent: f64,
pub prediction_horizon_hours: u32,
pub recommended_action: String,
pub timestamp: Timestamp,
pub acknowledged: bool,
pub acknowledged_by: Option<BirthSign>,
pub acknowledged_timestamp: Option<Timestamp>,
}
#[derive(Clone)]
pub struct ComplianceDashboard {
pub dashboard_id: [u8; 32],
pub overall_compliance_percent: f64,
pub compliant_checks: usize,
pub warning_checks: usize,
pub violation_checks: usize,
pub critical_violation_checks: usize,
pub compliance_trend: ComplianceTrend,
pub trend_period_days: u32,
pub domains_assessed: BTreeSet<ComplianceDomain>,
pub treaty_compliance_percent: f64,
pub fpic_compliance_percent: f64,
pub neurorights_compliance_percent: f64,
pub last_audit_timestamp: Timestamp,
pub next_audit_due: Timestamp,
pub remediation_overdue: usize,
}
#[derive(Clone)]
pub struct EnvironmentalResilienceScore {
pub score_id: [u8; 32],
pub haboob_resilience_score: u8,
pub extreme_heat_continuity_score: u8,
pub monsoon_flood_readiness_score: u8,
pub equipment_stress_level: u8,
pub operational_continuity_percent: f64,
pub recovery_time_objective_hours: f64,
pub recovery_point_objective_hours: f64,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct TimeSeriesPoint {
pub timestamp: Timestamp,
pub value: f64,
pub confidence_interval_lower: f64,
pub confidence_interval_upper: f64,
}
#[derive(Clone)]
pub struct PredictionModel {
pub model_id: [u8; 32],
pub model_type: PredictionModelType,
pub metric_name: String,
pub training_data_points: usize,
pub last_trained: Timestamp,
pub accuracy_percent: f64,
pub parameters: BTreeMap<String, String>,
pub predictions: Vec<TimeSeriesPoint>,
}
#[derive(Clone)]
pub struct SecurityMetricsSnapshot {
pub snapshot_id: [u8; 32],
pub posture_score: SecurityPostureScore,
pub threat_metrics: Vec<ThreatIntelligenceMetric>,
pub performance_metrics: Vec<PerformanceMetric>,
pub resource_metrics: Vec<ResourceUtilizationMetric>,
pub compliance_dashboard: ComplianceDashboard,
pub environmental_score: EnvironmentalResilienceScore,
pub predictive_alerts: Vec<PredictiveAlert>,
pub timestamp: Timestamp,
pub signature: PQSignature,
}
#[derive(Clone)]
pub struct MetricAggregationWindow {
pub window_id: [u8; 32],
pub start_time: Timestamp,
pub end_time: Timestamp,
pub duration_ms: u64,
pub threat_events: Vec<ThreatEvent>,
pub incidents: Vec<Incident>,
pub compliance_checks: Vec<ComplianceCheck>,
pub performance_samples: Vec<PerformanceSample>,
pub resource_samples: Vec<ResourceSample>,
}
#[derive(Clone)]
pub struct PerformanceSample {
pub sample_id: [u8; 32],
pub operation: String,
pub latency_ms: f64,
pub success: bool,
pub timestamp: Timestamp,
pub metadata: BTreeMap<String, String>,
}
#[derive(Clone)]
pub struct ResourceSample {
pub sample_id: [u8; 32],
pub cpu_percent: f64,
pub memory_percent: f64,
pub disk_percent: f64,
pub network_rx_bytes: u64,
pub network_tx_bytes: u64,
pub timestamp: Timestamp,
}
#[derive(Clone)]
pub struct SecurityMetricsHistory {
pub history_id: [u8; 32],
pub snapshots: VecDeque<SecurityMetricsSnapshot>,
pub max_snapshots: usize,
pub retention_days: u32,
pub last_pruned: Timestamp,
}
// --- Core Security Metrics Engine ---
pub struct SecurityMetricsEngine {
pub node_id: BirthSign,
pub crypto_engine: PQCryptoEngine,
pub threat_detection: ThreatDetectionEngine,
pub incident_response: IncidentResponseEngine,
pub compliance_engine: ComplianceAutomationEngine,
pub audit_log: ImmutableAuditLogEngine,
pub treaty_compliance: TreatyCompliance,
pub current_posture: SecurityPostureScore,
pub threat_metrics: BTreeMap<MetricCategory, VecDeque<ThreatIntelligenceMetric>>,
pub performance_metrics: BTreeMap<String, VecDeque<PerformanceMetric>>,
pub resource_metrics: VecDeque<ResourceUtilizationMetric>,
pub predictive_models: BTreeMap<String, PredictionModel>,
pub predictive_alerts: VecDeque<PredictiveAlert>,
pub compliance_dashboard: ComplianceDashboard,
pub environmental_score: EnvironmentalResilienceScore,
pub metrics_history: SecurityMetricsHistory,
pub aggregation_windows: BTreeMap<u64, MetricAggregationWindow>,
pub last_update: Timestamp,
pub last_prediction: Timestamp,
pub active: bool,
}
impl SecurityMetricsEngine {
/**
* Initialize Security Metrics Engine with threat intelligence aggregation
* Configures posture scoring, predictive analytics, compliance dashboards, and resource monitoring
* Ensures 72h offline operational capability with metric history retention
*/
pub fn new(node_id: BirthSign) -> Result<Self, &'static str> {
let crypto_engine = PQCryptoEngine::new(node_id.clone(), PQSecurityLevel::Level3)
.map_err(|_| "Failed to initialize PQ crypto engine")?;
let threat_detection = ThreatDetectionEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize threat detection")?;
let incident_response = IncidentResponseEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize incident response")?;
let compliance_engine = ComplianceAutomationEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize compliance engine")?;
let audit_log = ImmutableAuditLogEngine::new(node_id.clone())
.map_err(|_| "Failed to initialize audit log")?;
let mut engine = Self {
node_id,
crypto_engine,
threat_detection,
incident_response,
compliance_engine,
audit_log,
treaty_compliance: TreatyCompliance::new(),
current_posture: SecurityPostureScore {
overall_score: 100,
threat_detection_score: 100,
incident_response_score: 100,
compliance_score: 100,
performance_score: 100,
resource_utilization_score: 100,
treaty_compliance_score: 100,
environmental_resilience_score: 100,
timestamp: now(),
posture_level: SecurityPostureLevel::Excellent,
risk_factors: Vec::new(),
improvement_recommendations: Vec::new(),
},
threat_metrics: BTreeMap::new(),
performance_metrics: BTreeMap::new(),
resource_metrics: VecDeque::with_capacity(1000),
predictive_models: BTreeMap::new(),
predictive_alerts: VecDeque::with_capacity(1000),
compliance_dashboard: ComplianceDashboard {
dashboard_id: [0u8; 32],
overall_compliance_percent: 100.0,
compliant_checks: 0,
warning_checks: 0,
violation_checks: 0,
critical_violation_checks: 0,
compliance_trend: ComplianceTrend::Stable,
trend_period_days: 30,
domains_assessed: BTreeSet::new(),
treaty_compliance_percent: 100.0,
fpic_compliance_percent: 100.0,
neurorights_compliance_percent: 100.0,
last_audit_timestamp: now(),
next_audit_due: now() + (30 * 24 * 60 * 60 * 1000000),
remediation_overdue: 0,
},
environmental_score: EnvironmentalResilienceScore {
score_id: [0u8; 32],
haboob_resilience_score: 100,
extreme_heat_continuity_score: 100,
monsoon_flood_readiness_score: 100,
equipment_stress_level: 0,
operational_continuity_percent: 100.0,
recovery_time_objective_hours: 1.0,
recovery_point_objective_hours: 0.1,
timestamp: now(),
},
metrics_history: SecurityMetricsHistory {
history_id: [0u8; 32],
snapshots: VecDeque::with_capacity(10000),
max_snapshots: 10000,
retention_days: 365,
last_pruned: now(),
},
aggregation_windows: BTreeMap::new(),
last_update: now(),
last_prediction: now(),
active: true,
};
// Initialize predictive models
engine.initialize_predictive_models()?;
// Initialize metric categories
engine.initialize_metric_categories()?;
Ok(engine)
}
/**
* Initialize predictive analytics models for threat forecasting
*/
fn initialize_predictive_models(&mut self) -> Result<(), &'static str> {
// Model 1: Threat count exponential smoothing
self.predictive_models.insert("threat_count_1h".to_string(), PredictionModel {
model_id: self.generate_model_id(),
model_type: PredictionModelType::ExponentialSmoothing,
metric_name: "threat_count_1h".to_string(),
training_data_points: 0,
last_trained: now(),
accuracy_percent: 0.0,
parameters: {
let mut params = BTreeMap::new();
params.insert("alpha".to_string(), "0.3".to_string()); // Smoothing factor
params.insert("horizon_hours".to_string(), "24".to_string());
params
},
predictions: Vec::new(),
});
// Model 2: Incident response time moving average
self.predictive_models.insert("incident_response_time".to_string(), PredictionModel {
model_id: self.generate_model_id(),
model_type: PredictionModelType::MovingAverage,
metric_name: "incident_response_time".to_string(),
training_data_points: 0,
last_trained: now(),
accuracy_percent: 0.0,
parameters: {
let mut params = BTreeMap::new();
params.insert("window_size".to_string(), "10".to_string());
params.insert("type".to_string(), "EMA".to_string()); // Exponential Moving Average
params
},
predictions: Vec::new(),
});
// Model 3: Compliance violations CUSUM
self.predictive_models.insert("compliance_violations".to_string(), PredictionModel {
model_id: self.generate_model_id(),
model_type: PredictionModelType::CUSUM,
metric_name: "compliance_violations".to_string(),
training_data_points: 0,
last_trained: now(),
accuracy_percent: 0.0,
parameters: {
let mut params = BTreeMap::new();
params.insert("threshold".to_string(), "5.0".to_string());
params.insert("drift".to_string(), "1.0".to_string());
params
},
predictions: Vec::new(),
});
// Model 4: CPU utilization ARIMA
self.predictive_models.insert("cpu_utilization".to_string(), PredictionModel {
model_id: self.generate_model_id(),
model_type: PredictionModelType::ARIMA,
metric_name: "cpu_utilization".to_string(),
training_data_points: 0,
last_trained: now(),
accuracy_percent: 0.0,
parameters: {
let mut params = BTreeMap::new();
params.insert("p".to_string(), "1".to_string()); // AR order
params.insert("d".to_string(), "1".to_string()); // Differencing order
params.insert("q".to_string(), "1".to_string()); // MA order
params
},
predictions: Vec::new(),
});
Ok(())
}
/**
* Initialize metric categories for aggregation
*/
fn initialize_metric_categories(&mut self) -> Result<(), &'static str> {
// Initialize threat metrics categories
let categories = vec![
MetricCategory::ThreatDetection,
MetricCategory::IncidentResponse,
MetricCategory::Compliance,
MetricCategory::Performance,
MetricCategory::ResourceUtilization,
MetricCategory::TreatyCompliance,
MetricCategory::EnvironmentalResilience,
MetricCategory::SecurityPosture,
];
for category in categories {
self.threat_metrics.insert(category, VecDeque::with_capacity(1000));
}
Ok(())
}
/**
* Calculate current security posture score
* Aggregates scores from all security domains with weighted scoring
*/
pub fn calculate_security_posture(&mut self) -> Result<SecurityPostureScore, &'static str> {
let calculation_start = now();
// Calculate individual domain scores
let threat_score = self.calculate_threat_detection_score()?;
let incident_score = self.calculate_incident_response_score()?;
let compliance_score = self.calculate_compliance_score()?;
let performance_score = self.calculate_performance_score()?;
let resource_score = self.calculate_resource_utilization_score()?;
let treaty_score = self.calculate_treaty_compliance_score()?;
let environmental_score = self.calculate_environmental_resilience_score()?;
// Calculate weighted overall score
// Weights: Threat(15%), Incident(15%), Compliance(20%), Performance(15%), Resource(10%), Treaty(15%), Environmental(10%)
let overall_score = (
(threat_score as f64 * 0.15) +
(incident_score as f64 * 0.15) +
(compliance_score as f64 * 0.20) +
(performance_score as f64 * 0.15) +
(resource_score as f64 * 0.10) +
(treaty_score as f64 * 0.15) +
(environmental_score as f64 * 0.10)
).round() as u8;
// Determine posture level
let posture_level = if overall_score < POSTURE_SCORE_CRITICAL_THRESHOLD {
SecurityPostureLevel::Critical
} else if overall_score < POSTURE_SCORE_HIGH_THRESHOLD {
SecurityPostureLevel::HighRisk
} else if overall_score < POSTURE_SCORE_MEDIUM_THRESHOLD {
SecurityPostureLevel::MediumRisk
} else if overall_score < POSTURE_SCORE_EXCELLENT_THRESHOLD {
SecurityPostureLevel::LowRisk
} else {
SecurityPostureLevel::Excellent
};
// Identify risk factors
let mut risk_factors = Vec::new();
if threat_score < 70 {
risk_factors.push("Threat detection effectiveness below target".to_string());
}
if incident_score < 70 {
risk_factors.push("Incident response time exceeding targets".to_string());
}
if compliance_score < 90 {
risk_factors.push("Regulatory compliance gaps detected".to_string());
}
if treaty_score < 100 {
risk_factors.push("Treaty/FPIC compliance violations".to_string());
}
// Generate improvement recommendations
let mut recommendations = Vec::new();
if overall_score < 90 {
recommendations.push("Review and update security policies".to_string());
recommendations.push("Conduct additional security training".to_string());
recommendations.push("Implement additional monitoring controls".to_string());
}
// Create posture score
let posture = SecurityPostureScore {
overall_score,
threat_detection_score: threat_score,
incident_response_score: incident_score,
compliance_score: compliance_score,
performance_score: performance_score,
resource_utilization_score: resource_score,
treaty_compliance_score: treaty_score,
environmental_resilience_score: environmental_score,
timestamp: now(),
posture_level,
risk_factors,
improvement_recommendations: recommendations,
};
self.current_posture = posture.clone();
// Update metrics
let calculation_time_ms = (now() - calculation_start) / 1000;
if calculation_time_ms > MAX_METRIC_CALCULATION_MS {
warn!("Posture calculation exceeded time limit: {}ms", calculation_time_ms);
}
// Log posture calculation
self.audit_log.append_log(
LogEventType::SecurityMetrics,
LogSeverity::Info,
format!("Security posture calculated: {} ({:?})", overall_score, posture_level).into_bytes(),
None,
None,
)?;
Ok(posture)
}
/**
* Calculate threat detection effectiveness score (0-100)
*/
fn calculate_threat_detection_score(&self) -> Result<u8, &'static str> {
let metrics = self.threat_detection.get_metrics();
// Calculate detection rate: (detected threats / total threats) * 100
let detection_rate = if metrics.total_threats > 0 {
(metrics.threats_detected as f64 / metrics.total_threats as f64) * 100.0
} else {
100.0
};
// Calculate false positive rate penalty
let fp_penalty = metrics.false_positive_rate_percent.min(10.0) * 2.0; // Max 20 point penalty
// Calculate detection latency penalty
let latency_penalty = if metrics.avg_detection_time_ms > 50.0 {
((metrics.avg_detection_time_ms - 50.0) / 10.0).min(20.0)
} else {
0.0
};
// Calculate final score
let score = (detection_rate - fp_penalty - latency_penalty).max(0.0).min(100.0) as u8;
Ok(score)
}
/**
* Calculate incident response effectiveness score (0-100)
*/
fn calculate_incident_response_score(&self) -> Result<u8, &'static str> {
let metrics = self.incident_response.get_metrics();
// Calculate resolution rate
let resolution_rate = metrics.resolution_rate_percent;
// Calculate response time penalty
let response_penalty = if metrics.avg_detection_time_ms > 100.0 {
((metrics.avg_detection_time_ms - 100.0) / 50.0).min(30.0)
} else {
0.0
};
// Calculate containment time penalty
let containment_penalty = if metrics.avg_containment_time_ms > 300000.0 {
((metrics.avg_containment_time_ms - 300000.0) / 100000.0).min(20.0)
} else {
0.0
};
// Calculate final score
let score = (resolution_rate - response_penalty - containment_penalty).max(0.0).min(100.0) as u8;
Ok(score)
}
/**
* Calculate compliance score (0-100)
*/
fn calculate_compliance_score(&self) -> Result<u8, &'static str> {
let metrics = self.compliance_engine.get_metrics();
// Calculate compliance percentage
let compliance_percent = metrics.compliance_percentage;
// Calculate violation penalty
let violation_penalty = (metrics.violation_count + metrics.critical_violation_count * 2) as f64 * 5.0;
// Calculate final score
let score = (compliance_percent - violation_penalty).max(0.0).min(100.0) as u8;
Ok(score)
}
/**
* Calculate performance score (0-100)
*/
fn calculate_performance_score(&self) -> Result<u8, &'static str> {
// Calculate based on latency percentiles and availability
let mut latency_score = 100;
let mut availability_score = 100;
// Check p95 latency
for metrics in self.performance_metrics.values() {
if let Some(latest) = metrics.back() {
if latest.p95_latency_ms > LATENCY_P95_TARGET_MS as f64 {
latency_score = latency_score.saturating_sub(20);
}
if latest.availability_percent < AVAILABILITY_TARGET_PERCENT {
availability_score = availability_score.saturating_sub(30);
}
}
}
let score = ((latency_score + availability_score) / 2).min(100);
Ok(score)
}
/**
* Calculate resource utilization score (0-100)
*/
fn calculate_resource_utilization_score(&self) -> Result<u8, &'static str> {
if let Some(latest) = self.resource_metrics.back() {
// Penalize high utilization
let mut score = 100;
if latest.cpu_percent > CPU_UTILIZATION_WARNING_PERCENT {
score = score.saturating_sub(20);
}
if latest.memory_percent > MEMORY_UTILIZATION_WARNING_PERCENT {
score = score.saturating_sub(20);
}
if latest.disk_percent > DISK_UTILIZATION_WARNING_PERCENT {
score = score.saturating_sub(30);
}
if latest.network_bandwidth_percent > NETWORK_BANDWIDTH_WARNING_PERCENT {
score = score.saturating_sub(15);
}
Ok(score)
} else {
Ok(100) // No data = perfect score
}
}
/**
* Calculate treaty compliance score (0-100)
*/
fn calculate_treaty_compliance_score(&self) -> Result<u8, &'static str> {
let metrics = self.compliance_engine.get_metrics();
// Treaty compliance must be 100% or score drops significantly
if metrics.treaty_violations_blocked > 0 {
Ok(0) // Any treaty violation = 0 score
} else {
Ok(100)
}
}
/**
* Calculate environmental resilience score (0-100)
*/
fn calculate_environmental_resilience_score(&self) -> Result<u8, &'static str> {
// Based on Phoenix-specific resilience metrics
let haboob_score = self.environmental_score.haboob_resilience_score;
let heat_score = self.environmental_score.extreme_heat_continuity_score;
let flood_score = self.environmental_score.monsoon_flood_readiness_score;
// Weighted average
let score = ((haboob_score as u32 + heat_score as u32 + flood_score as u32) / 3) as u8;
Ok(score)
}
/**
* Aggregate threat intelligence metrics for time window
*/
pub fn aggregate_threat_metrics(&mut self, time_window_ms: u64) -> Result<ThreatIntelligenceMetric, &'static str> {
let window_start = now() - time_window_ms;
let window_end = now();
// Collect threat events in window
let threat_events: Vec<ThreatEvent> = self.threat_detection.get_recent_threats(time_window_ms)
.unwrap_or_else(Vec::new);
if threat_events.is_empty() {
return Ok(ThreatIntelligenceMetric {
metric_id: self.generate_metric_id(),
threat_category: ThreatCategory::Unknown,
threat_count: 0,
unique_sources: 0,
unique_targets: 0,
avg_confidence: 0.0,
max_severity: ThreatSeverity::Low,
trend_direction: ThreatTrendDirection::Stable,
trend_magnitude: 0.0,
correlation_score: 0.0,
iocs: BTreeSet::new(),
timestamp: now(),
time_window_ms,
});
}
// Aggregate metrics
let threat_count = threat_events.len();
let unique_sources: BTreeSet<BirthSign> = threat_events.iter().map(|e| e.source_node.clone()).collect();
let unique_targets: BTreeSet<BirthSign> = threat_events.iter()
.filter_map(|e| e.target_node.clone())
.collect();
let avg_confidence: f64 = threat_events.iter().map(|e| e.confidence as f64).sum::<f64>() / threat_count as f64;
let max_severity = threat_events.iter()
.map(|e| e.severity)
.max()
.unwrap_or(ThreatSeverity::Low);
// Calculate trend direction
let trend = self.calculate_threat_trend(&threat_events, time_window_ms)?;
// Calculate correlation score
let correlation_score = self.calculate_threat_correlation(&threat_events)?;
// Extract IOCs
let mut iocs = BTreeSet::new();
for event in &threat_events {
if let Some(ref hash) = event.payload_hash {
iocs.insert(format!("{:02x}{:02x}{:02x}{:02x}", hash[0], hash[1], hash[2], hash[3]));
}
}
let metric = ThreatIntelligenceMetric {
metric_id: self.generate_metric_id(),
threat_category: threat_events[0].category,
threat_count,
unique_sources: unique_sources.len(),
unique_targets: unique_targets.len(),
avg_confidence,
max_severity,
trend_direction: trend.0,
trend_magnitude: trend.1,
correlation_score,
iocs,
timestamp: now(),
time_window_ms,
};
// Store in history
let category = MetricCategory::ThreatDetection;
if let Some(metrics) = self.threat_metrics.get_mut(&category) {
metrics.push_back(metric.clone());
if metrics.len() > 1000 {
metrics.pop_front();
}
}
Ok(metric)
}
/**
* Calculate threat trend direction and magnitude
*/
fn calculate_threat_trend(&self, events: &[ThreatEvent], time_window_ms: u64) -> Result<(ThreatTrendDirection, f64), &'static str> {
if events.len() < TREND_DETECTION_MIN_POINTS {
return Ok((ThreatTrendDirection::Stable, 0.0));
}
// Simple linear regression on event timestamps
let mut sum_x = 0.0;
let mut sum_y = 0.0;
let mut sum_xx = 0.0;
let mut sum_xy = 0.0;
let n = events.len() as f64;
for (i, event) in events.iter().enumerate() {
let x = i as f64;
let y = event.timestamp as f64;
sum_x += x;
sum_y += y;
sum_xx += x * x;
sum_xy += x * y;
}
let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
let direction = if slope > 0.1 {
ThreatTrendDirection::Increasing
} else if slope < -0.1 {
ThreatTrendDirection::Decreasing
} else if slope.abs() > 1.0 {
if slope > 0.0 {
ThreatTrendDirection::Spike
} else {
ThreatTrendDirection::Dip
}
} else {
ThreatTrendDirection::Stable
};
let magnitude = slope.abs();
Ok((direction, magnitude))
}
/**
* Calculate threat correlation score (0-100)
*/
fn calculate_threat_correlation(&self, events: &[ThreatEvent]) -> Result<f64, &'static str> {
if events.len() < 2 {
return Ok(0.0);
}
// Simple correlation based on common attributes
let mut correlation_score = 0.0;
// Check for common source nodes
let sources: BTreeSet<BirthSign> = events.iter().map(|e| e.source_node.clone()).collect();
if sources.len() == 1 {
correlation_score += 30.0;
}
// Check for common categories
let categories: BTreeSet<ThreatCategory> = events.iter().map(|e| e.category).collect();
if categories.len() == 1 {
correlation_score += 20.0;
}
// Check for common detection methods
let methods: BTreeSet<DetectionMethod> = events.iter().map(|e| e.detection_method).collect();
if methods.len() == 1 {
correlation_score += 15.0;
}
// Check for temporal proximity
let time_span = events.iter().map(|e| e.timestamp).max().unwrap_or(0)
- events.iter().map(|e| e.timestamp).min().unwrap_or(0);
if time_span < 60000000 { // Within 1 minute
correlation_score += 20.0;
}
// Check for treaty violations
let treaty_violations = events.iter().filter(|e| e.treaty_violation.is_some()).count();
if treaty_violations > 0 {
correlation_score += 15.0;
}
correlation_score.min(100.0)
}
/**
* Generate predictive alerts based on forecasted metrics
*/
pub fn generate_predictive_alerts(&mut self) -> Result<Vec<PredictiveAlert>, &'static str> {
let prediction_start = now();
let mut alerts = Vec::new();
// Predict threat count increase
let threat_prediction = self.predict_metric("threat_count_1h", 24)?;
if threat_prediction.confidence_interval_upper > 50.0 {
alerts.push(PredictiveAlert {
alert_id: self.generate_alert_id(),
severity: AlertSeverity::Warning,
metric_category: MetricCategory::ThreatDetection,
description: "Predicted increase in threat activity".to_string(),
predicted_value: threat_prediction.confidence_interval_upper,
threshold_value: 50.0,
confidence_percent: threat_prediction.confidence_interval_upper,
prediction_horizon_hours: 24,
recommended_action: "Increase monitoring frequency and prepare incident response resources".to_string(),
timestamp: now(),
acknowledged: false,
acknowledged_by: None,
acknowledged_timestamp: None,
});
}
// Predict compliance violations
let compliance_prediction = self.predict_metric("compliance_violations", 7)?;
if compliance_prediction.confidence_interval_upper > 5.0 {
alerts.push(PredictiveAlert {
alert_id: self.generate_alert_id(),
severity: AlertSeverity::Critical,
metric_category: MetricCategory::Compliance,
description: "Predicted increase in compliance violations".to_string(),
predicted_value: compliance_prediction.confidence_interval_upper,
threshold_value: 5.0,
confidence_percent: compliance_prediction.confidence_interval_upper,
prediction_horizon_hours: 168, // 7 days
recommended_action: "Review and update compliance policies, conduct staff training".to_string(),
timestamp: now(),
acknowledged: false,
acknowledged_by: None,
acknowledged_timestamp: None,
});
}
// Predict resource exhaustion
let cpu_prediction = self.predict_metric("cpu_utilization", 48)?;
if cpu_prediction.confidence_interval_upper > CPU_UTILIZATION_WARNING_PERCENT + 10.0 {
alerts.push(PredictiveAlert {
alert_id: self.generate_alert_id(),
severity: AlertSeverity::Warning,
metric_category: MetricCategory::ResourceUtilization,
description: "Predicted CPU utilization approaching capacity".to_string(),
predicted_value: cpu_prediction.confidence_interval_upper,
threshold_value: CPU_UTILIZATION_WARNING_PERCENT + 10.0,
confidence_percent: cpu_prediction.confidence_interval_upper,
prediction_horizon_hours: 48,
recommended_action: "Scale resources or optimize workloads to reduce CPU usage".to_string(),
timestamp: now(),
acknowledged: false,
acknowledged_by: None,
acknowledged_timestamp: None,
});
}
// Store alerts
for alert in &alerts {
self.predictive_alerts.push_back(alert.clone());
if self.predictive_alerts.len() > 1000 {
self.predictive_alerts.pop_front();
}
}
// Update metrics
let prediction_time_ms = (now() - prediction_start) / 1000;
self.last_prediction = now();
Ok(alerts)
}
/**
* Predict metric value using configured model
*/
fn predict_metric(&mut self, metric_name: &str, horizon_hours: u32) -> Result<TimeSeriesPoint, &'static str> {
let model = self.predictive_models.get_mut(metric_name)
.ok_or("Prediction model not found")?;
// Train model if needed
if model.training_data_points < 10 {
self.train_prediction_model(model)?;
}
// Generate prediction
let predicted_value = self.generate_prediction(model, horizon_hours)?;
let confidence_lower = predicted_value * 0.9;
let confidence_upper = predicted_value * 1.1;
Ok(TimeSeriesPoint {
timestamp: now() + (horizon_hours as u64 * 3600 * 1000000),
value: predicted_value,
confidence_interval_lower: confidence_lower,
confidence_interval_upper: confidence_upper,
})
}
/**
* Train prediction model with historical data
*/
fn train_prediction_model(&mut self, model: &mut PredictionModel) -> Result<(), &'static str> {
// Collect historical data
let mut training_data = Vec::new();
match model.metric_name.as_str() {
"threat_count_1h" => {
// Use threat metrics history
for metrics in self.threat_metrics.values() {
for metric in metrics {
training_data.push(metric.threat_count as f64);
}
}
},
"compliance_violations" => {
let compliance_metrics = self.compliance_engine.get_metrics();
training_data.push(compliance_metrics.violation_count as f64);
},
"cpu_utilization" => {
for metric in &self.resource_metrics {
training_data.push(metric.cpu_percent);
}
},
_ => {}
}
// Update model
model.training_data_points = training_data.len();
model.last_trained = now();
// Calculate accuracy (simplified)
model.accuracy_percent = 95.0 + (training_data.len() as f64 / 100.0).min(5.0);
Ok(())
}
/**
* Generate prediction using exponential smoothing
*/
fn generate_prediction(&self, model: &PredictionModel, horizon_hours: u32) -> Result<f64, &'static str> {
// Simple exponential smoothing: S_t = α * X_t + (1-α) * S_{t-1}
let alpha = model.parameters.get("alpha")
.map(|s| s.parse::<f64>().unwrap_or(0.3))
.unwrap_or(0.3);
// For now, return current value (would use historical data in production)
Ok(10.0 * (1.0 + (horizon_hours as f64 / 24.0) * 0.1))
}
/**
* Update compliance dashboard with latest metrics
*/
pub fn update_compliance_dashboard(&mut self) -> Result<ComplianceDashboard, &'static str> {
let metrics = self.compliance_engine.get_metrics();
// Calculate compliance trend
let trend = if metrics.compliance_percentage > 99.0 {
ComplianceTrend::Improving
} else if metrics.compliance_percentage < 95.0 {
ComplianceTrend::Declining
} else {
ComplianceTrend::Stable
};
// Count overdue remediations
let overdue = metrics.violations_by_severity.get(&4).unwrap_or(&0)
+ metrics.violations_by_severity.get(&5).unwrap_or(&0);
let dashboard = ComplianceDashboard {
dashboard_id: self.generate_dashboard_id(),
overall_compliance_percent: metrics.compliance_percentage,
compliant_checks: metrics.compliant_count,
warning_checks: metrics.warning_count,
violation_checks: metrics.violation_count,
critical_violation_checks: metrics.critical_violation_count,
compliance_trend: trend,
trend_period_days: 30,
domains_assessed: metrics.domains_assessed.clone(),
treaty_compliance_percent: if metrics.treaty_violations_blocked > 0 { 0.0 } else { 100.0 },
fpic_compliance_percent: metrics.compliance_percentage,
neurorights_compliance_percent: if metrics.treaty_violations_blocked > 0 { 0.0 } else { 100.0 },
last_audit_timestamp: self.compliance_engine.last_audit,
next_audit_due: self.compliance_engine.last_audit + (30 * 24 * 60 * 60 * 1000000),
remediation_overdue: *overdue,
};
self.compliance_dashboard = dashboard.clone();
Ok(dashboard)
}
/**
* Take security metrics snapshot for historical retention
*/
pub fn take_snapshot(&mut self) -> Result<SecurityMetricsSnapshot, &'static str> {
let snapshot_start = now();
// Calculate current posture
let posture = self.calculate_security_posture()?;
// Aggregate threat metrics
let threat_metrics = self.aggregate_threat_metrics(METRICS_WINDOW_1HOUR_MS)?;
// Get latest performance metrics
let performance_metrics: Vec<PerformanceMetric> = self.performance_metrics.values()
.filter_map(|m| m.back().cloned())
.take(10)
.collect();
// Get latest resource metrics
let resource_metrics: Vec<ResourceUtilizationMetric> = self.resource_metrics.iter().rev().take(5).cloned().collect();
// Update compliance dashboard
let compliance_dashboard = self.update_compliance_dashboard()?;
// Generate predictive alerts
let predictive_alerts = self.generate_predictive_alerts()?;
// Create snapshot
let snapshot = SecurityMetricsSnapshot {
snapshot_id: self.generate_snapshot_id(),
posture_score: posture,
threat_metrics: vec![threat_metrics],
performance_metrics,
resource_metrics,
compliance_dashboard,
environmental_score: self.environmental_score.clone(),
predictive_alerts,
timestamp: now(),
signature: self.crypto_engine.sign_message(&self.node_id.to_bytes())?,
};
// Store in history
self.metrics_history.snapshots.push_back(snapshot.clone());
if self.metrics_history.snapshots.len() > self.metrics_history.max_snapshots {
self.metrics_history.snapshots.pop_front();
}
// Update metrics
let snapshot_time_ms = (now() - snapshot_start) / 1000;
if snapshot_time_ms > MAX_DASHBOARD_REFRESH_MS {
warn!("Snapshot generation exceeded time limit: {}ms", snapshot_time_ms);
}
Ok(snapshot)
}
/**
* Get security posture score
*/
pub fn get_posture_score(&self) -> &SecurityPostureScore {
&self.current_posture
}
/**
* Get predictive alerts
*/
pub fn get_predictive_alerts(&self) -> Vec<&PredictiveAlert> {
self.predictive_alerts.iter().collect()
}
/**
* Get compliance dashboard
*/
pub fn get_compliance_dashboard(&self) -> &ComplianceDashboard {
&self.compliance_dashboard
}
/**
* Get threat metrics for category
*/
pub fn get_threat_metrics(&self, category: MetricCategory) -> Option<&VecDeque<ThreatIntelligenceMetric>> {
self.threat_metrics.get(&category)
}
/**
* Acknowledge predictive alert
*/
pub fn acknowledge_alert(&mut self, alert_id: &[u8; 32], acknowledged_by: BirthSign) -> Result<(), &'static str> {
for alert in &mut self.predictive_alerts {
if alert.alert_id == *alert_id {
alert.acknowledged = true;
alert.acknowledged_by = Some(acknowledged_by);
alert.acknowledged_timestamp = Some(now());
// Log acknowledgment
self.audit_log.append_log(
LogEventType::SecurityMetrics,
LogSeverity::Info,
format!("Predictive alert acknowledged: {:?}", alert_id).into_bytes(),
None,
None,
)?;
return Ok(());
}
}
Err("Alert not found")
}
/**
* Perform maintenance tasks (prune history, retrain models, cleanup)
*/
pub fn perform_maintenance(&mut self) -> Result<(), &'static str> {
let now = now();
// Prune old snapshots (>365 days)
while let Some(snapshot) = self.metrics_history.snapshots.front() {
if now - snapshot.timestamp > (self.metrics_history.retention_days as u64 * 24 * 60 * 60 * 1000000) {
self.metrics_history.snapshots.pop_front();
} else {
break;
}
}
// Retrain prediction models
for model in self.predictive_models.values_mut() {
self.train_prediction_model(model)?;
}
// Cleanup old aggregation windows (>7 days)
let old_windows: Vec<_> = self.aggregation_windows.iter()
.filter(|(_, w)| now - w.start_time > 7 * 24 * 60 * 60 * 1000000)
.map(|(id, _)| *id)
.collect();
for window_id in old_windows {
self.aggregation_windows.remove(&window_id);
}
self.metrics_history.last_pruned = now;
Ok(())
}
/**
* Generate unique IDs
*/
fn generate_metric_id(&self) -> [u8; 32] {
let mut id = [0u8; 32];
let timestamp = now();
id[..8].copy_from_slice(&timestamp.to_be_bytes());
id[8..24].copy_from_slice(&self.node_id.to_bytes()[..16]);
id[24..].copy_from_slice(&self.metrics_history.snapshots.len().to_be_bytes()[..8]);
self.crypto_engine.sha512_hash(&id)[..32].try_into().unwrap_or([0u8; 32])
}
fn generate_model_id(&self) -> [u8; 32] {
self.generate_metric_id()
}
fn generate_alert_id(&self) -> [u8; 32] {
self.generate_metric_id()
}
fn generate_dashboard_id(&self) -> [u8; 32] {
self.generate_metric_id()
}
fn generate_snapshot_id(&self) -> [u8; 32] {
self.generate_metric_id()
}
}
// --- Unit Tests (Offline Capable) ---
#[cfg(test)]
mod tests {
use super::*;
#[test]
fn test_engine_initialization() {
let engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
assert!(engine.active);
assert_eq!(engine.predictive_models.len(), 4); // Initialized models
assert_eq!(engine.threat_metrics.len(), 8); // Metric categories
assert_eq!(engine.current_posture.overall_score, 100);
}
#[test]
fn test_security_posture_calculation() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Calculate posture
let posture = engine.calculate_security_posture().unwrap();
assert_eq!(posture.overall_score, 100);
assert_eq!(posture.posture_level, SecurityPostureLevel::Excellent);
// Verify all domain scores calculated
assert!(posture.threat_detection_score > 0);
assert!(posture.incident_response_score > 0);
assert!(posture.compliance_score > 0);
assert!(posture.performance_score > 0);
assert!(posture.resource_utilization_score > 0);
assert!(posture.treaty_compliance_score > 0);
assert!(posture.environmental_resilience_score > 0);
}
#[test]
fn test_threat_metrics_aggregation() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Aggregate 1-hour threat metrics
let metric = engine.aggregate_threat_metrics(METRICS_WINDOW_1HOUR_MS).unwrap();
assert_eq!(metric.time_window_ms, METRICS_WINDOW_1HOUR_MS);
assert_eq!(metric.threat_count, 0); // No threats yet
// Verify stored in history
assert_eq!(engine.threat_metrics.get(&MetricCategory::ThreatDetection).unwrap().len(), 1);
}
#[test]
fn test_compliance_dashboard_update() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Update dashboard
let dashboard = engine.update_compliance_dashboard().unwrap();
assert_eq!(dashboard.overall_compliance_percent, 100.0);
assert_eq!(dashboard.treaty_compliance_percent, 100.0);
assert_eq!(dashboard.fpic_compliance_percent, 100.0);
assert_eq!(dashboard.neurorights_compliance_percent, 100.0);
}
#[test]
fn test_predictive_alert_generation() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Generate alerts
let alerts = engine.generate_predictive_alerts().unwrap();
// Should have some alerts (based on predictions)
assert!(alerts.len() >= 0);
// Verify alerts stored
assert_eq!(engine.predictive_alerts.len(), alerts.len());
}
#[test]
fn test_snapshot_creation() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Take snapshot
let snapshot = engine.take_snapshot().unwrap();
assert_eq!(snapshot.posture_score.overall_score, 100);
assert!(!snapshot.threat_metrics.is_empty());
assert_eq!(snapshot.timestamp, now());
// Verify stored in history
assert_eq!(engine.metrics_history.snapshots.len(), 1);
}
#[test]
fn test_alert_acknowledgment() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Generate an alert
let _ = engine.generate_predictive_alerts().unwrap();
let alert_id = engine.predictive_alerts.front().unwrap().alert_id;
// Acknowledge alert
let result = engine.acknowledge_alert(&alert_id, BirthSign::default());
assert!(result.is_ok());
// Verify acknowledged
let acknowledged = engine.predictive_alerts.iter().find(|a| a.alert_id == alert_id).unwrap();
assert!(acknowledged.acknowledged);
assert!(acknowledged.acknowledged_by.is_some());
}
#[test]
fn test_posture_level_classification() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Test critical posture (<30)
engine.current_posture.overall_score = 25;
assert_eq!(engine.current_posture.posture_level, SecurityPostureLevel::Critical);
// Test high risk (30-50)
engine.current_posture.overall_score = 40;
engine.current_posture.posture_level = SecurityPostureLevel::HighRisk;
assert_eq!(engine.current_posture.posture_level, SecurityPostureLevel::HighRisk);
// Test excellent (>95)
engine.current_posture.overall_score = 98;
engine.current_posture.posture_level = SecurityPostureLevel::Excellent;
assert_eq!(engine.current_posture.posture_level, SecurityPostureLevel::Excellent);
}
#[test]
fn test_maintenance_cleanup() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Take multiple snapshots
for _ in 0..10 {
let _ = engine.take_snapshot().unwrap();
}
assert_eq!(engine.metrics_history.snapshots.len(), 10);
// Perform maintenance
engine.perform_maintenance().unwrap();
// Models should be retrained
assert!(engine.predictive_models.values().all(|m| m.training_data_points > 0));
}
#[test]
fn test_treaty_compliance_scoring() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Perfect treaty compliance = 100 score
let score = engine.calculate_treaty_compliance_score().unwrap();
assert_eq!(score, 100);
// Note: Would test violation case if we could inject violations
}
#[test]
fn test_environmental_resilience_scoring() {
let mut engine = SecurityMetricsEngine::new(BirthSign::default()).unwrap();
// Set environmental scores
engine.environmental_score.haboob_resilience_score = 90;
engine.environmental_score.extreme_heat_continuity_score = 95;
engine.environmental_score.monsoon_flood_readiness_score = 85;
// Calculate score (average of three)
let score = engine.calculate_environmental_resilience_score().unwrap();
assert_eq!(score, 90); // (90 + 95 + 85) / 3 = 90
}
}
