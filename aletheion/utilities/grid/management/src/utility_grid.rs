#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const UTILITY_GRID_VERSION: u32 = 20260310;
pub const MAX_UTILITY_NODES: usize = 8192;
pub const MAX_METER_READINGS: usize = 1048576;
pub const MAX_SERVICE_CONNECTIONS: usize = 262144;
pub const PHOENIX_WATER_TARGET_GPD: f64 = 50.0;
pub const PHOENIX_ENERGY_TARGET_KWH_DAY: f64 = 15.0;
pub const LEAK_DETECTION_THRESHOLD_PCT: f64 = 0.15;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UtilityType {
    Electricity = 0, Water = 1, Gas = 2, Sewer = 3,
    Telecom = 4, Internet = 5, Cable = 6, Stormwater = 7,
    RecycledWater = 8, DistrictCooling = 9, DistrictHeating = 10, Waste = 11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeterStatus {
    Active = 0, Inactive = 1, Tampered = 2, Fault = 3,
    Offline = 4, LowBattery = 5, LeakDetected = 6, Overload = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct UtilityNode {
    pub node_id: u64,
    pub utility_type: UtilityType,
    pub latitude: f64,
    pub longitude: f64,
    pub capacity: f64,
    pub current_load: f64,
    pub efficiency_pct: f64,
    pub last_maintenance_ns: u64,
    pub next_maintenance_ns: u64,
    pub operational: bool,
    pub redundancy_level: u8,
    pub backup_available: bool,
    pub smart_grid_enabled: bool,
    pub demand_response_capable: bool,
}

impl UtilityNode {
    pub fn load_ratio(&self) -> f64 {
        if self.capacity == 0.0 { return 0.0; }
        self.current_load / self.capacity
    }
    pub fn requires_maintenance(&self, now_ns: u64) -> bool {
        now_ns >= self.next_maintenance_ns || !self.operational || self.efficiency_pct < 70.0
    }
    pub fn is_overloaded(&self) -> bool {
        self.load_ratio() > 0.95
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MeterReading {
    pub reading_id: u64,
    pub meter_id: u64,
    pub utility_type: UtilityType,
    pub consumption_value: f64,
    pub unit: [u8; 16],
    pub timestamp_ns: u64,
    pub previous_reading: f64,
    pub delta: f64,
    pub estimated: bool,
    pub validated: bool,
    pub anomaly_detected: bool,
    pub citizen_did: [u8; 32],
}

impl MeterReading {
    pub fn compute_daily_average(&self, previous: &MeterReading) -> f64 {
        let time_delta_days = (self.timestamp_ns - previous.timestamp_ns) as f64 / 86400000000000.0;
        if time_delta_days <= 0.0 { return 0.0; }
        self.delta / time_delta_days
    }
    pub fn exceeds_target(&self, target: f64) -> bool {
        self.delta > target
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ServiceConnection {
    pub connection_id: u64,
    pub citizen_did: [u8; 32],
    pub property_id: u64,
    pub utility_type: UtilityType,
    pub meter_id: u64,
    pub node_id: u64,
    pub connection_date_ns: u64,
    pub status: MeterStatus,
    pub monthly_average: f64,
    pub target_compliance: bool,
    pub leak_detected: bool,
    pub last_reading_ns: u64,
    pub billing_account_id: u64,
    pub low_income_assistance: bool,
    pub accessibility_accommodations: bool,
}

pub struct SmartUtilityGrid {
    pub grid_id: u64,
    pub city_code: [u8; 8],
    pub utility_nodes: [Option<UtilityNode>; MAX_UTILITY_NODES],
    pub node_count: usize,
    pub meter_readings: [Option<MeterReading>; MAX_METER_READINGS],
    pub reading_count: usize,
    pub service_connections: [Option<ServiceConnection>; MAX_SERVICE_CONNECTIONS],
    pub connection_count: usize,
    pub total_consumption: [f64; 12],
    pub target_compliance_rate: f64,
    pub leak_detection_rate: f64,
    pub grid_efficiency: f64,
    pub demand_response_participation: f64,
    pub service_interruptions: u64,
    pub average_restoration_time_min: f64,
    pub last_grid_optimization_ns: u64,
    pub audit_checksum: u64,
}

impl SmartUtilityGrid {
    pub fn new(grid_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            grid_id,
            city_code,
            utility_nodes: Default::default(),
            node_count: 0,
            meter_readings: Default::default(),
            reading_count: 0,
            service_connections: Default::default(),
            connection_count: 0,
            total_consumption: [0.0; 12],
            target_compliance_rate: 1.0,
            leak_detection_rate: 0.0,
            grid_efficiency: 1.0,
            demand_response_participation: 0.0,
            service_interruptions: 0,
            average_restoration_time_min: 0.0,
            last_grid_optimization_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_utility_node(&mut self, node: UtilityNode) -> Result<u64, &'static str> {
        if self.node_count >= MAX_UTILITY_NODES { return Err("NODE_LIMIT_EXCEEDED"); }
        self.utility_nodes[self.node_count] = Some(node);
        self.node_count += 1;
        self.update_audit_checksum();
        Ok(node.node_id)
    }
    pub fn record_meter_reading(&mut self, reading: MeterReading) -> Result<u64, &'static str> {
        if self.reading_count >= MAX_METER_READINGS { return Err("READING_LIMIT_EXCEEDED"); }
        self.meter_readings[self.reading_count] = Some(reading);
        self.reading_count += 1;
        self.total_consumption[reading.utility_type as usize] += reading.delta;
        if reading.anomaly_detected {
            self.leak_detection_rate += 0.01;
        }
        self.update_audit_checksum();
        Ok(reading.reading_id)
    }
    pub fn register_service_connection(&mut self, connection: ServiceConnection) -> Result<u64, &'static str> {
        if self.connection_count >= MAX_SERVICE_CONNECTIONS { return Err("CONNECTION_LIMIT_EXCEEDED"); }
        self.service_connections[self.connection_count] = Some(connection);
        self.connection_count += 1;
        self.update_audit_checksum();
        Ok(connection.connection_id)
    }
    pub fn detect_leaks(&mut self, connection_id: u64, now_ns: u64) -> Result<bool, &'static str> {
        for i in 0..self.connection_count {
            if let Some(ref mut conn) = self.service_connections[i] {
                if conn.connection_id == connection_id {
                    let recent_readings = self.meter_readings.iter()
                        .filter_map(|r| r.as_ref())
                        .filter(|r| r.meter_id == conn.meter_id)
                        .take(10)
                        .collect::<Vec<_>>();
                    if recent_readings.len() >= 3 {
                        let avg_delta = recent_readings.iter().map(|r| r.delta).sum::<f64>() / recent_readings.len() as f64;
                        let expected = conn.monthly_average / 30.0;
                        if avg_delta > expected * (1.0 + LEAK_DETECTION_THRESHOLD_PCT) {
                            conn.leak_detected = true;
                            conn.status = MeterStatus::LeakDetected;
                            self.update_audit_checksum();
                            return Ok(true);
                        }
                    }
                    conn.leak_detected = false;
                    return Ok(false);
                }
            }
        }
        Err("CONNECTION_NOT_FOUND")
    }
    pub fn compute_target_compliance(&mut self) -> f64 {
        let mut compliant = 0u64;
        for i in 0..self.connection_count {
            if let Some(ref conn) = self.service_connections[i] {
                if conn.target_compliance { compliant += 1; }
            }
        }
        self.target_compliance_rate = compliant as f64 / self.connection_count.max(1) as f64;
        self.target_compliance_rate
    }
    pub fn compute_grid_efficiency(&mut self) -> f64 {
        let mut total_efficiency = 0.0;
        let mut operational_nodes = 0u64;
        for i in 0..self.node_count {
            if let Some(ref node) = self.utility_nodes[i] {
                if node.operational {
                    total_efficiency += node.efficiency_pct;
                    operational_nodes += 1;
                }
            }
        }
        self.grid_efficiency = if operational_nodes > 0 {
            total_efficiency / operational_nodes as f64 / 100.0
        } else { 0.0 };
        self.grid_efficiency
    }
    pub fn record_service_interruption(&mut self, duration_min: f64, now_ns: u64) {
        self.service_interruptions += 1;
        self.average_restoration_time_min = (self.average_restoration_time_min * 
            (self.service_interruptions - 1) as f64 + duration_min) / self.service_interruptions as f64;
        self.update_audit_checksum();
    }
    pub fn get_grid_status(&self, now_ns: u64) -> UtilityGridStatus {
        let operational_nodes = self.utility_nodes.iter()
            .filter(|n| n.as_ref().map(|node| node.operational).unwrap_or(false))
            .count();
        let nodes_needing_maintenance = self.utility_nodes.iter()
            .filter(|n| n.as_ref().map(|node| node.requires_maintenance(now_ns)).unwrap_or(false))
            .count();
        let leak_connections = self.service_connections.iter()
            .filter(|c| c.as_ref().map(|conn| conn.leak_detected).unwrap_or(false))
            .count();
        UtilityGridStatus {
            grid_id: self.grid_id,
            total_nodes: self.node_count,
            operational_nodes,
            nodes_needing_maintenance,
            total_connections: self.connection_count,
            active_connections: self.connection_count - leak_connections,
            leak_detected_connections: leak_connections,
            total_readings: self.reading_count,
            target_compliance_rate: self.target_compliance_rate,
            leak_detection_rate: self.leak_detection_rate,
            grid_efficiency: self.grid_efficiency,
            service_interruptions: self.service_interruptions,
            average_restoration_time_min: self.average_restoration_time_min,
            last_optimization_ns: self.last_grid_optimization_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.node_count as u64).wrapping_mul(self.connection_count as u64);
        sum ^= self.service_interruptions;
        sum ^= (self.grid_efficiency * 1e6) as u64;
        for i in 0..self.node_count {
            if let Some(ref node) = self.utility_nodes[i] {
                sum ^= node.node_id.wrapping_mul((node.efficiency_pct * 100.0) as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.node_count as u64).wrapping_mul(self.connection_count as u64);
        sum ^= self.service_interruptions;
        sum ^= (self.grid_efficiency * 1e6) as u64;
        for i in 0..self.node_count {
            if let Some(ref node) = self.utility_nodes[i] {
                sum ^= node.node_id.wrapping_mul((node.efficiency_pct * 100.0) as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct UtilityGridStatus {
    pub grid_id: u64,
    pub total_nodes: usize,
    pub operational_nodes: usize,
    pub nodes_needing_maintenance: usize,
    pub total_connections: usize,
    pub active_connections: usize,
    pub leak_detected_connections: usize,
    pub total_readings: usize,
    pub target_compliance_rate: f64,
    pub leak_detection_rate: f64,
    pub grid_efficiency: f64,
    pub service_interruptions: u64,
    pub average_restoration_time_min: f64,
    pub last_optimization_ns: u64,
    pub last_update_ns: u64,
}

impl UtilityGridStatus {
    pub fn utility_reliability_index(&self) -> f64 {
        let availability = self.operational_nodes as f64 / self.total_nodes.max(1) as f64;
        let leak_free_rate = 1.0 - (self.leak_detected_connections as f64 / self.total_connections.max(1) as f64);
        let restoration_score = if self.average_restoration_time_min < 60.0 { 1.0 } 
            else { 60.0 / self.average_restoration_time_min };
        (availability * 0.4 + leak_free_rate * 0.35 + restoration_score * 0.25).min(1.0)
    }
}
