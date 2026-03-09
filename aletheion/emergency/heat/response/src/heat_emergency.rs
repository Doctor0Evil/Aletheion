#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const HEAT_EMERGENCY_VERSION: u32 = 20260310;
pub const MAX_COOLING_CENTERS: usize = 512;
pub const MAX_HEAT_ALERTS: usize = 65536;
pub const MAX_VULNERABLE_CITIZENS: usize = 131072;
pub const PHOENIX_HEAT_EMERGENCY_THRESHOLD_F: f64 = 115.0;
pub const HEAT_WARNING_THRESHOLD_F: f64 = 110.0;
pub const HEAT_ADVISORY_THRESHOLD_F: f64 = 105.0;
pub const MAX_EXPOSURE_TIME_MIN: f64 = 30.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeatAlertLevel {
    Normal = 0, Advisory = 1, Watch = 2, Warning = 3, Emergency = 4, Critical = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VulnerabilityCategory {
    Elderly = 0, Child = 1, ChronicIllness = 2, Homeless = 3,
    OutdoorWorker = 4, Disabled = 5, LowIncome = 6, Isolated = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct HeatAlert {
    pub alert_id: u64,
    pub alert_level: HeatAlertLevel,
    pub issued_at_ns: u64,
    pub expires_at_ns: u64,
    pub temperature_f: f64,
    pub heat_index_f: f64,
    pub humidity_pct: f64,
    pub affected_zones: [u32; 16],
    pub zone_count: u8,
    public_notification_sent: bool,
    emergency_services_activated: bool,
    cooling_centers_opened: u32,
    casualties_reported: u32,
    hospitalizations: u32,
}

impl HeatAlert {
    pub fn is_active(&self, now_ns: u64) -> bool {
        now_ns >= self.issued_at_ns && now_ns < self.expires_at_ns
    }
    pub fn requires_emergency_response(&self) -> bool {
        self.alert_level >= HeatAlertLevel::Emergency
    }
    pub fn duration_hours(&self, now_ns: u64) -> f64 {
        let end = if now_ns < self.expires_at_ns { now_ns } else { self.expires_at_ns };
        (end - self.issued_at_ns) as f64 / 3600000000000.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CoolingCenter {
    pub center_id: u32,
    pub name: [u8; 128],
    pub latitude: f64,
    pub longitude: f64,
    pub capacity: u32,
    pub current_occupancy: u32,
    pub operational: bool,
    pub accessibility_compliant: bool,
    pub medical_support_available: bool,
    pub pet_friendly: bool,
    pub hours_of_operation: [u8; 64],
    pub contact_phone: [u8; 32],
    pub power_backup_available: bool,
    pub water_supply_available: bool,
    pub last_inspection_ns: u64,
    pub activation_status: u8,
}

impl CoolingCenter {
    pub fn availability_ratio(&self) -> f64 {
        if self.capacity == 0 { return 0.0; }
        (self.capacity - self.current_occupancy) as f64 / self.capacity as f64
    }
    pub fn can_accept_patients(&self) -> bool {
        self.operational && self.availability_ratio() > 0.1 && self.activation_status >= 2
    }
    pub fn is_emergency_ready(&self) -> bool {
        self.power_backup_available && self.water_supply_available && self.medical_support_available
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VulnerableCitizen {
    pub citizen_id: u64,
    pub citizen_did: [u8; 32],
    pub vulnerability_categories: u8,
    pub age: u8,
    pub has_chronic_conditions: bool,
    pub is_homeless: bool,
    pub is_outdoor_worker: bool,
    pub emergency_contact_id: u64,
    pub preferred_language: [u8; 8],
    pub accessibility_needs: bool,
    pub registered_for_alerts: bool,
    pub last_welfare_check_ns: u64,
    pub welfare_check_required: bool,
    pub location_zone_id: u32,
    pub risk_score: f64,
}

impl VulnerableCitizen {
    pub fn compute_risk_score(&mut self, temperature_f: f64) -> f64 {
        let mut score = 0.0;
        if self.age >= 65 { score += 0.2; }
        if self.age <= 5 { score += 0.2; }
        if self.has_chronic_conditions { score += 0.25; }
        if self.is_homeless { score += 0.3; }
        if self.is_outdoor_worker { score += 0.25; }
        if self.accessibility_needs { score += 0.15; }
        if temperature_f >= PHOENIX_HEAT_EMERGENCY_THRESHOLD_F { score += 0.3; }
        self.risk_score = score.min(1.0);
        self.risk_score
    }
    pub fn requires_welfare_check(&self, now_ns: u64, alert_level: HeatAlertLevel) -> bool {
        let check_interval_ns = match alert_level {
            HeatAlertLevel::Critical => 3600000000000,
            HeatAlertLevel::Emergency => 7200000000000,
            HeatAlertLevel::Warning => 14400000000000,
            _ => 86400000000000,
        };
        now_ns - self.last_welfare_check_ns > check_interval_ns || self.welfare_check_required
    }
}

pub struct HeatEmergencyResponseSystem {
    pub system_id: u64,
    pub city_code: [u8; 8],
    pub heat_alerts: [Option<HeatAlert>; MAX_HEAT_ALERTS],
    pub alert_count: usize,
    pub cooling_centers: [Option<CoolingCenter>; MAX_COOLING_CENTERS],
    pub center_count: usize,
    pub vulnerable_citizens: [Option<VulnerableCitizen>; MAX_VULNERABLE_CITIZENS],
    pub citizen_count: usize,
    pub current_temperature_f: f64,
    pub current_heat_index_f: f64,
    pub current_alert_level: HeatAlertLevel,
    pub total_alerts_issued: u64,
    pub total_emergency_activations: u64,
    pub total_welfare_checks_completed: u64,
    pub heat_related_casualties: u64,
    pub heat_related_hospitalizations: u64,
    pub average_response_time_min: f64,
    pub last_emergency_activation_ns: u64,
    pub audit_checksum: u64,
}

impl HeatEmergencyResponseSystem {
    pub fn new(system_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            system_id,
            city_code,
            heat_alerts: Default::default(),
            alert_count: 0,
            cooling_centers: Default::default(),
            center_count: 0,
            vulnerable_citizens: Default::default(),
            citizen_count: 0,
            current_temperature_f: 0.0,
            current_heat_index_f: 0.0,
            current_alert_level: HeatAlertLevel::Normal,
            total_alerts_issued: 0,
            total_emergency_activations: 0,
            total_welfare_checks_completed: 0,
            heat_related_casualties: 0,
            heat_related_hospitalizations: 0,
            average_response_time_min: 0.0,
            last_emergency_activation_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_cooling_center(&mut self, center: CoolingCenter) -> Result<u32, &'static str> {
        if self.center_count >= MAX_COOLING_CENTERS { return Err("CENTER_LIMIT_EXCEEDED"); }
        if !center.accessibility_compliant { return Err("ACCESSIBILITY_COMPLIANCE_REQUIRED"); }
        self.cooling_centers[self.center_count] = Some(center);
        self.center_count += 1;
        self.update_audit_checksum();
        Ok(center.center_id)
    }
    pub fn register_vulnerable_citizen(&mut self, mut citizen: VulnerableCitizen, temperature_f: f64) -> Result<u64, &'static str> {
        if self.citizen_count >= MAX_VULNERABLE_CITIZENS { return Err("CITIZEN_LIMIT_EXCEEDED"); }
        citizen.compute_risk_score(temperature_f);
        self.vulnerable_citizens[self.citizen_count] = Some(citizen);
        self.citizen_count += 1;
        self.update_audit_checksum();
        Ok(citizen.citizen_id)
    }
    pub fn issue_heat_alert(&mut self, mut alert: HeatAlert, now_ns: u64) -> Result<u64, &'static str> {
        if self.alert_count >= MAX_HEAT_ALERTS { return Err("ALERT_LIMIT_EXCEEDED"); }
        self.current_alert_level = alert.alert_level;
        self.current_temperature_f = alert.temperature_f;
        self.current_heat_index_f = alert.heat_index_f;
        if alert.alert_level >= HeatAlertLevel::Emergency {
            self.total_emergency_activations += 1;
            self.last_emergency_activation_ns = now_ns;
        }
        self.heat_alerts[self.alert_count] = Some(alert);
        self.alert_count += 1;
        self.total_alerts_issued += 1;
        self.update_audit_checksum();
        Ok(alert.alert_id)
    }
    pub fn activate_cooling_center(&mut self, center_id: u32, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.center_count {
            if let Some(ref mut center) = self.cooling_centers[i] {
                if center.center_id == center_id {
                    center.activation_status = 3;
                    center.last_inspection_ns = now_ns;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("CENTER_NOT_FOUND")
    }
    pub fn perform_welfare_check(&mut self, citizen_id: u64, now_ns: u64, welfare_status: u8) -> Result<(), &'static str> {
        for i in 0..self.citizen_count {
            if let Some(ref mut citizen) = self.vulnerable_citizens[i] {
                if citizen.citizen_id == citizen_id {
                    citizen.last_welfare_check_ns = now_ns;
                    citizen.welfare_check_required = welfare_status < 2;
                    self.total_welfare_checks_completed += 1;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("CITIZEN_NOT_FOUND")
    }
    pub fn record_heat_casualty(&mut self, casualty_count: u32, hospitalization_count: u32) {
        self.heat_related_casualties += casualty_count as u64;
        self.heat_related_hospitalizations += hospitalization_count as u64;
        self.update_audit_checksum();
    }
    pub fn find_nearest_cooling_center(&self, lat: f64, lon: f64) -> Option<&CoolingCenter> {
        let mut best_center: Option<&CoolingCenter> = None;
        let mut best_distance = f64::MAX;
        for i in 0..self.center_count {
            if let Some(ref center) = self.cooling_centers[i] {
                if center.can_accept_patients() {
                    let distance = self.compute_distance(lat, lon, center.latitude, center.longitude);
                    if distance < best_distance {
                        best_distance = distance;
                        best_center = Some(center);
                    }
                }
            }
        }
        best_center
    }
    fn compute_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let earth_radius_km = 6371.0;
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let a = (d_lat / 2.0).sin().powi(2) +
                lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        earth_radius_km * c
    }
    pub fn compute_system_readiness(&self, now_ns: u64) -> f64 {
        let operational_centers = self.cooling_centers.iter()
            .filter(|c| c.as_ref().map(|center| center.operational).unwrap_or(false))
            .count() as f64;
        let center_readiness = operational_centers / self.center_count.max(1) as f64;
        let emergency_ready_centers = self.cooling_centers.iter()
            .filter(|c| c.as_ref().map(|center| center.is_emergency_ready()).unwrap_or(false))
            .count() as f64;
        let emergency_readiness = emergency_ready_centers / self.center_count.max(1) as f64;
        let welfare_check_coverage = if self.citizen_count > 0 {
            self.total_welfare_checks_completed as f64 / self.citizen_count as f64
        } else { 1.0 };
        (center_readiness * 0.35 + emergency_readiness * 0.35 + welfare_check_coverage * 0.30).min(1.0)
    }
    pub fn get_system_status(&self, now_ns: u64) -> HeatEmergencyStatus {
        let active_alerts = self.heat_alerts.iter()
            .filter(|a| a.as_ref().map(|alert| alert.is_active(now_ns)).unwrap_or(false))
            .count();
        let open_centers = self.cooling_centers.iter()
            .filter(|c| c.as_ref().map(|center| center.can_accept_patients()).unwrap_or(false))
            .count();
        let citizens_needing_checks = self.vulnerable_citizens.iter()
            .filter(|c| c.as_ref().map(|citizen| citizen.welfare_check_required).unwrap_or(false))
            .count();
        HeatEmergencyStatus {
            system_id: self.system_id,
            current_temperature_f: self.current_temperature_f,
            current_heat_index_f: self.current_heat_index_f,
            current_alert_level: self.current_alert_level,
            active_alerts,
            total_alerts_issued: self.total_alerts_issued,
            total_cooling_centers: self.center_count,
            open_cooling_centers: open_centers,
            total_vulnerable_citizens: self.citizen_count,
            citizens_needing_welfare_checks: citizens_needing_checks,
            total_welfare_checks_completed: self.total_welfare_checks_completed,
            heat_casualties: self.heat_related_casualties,
            heat_hospitalizations: self.heat_related_hospitalizations,
            system_readiness_score: self.compute_system_readiness(now_ns),
            last_emergency_activation_ns: self.last_emergency_activation_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.alert_count as u64).wrapping_mul(self.center_count as u64);
        sum ^= self.total_alerts_issued;
        sum ^= self.total_emergency_activations;
        sum ^= self.heat_related_casualties;
        for i in 0..self.center_count {
            if let Some(ref center) = self.cooling_centers[i] {
                sum ^= center.center_id as u64;
                sum ^= center.activation_status as u64;
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.alert_count as u64).wrapping_mul(self.center_count as u64);
        sum ^= self.total_alerts_issued;
        sum ^= self.total_emergency_activations;
        sum ^= self.heat_related_casualties;
        for i in 0..self.center_count {
            if let Some(ref center) = self.cooling_centers[i] {
                sum ^= center.center_id as u64;
                sum ^= center.activation_status as u64;
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct HeatEmergencyStatus {
    pub system_id: u64,
    pub current_temperature_f: f64,
    pub current_heat_index_f: f64,
    pub current_alert_level: HeatAlertLevel,
    pub active_alerts: usize,
    pub total_alerts_issued: u64,
    pub total_cooling_centers: usize,
    pub open_cooling_centers: usize,
    pub total_vulnerable_citizens: usize,
    pub citizens_needing_welfare_checks: usize,
    pub total_welfare_checks_completed: u64,
    pub heat_casualties: u64,
    pub heat_hospitalizations: u64,
    pub system_readiness_score: f64,
    pub last_emergency_activation_ns: u64,
    pub last_update_ns: u64,
}

impl HeatEmergencyStatus {
    pub fn heat_resilience_index(&self) -> f64 {
        let center_availability = self.open_cooling_centers as f64 / self.total_cooling_centers.max(1) as f64;
        let welfare_coverage = self.total_welfare_checks_completed as f64 / self.total_vulnerable_citizens.max(1) as f64;
        let casualty_rate = if self.total_alerts_issued > 0 {
            1.0 - (self.heat_casualties as f64 / self.total_alerts_issued as f64).min(1.0)
        } else { 1.0 };
        (center_availability * 0.4 + welfare_coverage * 0.35 + casualty_rate * 0.25).min(1.0)
    }
}
