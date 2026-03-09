#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const INFRASTRUCTURE_PROTECTION_VERSION: u32 = 20260310;
pub const MAX_CRITICAL_FACILITIES: usize = 2048;
pub const MAX_INFRASTRUCTURE_TYPES: usize = 128;
pub const MAX_SECURITY_ALERTS: usize = 32768;
pub const MAX_DEPENDENCY_MAPPINGS: usize = 65536;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InfrastructureType {
    PowerPlant = 0, WaterTreatment = 1, Hospital = 2, EmergencyServices = 3,
    Communications = 4, Transportation = 5, Financial = 6, Government = 7,
    FuelStorage = 8, FoodDistribution = 9, DataCenter = 10, WasteTreatment = 11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThreatLevel {
    Minimal = 0, Low = 1, Guarded = 2, Elevated = 3, High = 4, Severe = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FacilityStatus {
    Operational = 0, Degraded = 1, Compromised = 2, Offline = 3,
    UnderAttack = 4, Evacuated = 5, Quarantined = 6, Destroyed = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct CriticalFacility {
    pub facility_id: u64,
    pub facility_type: InfrastructureType,
    pub name: [u8; 128],
    pub latitude: f64,
    pub longitude: f64,
    pub criticality_score: f64,
    pub redundancy_level: u8,
    pub backup_systems_available: bool,
    pub cybersecurity_level: u8,
    pub physical_security_level: u8,
    pub staff_count: u32,
    pub capacity_pct: f64,
    pub status: FacilityStatus,
    pub threat_level: ThreatLevel,
    pub last_inspection_ns: u64,
    pub last_security_audit_ns: u64,
    pub emergency_contact_id: u64,
    pub dependency_count: u8,
    pub protected: bool,
}

impl CriticalFacility {
    pub fn is_operational(&self) -> bool {
        self.status == FacilityStatus::Operational || self.status == FacilityStatus::Degraded
    }
    pub fn requires_immediate_protection(&self) -> bool {
        self.threat_level >= ThreatLevel::High ||
        self.status == FacilityStatus::UnderAttack ||
        self.status == FacilityStatus::Compromised
    }
    pub fn resilience_score(&self) -> f64 {
        let mut score = 0.0;
        if self.backup_systems_available { score += 0.3; }
        score += self.redundancy_level as f64 * 0.1;
        score += self.cybersecurity_level as f64 * 0.1;
        score += self.physical_security_level as f64 * 0.1;
        if self.status == FacilityStatus::Operational { score += 0.3; }
        score.min(1.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SecurityAlert {
    pub alert_id: u64,
    pub facility_id: u64,
    pub threat_type: u8,
    pub severity: u8,
    pub detected_at_ns: u64,
    pub resolved_at_ns: u64,
    pub status: u8,
    pub false_positive: bool,
    pub response_time_min: u32,
    pub casualties: u32,
    pub economic_loss_usd: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct DependencyMapping {
    pub mapping_id: u64,
    pub source_facility_id: u64,
    pub dependent_facility_id: u64,
    pub dependency_type: u8,
    pub criticality: f64,
    pub redundancy_available: bool,
    pub failover_time_min: u32,
}

pub struct CriticalInfrastructureProtectionSystem {
    pub system_id: u64,
    pub city_code: [u8; 8],
    pub facilities: [Option<CriticalFacility>; MAX_CRITICAL_FACILITIES],
    pub facility_count: usize,
    pub security_alerts: [Option<SecurityAlert>; MAX_SECURITY_ALERTS],
    pub alert_count: usize,
    pub dependencies: [Option<DependencyMapping>; MAX_DEPENDENCY_MAPPINGS],
    pub dependency_count: usize,
    pub total_alerts: u64,
    pub false_positives: u64,
    pub successful_defenses: u64,
    pub facilities_compromised: u64,
    pub average_response_time_min: f64,
    pub infrastructure_resilience_index: f64,
    pub last_major_incident_ns: u64,
    pub audit_checksum: u64,
}

impl CriticalInfrastructureProtectionSystem {
    pub fn new(system_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            system_id,
            city_code,
            facilities: Default::default(),
            facility_count: 0,
            security_alerts: Default::default(),
            alert_count: 0,
            dependencies: Default::default(),
            dependency_count: 0,
            total_alerts: 0,
            false_positives: 0,
            successful_defenses: 0,
            facilities_compromised: 0,
            average_response_time_min: 0.0,
            infrastructure_resilience_index: 1.0,
            last_major_incident_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_facility(&mut self, facility: CriticalFacility) -> Result<u64, &'static str> {
        if self.facility_count >= MAX_CRITICAL_FACILITIES { return Err("FACILITY_LIMIT_EXCEEDED"); }
        if facility.criticality_score < 0.7 { return Err("CRITICALITY_THRESHOLD_NOT_MET"); }
        self.facilities[self.facility_count] = Some(facility);
        self.facility_count += 1;
        self.update_audit_checksum();
        Ok(facility.facility_id)
    }
    pub fn report_security_alert(&mut self, alert: SecurityAlert, now_ns: u64) -> Result<u64, &'static str> {
        if self.alert_count >= MAX_SECURITY_ALERTS { return Err("ALERT_LIMIT_EXCEEDED"); }
        self.security_alerts[self.alert_count] = Some(alert);
        self.alert_count += 1;
        self.total_alerts += 1;
        if alert.severity >= 4 {
            self.last_major_incident_ns = now_ns;
        }
        self.update_audit_checksum();
        Ok(alert.alert_id)
    }
    pub fn resolve_security_alert(&mut self, alert_id: u64, resolved_at_ns: u64, 
                                   response_time_min: u32, false_positive: bool) -> Result<(), &'static str> {
        for i in 0..self.alert_count {
            if let Some(ref mut alert) = self.security_alerts[i] {
                if alert.alert_id == alert_id {
                    alert.resolved_at_ns = resolved_at_ns;
                    alert.status = 2;
                    alert.response_time_min = response_time_min;
                    alert.false_positive = false_positive;
                    if false_positive {
                        self.false_positives += 1;
                    } else {
                        self.successful_defenses += 1;
                    }
                    self.average_response_time_min = (self.average_response_time_min * 
                        (self.successful_defenses - 1) as f64 + response_time_min as f64) / 
                        self.successful_defenses.max(1) as f64;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("ALERT_NOT_FOUND")
    }
    pub fn register_dependency(&mut self, dependency: DependencyMapping) -> Result<u64, &'static str> {
        if self.dependency_count >= MAX_DEPENDENCY_MAPPINGS { return Err("DEPENDENCY_LIMIT_EXCEEDED"); }
        self.dependencies[self.dependency_count] = Some(dependency);
        self.dependency_count += 1;
        self.update_audit_checksum();
        Ok(dependency.mapping_id)
    }
    pub fn update_facility_status(&mut self, facility_id: u64, status: FacilityStatus, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.facility_count {
            if let Some(ref mut facility) = self.facilities[i] {
                if facility.facility_id == facility_id {
                    if status == FacilityStatus::Compromised || status == FacilityStatus::Destroyed {
                        self.facilities_compromised += 1;
                    }
                    facility.status = status;
                    facility.last_inspection_ns = now_ns;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("FACILITY_NOT_FOUND")
    }
    pub fn compute_infrastructure_resilience_index(&mut self) -> f64 {
        let mut total_resilience = 0.0;
        let mut operational_facilities = 0u64;
        for i in 0..self.facility_count {
            if let Some(ref facility) = self.facilities[i] {
                if facility.is_operational() {
                    total_resilience += facility.resilience_score();
                    operational_facilities += 1;
                }
            }
        }
        let avg_resilience = if operational_facilities > 0 {
            total_resilience / operational_facilities as f64
        } else { 0.0 };
        let availability_ratio = operational_facilities as f64 / self.facility_count.max(1) as f64;
        let defense_success_rate = if self.total_alerts > 0 {
            self.successful_defenses as f64 / self.total_alerts as f64
        } else { 1.0 };
        self.infrastructure_resilience_index = (avg_resilience * 0.4 + availability_ratio * 0.35 + 
            defense_success_rate * 0.25).min(1.0);
        self.infrastructure_resilience_index
    }
    pub fn identify_cascade_failures(&self, compromised_facility_id: u64) -> Vec<u64> {
        let mut at_risk = Vec::new();
        for i in 0..self.dependency_count {
            if let Some(ref dep) = self.dependencies[i] {
                if dep.source_facility_id == compromised_facility_id {
                    if !dep.redundancy_available {
                        at_risk.push(dep.dependent_facility_id);
                    }
                }
            }
        }
        at_risk
    }
    pub fn get_system_status(&self, now_ns: u64) -> InfrastructureProtectionStatus {
        let operational_facilities = self.facilities.iter()
            .filter(|f| f.as_ref().map(|fac| fac.is_operational()).unwrap_or(false))
            .count();
        let high_threat_facilities = self.facilities.iter()
            .filter(|f| f.as_ref().map(|fac| fac.requires_immediate_protection()).unwrap_or(false))
            .count();
        let active_alerts = self.security_alerts.iter()
            .filter(|a| a.as_ref().map(|alert| alert.status == 1).unwrap_or(false))
            .count();
        InfrastructureProtectionStatus {
            system_id: self.system_id,
            total_facilities: self.facility_count,
            operational_facilities,
            high_threat_facilities,
            total_alerts: self.total_alerts,
            active_alerts,
            false_positives: self.false_positives,
            successful_defenses: self.successful_defenses,
            facilities_compromised: self.facilities_compromised,
            average_response_time_min: self.average_response_time_min,
            infrastructure_resilience_index: self.infrastructure_resilience_index,
            last_major_incident_ns: self.last_major_incident_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.facility_count as u64).wrapping_mul(self.alert_count as u64);
        sum ^= self.total_alerts;
        sum ^= self.facilities_compromised;
        for i in 0..self.facility_count {
            if let Some(ref facility) = self.facilities[i] {
                sum ^= facility.facility_id.wrapping_mul(facility.criticality_score as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.facility_count as u64).wrapping_mul(self.alert_count as u64);
        sum ^= self.total_alerts;
        sum ^= self.facilities_compromised;
        for i in 0..self.facility_count {
            if let Some(ref facility) = self.facilities[i] {
                sum ^= facility.facility_id.wrapping_mul(facility.criticality_score as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct InfrastructureProtectionStatus {
    pub system_id: u64,
    pub total_facilities: usize,
    pub operational_facilities: usize,
    pub high_threat_facilities: usize,
    pub total_alerts: u64,
    pub active_alerts: usize,
    pub false_positives: u64,
    pub successful_defenses: u64,
    pub facilities_compromised: u64,
    pub average_response_time_min: f64,
    pub infrastructure_resilience_index: f64,
    pub last_major_incident_ns: u64,
    pub last_update_ns: u64,
}

impl InfrastructureProtectionStatus {
    pub fn protection_effectiveness(&self) -> f64 {
        let facility_availability = self.operational_facilities as f64 / self.total_facilities.max(1) as f64;
        let defense_rate = if self.total_alerts > 0 {
            self.successful_defenses as f64 / self.total_alerts as f64
        } else { 1.0 };
        let compromise_rate = 1.0 - (self.facilities_compromised as f64 / self.total_facilities.max(1) as f64);
        (facility_availability * 0.4 + defense_rate * 0.35 + compromise_rate * 0.25).min(1.0)
    }
}
