#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const WATER_QUALITY_VERSION: u32 = 20260310;
pub const MAX_MONITORING_STATIONS: usize = 4096;
pub const MAX_WATER_SAMPLES: usize = 1048576;
pub const MAX_CONTAMINANT_ALERTS: usize = 32768;
pub const EPA_PH_MIN: f64 = 6.5;
pub const EPA_PH_MAX: f64 = 8.5;
pub const EPA_TURBIDITY_MAX_NTU: f64 = 0.3;
pub const EPA_LEAD_MAX_PPB: f64 = 15.0;
pub const EPA_PFAS_MAX_NGL: f64 = 10.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaterSourceType {
    SurfaceWater = 0, Groundwater = 1, ReclaimedWater = 2, Desalinated = 3,
    Atmospheric = 4, Rainwater = 5, Well = 6, Reservoir = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlertLevel {
    Normal = 0, Advisory = 1, Watch = 2, Warning = 3, Emergency = 4, Critical = 5,
}

#[derive(Clone, Copy, Debug)]
pub struct WaterQualityParameters {
    pub ph: f64,
    pub turbidity_ntu: f64,
    pub total_dissolved_solids_mgl: f64,
    pub dissolved_oxygen_mgl: f64,
    pub chlorine_residual_mgl: f64,
    pub fluoride_mgl: f64,
    pub nitrate_mgl: f64,
    pub phosphate_mgl: f64,
    pub conductivity_uscm: f64,
    pub temperature_celsius: f64,
    pub total_colonies_cfu_100ml: u64,
    pub e_coli_cfu_100ml: u64,
    pub lead_ppb: f64,
    pub copper_ppb: f64,
    pub arsenic_ppb: f64,
    pub pfas_ngl: f64,
    pub pharmaceuticals_ngl: f64,
}

impl WaterQualityParameters {
    pub fn is_potable(&self) -> bool {
        self.ph >= EPA_PH_MIN && self.ph <= EPA_PH_MAX &&
        self.turbidity_ntu <= EPA_TURBIDITY_MAX_NTU &&
        self.e_coli_cfu_100ml == 0 &&
        self.lead_ppb <= EPA_LEAD_MAX_PPB &&
        self.pfas_ngl <= EPA_PFAS_MAX_NGL &&
        self.chlorine_residual_mgl >= 0.2
    }
    pub fn compute_quality_score(&self) -> f64 {
        let mut score = 1.0;
        if self.ph < EPA_PH_MIN || self.ph > EPA_PH_MAX { score -= 0.15; }
        if self.turbidity_ntu > EPA_TURBIDITY_MAX_NTU { score -= 0.15; }
        if self.e_coli_cfu_100ml > 0 { score -= 0.25; }
        if self.lead_ppb > EPA_LEAD_MAX_PPB { score -= 0.20; }
        if self.pfas_ngl > EPA_PFAS_MAX_NGL { score -= 0.20; }
        if self.chlorine_residual_mgl < 0.2 { score -= 0.15; }
        score.max(0.0)
    }
    pub fn requires_alert(&self) -> bool {
        !self.is_potable() || self.compute_quality_score() < 0.7
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MonitoringStation {
    pub station_id: u64,
    pub station_name: [u8; 128],
    pub latitude: f64,
    pub longitude: f64,
    pub water_source_type: WaterSourceType,
    pub depth_m: f64,
    pub flow_rate_m3h: f64,
    pub current_parameters: WaterQualityParameters,
    pub last_sample_ns: u64,
    pub next_sample_ns: u64,
    pub operational: bool,
    pub telemetry_active: bool,
    pub calibration_valid: bool,
    pub last_calibration_ns: u64,
    pub last_maintenance_ns: u64,
    pub alert_level: AlertLevel,
    pub served_population: u32,
    pub indigenous_community_served: bool,
}

impl MonitoringStation {
    pub fn requires_sampling(&self, now_ns: u64) -> bool {
        now_ns >= self.next_sample_ns || !self.operational || !self.calibration_valid
    }
    pub fn requires_maintenance(&self, now_ns: u64) -> bool {
        now_ns - self.last_maintenance_ns > 7776000000000000 || !self.operational
    }
    pub fn is_compliant(&self) -> bool {
        self.current_parameters.is_potable() && self.alert_level == AlertLevel::Normal
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WaterSample {
    pub sample_id: u64,
    pub station_id: u64,
    pub parameters: WaterQualityParameters,
    pub collected_ns: u64,
    pub analyzed_ns: u64,
    pub validated: bool,
    pub lab_id: u32,
    pub technician_id: u32,
    pub chain_of_custody_verified: bool,
    public_notification_required: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ContaminantAlert {
    pub alert_id: u64,
    pub station_id: u64,
    pub contaminant_type: u8,
    pub alert_level: AlertLevel,
    pub detected_concentration: f64,
    pub regulatory_limit: f64,
    pub detected_at_ns: u64,
    pub resolved_at_ns: u64,
    public_notification_sent: bool,
    boil_water_advisory: bool,
    affected_population: u32,
    alternative_water_provided: bool,
    resolution_notes: [u8; 256],
}

pub struct WaterQualityMonitoringNetwork {
    pub network_id: u64,
    pub city_code: [u8; 8],
    pub monitoring_stations: [Option<MonitoringStation>; MAX_MONITORING_STATIONS],
    pub station_count: usize,
    pub water_samples: [Option<WaterSample>; MAX_WATER_SAMPLES],
    pub sample_count: usize,
    pub contaminant_alerts: [Option<ContaminantAlert>; MAX_CONTAMINANT_ALERTS],
    pub alert_count: usize,
    pub total_samples_collected: u64,
    pub compliant_samples: u64,
    pub total_alerts_issued: u64,
    pub alerts_resolved: u64,
    pub average_response_time_min: f64,
    pub network_compliance_rate: f64,
    pub public_notification_count: u64,
    pub last_contamination_event_ns: u64,
    pub audit_checksum: u64,
}

impl WaterQualityMonitoringNetwork {
    pub fn new(network_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            network_id,
            city_code,
            monitoring_stations: Default::default(),
            station_count: 0,
            water_samples: Default::default(),
            sample_count: 0,
            contaminant_alerts: Default::default(),
            alert_count: 0,
            total_samples_collected: 0,
            compliant_samples: 0,
            total_alerts_issued: 0,
            alerts_resolved: 0,
            average_response_time_min: 0.0,
            network_compliance_rate: 1.0,
            public_notification_count: 0,
            last_contamination_event_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_monitoring_station(&mut self, station: MonitoringStation) -> Result<u64, &'static str> {
        if self.station_count >= MAX_MONITORING_STATIONS { return Err("STATION_LIMIT_EXCEEDED"); }
        if !station.calibration_valid { return Err("CALIBRATION_REQUIRED"); }
        self.monitoring_stations[self.station_count] = Some(station);
        self.station_count += 1;
        self.update_audit_checksum();
        Ok(station.station_id)
    }
    pub fn record_water_sample(&mut self, sample: WaterSample) -> Result<u64, &'static str> {
        if self.sample_count >= MAX_WATER_SAMPLES { return Err("SAMPLE_LIMIT_EXCEEDED"); }
        if !sample.chain_of_custody_verified { return Err("CHAIN_OF_CUSTODY_REQUIRED"); }
        self.water_samples[self.sample_count] = Some(sample);
        self.sample_count += 1;
        self.total_samples_collected += 1;
        if sample.parameters.is_potable() {
            self.compliant_samples += 1;
        }
        if sample.parameters.requires_alert() {
            self.trigger_contaminant_alert(sample.station_id, &sample.parameters, sample.analyzed_ns);
        }
        self.update_audit_checksum();
        Ok(sample.sample_id)
    }
    pub fn trigger_contaminant_alert(&mut self, station_id: u64, parameters: &WaterQualityParameters, now_ns: u64) -> Result<u64, &'static str> {
        if self.alert_count >= MAX_CONTAMINANT_ALERTS { return Err("ALERT_LIMIT_EXCEEDED"); }
        let mut contaminant_type = 0u8;
        let mut detected_concentration = 0.0;
        let mut regulatory_limit = 0.0;
        if parameters.lead_ppb > EPA_LEAD_MAX_PPB {
            contaminant_type = 1;
            detected_concentration = parameters.lead_ppb;
            regulatory_limit = EPA_LEAD_MAX_PPB;
        } else if parameters.pfas_ngl > EPA_PFAS_MAX_NGL {
            contaminant_type = 2;
            detected_concentration = parameters.pfas_ngl;
            regulatory_limit = EPA_PFAS_MAX_NGL;
        } else if parameters.e_coli_cfu_100ml > 0 {
            contaminant_type = 3;
            detected_concentration = parameters.e_coli_cfu_100ml as f64;
            regulatory_limit = 0.0;
        }
        let alert_level = if detected_concentration > regulatory_limit * 2.0 {
            AlertLevel::Emergency
        } else {
            AlertLevel::Warning
        };
        let alert = ContaminantAlert {
            alert_id: self.alert_count as u64,
            station_id,
            contaminant_type,
            alert_level,
            detected_concentration,
            regulatory_limit,
            detected_at_ns: now_ns,
            resolved_at_ns: 0,
            public_notification_sent: alert_level >= AlertLevel::Warning,
            boil_water_advisory: alert_level >= AlertLevel::Emergency,
            affected_population: 0,
            alternative_water_provided: false,
            resolution_notes: [0u8; 256],
        };
        self.contaminant_alerts[self.alert_count] = Some(alert);
        self.alert_count += 1;
        self.total_alerts_issued += 1;
        self.last_contamination_event_ns = now_ns;
        if alert.public_notification_sent {
            self.public_notification_count += 1;
        }
        self.update_audit_checksum();
        Ok(alert.alert_id)
    }
    pub fn resolve_alert(&mut self, alert_id: u64, resolution_notes: [u8; 256], now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.alert_count {
            if let Some(ref mut alert) = self.contaminant_alerts[i] {
                if alert.alert_id == alert_id {
                    alert.resolved_at_ns = now_ns;
                    alert.resolution_notes = resolution_notes;
                    self.alerts_resolved += 1;
                    let response_time = (now_ns - alert.detected_at_ns) / 60000000000;
                    self.average_response_time_min = (self.average_response_time_min * 
                        (self.alerts_resolved - 1) as f64 + response_time as f64) / 
                        self.alerts_resolved.max(1) as f64;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("ALERT_NOT_FOUND")
    }
    pub fn compute_network_compliance(&mut self) -> f64 {
        self.network_compliance_rate = if self.total_samples_collected > 0 {
            self.compliant_samples as f64 / self.total_samples_collected as f64
        } else { 1.0 };
        self.network_compliance_rate
    }
    pub fn get_network_status(&self, now_ns: u64) -> WaterQualityStatus {
        let operational_stations = self.monitoring_stations.iter()
            .filter(|s| s.as_ref().map(|station| station.operational).unwrap_or(false))
            .count();
        let compliant_stations = self.monitoring_stations.iter()
            .filter(|s| s.as_ref().map(|station| station.is_compliant()).unwrap_or(false))
            .count();
        let active_alerts = self.contaminant_alerts.iter()
            .filter(|a| a.as_ref().map(|alert| alert.resolved_at_ns == 0).unwrap_or(false))
            .count();
        WaterQualityStatus {
            network_id: self.network_id,
            total_stations: self.station_count,
            operational_stations,
            compliant_stations,
            total_samples: self.total_samples_collected,
            compliant_samples: self.compliant_samples,
            total_alerts: self.total_alerts_issued,
            active_alerts,
            alerts_resolved: self.alerts_resolved,
            network_compliance_rate: self.network_compliance_rate,
            average_response_time_min: self.average_response_time_min,
            public_notifications: self.public_notification_count,
            last_contamination_event_ns: self.last_contamination_event_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.station_count as u64).wrapping_mul(self.sample_count as u64);
        sum ^= self.total_alerts_issued;
        sum ^= (self.network_compliance_rate * 1e6) as u64;
        for i in 0..self.station_count {
            if let Some(ref station) = self.monitoring_stations[i] {
                sum ^= station.station_id.wrapping_mul(station.alert_level as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.station_count as u64).wrapping_mul(self.sample_count as u64);
        sum ^= self.total_alerts_issued;
        sum ^= (self.network_compliance_rate * 1e6) as u64;
        for i in 0..self.station_count {
            if let Some(ref station) = self.monitoring_stations[i] {
                sum ^= station.station_id.wrapping_mul(station.alert_level as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct WaterQualityStatus {
    pub network_id: u64,
    pub total_stations: usize,
    pub operational_stations: usize,
    pub compliant_stations: usize,
    pub total_samples: u64,
    pub compliant_samples: u64,
    pub total_alerts: u64,
    pub active_alerts: usize,
    pub alerts_resolved: u64,
    pub network_compliance_rate: f64,
    pub average_response_time_min: f64,
    pub public_notifications: u64,
    pub last_contamination_event_ns: u64,
    pub last_update_ns: u64,
}

impl WaterQualityStatus {
    pub fn water_safety_index(&self) -> f64 {
        let station_compliance = self.compliant_stations as f64 / self.total_stations.max(1) as f64;
        let sample_compliance = self.network_compliance_rate;
        let alert_resolution_rate = if self.total_alerts > 0 {
            self.alerts_resolved as f64 / self.total_alerts as f64
        } else { 1.0 };
        let response_score = if self.average_response_time_min < 60.0 { 1.0 } 
            else { 60.0 / self.average_response_time_min };
        (station_compliance * 0.35 + sample_compliance * 0.35 + 
         alert_resolution_rate * 0.15 + response_score * 0.15).min(1.0)
    }
}
