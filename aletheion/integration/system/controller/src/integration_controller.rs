#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const INTEGRATION_CONTROLLER_VERSION: u32 = 20260310;
pub const MAX_SUBSYSTEMS: usize = 64;
pub const MAX_INTEGRATION_POINTS: usize = 4096;
pub const MAX_DATA_PIPELINES: usize = 2048;
pub const HEALTH_CHECK_INTERVAL_S: u64 = 60;
pub const SYNC_TOLERANCE_MS: u64 = 100;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SubsystemType {
    Energy = 0, Water = 1, Waste = 2, Transportation = 3,
    Housing = 4, Healthcare = 5, Safety = 6, Communications = 7,
    Finance = 8, Legal = 9, Cultural = 10, Agriculture = 11,
    Environment = 12, Education = 13, CitizenServices = 14, Governance = 15,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IntegrationStatus {
    Offline = 0, Connecting = 1, Online = 2, Degraded = 3,
    Fault = 4, Maintenance = 5, Syncing = 6, Mismatch = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct SubsystemHealth {
    pub subsystem_type: SubsystemType,
    pub status: IntegrationStatus,
    pub uptime_s: u64,
    pub last_heartbeat_ns: u64,
    pub error_count: u32,
    pub warning_count: u32,
    pub data_latency_ms: u64,
    pub sync_offset_ms: u64,
    pub health_score: f64,
}

impl SubsystemHealth {
    pub fn is_healthy(&self, now_ns: u64) -> bool {
        self.status == IntegrationStatus::Online &&
        self.health_score > 0.7 &&
        now_ns - self.last_heartbeat_ns < HEALTH_CHECK_INTERVAL_S * 1000000000 &&
        self.data_latency_ms < 500 &&
        self.sync_offset_ms < SYNC_TOLERANCE_MS
    }
    pub fn requires_attention(&self) -> bool {
        self.status == IntegrationStatus::Fault ||
        self.status == IntegrationStatus::Degraded ||
        self.health_score < 0.5 ||
        self.error_count > 10
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IntegrationPoint {
    pub point_id: u64,
    pub source_subsystem: SubsystemType,
    pub destination_subsystem: SubsystemType,
    pub data_type: u32,
    pub protocol: u8,
    pub bandwidth_mbps: f64,
    pub current_throughput_mbps: f64,
    pub latency_ms: u64,
    pub packet_loss_pct: f64,
    pub encryption_enabled: bool,
    pub compression_enabled: bool,
    pub operational: bool,
    pub last_test_ns: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct DataPipeline {
    pub pipeline_id: u64,
    pub name: [u8; 64],
    pub source_subsystem: SubsystemType,
    pub destination_subsystem: SubsystemType,
    pub priority: u8,
    pub max_throughput_mbps: f64,
    pub current_throughput_mbps: f64,
    pub buffer_size_kb: u32,
    pub buffer_used_kb: u32,
    pub messages_processed: u64,
    pub messages_dropped: u64,
    pub last_message_ns: u64,
    pub operational: bool,
}

pub struct SystemIntegrationController {
    pub controller_id: u64,
    pub city_code: [u8; 8],
    pub subsystems: [Option<SubsystemHealth>; MAX_SUBSYSTEMS],
    pub subsystem_count: usize,
    pub integration_points: [Option<IntegrationPoint>; MAX_INTEGRATION_POINTS],
    pub point_count: usize,
    pub data_pipelines: [Option<DataPipeline>; MAX_DATA_PIPELINES],
    pub pipeline_count: usize,
    pub total_messages_processed: u64,
    pub total_messages_dropped: u64,
    pub average_latency_ms: f64,
    pub system_sync_score: f64,
    pub integration_health_index: f64,
    pub cascade_failure_risk: f64,
    pub last_full_sync_ns: u64,
    pub audit_checksum: u64,
}

impl SystemIntegrationController {
    pub fn new(controller_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            controller_id,
            city_code,
            subsystems: Default::default(),
            subsystem_count: 0,
            integration_points: Default::default(),
            point_count: 0,
            data_pipelines: Default::default(),
            pipeline_count: 0,
            total_messages_processed: 0,
            total_messages_dropped: 0,
            average_latency_ms: 0.0,
            system_sync_score: 1.0,
            integration_health_index: 1.0,
            cascade_failure_risk: 0.0,
            last_full_sync_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_subsystem(&mut self, health: SubsystemHealth) -> Result<u8, &'static str> {
        if self.subsystem_count >= MAX_SUBSYSTEMS {
            return Err("SUBSYSTEM_LIMIT_EXCEEDED");
        }
        self.subsystems[self.subsystem_count] = Some(health);
        self.subsystem_count += 1;
        self.update_audit_checksum();
        Ok(health.subsystem_type as u8)
    }
    pub fn update_subsystem_health(&mut self, subsystem_type: SubsystemType, health: SubsystemHealth, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.subsystem_count {
            if let Some(ref mut subsystem) = self.subsystems[i] {
                if subsystem.subsystem_type == subsystem_type {
                    *subsystem = health;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("SUBSYSTEM_NOT_FOUND")
    }
    pub fn register_integration_point(&mut self, point: IntegrationPoint) -> Result<u64, &'static str> {
        if self.point_count >= MAX_INTEGRATION_POINTS {
            return Err("INTEGRATION_POINT_LIMIT_EXCEEDED");
        }
        self.integration_points[self.point_count] = Some(point);
        self.point_count += 1;
        self.update_audit_checksum();
        Ok(point.point_id)
    }
    pub fn register_data_pipeline(&mut self, pipeline: DataPipeline) -> Result<u64, &'static str> {
        if self.pipeline_count >= MAX_DATA_PIPELINES {
            return Err("PIPELINE_LIMIT_EXCEEDED");
        }
        self.data_pipelines[self.pipeline_count] = Some(pipeline);
        self.pipeline_count += 1;
        self.update_audit_checksum();
        Ok(pipeline.pipeline_id)
    }
    pub fn compute_system_sync_score(&mut self, now_ns: u64) -> f64 {
        let mut total_offset = 0u64;
        let mut valid_count = 0u64;
        for i in 0..self.subsystem_count {
            if let Some(ref subsystem) = self.subsystems[i] {
                if subsystem.status == IntegrationStatus::Online {
                    total_offset += subsystem.sync_offset_ms;
                    valid_count += 1;
                }
            }
        }
        if valid_count == 0 { return 0.0; }
        let avg_offset = total_offset / valid_count;
        self.system_sync_score = if avg_offset < SYNC_TOLERANCE_MS { 1.0 }
            else { 1.0 - (avg_offset - SYNC_TOLERANCE_MS) as f64 / 1000.0 }
            .max(0.0);
        self.system_sync_score
    }
    pub fn compute_integration_health_index(&mut self, now_ns: u64) -> f64 {
        let healthy_subsystems = self.subsystems.iter()
            .filter(|s| s.as_ref().map(|sub| sub.is_healthy(now_ns)).unwrap_or(false))
            .count() as f64;
        let operational_points = self.integration_points.iter()
            .filter(|p| p.as_ref().map(|pt| pt.operational).unwrap_or(false))
            .count() as f64;
        let operational_pipelines = self.data_pipelines.iter()
            .filter(|p| p.as_ref().map(|pl| pl.operational).unwrap_or(false))
            .count() as f64;
        let subsystem_health = healthy_subsystems / self.subsystem_count.max(1) as f64;
        let point_health = operational_points / self.point_count.max(1) as f64;
        let pipeline_health = operational_pipelines / self.pipeline_count.max(1) as f64;
        self.integration_health_index = (subsystem_health * 0.4 + point_health * 0.3 + pipeline_health * 0.3).min(1.0);
        self.integration_health_index
    }
    pub fn compute_cascade_failure_risk(&mut self) -> f64 {
        let mut critical_subsystems = 0u64;
        let mut degraded_subsystems = 0u64;
        for i in 0..self.subsystem_count {
            if let Some(ref subsystem) = self.subsystems[i] {
                if subsystem.status == IntegrationStatus::Fault {
                    critical_subsystems += 1;
                } else if subsystem.status == IntegrationStatus::Degraded {
                    degraded_subsystems += 1;
                }
            }
        }
        self.cascade_failure_risk = (critical_subsystems as f64 * 0.3 + degraded_subsystems as f64 * 0.1)
            .min(1.0);
        self.cascade_failure_risk
    }
    pub fn get_controller_status(&self, now_ns: u64) -> IntegrationStatusReport {
        let healthy_subsystems = self.subsystems.iter()
            .filter(|s| s.as_ref().map(|sub| sub.is_healthy(now_ns)).unwrap_or(false))
            .count();
        let subsystems_requiring_attention = self.subsystems.iter()
            .filter(|s| s.as_ref().map(|sub| sub.requires_attention()).unwrap_or(false))
            .count();
        IntegrationStatusReport {
            controller_id: self.controller_id,
            total_subsystems: self.subsystem_count,
            healthy_subsystems,
            subsystems_requiring_attention,
            total_integration_points: self.point_count,
            operational_points: self.integration_points.iter()
                .filter(|p| p.as_ref().map(|pt| pt.operational).unwrap_or(false))
                .count(),
            total_pipelines: self.pipeline_count,
            operational_pipelines: self.data_pipelines.iter()
                .filter(|p| p.as_ref().map(|pl| pl.operational).unwrap_or(false))
                .count(),
            total_messages_processed: self.total_messages_processed,
            total_messages_dropped: self.total_messages_dropped,
            average_latency_ms: self.average_latency_ms,
            system_sync_score: self.system_sync_score,
            integration_health_index: self.integration_health_index,
            cascade_failure_risk: self.cascade_failure_risk,
            last_full_sync_ns: self.last_full_sync_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.subsystem_count as u64).wrapping_mul(self.point_count as u64);
        sum ^= (self.pipeline_count as u64);
        sum ^= self.total_messages_processed;
        sum ^= self.total_messages_dropped;
        for i in 0..self.subsystem_count {
            if let Some(ref subsystem) = self.subsystems[i] {
                sum ^= (subsystem.subsystem_type as u64).wrapping_mul(subsystem.error_count as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.subsystem_count as u64).wrapping_mul(self.point_count as u64);
        sum ^= (self.pipeline_count as u64);
        sum ^= self.total_messages_processed;
        sum ^= self.total_messages_dropped;
        for i in 0..self.subsystem_count {
            if let Some(ref subsystem) = self.subsystems[i] {
                sum ^= (subsystem.subsystem_type as u64).wrapping_mul(subsystem.error_count as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct IntegrationStatusReport {
    pub controller_id: u64,
    pub total_subsystems: usize,
    pub healthy_subsystems: usize,
    pub subsystems_requiring_attention: usize,
    pub total_integration_points: usize,
    pub operational_points: usize,
    pub total_pipelines: usize,
    pub operational_pipelines: usize,
    pub total_messages_processed: u64,
    pub total_messages_dropped: u64,
    pub average_latency_ms: f64,
    pub system_sync_score: f64,
    pub integration_health_index: f64,
    pub cascade_failure_risk: f64,
    pub last_full_sync_ns: u64,
    pub last_update_ns: u64,
}

impl IntegrationStatusReport {
    pub fn system_reliability_index(&self) -> f64 {
        let subsystem_availability = self.healthy_subsystems as f64 / self.total_subsystems.max(1) as f64;
        let point_availability = self.operational_points as f64 / self.total_integration_points.max(1) as f64;
        let pipeline_availability = self.operational_pipelines as f64 / self.total_pipelines.max(1) as f64;
        let message_success_rate = if self.total_messages_processed + self.total_messages_dropped > 0 {
            self.total_messages_processed as f64 / (self.total_messages_processed + self.total_messages_dropped) as f64
        } else { 1.0 };
        let risk_penalty = self.cascade_failure_risk * 0.3;
        (subsystem_availability * 0.3 + point_availability * 0.25 + 
         pipeline_availability * 0.25 + message_success_rate * 0.2 - risk_penalty).max(0.0)
    }
}
