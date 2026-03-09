#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const TRAFFIC_CONTROLLER_VERSION: u32 = 20260310;
pub const MAX_INTERSECTIONS: usize = 2048;
pub const MAX_TRAFFIC_SIGNALS: usize = 8192;
pub const OPTIMIZATION_INTERVAL_S: u64 = 60;
pub const EMERGENCY_VEHICLE_PRIORITY_MS: u32 = 500;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SignalState {
    Red = 0, Yellow = 1, Green = 2, FlashingRed = 3, FlashingYellow = 4, Off = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VehicleType {
    Passenger = 0, Commercial = 1, Transit = 2, Emergency = 3,
    Bicycle = 4, Pedestrian = 5, Autonomous = 6, Freight = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct TrafficSignal {
    pub signal_id: u32,
    pub intersection_id: u32,
    pub direction: u8,
    pub current_state: SignalState,
    pub green_duration_s: u32,
    pub yellow_duration_s: u32,
    pub red_duration_s: u32,
    pub state_change_ns: u64,
    pub pedestrian_active: bool,
    pub bicycle_detector: bool,
    pub transit_priority: bool,
    pub operational: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct IntersectionMetrics {
    pub intersection_id: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub approach_volumes: [u16; 4],
    pub queue_lengths_m: [u16; 4],
    pub average_delay_s: f64,
    pub level_of_service: u8,
    pub pedestrian_count: u16,
    pub bicycle_count: u16,
    pub transit_vehicles: u8,
    pub emergency_vehicle_waiting: bool,
    pub last_optimization_ns: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct TrafficFlow {
    pub flow_id: u64,
    pub origin_zone: u32,
    pub destination_zone: u32,
    pub vehicle_type: VehicleType,
    pub vehicle_count: u16,
    pub average_speed_kmh: f64,
    pub density_veh_km: f64,
    pub timestamp_ns: u64,
}

pub struct UrbanTrafficController {
    pub controller_id: u32,
    pub city_code: [u8; 8],
    pub intersections: [Option<IntersectionMetrics>; MAX_INTERSECTIONS],
    pub intersection_count: usize,
    pub traffic_signals: [Option<TrafficSignal>; MAX_TRAFFIC_SIGNALS],
    pub signal_count: usize,
    pub active_flows: [Option<TrafficFlow>; 512],
    pub flow_count: usize,
    pub total_vehicles_processed: u64,
    pub total_pedestrians_served: u64,
    pub emergency_vehicle_priorities: u64,
    pub transit_priority_activations: u64,
    pub average_citywide_delay_s: f64,
    pub system_health_score: f64,
    pub last_optimization_ns: u64,
    pub audit_checksum: u64,
}

impl UrbanTrafficController {
    pub fn new(controller_id: u32, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            controller_id,
            city_code,
            intersections: Default::default(),
            intersection_count: 0,
            traffic_signals: Default::default(),
            signal_count: 0,
            active_flows: Default::default(),
            flow_count: 0,
            total_vehicles_processed: 0,
            total_pedestrians_served: 0,
            emergency_vehicle_priorities: 0,
            transit_priority_activations: 0,
            average_citywide_delay_s: 0.0,
            system_health_score: 1.0,
            last_optimization_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_intersection(&mut self, metrics: IntersectionMetrics) -> Result<u32, &'static str> {
        if self.intersection_count >= MAX_INTERSECTIONS {
            return Err("INTERSECTION_LIMIT_EXCEEDED");
        }
        self.intersections[self.intersection_count] = Some(metrics);
        self.intersection_count += 1;
        self.update_audit_checksum();
        Ok(metrics.intersection_id)
    }
    pub fn register_traffic_signal(&mut self, signal: TrafficSignal) -> Result<u32, &'static str> {
        if self.signal_count >= MAX_TRAFFIC_SIGNALS {
            return Err("SIGNAL_LIMIT_EXCEEDED");
        }
        self.traffic_signals[self.signal_count] = Some(signal);
        self.signal_count += 1;
        self.update_audit_checksum();
        Ok(signal.signal_id)
    }
    pub fn update_intersection_metrics(&mut self, intersection_id: u32, metrics: IntersectionMetrics, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.intersection_count {
            if let Some(ref mut intersection) = self.intersections[i] {
                if intersection.intersection_id == intersection_id {
                    *intersection = metrics;
                    intersection.last_optimization_ns = now_ns;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("INTERSECTION_NOT_FOUND")
    }
    pub fn optimize_signal_timing(&mut self, intersection_id: u32, now_ns: u64) -> Result<[u32; 4], &'static str> {
        for i in 0..self.intersection_count {
            if let Some(ref intersection) = self.intersections[i] {
                if intersection.intersection_id == intersection_id {
                    let mut timings = [30u32, 30, 30, 30];
                    let total_volume: u32 = intersection.approach_volumes.iter().map(|&v| v as u32).sum();
                    if total_volume > 0 {
                        for j in 0..4 {
                            let ratio = intersection.approach_volumes[j] as f32 / total_volume as f32;
                            timings[j] = (ratio * 120.0) as u32;
                            timings[j] = timings[j].max(15).min(90);
                        }
                    }
                    if intersection.emergency_vehicle_waiting {
                        self.emergency_vehicle_priorities += 1;
                        for j in 0..4 {
                            timings[j] = 15;
                        }
                    }
                    if intersection.transit_vehicles > 0 {
                        self.transit_priority_activations += 1;
                    }
                    self.update_audit_checksum();
                    return Ok(timings);
                }
            }
        }
        Err("INTERSECTION_NOT_FOUND")
    }
    pub fn record_traffic_flow(&mut self, flow: TrafficFlow) -> Result<u64, &'static str> {
        if self.flow_count >= 512 {
            return Err("FLOW_RECORD_LIMIT");
        }
        self.active_flows[self.flow_count] = Some(flow);
        self.flow_count += 1;
        self.total_vehicles_processed += flow.vehicle_count as u64;
        self.update_audit_checksum();
        Ok(flow.flow_id)
    }
    pub fn compute_intersection_delay(&self, intersection_id: u32) -> f64 {
        for i in 0..self.intersection_count {
            if let Some(ref intersection) = self.intersections[i] {
                if intersection.intersection_id == intersection_id {
                    let total_volume: u32 = intersection.approach_volumes.iter().map(|&v| v as u32).sum();
                    let total_queue: u32 = intersection.queue_lengths_m.iter().map(|&q| q as u32).sum();
                    if total_volume == 0 { return 0.0; }
                    return (total_queue as f64 / total_volume as f64) * 10.0;
                }
            }
        }
        0.0
    }
    pub fn compute_citywide_average_delay(&mut self) -> f64 {
        if self.intersection_count == 0 { return 0.0; }
        let mut total_delay = 0.0;
        let mut valid_intersections = 0;
        for i in 0..self.intersection_count {
            if let Some(ref intersection) = self.intersections[i] {
                total_delay += intersection.average_delay_s;
                valid_intersections += 1;
            }
        }
        self.average_citywide_delay_s = total_delay / valid_intersections.max(1) as f64;
        self.average_citywide_delay_s
    }
    pub fn activate_emergency_priority(&mut self, intersection_id: u32, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.intersection_count {
            if let Some(ref mut intersection) = self.intersections[i] {
                if intersection.intersection_id == intersection_id {
                    intersection.emergency_vehicle_waiting = true;
                    self.emergency_vehicle_priorities += 1;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("INTERSECTION_NOT_FOUND")
    }
    pub fn get_system_status(&self, now_ns: u64) -> TrafficSystemStatus {
        let mut operational_signals = 0;
        let mut operational_intersections = 0;
        for i in 0..self.signal_count {
            if let Some(ref signal) = self.traffic_signals[i] {
                if signal.operational { operational_signals += 1; }
            }
        }
        for i in 0..self.intersection_count {
            if let Some(ref intersection) = self.intersections[i] {
                if intersection.last_optimization_ns > now_ns - 3600000000000 {
                    operational_intersections += 1;
                }
            }
        }
        TrafficSystemStatus {
            controller_id: self.controller_id,
            total_intersections: self.intersection_count,
            operational_intersections,
            total_signals: self.signal_count,
            operational_signals,
            active_flows: self.flow_count,
            total_vehicles_processed: self.total_vehicles_processed,
            total_pedestrians_served: self.total_pedestrians_served,
            emergency_priorities: self.emergency_vehicle_priorities,
            transit_priorities: self.transit_priority_activations,
            average_delay_s: self.average_citywide_delay_s,
            system_health_score: self.compute_health_score(now_ns),
            last_optimization_ns: self.last_optimization_ns,
        }
    }
    fn compute_health_score(&self, now_ns: u64) -> f64 {
        let mut score = 1.0;
        let signal_operational_ratio = self.traffic_signals.iter()
            .filter(|s| s.as_ref().map(|sig| sig.operational).unwrap_or(false))
            .count() as f64 / self.signal_count.max(1) as f64;
        score *= signal_operational_ratio;
        if self.average_citywide_delay_s > 60.0 { score *= 0.8; }
        if self.emergency_vehicle_priorities > 100 { score *= 0.95; }
        score.max(0.0)
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.intersection_count as u64).wrapping_mul(self.signal_count as u64);
        sum ^= self.total_vehicles_processed;
        sum ^= self.emergency_vehicle_priorities;
        for i in 0..self.intersection_count {
            if let Some(ref intersection) = self.intersections[i] {
                sum ^= intersection.intersection_id as u64;
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.intersection_count as u64).wrapping_mul(self.signal_count as u64);
        sum ^= self.total_vehicles_processed;
        sum ^= self.emergency_vehicle_priorities;
        for i in 0..self.intersection_count {
            if let Some(ref intersection) = self.intersections[i] {
                sum ^= intersection.intersection_id as u64;
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct TrafficSystemStatus {
    pub controller_id: u32,
    pub total_intersections: usize,
    pub operational_intersections: usize,
    pub total_signals: usize,
    pub operational_signals: usize,
    pub active_flows: usize,
    pub total_vehicles_processed: u64,
    pub total_pedestrians_served: u64,
    pub emergency_priorities: u64,
    pub transit_priorities: u64,
    pub average_delay_s: f64,
    pub system_health_score: f64,
    pub last_optimization_ns: u64,
}

impl TrafficSystemStatus {
    pub fn efficiency_index(&self) -> f64 {
        let intersection_health = self.operational_intersections as f64 / self.total_intersections.max(1) as f64;
        let signal_health = self.operational_signals as f64 / self.total_signals.max(1) as f64;
        let delay_penalty = if self.average_delay_s < 30.0 { 1.0 } else { 0.7 };
        (intersection_health * 0.4 + signal_health * 0.4 + delay_penalty * 0.2).min(1.0)
    }
}
