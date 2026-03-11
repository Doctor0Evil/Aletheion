use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use aletheion_core::av::safety::{SafetyState, ComponentHealth, ComponentType, AVSafetySystem};
use aletheion_core::av::routing::{RouteContext, GeoCoordinate};
use aletheion_core::biosignal::{BiosignalData, BiosignalType};
use aletheion_core::data_sovereignty::{DataPolicy, DataScope, SovereigntyGuard};
use aletheion_core::treaty::{FPICStatus, IndigenousNation, TreatyCompliance};
use aletheion_core::identity::{DID, AugmentedCitizenID, VehicleID};
use aletheion_core::temporal::{Timestamp, TimeWindow, Schedule};
use aletheion_core::metrics::{MetricCollector, MetricType};
use aletheion_core::resource::{ResourceAllocation, ResourcePool, ResourceType};
use aletheion_core::workflow::{WorkflowState, WorkflowEngine, TaskPriority};

pub mod predictive_algorithms;
pub mod health_monitoring;
pub mod repair_scheduling;
pub mod component_lifecycle;
pub mod failure_prediction;
pub mod maintenance_workflow;
pub mod resource_allocation;
pub mod treaty_compliance;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaintenanceType {
    Preventive,
    Predictive,
    Corrective,
    Emergency,
    Recertification,
    SoftwareUpdate,
    Calibration,
    Inspection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceTask {
    pub task_id: String,
    pub vehicle_id: VehicleID,
    pub maintenance_type: MaintenanceType,
    pub components: Vec<ComponentType>,
    pub priority: TaskPriority,
    pub estimated_duration: Duration,
    pub required_resources: HashSet<ResourceType>,
    pub skill_requirements: HashSet<MaintenanceSkill>,
    pub safety_critical: bool,
    pub treaty_sensitive: bool,
    pub scheduled_start: Option<Timestamp>,
    pub scheduled_end: Option<Timestamp>,
    pub actual_start: Option<Timestamp>,
    pub actual_end: Option<Timestamp>,
    pub status: MaintenanceStatus,
    pub assigned_technicians: Vec<DID>,
    pub location: MaintenanceLocation,
    pub treaty_compliance_check: TreatyComplianceCheck,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaintenanceStatus {
    Pending,
    Scheduled,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaintenanceSkill {
    ElectricalSystems,
    MechanicalSystems,
    SoftwareDiagnostics,
    SensorCalibration,
    BatteryManagement,
    HydraulicSystems,
    StructuralRepair,
    SafetyCertification,
    IndigenousProtocolTraining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceLocation {
    pub facility_id: String,
    pub coordinates: GeoCoordinate,
    pub facility_type: FacilityType,
    pub capacity: u32,
    pub current_utilization: u32,
    pub treaty_jurisdiction: Option<IndigenousNation>,
    pub fpic_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacilityType {
    ServiceCenter,
    MobileUnit,
    FieldRepair,
    EmergencyResponse,
    CalibrationLab,
    SoftwareDepot,
    InspectionStation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyComplianceCheck {
    pub fpic_verified: bool,
    pub nation_consulted: Option<IndigenousNation>,
    pub protocol_adhered: bool,
    pub cultural_sensitivity_score: f32,
    pub verification_timestamp: Option<Timestamp>,
    pub verifier_id: Option<DID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthRecord {
    pub component_type: ComponentType,
    pub component_id: String,
    pub vehicle_id: VehicleID,
    pub current_health: ComponentHealth,
    pub baseline_health: ComponentHealth,
    pub degradation_rate: f32,
    pub predicted_failure_time: Option<Timestamp>,
    pub maintenance_history: Vec<MaintenanceEvent>,
    pub sensor_readings: VecDeque<SensorReading>,
    pub anomaly_detected: bool,
    pub anomaly_details: Option<AnomalyDetails>,
    pub last_inspection: Option<Timestamp>,
    pub next_inspection_due: Timestamp,
    pub warranty_status: WarrantyStatus,
    pub lifecycle_stage: LifecycleStage,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceEvent {
    pub event_id: String,
    pub timestamp: Timestamp,
    pub event_type: MaintenanceEventType,
    pub component_type: ComponentType,
    pub severity: EventSeverity,
    pub description: String,
    pub corrective_action: Option<String>,
    pub technician_id: Option<DID>,
    pub parts_replaced: Vec<String>,
    pub labor_hours: f32,
    pub cost_estimate: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaintenanceEventType {
    Inspection,
    Repair,
    Replacement,
    Calibration,
    SoftwareUpdate,
    AnomalyDetected,
    FailurePredicted,
    SafetyAlert,
    Recertification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub timestamp: Timestamp,
    pub value: f32,
    pub unit: String,
    pub threshold_violation: bool,
    pub anomaly_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetails {
    pub anomaly_type: AnomalyType,
    pub confidence: f32,
    pub contributing_factors: Vec<String>,
    pub recommended_action: String,
    pub urgency: AnomalyUrgency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyType {
    PerformanceDegradation,
    UnexpectedVariance,
    WearPatternDeviation,
    CalibrationDrift,
    EnvironmentalStress,
    ManufacturingDefect,
    SoftwareGlitch,
    SensorFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyUrgency {
    Monitor,
    ScheduleSoon,
    ImmediateAttention,
    EmergencyShutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WarrantyStatus {
    Active,
    Expired,
    Voided,
    Extended,
    UnderClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleStage {
    New,
    BrokenIn,
    NormalOperation,
    EarlyDegradation,
    AdvancedDegradation,
    EndOfLife,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveModel {
    pub model_id: String,
    pub component_type: ComponentType,
    pub algorithm: PredictionAlgorithm,
    pub training_data_points: usize,
    pub accuracy_score: f32,
    pub false_positive_rate: f32,
    pub false_negative_rate: f32,
    pub prediction_horizon: Duration,
    pub feature_importance: HashMap<String, f32>,
    pub last_trained: Timestamp,
    pub next_retraining_due: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredictionAlgorithm {
    RandomForest,
    GradientBoosting,
    NeuralNetwork,
    SurvivalAnalysis,
    BayesianNetwork,
    TimeSeriesForecasting,
    EnsembleMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetHealthSnapshot {
    pub snapshot_id: String,
    pub timestamp: Timestamp,
    pub total_vehicles: usize,
    pub vehicles_operational: usize,
    pub vehicles_under_maintenance: usize,
    pub vehicles_offline: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub informational_alerts: usize,
    pub average_fleet_health: f32,
    pub component_health_summary: HashMap<ComponentType, ComponentHealthSummary>,
    pub maintenance_backlog: usize,
    pub estimated_downtime_hours: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthSummary {
    pub total_components: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub critical: usize,
    pub failed: usize,
    pub average_health_score: f32,
    pub predicted_failures_next_30d: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceSchedule {
    pub schedule_id: String,
    pub vehicle_id: VehicleID,
    pub tasks: Vec<MaintenanceTask>,
    pub total_estimated_duration: Duration,
    pub resource_requirements: ResourceAllocationPlan,
    pub treaty_compliance_status: TreatyComplianceStatus,
    pub safety_verification: SafetyVerification,
    pub created_at: Timestamp,
    pub valid_until: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationPlan {
    pub required_technicians: usize,
    pub required_skills: HashSet<MaintenanceSkill>,
    pub equipment_needed: HashSet<ResourceType>,
    pub parts_inventory: HashMap<String, u32>,
    pub facility_requirements: Vec<MaintenanceLocation>,
    pub estimated_cost: f64,
    pub priority_adjustment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyComplianceStatus {
    pub fpic_status: FPICStatus,
    pub nations_affected: Vec<IndigenousNation>,
    pub protocols_required: Vec<String>,
    pub consultation_completed: bool,
    pub cultural_liaison_assigned: Option<DID>,
    pub compliance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyVerification {
    pub pre_maintenance_checks: Vec<SafetyCheck>,
    pub post_maintenance_validation: Vec<SafetyCheck>,
    pub safety_critical_components: Vec<ComponentType>,
    pub recertification_required: bool,
    pub safety_officer_approval: Option<DID>,
    pub verification_timestamp: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheck {
    pub check_id: String,
    pub check_type: SafetyCheckType,
    pub component: ComponentType,
    pub pass_threshold: f32,
    pub actual_value: Option<f32>,
    pub passed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyCheckType {
    FunctionalTest,
    PerformanceBenchmark,
    CalibrationVerification,
    StructuralIntegrity,
    SoftwareIntegrity,
    SensorAccuracy,
    CommunicationLatency,
    EmergencyStopTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWorkflow {
    pub workflow_id: String,
    pub task_id: String,
    pub current_state: WorkflowState,
    pub state_history: Vec<WorkflowStateTransition>,
    pub assigned_personnel: Vec<DID>,
    pub quality_checks: Vec<QualityCheck>,
    pub documentation_requirements: Vec<DocumentType>,
    pub treaty_protocol_steps: Vec<TreatyProtocolStep>,
    pub safety_gates: Vec<SafetyGate>,
    pub completion_criteria: Vec<CompletionCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStateTransition {
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
    pub timestamp: Timestamp,
    pub triggered_by: DID,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheck {
    pub check_id: String,
    pub check_type: QualityCheckType,
    pub component: ComponentType,
    pub inspector: DID,
    pub timestamp: Timestamp,
    pub result: QualityResult,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityCheckType {
    VisualInspection,
    FunctionalTest,
    DimensionalMeasurement,
    MaterialAnalysis,
    SoftwareValidation,
    CalibrationCheck,
    PerformanceTest,
    SafetyVerification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityResult {
    Pass,
    Fail,
    ReworkRequired,
    ConditionalPass,
    PendingReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    WorkOrder,
    InspectionReport,
    RepairLog,
    PartsInventory,
    CalibrationCertificate,
    SafetyCertification,
    TreatyComplianceReport,
    QualityAssuranceReport,
    TrainingRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyProtocolStep {
    pub step_id: String,
    pub protocol_name: String,
    pub nation: IndigenousNation,
    pub required_action: String,
    pub responsible_party: DID,
    pub completion_timestamp: Option<Timestamp>,
    pub verification_method: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGate {
    pub gate_id: String,
    pub gate_type: SafetyGateType,
    pub prerequisite_checks: Vec<String>,
    pub approval_required: bool,
    pub approver_role: SafetyRole,
    pub bypass_allowed: bool,
    passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyGateType {
    PreWorkAuthorization,
    ComponentIsolation,
    SystemShutdown,
    PostRepairValidation,
    RecertificationApproval,
    FinalInspection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyRole {
    SafetyOfficer,
    LeadTechnician,
    QualityInspector,
    SystemEngineer,
    TreatyLiaison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCriterion {
    pub criterion_id: String,
    pub description: String,
    pub verification_method: String,
    pub verifier_role: SafetyRole,
    pub verified: bool,
    pub verification_timestamp: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    pub predictive_models: HashMap<ComponentType, PredictiveModel>,
    pub health_thresholds: HashMap<ComponentType, HealthThresholds>,
    pub maintenance_intervals: HashMap<ComponentType, Duration>,
    pub resource_pools: HashMap<ResourceType, ResourcePool>,
    pub facility_inventory: HashMap<String, MaintenanceLocation>,
    pub technician_skills: HashMap<DID, HashSet<MaintenanceSkill>>,
    pub treaty_protocols: HashMap<IndigenousNation, TreatyMaintenanceProtocol>,
    pub safety_checklists: HashMap<MaintenanceType, Vec<SafetyCheckType>>,
    pub workflow_templates: HashMap<MaintenanceType, MaintenanceWorkflowTemplate>,
    pub metric_collector: Arc<MetricCollector>,
    pub update_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    pub warning_threshold: f32,
    pub critical_threshold: f32,
    pub failure_threshold: f32,
    pub hysteresis: f32,
    pub sampling_frequency_hz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatyMaintenanceProtocol {
    pub nation: IndigenousNation,
    pub consultation_requirements: Vec<ConsultationRequirement>,
    pub cultural_protocols: Vec<CulturalProtocol>,
    pub fpic_process: FPICProcess,
    pub liaison_requirements: bool,
    pub reporting_obligations: Vec<ReportingObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultationRequirement {
    pub requirement_type: ConsultationType,
    pub timing: ConsultationTiming,
    pub participants: Vec<String>,
    pub documentation_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsultationType {
    PreWorkNotification,
    ImpactAssessmentReview,
    CulturalSiteSurvey,
    TraditionalKnowledgeIntegration,
    PostWorkDebrief,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsultationTiming {
    Immediate,
    Within24Hours,
    Within7Days,
    DuringPlanningPhase,
    Ongoing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalProtocol {
    pub protocol_name: String,
    pub description: String,
    pub required_actions: Vec<String>,
    pub prohibited_actions: Vec<String>,
    pub ceremonial_requirements: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FPICProcess {
    pub notification_method: String,
    pub consultation_period_days: u32,
    pub consent_verification: ConsentVerificationMethod,
    pub documentation_requirements: Vec<String>,
    pub appeal_process: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsentVerificationMethod {
    WrittenDocument,
    CommunityAssembly,
    TribalCouncilResolution,
    DigitalSignature,
    WitnessedVerbal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingObligation {
    pub report_type: ReportType,
    pub frequency: ReportingFrequency,
    pub recipients: Vec<String>,
    pub content_requirements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    ActivityLog,
    ImpactAssessment,
    CulturalResourceReport,
    EnvironmentalMonitoring,
    SafetyIncident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportingFrequency {
    RealTime,
    Daily,
    Weekly,
    Monthly,
    PerProject,
    UponCompletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWorkflowTemplate {
    pub template_id: String,
    pub maintenance_type: MaintenanceType,
    pub default_states: Vec<WorkflowState>,
    pub default_transitions: Vec<WorkflowTransition>,
    pub required_approvals: Vec<SafetyRole>,
    pub quality_checkpoints: Vec<QualityCheckType>,
    pub treaty_protocol_steps: Vec<TreatyProtocolStep>,
    pub safety_gates: Vec<SafetyGateType>,
    pub documentation_requirements: Vec<DocumentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
    pub trigger_condition: String,
    pub required_role: SafetyRole,
    approval_required: bool,
}

pub struct AVMaintenanceSystem {
    config: Arc<RwLock<MaintenanceConfig>>,
    fleet_health: Arc<RwLock<HashMap<VehicleID, VehicleHealthRecord>>>,
    active_tasks: Arc<Mutex<HashMap<String, MaintenanceTask>>>,
    pending_tasks: Arc<Mutex<VecDeque<MaintenanceTask>>>,
    completed_tasks: Arc<Mutex<VecDeque<MaintenanceTask>>>,
    predictive_engine: Arc<PredictiveMaintenanceEngine>,
    scheduler: Arc<MaintenanceScheduler>,
    resource_allocator: Arc<ResourceAllocator>,
    treaty_compliance_engine: Arc<TreatyComplianceEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    metric_collector: Arc<MetricCollector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleHealthRecord {
    pub vehicle_id: VehicleID,
    pub last_health_update: Timestamp,
    pub overall_health_score: f32,
    pub component_records: HashMap<ComponentType, ComponentHealthRecord>,
    pub active_alerts: Vec<MaintenanceAlert>,
    pub predicted_maintenance_needs: Vec<PredictedMaintenanceNeed>,
    pub maintenance_history: Vec<MaintenanceEvent>,
    pub current_location: GeoCoordinate,
    pub operational_status: VehicleOperationalStatus,
    pub treaty_jurisdiction: Option<IndigenousNation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceAlert {
    pub alert_id: String,
    pub timestamp: Timestamp,
    pub component: ComponentType,
    pub severity: EventSeverity,
    pub description: String,
    pub recommended_action: String,
    pub urgency: AlertUrgency,
    acknowledged: bool,
    acknowledged_by: Option<DID>,
    acknowledged_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertUrgency {
    Low,
    Medium,
    High,
    Immediate,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedMaintenanceNeed {
    pub component: ComponentType,
    pub predicted_failure_time: Timestamp,
    pub confidence: f32,
    pub recommended_maintenance_window: TimeWindow,
    pub estimated_impact_if_delayed: ImpactAssessment,
    maintenance_type: MaintenanceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleOperationalStatus {
    Operational,
    LimitedOperation,
    MaintenanceRequired,
    OfflineForMaintenance,
    SafetyHold,
    Decommissioned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub safety_impact: SafetyImpactLevel,
    pub operational_impact: OperationalImpactLevel,
    pub financial_impact: f64,
    pub environmental_impact: EnvironmentalImpactLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyImpactLevel {
    Negligible,
    Minor,
    Moderate,
    Significant,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationalImpactLevel {
    None,
    Minimal,
    Moderate,
    Substantial,
    Severe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvironmentalImpactLevel {
    None,
    Low,
    Medium,
    High,
    Severe,
}

impl AVMaintenanceSystem {
    pub fn new(config: MaintenanceConfig) -> Self {
        let config_arc = Arc::new(RwLock::new(config));
        let metric_collector = config_arc.read().await.metric_collector.clone();
        
        Self {
            config: config_arc.clone(),
            fleet_health: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            pending_tasks: Arc::new(Mutex::new(VecDeque::new())),
            completed_tasks: Arc::new(Mutex::new(VecDeque::new())),
            predictive_engine: Arc::new(PredictiveMaintenanceEngine::new(config_arc.clone())),
            scheduler: Arc::new(MaintenanceScheduler::new(config_arc.clone())),
            resource_allocator: Arc::new(ResourceAllocator::new(config_arc.clone())),
            treaty_compliance_engine: Arc::new(TreatyComplianceEngine::new(config_arc.clone())),
            workflow_engine: Arc::new(WorkflowEngine::new()),
            metric_collector,
        }
    }

    pub async fn register_vehicle(&self, vehicle_id: VehicleID, initial_health: VehicleHealthRecord) -> Result<()> {
        let mut fleet = self.fleet_health.write().await;
        fleet.insert(vehicle_id.clone(), initial_health);
        
        self.metric_collector.increment_counter(MetricType::VehiclesRegistered, 1).await;
        info!("Vehicle {} registered in maintenance system", vehicle_id);
        
        Ok(())
    }

    pub async fn update_vehicle_health(&self, vehicle_id: &VehicleID, component_updates: HashMap<ComponentType, ComponentHealth>) -> Result<()> {
        let mut fleet = self.fleet_health.write().await;
        let vehicle_record = fleet.get_mut(vehicle_id)
            .context("Vehicle not found in maintenance system")?;
        
        for (component_type, new_health) in component_updates {
            if let Some(component_record) = vehicle_record.component_records.get_mut(&component_type) {
                let previous_health = component_record.current_health.clone();
                component_record.current_health = new_health;
                component_record.updated_at = Timestamp::now();
                
                self.evaluate_health_thresholds(vehicle_id, &component_type, &previous_health, &new_health).await?;
            }
        }
        
        vehicle_record.last_health_update = Timestamp::now();
        vehicle_record.overall_health_score = self.calculate_overall_health(&vehicle_record.component_records).await;
        
        self.metric_collector.record_gauge(MetricType::VehicleHealthScore, vehicle_record.overall_health_score as f64).await;
        
        Ok(())
    }

    async fn evaluate_health_thresholds(&self, vehicle_id: &VehicleID, component: &ComponentType, previous: &ComponentHealth, current: &ComponentHealth) -> Result<()> {
        let config = self.config.read().await;
        if let Some(thresholds) = config.health_thresholds.get(component) {
            let health_score = current.health_score();
            
            if health_score <= thresholds.critical_threshold && previous.health_score() > thresholds.critical_threshold {
                self.create_critical_alert(vehicle_id, component, health_score).await?;
            } else if health_score <= thresholds.warning_threshold && previous.health_score() > thresholds.warning_threshold {
                self.create_warning_alert(vehicle_id, component, health_score).await?;
            }
        }
        
        Ok(())
    }

    async fn create_critical_alert(&self, vehicle_id: &VehicleID, component: &ComponentType, health_score: f32) -> Result<()> {
        let alert = MaintenanceAlert {
            alert_id: format!("alert-{}-{}-{}", vehicle_id, component, Timestamp::now().timestamp),
            timestamp: Timestamp::now(),
            component: component.clone(),
            severity: EventSeverity::Critical,
            description: format!("Component {} health critical: {:.2}", component, health_score),
            recommended_action: "Schedule immediate maintenance".to_string(),
            urgency: AlertUrgency::Immediate,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };
        
        let mut fleet = self.fleet_health.write().await;
        if let Some(vehicle) = fleet.get_mut(vehicle_id) {
            vehicle.active_alerts.push(alert);
            self.metric_collector.increment_counter(MetricType::CriticalAlertsGenerated, 1).await;
        }
        
        Ok(())
    }

    async fn create_warning_alert(&self, vehicle_id: &VehicleID, component: &ComponentType, health_score: f32) -> Result<()> {
        let alert = MaintenanceAlert {
            alert_id: format!("alert-{}-{}-{}", vehicle_id, component, Timestamp::now().timestamp),
            timestamp: Timestamp::now(),
            component: component.clone(),
            severity: EventSeverity::High,
            description: format!("Component {} health degraded: {:.2}", component, health_score),
            recommended_action: "Schedule maintenance within 7 days".to_string(),
            urgency: AlertUrgency::High,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };
        
        let mut fleet = self.fleet_health.write().await;
        if let Some(vehicle) = fleet.get_mut(vehicle_id) {
            vehicle.active_alerts.push(alert);
            self.metric_collector.increment_counter(MetricType::WarningAlertsGenerated, 1).await;
        }
        
        Ok(())
    }

    async fn calculate_overall_health(&self, components: &HashMap<ComponentType, ComponentHealthRecord>) -> f32 {
        if components.is_empty() {
            return 1.0;
        }
        
        let total_score: f32 = components.values().map(|r| r.current_health.health_score()).sum();
        total_score / components.len() as f32
    }

    pub async fn predict_maintenance_needs(&self, vehicle_id: &VehicleID) -> Result<Vec<PredictedMaintenanceNeed>> {
        let fleet = self.fleet_health.read().await;
        let vehicle = fleet.get(vehicle_id)
            .context("Vehicle not found")?;
        
        let mut predictions = Vec::new();
        
        for (component_type, component_record) in &vehicle.component_records {
            if let Some(predicted_time) = component_record.predicted_failure_time {
                let confidence = self.predictive_engine.calculate_confidence(component_type, &component_record).await?;
                
                if confidence >= 0.7 {
                    predictions.push(PredictedMaintenanceNeed {
                        component: component_type.clone(),
                        predicted_failure_time: predicted_time,
                        confidence,
                        recommended_maintenance_window: self.calculate_maintenance_window(&predicted_time).await?,
                        estimated_impact_if_delayed: self.assess_delay_impact(component_type, &predicted_time).await?,
                        maintenance_type: MaintenanceType::Predictive,
                    });
                }
            }
        }
        
        predictions.sort_by_key(|p| p.predicted_failure_time.timestamp);
        
        Ok(predictions)
    }

    async fn calculate_maintenance_window(&self, predicted_failure: &Timestamp) -> Result<TimeWindow> {
        let now = Timestamp::now();
        let window_start = if predicted_failure.timestamp > now.timestamp + 86400 {
            Timestamp { timestamp: now.timestamp + 3600 }
        } else {
            now
        };
        
        let window_end = Timestamp { timestamp: predicted_failure.timestamp - 3600 };
        
        Ok(TimeWindow {
            start: window_start,
            end: window_end,
        })
    }

    async fn assess_delay_impact(&self, component: &ComponentType, predicted_time: &Timestamp) -> Result<ImpactAssessment> {
        let safety_impact = match component {
            ComponentType::BrakingSystem | ComponentType::SteeringSystem | ComponentType::CollisionAvoidance => SafetyImpactLevel::Critical,
            ComponentType::PropulsionSystem | ComponentType::PowerDistribution => SafetyImpactLevel::Significant,
            ComponentType::SensorArray | ComponentType::CommunicationSystem => SafetyImpactLevel::Moderate,
            _ => SafetyImpactLevel::Minor,
        };
        
        let time_until_failure = predicted_time.timestamp - Timestamp::now().timestamp;
        let operational_impact = if time_until_failure < 3600 {
            OperationalImpactLevel::Severe
        } else if time_until_failure < 86400 {
            OperationalImpactLevel::Substantial
        } else if time_until_failure < 604800 {
            OperationalImpactLevel::Moderate
        } else {
            OperationalImpactLevel::Minimal
        };
        
        Ok(ImpactAssessment {
            safety_impact,
            operational_impact,
            financial_impact: self.estimate_financial_impact(component).await?,
            environmental_impact: EnvironmentalImpactLevel::Low,
        })
    }

    async fn estimate_financial_impact(&self, component: &ComponentType) -> Result<f64> {
        let base_cost = match component {
            ComponentType::BatterySystem => 15000.0,
            ComponentType::PropulsionSystem => 25000.0,
            ComponentType::BrakingSystem => 8000.0,
            ComponentType::SteeringSystem => 6000.0,
            ComponentType::SensorArray => 12000.0,
            ComponentType::CommunicationSystem => 5000.0,
            ComponentType::PowerDistribution => 7000.0,
            ComponentType::CollisionAvoidance => 18000.0,
            ComponentType::ClimateControl => 3000.0,
            ComponentType::LightingSystem => 2000.0,
            ComponentType::StructuralFrame => 30000.0,
            _ => 1000.0,
        };
        
        Ok(base_cost * 1.2)
    }

    pub async fn generate_maintenance_task(&self, vehicle_id: &VehicleID, component: ComponentType, maintenance_type: MaintenanceType) -> Result<MaintenanceTask> {
        let fleet = self.fleet_health.read().await;
        let vehicle = fleet.get(vehicle_id)
            .context("Vehicle not found")?;
        
        let config = self.config.read().await;
        let estimated_duration = config.maintenance_intervals.get(&component)
            .copied()
            .unwrap_or(Duration::from_hours(2));
        
        let required_resources = self.determine_required_resources(&component, &maintenance_type).await?;
        let skill_requirements = self.determine_skill_requirements(&component, &maintenance_type).await?;
        let location = self.select_maintenance_location(vehicle_id, &maintenance_type).await?;
        
        let treaty_check = if location.fpic_required {
            self.treaty_compliance_engine.verify_compliance(vehicle_id, &location).await?
        } else {
            TreatyComplianceCheck {
                fpic_verified: true,
                nation_consulted: None,
                protocol_adhered: true,
                cultural_sensitivity_score: 1.0,
                verification_timestamp: Some(Timestamp::now()),
                verifier_id: None,
            }
        };
        
        let task_id = format!("task-{}-{}-{}", vehicle_id, component, Timestamp::now().timestamp);
        
        let task = MaintenanceTask {
            task_id: task_id.clone(),
            vehicle_id: vehicle_id.clone(),
            maintenance_type,
            components: vec![component],
            priority: self.calculate_task_priority(vehicle_id, &component).await?,
            estimated_duration,
            required_resources,
            skill_requirements,
            safety_critical: self.is_safety_critical(&component),
            treaty_sensitive: location.fpic_required,
            scheduled_start: None,
            scheduled_end: None,
            actual_start: None,
            actual_end: None,
            status: MaintenanceStatus::Pending,
            assigned_technicians: Vec::new(),
            location,
            treaty_compliance_check: treaty_check,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
        };
        
        drop(config);
        drop(fleet);
        
        self.pending_tasks.lock().await.push_back(task.clone());
        self.metric_collector.increment_counter(MetricType::MaintenanceTasksCreated, 1).await;
        
        Ok(task)
    }

    async fn determine_required_resources(&self, component: &ComponentType, maintenance_type: &MaintenanceType) -> Result<HashSet<ResourceType>> {
        let mut resources = HashSet::new();
        
        match component {
            ComponentType::BatterySystem => {
                resources.insert(ResourceType::LiftingEquipment);
                resources.insert(ResourceType::DiagnosticEquipment);
                resources.insert(ResourceType::SafetyEquipment);
            }
            ComponentType::PropulsionSystem => {
                resources.insert(ResourceType::LiftingEquipment);
                resources.insert(ResourceType::SpecializedTools);
                resources.insert(ResourceType::TestEquipment);
            }
            ComponentType::BrakingSystem => {
                resources.insert(ResourceType::HydraulicEquipment);
                resources.insert(ResourceType::TestEquipment);
                resources.insert(ResourceType::SafetyEquipment);
            }
            _ => {
                resources.insert(ResourceType::DiagnosticEquipment);
                resources.insert(ResourceType::GeneralTools);
            }
        }
        
        if matches!(maintenance_type, MaintenanceType::Emergency | MaintenanceType::Corrective) {
            resources.insert(ResourceType::EmergencyEquipment);
        }
        
        Ok(resources)
    }

    async fn determine_skill_requirements(&self, component: &ComponentType, maintenance_type: &MaintenanceType) -> Result<HashSet<MaintenanceSkill>> {
        let mut skills = HashSet::new();
        
        match component {
            ComponentType::BatterySystem => {
                skills.insert(MaintenanceSkill::ElectricalSystems);
                skills.insert(MaintenanceSkill::BatteryManagement);
            }
            ComponentType::PropulsionSystem => {
                skills.insert(MaintenanceSkill::MechanicalSystems);
                skills.insert(MaintenanceSkill::HydraulicSystems);
            }
            ComponentType::BrakingSystem => {
                skills.insert(MaintenanceSkill::MechanicalSystems);
                skills.insert(MaintenanceSkill::HydraulicSystems);
            }
            ComponentType::SensorArray => {
                skills.insert(MaintenanceSkill::SensorCalibration);
                skills.insert(MaintenanceSkill::SoftwareDiagnostics);
            }
            ComponentType::CommunicationSystem => {
                skills.insert(MaintenanceSkill::ElectricalSystems);
                skills.insert(MaintenanceSkill::SoftwareDiagnostics);
            }
            _ => {
                skills.insert(MaintenanceSkill::MechanicalSystems);
            }
        }
        
        if self.is_safety_critical(component) {
            skills.insert(MaintenanceSkill::SafetyCertification);
        }
        
        Ok(skills)
    }

    async fn select_maintenance_location(&self, vehicle_id: &VehicleID, maintenance_type: &MaintenanceType) -> Result<MaintenanceLocation> {
        let config = self.config.read().await;
        let fleet = self.fleet_health.read().await;
        
        let vehicle = fleet.get(vehicle_id)
            .context("Vehicle not found")?;
        
        let mut suitable_facilities: Vec<&MaintenanceLocation> = config.facility_inventory.values()
            .filter(|facility| {
                match maintenance_type {
                    MaintenanceType::Emergency => matches!(facility.facility_type, FacilityType::EmergencyResponse | FacilityType::MobileUnit),
                    MaintenanceType::Corrective => matches!(facility.facility_type, FacilityType::ServiceCenter | FacilityType::FieldRepair),
                    MaintenanceType::Preventive | MaintenanceType::Predictive => matches!(facility.facility_type, FacilityType::ServiceCenter | FacilityType::InspectionStation),
                    MaintenanceType::Recertification => matches!(facility.facility_type, FacilityType::InspectionStation | FacilityType::CalibrationLab),
                    MaintenanceType::SoftwareUpdate => matches!(facility.facility_type, FacilityType::SoftwareDepot | FacilityType::ServiceCenter),
                    MaintenanceType::Calibration => matches!(facility.facility_type, FacilityType::CalibrationLab),
                    MaintenanceType::Inspection => matches!(facility.facility_type, FacilityType::InspectionStation),
                }
            })
            .filter(|facility| facility.current_utilization < facility.capacity)
            .collect();
        
        suitable_facilities.sort_by(|a, b| {
            let dist_a = self.calculate_distance(&vehicle.current_location, &a.coordinates);
            let dist_b = self.calculate_distance(&vehicle.current_location, &b.coordinates);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        suitable_facilities.get(0)
            .map(|loc| (*loc).clone())
            .context("No suitable maintenance facility available")
    }

    fn calculate_distance(&self, coord1: &GeoCoordinate, coord2: &GeoCoordinate) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        
        let lat1 = coord1.latitude.to_radians();
        let lon1 = coord1.longitude.to_radians();
        let lat2 = coord2.latitude.to_radians();
        let lon2 = coord2.longitude.to_radians();
        
        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;
        
        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        EARTH_RADIUS_KM * c
    }

    async fn calculate_task_priority(&self, vehicle_id: &VehicleID, component: &ComponentType) -> Result<TaskPriority> {
        let fleet = self.fleet_health.read().await;
        let vehicle = fleet.get(vehicle_id)
            .context("Vehicle not found")?;
        
        let component_record = vehicle.component_records.get(component)
            .context("Component not found")?;
        
        let health_score = component_record.current_health.health_score();
        let time_since_last_maintenance = Timestamp::now().timestamp - component_record.last_inspection.unwrap_or_default().timestamp;
        
        let priority_score = if health_score < 0.3 {
            100.0
        } else if health_score < 0.5 {
            75.0
        } else if health_score < 0.7 {
            50.0
        } else if time_since_last_maintenance > 2592000 {
            40.0
        } else {
            25.0
        };
        
        Ok(if priority_score >= 75.0 {
            TaskPriority::Critical
        } else if priority_score >= 50.0 {
            TaskPriority::High
        } else if priority_score >= 30.0 {
            TaskPriority::Medium
        } else {
            TaskPriority::Low
        })
    }

    fn is_safety_critical(&self, component: &ComponentType) -> bool {
        matches!(component, 
            ComponentType::BrakingSystem | 
            ComponentType::SteeringSystem | 
            ComponentType::CollisionAvoidance | 
            ComponentType::PropulsionSystem
        )
    }

    pub async fn schedule_maintenance(&self, task_id: &str) -> Result<MaintenanceSchedule> {
        let mut pending = self.pending_tasks.lock().await;
        let task = pending.iter().find(|t| t.task_id == task_id)
            .context("Task not found in pending queue")?
            .clone();
        
        let schedule = self.scheduler.create_schedule(&task).await?;
        
        pending.retain(|t| t.task_id != task_id);
        self.active_tasks.lock().await.insert(task_id.to_string(), task);
        
        self.metric_collector.increment_counter(MetricType::MaintenanceTasksScheduled, 1).await;
        
        Ok(schedule)
    }

    pub async fn start_maintenance_task(&self, task_id: &str, technicians: Vec<DID>) -> Result<()> {
        let mut active = self.active_tasks.lock().await;
        let task = active.get_mut(task_id)
            .context("Task not found")?;
        
        if task.status != MaintenanceStatus::Scheduled {
            anyhow::bail!("Task must be scheduled before starting");
        }
        
        task.status = MaintenanceStatus::InProgress;
        task.actual_start = Some(Timestamp::now());
        task.assigned_technicians = technicians;
        task.updated_at = Timestamp::now();
        
        self.workflow_engine.start_workflow(task_id).await?;
        
        self.metric_collector.increment_counter(MetricType::MaintenanceTasksStarted, 1).await;
        
        Ok(())
    }

    pub async fn complete_maintenance_task(&self, task_id: &str, completion_notes: String) -> Result<()> {
        let mut active = self.active_tasks.lock().await;
        let task = active.remove(task_id)
            .context("Task not found")?;
        
        let mut completed = task.clone();
        completed.status = MaintenanceStatus::Completed;
        completed.actual_end = Some(Timestamp::now());
        completed.updated_at = Timestamp::now();
        
        self.completed_tasks.lock().await.push_back(completed);
        
        self.update_vehicle_maintenance_history(&task.vehicle_id, &task).await?;
        self.workflow_engine.complete_workflow(task_id).await?;
        
        self.metric_collector.increment_counter(MetricType::MaintenanceTasksCompleted, 1).await;
        self.metric_collector.record_duration(MetricType::MaintenanceTaskDuration, task.actual_end.unwrap().elapsed()).await;
        
        Ok(())
    }

    async fn update_vehicle_maintenance_history(&self, vehicle_id: &VehicleID, task: &MaintenanceTask) -> Result<()> {
        let mut fleet = self.fleet_health.write().await;
        let vehicle = fleet.get_mut(vehicle_id)
            .context("Vehicle not found")?;
        
        for component in &task.components {
            let event = MaintenanceEvent {
                event_id: format!("event-{}-{}", task.task_id, component),
                timestamp: Timestamp::now(),
                event_type: match task.maintenance_type {
                    MaintenanceType::Preventive => MaintenanceEventType::Inspection,
                    MaintenanceType::Predictive => MaintenanceEventType::Repair,
                    MaintenanceType::Corrective => MaintenanceEventType::Replacement,
                    MaintenanceType::Emergency => MaintenanceEventType::Repair,
                    MaintenanceType::Recertification => MaintenanceEventType::Recertification,
                    MaintenanceType::SoftwareUpdate => MaintenanceEventType::SoftwareUpdate,
                    MaintenanceType::Calibration => MaintenanceEventType::Calibration,
                    MaintenanceType::Inspection => MaintenanceEventType::Inspection,
                },
                component_type: component.clone(),
                severity: EventSeverity::Informational,
                description: format!("Maintenance completed: {:?}", task.maintenance_type),
                corrective_action: Some(completion_notes.clone()),
                technician_id: task.assigned_technicians.get(0).cloned(),
                parts_replaced: Vec::new(),
                labor_hours: task.estimated_duration.as_secs() as f32 / 3600.0,
                cost_estimate: None,
            };
            
            vehicle.maintenance_history.push(event);
            
            if let Some(component_record) = vehicle.component_records.get_mut(component) {
                component_record.last_inspection = Some(Timestamp::now());
                component_record.next_inspection_due = Timestamp { timestamp: Timestamp::now().timestamp + 2592000 };
                component_record.current_health = ComponentHealth::Optimal;
                component_record.updated_at = Timestamp::now();
            }
        }
        
        vehicle.overall_health_score = self.calculate_overall_health(&vehicle.component_records).await;
        
        Ok(())
    }

    pub async fn get_fleet_health_snapshot(&self) -> Result<FleetHealthSnapshot> {
        let fleet = self.fleet_health.read().await;
        
        let total_vehicles = fleet.len();
        let mut operational = 0;
        let mut under_maintenance = 0;
        let mut offline = 0;
        let mut critical_alerts = 0;
        let mut warning_alerts = 0;
        let mut informational_alerts = 0;
        
        let mut component_summary: HashMap<ComponentType, ComponentHealthSummary> = HashMap::new();
        
        for (_, vehicle) in fleet.iter() {
            match vehicle.operational_status {
                VehicleOperationalStatus::Operational => operational += 1,
                VehicleOperationalStatus::LimitedOperation | VehicleOperationalStatus::MaintenanceRequired => under_maintenance += 1,
                VehicleOperationalStatus::OfflineForMaintenance | VehicleOperationalStatus::SafetyHold | VehicleOperationalStatus::Decommissioned => offline += 1,
            }
            
            for alert in &vehicle.active_alerts {
                match alert.severity {
                    EventSeverity::Critical => critical_alerts += 1,
                    EventSeverity::High | EventSeverity::Medium => warning_alerts += 1,
                    EventSeverity::Low | EventSeverity::Informational => informational_alerts += 1,
                }
            }
            
            for (component_type, record) in &vehicle.component_records {
                let summary = component_summary.entry(component_type.clone())
                    .or_insert_with(|| ComponentHealthSummary {
                        total_components: 0,
                        healthy: 0,
                        degraded: 0,
                        critical: 0,
                        failed: 0,
                        average_health_score: 0.0,
                        predicted_failures_next_30d: 0,
                    });
                
                let health_score = record.current_health.health_score();
                summary.total_components += 1;
                
                if health_score >= 0.8 {
                    summary.healthy += 1;
                } else if health_score >= 0.5 {
                    summary.degraded += 1;
                } else if health_score >= 0.2 {
                    summary.critical += 1;
                } else {
                    summary.failed += 1;
                }
                
                summary.average_health_score += health_score;
                
                if let Some(predicted_time) = record.predicted_failure_time {
                    let days_until_failure = (predicted_time.timestamp - Timestamp::now().timestamp) / 86400;
                    if days_until_failure <= 30 {
                        summary.predicted_failures_next_30d += 1;
                    }
                }
            }
        }
        
        for summary in component_summary.values_mut() {
            if summary.total_components > 0 {
                summary.average_health_score /= summary.total_components as f32;
            }
        }
        
        let average_fleet_health = if total_vehicles > 0 {
            fleet.values().map(|v| v.overall_health_score).sum::<f32>() / total_vehicles as f32
        } else {
            1.0
        };
        
        let maintenance_backlog = self.pending_tasks.lock().await.len();
        let estimated_downtime_hours = self.estimate_downtime().await?;
        
        Ok(FleetHealthSnapshot {
            snapshot_id: format!("snapshot-{}", Timestamp::now().timestamp),
            timestamp: Timestamp::now(),
            total_vehicles,
            vehicles_operational: operational,
            vehicles_under_maintenance: under_maintenance,
            vehicles_offline: offline,
            critical_alerts,
            warning_alerts,
            informational_alerts,
            average_fleet_health,
            component_health_summary: component_summary,
            maintenance_backlog,
            estimated_downtime_hours,
        })
    }

    async fn estimate_downtime(&self) -> Result<f32> {
        let pending = self.pending_tasks.lock().await;
        let active = self.active_tasks.lock().await;
        
        let pending_hours: f32 = pending.iter().map(|t| t.estimated_duration.as_secs() as f32 / 3600.0).sum();
        let active_hours: f32 = active.values().map(|t| {
            let elapsed = t.actual_start.map(|s| s.elapsed().as_secs() as f32 / 3600.0).unwrap_or(0.0);
            let remaining = t.estimated_duration.as_secs() as f32 / 3600.0 - elapsed;
            remaining.max(0.0)
        }).sum();
        
        Ok(pending_hours + active_hours)
    }

    pub async fn acknowledge_alert(&self, vehicle_id: &VehicleID, alert_id: &str, technician_id: DID) -> Result<()> {
        let mut fleet = self.fleet_health.write().await;
        let vehicle = fleet.get_mut(vehicle_id)
            .context("Vehicle not found")?;
        
        if let Some(alert) = vehicle.active_alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(technician_id);
            alert.acknowledged_at = Some(Timestamp::now());
            
            self.metric_collector.increment_counter(MetricType::AlertsAcknowledged, 1).await;
        } else {
            anyhow::bail!("Alert not found");
        }
        
        Ok(())
    }

    pub async fn get_vehicle_maintenance_history(&self, vehicle_id: &VehicleID) -> Result<Vec<MaintenanceEvent>> {
        let fleet = self.fleet_health.read().await;
        let vehicle = fleet.get(vehicle_id)
            .context("Vehicle not found")?;
        
        Ok(vehicle.maintenance_history.clone())
    }

    pub async fn optimize_maintenance_schedule(&self) -> Result<Vec<MaintenanceSchedule>> {
        let pending = self.pending_tasks.lock().await;
        let mut tasks: Vec<MaintenanceTask> = pending.iter().cloned().collect();
        
        tasks.sort_by_key(|t| t.priority as i32 * -1);
        
        let mut schedules = Vec::new();
        for task in tasks {
            let schedule = self.scheduler.create_schedule(&task).await?;
            schedules.push(schedule);
        }
        
        Ok(schedules)
    }
}

pub struct PredictiveMaintenanceEngine {
    config: Arc<RwLock<MaintenanceConfig>>,
    anomaly_detector: Arc<AnomalyDetector>,
    failure_predictor: Arc<FailurePredictor>,
    degradation_tracker: Arc<DegradationTracker>,
}

impl PredictiveMaintenanceEngine {
    pub fn new(config: Arc<RwLock<MaintenanceConfig>>) -> Self {
        Self {
            config,
            anomaly_detector: Arc::new(AnomalyDetector::new()),
            failure_predictor: Arc::new(FailurePredictor::new()),
            degradation_tracker: Arc::new(DegradationTracker::new()),
        }
    }

    pub async fn calculate_confidence(&self, component: &ComponentType, record: &ComponentHealthRecord) -> Result<f32> {
        let anomaly_confidence = self.anomaly_detector.calculate_confidence(&record.sensor_readings).await?;
        let failure_confidence = self.failure_predictor.calculate_confidence(record).await?;
        let degradation_confidence = self.degradation_tracker.calculate_confidence(record).await?;
        
        Ok((anomaly_confidence + failure_confidence + degradation_confidence) / 3.0)
    }
}

pub struct AnomalyDetector;
impl AnomalyDetector {
    pub fn new() -> Self { Self }
    pub async fn calculate_confidence(&self, readings: &[SensorReading]) -> Result<f32> {
        if readings.is_empty() {
            return Ok(0.0);
        }
        
        let anomaly_scores: Vec<f32> = readings.iter().map(|r| r.anomaly_score).collect();
        let avg_anomaly = anomaly_scores.iter().sum::<f32>() / anomaly_scores.len() as f32;
        
        Ok((avg_anomaly * 100.0).min(100.0) / 100.0)
    }
}

pub struct FailurePredictor;
impl FailurePredictor {
    pub fn new() -> Self { Self }
    pub async fn calculate_confidence(&self, record: &ComponentHealthRecord) -> Result<f32> {
        let health_score = record.current_health.health_score();
        let degradation = record.degradation_rate;
        
        let confidence = if health_score < 0.2 {
            0.95
        } else if health_score < 0.4 {
            0.85
        } else if health_score < 0.6 {
            0.70
        } else if degradation > 0.05 {
            0.60
        } else {
            0.40
        };
        
        Ok(confidence)
    }
}

pub struct DegradationTracker;
impl DegradationTracker {
    pub fn new() -> Self { Self }
    pub async fn calculate_confidence(&self, record: &ComponentHealthRecord) -> Result<f32> {
        let degradation = record.degradation_rate;
        
        Ok(degradation.min(1.0))
    }
}

pub struct MaintenanceScheduler {
    config: Arc<RwLock<MaintenanceConfig>>,
    resource_optimizer: Arc<ResourceOptimizer>,
    treaty_coordinator: Arc<TreatyCoordinator>,
}

impl MaintenanceScheduler {
    pub fn new(config: Arc<RwLock<MaintenanceConfig>>) -> Self {
        Self {
            config,
            resource_optimizer: Arc::new(ResourceOptimizer::new()),
            treaty_coordinator: Arc::new(TreatyCoordinator::new()),
        }
    }

    pub async fn create_schedule(&self, task: &MaintenanceTask) -> Result<MaintenanceSchedule> {
        let resource_plan = self.resource_optimizer.allocate_resources(task).await?;
        let treaty_status = self.treaty_coordinator.verify_compliance(task).await?;
        let safety_verification = self.create_safety_verification(task).await?;
        
        Ok(MaintenanceSchedule {
            schedule_id: format!("schedule-{}", task.task_id),
            vehicle_id: task.vehicle_id.clone(),
            tasks: vec![task.clone()],
            total_estimated_duration: task.estimated_duration,
            resource_requirements: resource_plan,
            treaty_compliance_status: treaty_status,
            safety_verification,
            created_at: Timestamp::now(),
            valid_until: Timestamp { timestamp: Timestamp::now().timestamp + 86400 },
        })
    }

    async fn create_safety_verification(&self, task: &MaintenanceTask) -> Result<SafetyVerification> {
        let config = self.config.read().await;
        let safety_checks = config.safety_checklists.get(&task.maintenance_type)
            .map(|checks| checks.iter().map(|&c| SafetyCheck {
                check_id: format!("check-{}-{}", task.task_id, c as i32),
                check_type: c,
                component: task.components.get(0).cloned().unwrap_or(ComponentType::Unknown),
                pass_threshold: 0.9,
                actual_value: None,
                passed: false,
                notes: None,
            }).collect())
            .unwrap_or_default();
        
        Ok(SafetyVerification {
            pre_maintenance_checks: safety_checks.clone(),
            post_maintenance_validation: safety_checks,
            safety_critical_components: task.components.iter()
                .filter(|&&c| self.is_safety_critical(&c))
                .cloned()
                .collect(),
            recertification_required: task.maintenance_type == MaintenanceType::Recertification,
            safety_officer_approval: None,
            verification_timestamp: None,
        })
    }

    fn is_safety_critical(&self, component: &ComponentType) -> bool {
        matches!(component, 
            ComponentType::BrakingSystem | 
            ComponentType::SteeringSystem | 
            ComponentType::CollisionAvoidance
        )
    }
}

pub struct ResourceOptimizer;
impl ResourceOptimizer {
    pub fn new() -> Self { Self }
    pub async fn allocate_resources(&self, task: &MaintenanceTask) -> Result<ResourceAllocationPlan> {
        Ok(ResourceAllocationPlan {
            required_technicians: task.skill_requirements.len().max(1),
            required_skills: task.skill_requirements.clone(),
            equipment_needed: task.required_resources.clone(),
            parts_inventory: HashMap::new(),
            facility_requirements: vec![task.location.clone()],
            estimated_cost: 500.0 * task.estimated_duration.as_secs() as f64 / 3600.0,
            priority_adjustment: match task.priority {
                TaskPriority::Critical => 1.5,
                TaskPriority::High => 1.2,
                TaskPriority::Medium => 1.0,
                TaskPriority::Low => 0.8,
                _ => 1.0,
            },
        })
    }
}

pub struct TreatyCoordinator;
impl TreatyCoordinator {
    pub fn new() -> Self { Self }
    pub async fn verify_compliance(&self, task: &MaintenanceTask) -> Result<TreatyComplianceStatus> {
        Ok(TreatyComplianceStatus {
            fpic_status: if task.treaty_sensitive {
                FPICStatus::Pending
            } else {
                FPICStatus::NotRequired
            },
            nations_affected: Vec::new(),
            protocols_required: Vec::new(),
            consultation_completed: !task.treaty_sensitive,
            cultural_liaison_assigned: None,
            compliance_score: if task.treaty_sensitive { 0.0 } else { 1.0 },
        })
    }
}

pub struct ResourceAllocator {
    config: Arc<RwLock<MaintenanceConfig>>,
}

impl ResourceAllocator {
    pub fn new(config: Arc<RwLock<MaintenanceConfig>>) -> Self {
        Self { config }
    }
}

pub struct TreatyComplianceEngine {
    config: Arc<RwLock<MaintenanceConfig>>,
}

impl TreatyComplianceEngine {
    pub fn new(config: Arc<RwLock<MaintenanceConfig>>) -> Self {
        Self { config }
    }

    pub async fn verify_compliance(&self, vehicle_id: &VehicleID, location: &MaintenanceLocation) -> Result<TreatyComplianceCheck> {
        if let Some(nation) = &location.treaty_jurisdiction {
            Ok(TreatyComplianceCheck {
                fpic_verified: false,
                nation_consulted: Some(nation.clone()),
                protocol_adhered: false,
                cultural_sensitivity_score: 0.0,
                verification_timestamp: None,
                verifier_id: None,
            })
        } else {
            Ok(TreatyComplianceCheck {
                fpic_verified: true,
                nation_consulted: None,
                protocol_adhered: true,
                cultural_sensitivity_score: 1.0,
                verification_timestamp: Some(Timestamp::now()),
                verifier_id: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aletheion_core::av::safety::ComponentHealth;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_vehicle_registration() -> Result<()> {
        let config = create_test_config();
        let system = AVMaintenanceSystem::new(config);
        
        let vehicle_id = VehicleID::new("vehicle-001");
        let health_record = create_test_health_record(&vehicle_id);
        
        system.register_vehicle(vehicle_id.clone(), health_record).await?;
        
        let snapshot = system.get_fleet_health_snapshot().await?;
        assert_eq!(snapshot.total_vehicles, 1);
        assert_eq!(snapshot.vehicles_operational, 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_health_update_and_alert_generation() -> Result<()> {
        let config = create_test_config();
        let system = AVMaintenanceSystem::new(config);
        
        let vehicle_id = VehicleID::new("vehicle-002");
        let mut health_record = create_test_health_record(&vehicle_id);
        
        system.register_vehicle(vehicle_id.clone(), health_record.clone()).await?;
        
        let mut updates = HashMap::new();
        updates.insert(ComponentType::BrakingSystem, ComponentHealth::Degraded(0.25));
        
        system.update_vehicle_health(&vehicle_id, updates).await?;
        
        let snapshot = system.get_fleet_health_snapshot().await?;
        assert!(snapshot.warning_alerts > 0 || snapshot.critical_alerts > 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_maintenance_task_generation() -> Result<()> {
        let config = create_test_config();
        let system = AVMaintenanceSystem::new(config);
        
        let vehicle_id = VehicleID::new("vehicle-003");
        system.register_vehicle(vehicle_id.clone(), create_test_health_record(&vehicle_id)).await?;
        
        let task = system.generate_maintenance_task(&vehicle_id, ComponentType::BrakingSystem, MaintenanceType::Predictive).await?;
        
        assert_eq!(task.vehicle_id, vehicle_id);
        assert_eq!(task.maintenance_type, MaintenanceType::Predictive);
        assert_eq!(task.components.len(), 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_maintenance_scheduling() -> Result<()> {
        let config = create_test_config();
        let system = AVMaintenanceSystem::new(config);
        
        let vehicle_id = VehicleID::new("vehicle-004");
        system.register_vehicle(vehicle_id.clone(), create_test_health_record(&vehicle_id)).await?;
        
        let task = system.generate_maintenance_task(&vehicle_id, ComponentType::BatterySystem, MaintenanceType::Preventive).await?;
        let schedule = system.schedule_maintenance(&task.task_id).await?;
        
        assert_eq!(schedule.vehicle_id, vehicle_id);
        assert!(schedule.treaty_compliance_status.compliance_score >= 0.0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_task_completion_workflow() -> Result<()> {
        let config = create_test_config();
        let system = AVMaintenanceSystem::new(config);
        
        let vehicle_id = VehicleID::new("vehicle-005");
        system.register_vehicle(vehicle_id.clone(), create_test_health_record(&vehicle_id)).await?;
        
        let task = system.generate_maintenance_task(&vehicle_id, ComponentType::SensorArray, MaintenanceType::Corrective).await?;
        system.schedule_maintenance(&task.task_id).await?;
        
        let technicians = vec![DID::new("technician-001")];
        system.start_maintenance_task(&task.task_id, technicians).await?;
        
        system.complete_maintenance_task(&task.task_id, "All repairs completed successfully".to_string()).await?;
        
        let history = system.get_vehicle_maintenance_history(&vehicle_id).await?;
        assert!(!history.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_fleet_health_snapshot() -> Result<()> {
        let config = create_test_config();
        let system = AVMaintenanceSystem::new(config);
        
        for i in 0..5 {
            let vehicle_id = VehicleID::new(&format!("vehicle-test-{}", i));
            system.register_vehicle(vehicle_id.clone(), create_test_health_record(&vehicle_id)).await?;
        }
        
        let snapshot = system.get_fleet_health_snapshot().await?;
        
        assert_eq!(snapshot.total_vehicles, 5);
        assert_eq!(snapshot.vehicles_operational, 5);
        assert_eq!(snapshot.average_fleet_health, 1.0);
        
        Ok(())
    }

    fn create_test_config() -> MaintenanceConfig {
        let mut health_thresholds = HashMap::new();
        health_thresholds.insert(ComponentType::BrakingSystem, HealthThresholds {
            warning_threshold: 0.7,
            critical_threshold: 0.4,
            failure_threshold: 0.2,
            hysteresis: 0.05,
            sampling_frequency_hz: 10,
        });
        
        let mut maintenance_intervals = HashMap::new();
        maintenance_intervals.insert(ComponentType::BrakingSystem, Duration::from_hours(2));
        maintenance_intervals.insert(ComponentType::BatterySystem, Duration::from_hours(4));
        
        let mut facility_inventory = HashMap::new();
        facility_inventory.insert("facility-001".to_string(), MaintenanceLocation {
            facility_id: "facility-001".to_string(),
            coordinates: GeoCoordinate { latitude: 33.4484, longitude: -112.0740, altitude_m: None },
            facility_type: FacilityType::ServiceCenter,
            capacity: 10,
            current_utilization: 0,
            treaty_jurisdiction: None,
            fpic_required: false,
        });
        
        MaintenanceConfig {
            predictive_models: HashMap::new(),
            health_thresholds,
            maintenance_intervals,
            resource_pools: HashMap::new(),
            facility_inventory,
            technician_skills: HashMap::new(),
            treaty_protocols: HashMap::new(),
            safety_checklists: HashMap::new(),
            workflow_templates: HashMap::new(),
            metric_collector: Arc::new(MetricCollector::new()),
            update_interval_ms: 1000,
        }
    }

    fn create_test_health_record(vehicle_id: &VehicleID) -> VehicleHealthRecord {
        let mut component_records = HashMap::new();
        
        for component in vec![ComponentType::BrakingSystem, ComponentType::BatterySystem, ComponentType::SensorArray] {
            component_records.insert(component, ComponentHealthRecord {
                component_type: component,
                component_id: format!("{}-{}", component, vehicle_id),
                vehicle_id: vehicle_id.clone(),
                current_health: ComponentHealth::Optimal,
                baseline_health: ComponentHealth::Optimal,
                degradation_rate: 0.0,
                predicted_failure_time: None,
                maintenance_history: Vec::new(),
                sensor_readings: VecDeque::new(),
                anomaly_detected: false,
                anomaly_details: None,
                last_inspection: None,
                next_inspection_due: Timestamp { timestamp: Timestamp::now().timestamp + 2592000 },
                warranty_status: WarrantyStatus::Active,
                lifecycle_stage: LifecycleStage::New,
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
            });
        }
        
        VehicleHealthRecord {
            vehicle_id: vehicle_id.clone(),
            last_health_update: Timestamp::now(),
            overall_health_score: 1.0,
            component_records,
            active_alerts: Vec::new(),
            predicted_maintenance_needs: Vec::new(),
            maintenance_history: Vec::new(),
            current_location: GeoCoordinate { latitude: 33.4484, longitude: -112.0740, altitude_m: None },
            operational_status: VehicleOperationalStatus::Operational,
            treaty_jurisdiction: None,
        }
    }
}
