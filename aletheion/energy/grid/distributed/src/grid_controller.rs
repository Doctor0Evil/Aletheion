#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const GRID_CONTROLLER_VERSION: u32 = 20260310;
pub const MAX_MICROGRIDS: usize = 512;
pub const MAX_ENERGY_NODES: usize = 4096;
pub const MAX_POWER_PURCHASE_AGREEMENTS: usize = 1024;
pub const GRID_FREQUENCY_HZ: f64 = 60.0;
pub const FREQUENCY_TOLERANCE_HZ: f64 = 0.5;
pub const VOLTAGE_NOMINAL_V: f64 = 120.0;
pub const VOLTAGE_TOLERANCE_PCT: f64 = 5.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnergySource {
    SolarPV = 0, Wind = 1, Battery = 2, Grid = 3,
    Generator = 4, Hydrogen = 5, Thermal = 6, Kinetic = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeStatus {
    Online = 0, Offline = 1, Maintenance = 2, Fault = 3,
    Overloaded = 4, Underperforming = 5, Islanded = 6, Syncing = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct EnergyNode {
    pub node_id: u64,
    pub microgrid_id: u32,
    pub energy_source: EnergySource,
    pub status: NodeStatus,
    pub capacity_kw: f64,
    pub current_output_kw: f64,
    pub efficiency_0_1: f64,
    pub battery_soc_pct: f64,
    pub voltage_v: f64,
    pub frequency_hz: f64,
    pub temperature_celsius: f64,
    pub last_maintenance_ns: u64,
    pub deployment_date_ns: u64,
    pub location_zone_id: u32,
}

impl EnergyNode {
    pub fn is_operational(&self, now_ns: u64) -> bool {
        self.status == NodeStatus::Online &&
        self.voltage_v >= VOLTAGE_NOMINAL_V * (1.0 - VOLTAGE_TOLERANCE_PCT / 100.0) &&
        self.voltage_v <= VOLTAGE_NOMINAL_V * (1.0 + VOLTAGE_TOLERANCE_PCT / 100.0) &&
        (self.frequency_hz - GRID_FREQUENCY_HZ).abs() < FREQUENCY_TOLERANCE_HZ &&
        now_ns - self.last_maintenance_ns < 7776000000000000
    }
    pub fn requires_maintenance(&self) -> bool {
        self.status == NodeStatus::Maintenance ||
        self.status == NodeStatus::Fault ||
        self.efficiency_0_1 < 0.7 ||
        self.temperature_celsius > 85.0
    }
    pub fn can_supply_power(&self) -> bool {
        self.is_operational(SystemTime::now_ns()) &&
        self.current_output_kw > 0.0 &&
        (self.energy_source != EnergySource::Battery || self.battery_soc_pct > 20.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Microgrid {
    pub microgrid_id: u32,
    pub zone_name: [u8; 32],
    pub total_capacity_kw: f64,
    pub current_load_kw: f64,
    pub renewable_percentage: f64,
    pub island_capable: bool,
    pub is_islanded: bool,
    pub frequency_hz: f64,
    pub voltage_v: f64,
    pub node_count: usize,
    pub priority_loads: u64,
    pub emergency_reserve_kwh: f64,
}

impl Microgrid {
    pub fn load_balance_ratio(&self) -> f64 {
        if self.total_capacity_kw == 0.0 { return 0.0; }
        self.current_load_kw / self.total_capacity_kw
    }
    pub fn has_sufficient_reserve(&self) -> bool {
        self.emergency_reserve_kwh >= self.current_load_kw * 2.0
    }
    pub fn renewable_score(&self) -> f64 {
        self.renewable_percentage / 100.0
    }
}

#[derive(Clone, Debug)]
pub struct PowerPurchaseAgreement {
    pub ppa_id: u64,
    pub provider_node_id: u64,
    pub consumer_zone_id: u32,
    pub contracted_kw: f64,
    pub price_per_kwh_cents: f64,
    pub start_date_ns: u64,
    pub end_date_ns: u64,
    pub renewable_required: bool,
    pub active: bool,
    pub total_energy_delivered_kwh: f64,
    pub total_payments_cents: f64,
}

impl PowerPurchaseAgreement {
    pub fn is_valid(&self, now_ns: u64) -> bool {
        self.active && now_ns >= self.start_date_ns && now_ns < self.end_date_ns
    }
    pub fn is_expired(&self, now_ns: u64) -> bool {
        now_ns >= self.end_date_ns
    }
}

pub struct DistributedEnergyGrid {
    pub grid_id: u64,
    pub city_code: [u8; 8],
    pub microgrids: [Option<Microgrid>; MAX_MICROGRIDS],
    pub microgrid_count: usize,
    pub energy_nodes: [Option<EnergyNode>; MAX_ENERGY_NODES],
    pub node_count: usize,
    pub ppas: [Option<PowerPurchaseAgreement>; MAX_POWER_PURCHASE_AGREEMENTS],
    pub ppa_count: usize,
    pub total_generation_kw: f64,
    pub total_consumption_kw: f64,
    pub grid_frequency_hz: f64,
    pub grid_voltage_v: f64,
    pub system_inertia_mw_s: f64,
    pub carbon_intensity_g_co2_kwh: f64,
    pub total_energy_generated_kwh: f64,
    pub total_energy_consumed_kwh: f64,
    pub blackout_events: u64,
    pub last_blackout_ns: u64,
    pub audit_checksum: u64,
}

impl DistributedEnergyGrid {
    pub fn new(grid_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            grid_id,
            city_code,
            microgrids: Default::default(),
            microgrid_count: 0,
            energy_nodes: Default::default(),
            node_count: 0,
            ppas: Default::default(),
            ppa_count: 0,
            total_generation_kw: 0.0,
            total_consumption_kw: 0.0,
            grid_frequency_hz: GRID_FREQUENCY_HZ,
            grid_voltage_v: VOLTAGE_NOMINAL_V,
            system_inertia_mw_s: 0.0,
            carbon_intensity_g_co2_kwh: 0.0,
            total_energy_generated_kwh: 0.0,
            total_energy_consumed_kwh: 0.0,
            blackout_events: 0,
            last_blackout_ns: 0,
            audit_checksum: 0,
        }
    }
    pub fn register_microgrid(&mut self, microgrid: Microgrid) -> Result<u32, &'static str> {
        if self.microgrid_count >= MAX_MICROGRIDS {
            return Err("MICROGRID_LIMIT_EXCEEDED");
        }
        self.microgrids[self.microgrid_count] = Some(microgrid);
        self.microgrid_count += 1;
        self.update_audit_checksum();
        Ok(microgrid.microgrid_id)
    }
    pub fn register_energy_node(&mut self, node: EnergyNode) -> Result<u64, &'static str> {
        if self.node_count >= MAX_ENERGY_NODES {
            return Err("NODE_LIMIT_EXCEEDED");
        }
        self.energy_nodes[self.node_count] = Some(node);
        self.node_count += 1;
        self.total_generation_kw += node.current_output_kw;
        self.update_audit_checksum();
        Ok(node.node_id)
    }
    pub fn register_ppa(&mut self, ppa: PowerPurchaseAgreement) -> Result<u64, &'static str> {
        if self.ppa_count >= MAX_POWER_PURCHASE_AGREEMENTS {
            return Err("PPA_LIMIT_EXCEEDED");
        }
        self.ppas[self.ppa_count] = Some(ppa);
        self.ppa_count += 1;
        self.update_audit_checksum();
        Ok(ppa.ppa_id)
    }
    pub fn update_node_output(&mut self, node_id: u64, output_kw: f64, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.node_count {
            if let Some(ref mut node) = self.energy_nodes[i] {
                if node.node_id == node_id {
                    let delta = output_kw - node.current_output_kw;
                    node.current_output_kw = output_kw;
                    self.total_generation_kw += delta;
                    node.last_maintenance_ns = now_ns;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("NODE_NOT_FOUND")
    }
    pub fn compute_load_balance(&mut self, consumption_kw: f64, now_ns: u64) -> (f64, bool) {
        self.total_consumption_kw = consumption_kw;
        let balance = self.total_generation_kw - self.total_consumption_kw;
        let stable = balance.abs() < self.total_generation_kw * 0.05;
        if !stable {
            self.grid_frequency_hz = GRID_FREQUENCY_HZ + (balance / self.system_inertia_mw_s.max(1.0)) * 0.01;
        }
        if (self.grid_frequency_hz - GRID_FREQUENCY_HZ).abs() > FREQUENCY_TOLERANCE_HZ {
            self.blackout_events += 1;
            self.last_blackout_ns = now_ns;
        }
        self.update_audit_checksum();
        (balance, stable)
    }
    pub fn island_microgrid(&mut self, microgrid_id: u32, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.microgrid_count {
            if let Some(ref mut mg) = self.microgrids[i] {
                if mg.microgrid_id == microgrid_id {
                    if !mg.island_capable {
                        return Err("MICROGRID_NOT_ISLAND_CAPABLE");
                    }
                    mg.is_islanded = true;
                    mg.status = NodeStatus::Islanded;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("MICROGRID_NOT_FOUND")
    }
    pub fn reconnect_microgrid(&mut self, microgrid_id: u32, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.microgrid_count {
            if let Some(ref mut mg) = self.microgrids[i] {
                if mg.microgrid_id == microgrid_id {
                    if !mg.is_islanded {
                        return Err("MICROGRID_NOT_ISLANDED");
                    }
                    mg.is_islanded = false;
                    mg.status = NodeStatus::Syncing;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("MICROGRID_NOT_FOUND")
    }
    pub fn get_renewable_percentage(&self) -> f64 {
        let mut renewable_kw = 0.0;
        let mut total_kw = 0.0;
        for i in 0..self.node_count {
            if let Some(ref node) = self.energy_nodes[i] {
                total_kw += node.current_output_kw;
                match node.energy_source {
                    EnergySource::SolarPV | EnergySource::Wind | EnergySource::Hydro | EnergySource::Kinetic => {
                        renewable_kw += node.current_output_kw;
                    }
                    _ => {}
                }
            }
        }
        if total_kw == 0.0 { return 0.0; }
        renewable_kw / total_kw * 100.0
    }
    pub fn compute_carbon_intensity(&mut self) -> f64 {
        let mut weighted_carbon = 0.0;
        let mut total_output = 0.0;
        for i in 0..self.node_count {
            if let Some(ref node) = self.energy_nodes[i] {
                let carbon_factor = match node.energy_source {
                    EnergySource::SolarPV | EnergySource::Wind | EnergySource::Hydro | EnergySource::Kinetic => 0.0,
                    EnergySource::Battery => 50.0,
                    EnergySource::Hydrogen => 10.0,
                    EnergySource::Thermal => 400.0,
                    EnergySource::Generator => 600.0,
                    EnergySource::Grid => 350.0,
                };
                weighted_carbon += node.current_output_kw * carbon_factor;
                total_output += node.current_output_kw;
            }
        }
        if total_output == 0.0 { return 0.0; }
        self.carbon_intensity_g_co2_kwh = weighted_carbon / total_output;
        self.carbon_intensity_g_co2_kwh
    }
    pub fn get_grid_status(&self, now_ns: u64) -> GridStatus {
        let operational_nodes = self.energy_nodes.iter()
            .filter(|n| n.as_ref().map(|node| node.is_operational(now_ns)).unwrap_or(false))
            .count();
        let islanded_microgrids = self.microgrids.iter()
            .filter(|m| m.as_ref().map(|mg| mg.is_islanded).unwrap_or(false))
            .count();
        let active_ppas = self.ppas.iter()
            .filter(|p| p.as_ref().map(|ppa| ppa.is_valid(now_ns)).unwrap_or(false))
            .count();
        GridStatus {
            grid_id: self.grid_id,
            total_nodes: self.node_count,
            operational_nodes,
            total_microgrids: self.microgrid_count,
            islanded_microgrids,
            active_ppas,
            total_generation_kw: self.total_generation_kw,
            total_consumption_kw: self.total_consumption_kw,
            grid_frequency_hz: self.grid_frequency_hz,
            grid_voltage_v: self.grid_voltage_v,
            renewable_percentage: self.get_renewable_percentage(),
            carbon_intensity: self.carbon_intensity_g_co2_kwh,
            blackout_events: self.blackout_events,
            grid_health_score: self.compute_health_score(now_ns),
            last_update_ns: now_ns,
        }
    }
    fn compute_health_score(&self, now_ns: u64) -> f64 {
        let mut score = 1.0;
        let operational_ratio = self.energy_nodes.iter()
            .filter(|n| n.as_ref().map(|node| node.is_operational(now_ns)).unwrap_or(false))
            .count() as f64 / self.node_count.max(1) as f64;
        score *= operational_ratio;
        let frequency_deviation = (self.grid_frequency_hz - GRID_FREQUENCY_HZ).abs();
        if frequency_deviation > FREQUENCY_TOLERANCE_HZ * 0.5 { score *= 0.9; }
        if frequency_deviation > FREQUENCY_TOLERANCE_HZ * 0.8 { score *= 0.8; }
        if self.blackout_events > 0 { score *= 0.95f64.powi(self.blackout_events as i32); }
        score.max(0.0)
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.node_count as u64).wrapping_mul(self.microgrid_count as u64);
        sum ^= (self.total_generation_kw as u64);
        sum ^= (self.total_consumption_kw as u64);
        sum ^= self.blackout_events;
        for i in 0..self.node_count {
            if let Some(ref node) = self.energy_nodes[i] {
                sum ^= node.node_id.wrapping_mul((node.current_output_kw * 100.0) as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.node_count as u64).wrapping_mul(self.microgrid_count as u64);
        sum ^= (self.total_generation_kw as u64);
        sum ^= (self.total_consumption_kw as u64);
        sum ^= self.blackout_events;
        for i in 0..self.node_count {
            if let Some(ref node) = self.energy_nodes[i] {
                sum ^= node.node_id.wrapping_mul((node.current_output_kw * 100.0) as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct GridStatus {
    pub grid_id: u64,
    pub total_nodes: usize,
    pub operational_nodes: usize,
    pub total_microgrids: usize,
    pub islanded_microgrids: usize,
    pub active_ppas: usize,
    pub total_generation_kw: f64,
    pub total_consumption_kw: f64,
    pub grid_frequency_hz: f64,
    pub grid_voltage_v: f64,
    pub renewable_percentage: f64,
    pub carbon_intensity: f64,
    pub blackout_events: u64,
    pub grid_health_score: f64,
    pub last_update_ns: u64,
}

impl GridStatus {
    pub fn reliability_index(&self) -> f64 {
        let availability = self.operational_nodes as f64 / self.total_nodes.max(1) as f64;
        let frequency_stability = if (self.grid_frequency_hz - GRID_FREQUENCY_HZ).abs() < FREQUENCY_TOLERANCE_HZ * 0.5 { 1.0 } else { 0.7 };
        let blackout_penalty = if self.blackout_events == 0 { 1.0 } else { 0.8 };
        (availability * 0.5 + frequency_stability * 0.3 + blackout_penalty * 0.2).min(1.0)
    }
}
