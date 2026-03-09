#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const IOT_CONTROLLER_VERSION: u32 = 20260310;
pub const MAX_DEVICE_NODES: usize = 4096;
pub const MAX_DEVICE_TYPES: usize = 256;
pub const HEALTH_CHECK_INTERVAL_S: u64 = 300;
pub const FIRMWARE_UPDATE_TIMEOUT_S: u64 = 1800;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeviceType {
    Sensor = 0, Actuator = 1, Gateway = 2, Controller = 3, Meter = 4,
    Camera = 5, Beacon = 6, Valve = 7, Pump = 8, Light = 9,
    HvacUnit = 10, SolarPanel = 11, BatteryPack = 12, WaterQuality = 13,
    AirQuality = 14, TrafficSignal = 15, FloodSensor = 16, HeatSensor = 17,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeviceStatus {
    Offline = 0, Online = 1, Maintenance = 2, Fault = 3,
    Updating = 4, Calibrating = 5, LowPower = 6, Compromised = 7,
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceIdentity {
    pub device_id: u64,
    pub device_type: DeviceType,
    pub manufacturer_id: u32,
    pub model_number: u32,
    pub serial_number: [u8; 32],
    pub firmware_version: u32,
    pub hardware_revision: u8,
    pub manufacturing_date_ns: u64,
    pub deployment_date_ns: u64,
    pub location_zone_id: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceHealth {
    pub status: DeviceStatus,
    pub battery_pct: f64,
    pub signal_strength_dbm: i16,
    pub uptime_s: u64,
    pub last_heartbeat_ns: u64,
    pub error_count: u32,
    pub warning_count: u32,
    pub temperature_celsius: f64,
    pub humidity_pct: f64,
}

impl DeviceHealth {
    pub fn is_healthy(&self, now_ns: u64) -> bool {
        self.status == DeviceStatus::Online &&
        self.battery_pct > 20.0 &&
        self.signal_strength_dbm > -90 &&
        now_ns - self.last_heartbeat_ns < HEALTH_CHECK_INTERVAL_S * 1000000000 &&
        self.error_count < 10
    }
    pub fn requires_maintenance(&self) -> bool {
        self.status == DeviceStatus::Maintenance ||
        self.status == DeviceStatus::Fault ||
        self.battery_pct < 30.0 ||
        self.error_count > 5
    }
}

#[derive(Clone, Debug)]
pub struct DeviceCommand {
    pub command_id: u64,
    pub device_id: u64,
    pub command_type: u8,
    pub payload: [u8; 256],
    pub payload_len: usize,
    pub priority: u8,
    pub created_ns: u64,
    pub expires_ns: u64,
    pub executed: bool,
    pub execution_result: i8,
}

impl DeviceCommand {
    pub fn is_valid(&self, now_ns: u64) -> bool {
        !self.executed && now_ns < self.expires_ns && now_ns >= self.created_ns
    }
    pub fn is_expired(&self, now_ns: u64) -> bool {
        now_ns >= self.expires_ns
    }
}

#[derive(Clone, Debug)]
pub struct DeviceRecord {
    pub identity: DeviceIdentity,
    pub health: DeviceHealth,
    pub capabilities: u64,
    pub configuration: [u8; 512],
    pub config_version: u32,
    pub pending_commands: [Option<DeviceCommand>; 8],
    pub pending_command_count: usize,
    pub total_commands_executed: u64,
    pub last_configuration_update_ns: u64,
}

pub struct IoTDeviceManager {
    pub manager_id: u64,
    pub city_code: [u8; 8],
    pub devices: [Option<DeviceRecord>; MAX_DEVICE_NODES],
    pub device_count: usize,
    pub command_queue: [Option<DeviceCommand>; 1024],
    pub queue_head: usize,
    pub queue_tail: usize,
    pub queue_size: usize,
    pub next_command_id: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub security_alerts: u64,
    pub audit_checksum: u64,
}

impl IoTDeviceManager {
    pub fn new(manager_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            manager_id,
            city_code,
            devices: Default::default(),
            device_count: 0,
            command_queue: Default::default(),
            queue_head: 0,
            queue_tail: 0,
            queue_size: 0,
            next_command_id: 1,
            total_messages_sent: 0,
            total_messages_received: 0,
            security_alerts: 0,
            audit_checksum: 0,
        }
    }
    pub fn register_device(&mut self, identity: DeviceIdentity, now_ns: u64) -> Result<u64, &'static str> {
        if self.device_count >= MAX_DEVICE_NODES {
            return Err("DEVICE_LIMIT_EXCEEDED");
        }
        let device_record = DeviceRecord {
            identity,
            health: DeviceHealth {
                status: DeviceStatus::Online,
                battery_pct: 100.0,
                signal_strength_dbm: -50,
                uptime_s: 0,
                last_heartbeat_ns: now_ns,
                error_count: 0,
                warning_count: 0,
                temperature_celsius: 25.0,
                humidity_pct: 50.0,
            },
            capabilities: 0,
            configuration: [0u8; 512],
            config_version: 1,
            pending_commands: Default::default(),
            pending_command_count: 0,
            total_commands_executed: 0,
            last_configuration_update_ns: now_ns,
        };
        self.devices[self.device_count] = Some(device_record);
        self.device_count += 1;
        self.update_audit_checksum();
        Ok(identity.device_id)
    }
    pub fn update_device_health(&mut self, device_id: u64, health: DeviceHealth, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.device_count {
            if let Some(ref mut device) = self.devices[i] {
                if device.identity.device_id == device_id {
                    device.health = health;
                    device.health.last_heartbeat_ns = now_ns;
                    if health.status == DeviceStatus::Compromised {
                        self.security_alerts += 1;
                    }
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("DEVICE_NOT_FOUND")
    }
    pub fn enqueue_command(&mut self, mut command: DeviceCommand) -> Result<u64, &'static str> {
        if self.queue_size >= 1024 {
            return Err("COMMAND_QUEUE_FULL");
        }
        command.command_id = self.next_command_id;
        self.next_command_id += 1;
        self.command_queue[self.queue_tail] = Some(command);
        self.queue_tail = (self.queue_tail + 1) % 1024;
        self.queue_size += 1;
        self.total_messages_sent += 1;
        self.update_audit_checksum();
        Ok(command.command_id)
    }
    pub fn dequeue_command(&mut self) -> Option<DeviceCommand> {
        if self.queue_size == 0 { return None; }
        let command = self.command_queue[self.queue_head].take();
        self.queue_head = (self.queue_head + 1) % 1024;
        self.queue_size -= 1;
        command
    }
    pub fn execute_command_for_device(&mut self, device_id: u64, command: DeviceCommand, now_ns: u64) -> Result<i8, &'static str> {
        if !command.is_valid(now_ns) {
            return Err("COMMAND_INVALID_OR_EXPIRED");
        }
        for i in 0..self.device_count {
            if let Some(ref mut device) = self.devices[i] {
                if device.identity.device_id == device_id {
                    if device.health.status != DeviceStatus::Online {
                        return Err("DEVICE_NOT_ONLINE");
                    }
                    if device.pending_command_count >= 8 {
                        return Err("DEVICE_COMMAND_LIMIT");
                    }
                    device.pending_commands[device.pending_command_count] = Some(command);
                    device.pending_command_count += 1;
                    self.update_audit_checksum();
                    return Ok(0);
                }
            }
        }
        Err("DEVICE_NOT_FOUND")
    }
    pub fn mark_command_executed(&mut self, device_id: u64, command_id: u64, result: i8, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.device_count {
            if let Some(ref mut device) = self.devices[i] {
                if device.identity.device_id == device_id {
                    for j in 0..device.pending_command_count {
                        if let Some(ref cmd) = device.pending_commands[j] {
                            if cmd.command_id == command_id {
                                device.pending_commands[j].as_mut().unwrap().executed = true;
                                device.pending_commands[j].as_mut().unwrap().execution_result = result;
                                device.total_commands_executed += 1;
                                if result != 0 {
                                    device.health.error_count += 1;
                                }
                                self.update_audit_checksum();
                                return Ok(());
                            }
                        }
                    }
                    return Err("COMMAND_NOT_FOUND");
                }
            }
        }
        Err("DEVICE_NOT_FOUND")
    }
    pub fn get_healthy_device_count(&self, now_ns: u64) -> usize {
        let mut count = 0;
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices[i] {
                if device.health.is_healthy(now_ns) {
                    count += 1;
                }
            }
        }
        count
    }
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<u64> {
        let mut result = Vec::new();
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices[i] {
                if device.identity.device_type == device_type {
                    result.push(device.identity.device_id);
                }
            }
        }
        result
    }
    pub fn get_devices_requiring_maintenance(&self) -> Vec<u64> {
        let mut result = Vec::new();
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices[i] {
                if device.health.requires_maintenance() {
                    result.push(device.identity.device_id);
                }
            }
        }
        result
    }
    pub fn compute_network_health_score(&self, now_ns: u64) -> f64 {
        if self.device_count == 0 { return 0.0; }
        let healthy_count = self.get_healthy_device_count(now_ns) as f64;
        let health_ratio = healthy_count / self.device_count as f64;
        let security_penalty = (self.security_alerts as f64 * 0.01).min(0.3);
        let queue_penalty = if self.queue_size > 500 { 0.1 } else { 0.0 };
        (health_ratio * 0.7 - security_penalty - queue_penalty).max(0.0)
    }
    pub fn get_manager_status(&self, now_ns: u64) -> ManagerStatus {
        let healthy_devices = self.get_healthy_device_count(now_ns);
        let maintenance_required = self.get_devices_requiring_maintenance().len();
        let offline_devices = self.device_count - healthy_devices - maintenance_required;
        ManagerStatus {
            manager_id: self.manager_id,
            total_devices: self.device_count,
            healthy_devices,
            maintenance_required,
            offline_devices,
            pending_commands: self.queue_size,
            security_alerts: self.security_alerts,
            network_health_score: self.compute_network_health_score(now_ns),
            total_messages_sent: self.total_messages_sent,
            total_messages_received: self.total_messages_received,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.device_count as u64).wrapping_mul(self.security_alerts);
        sum ^= self.total_messages_sent.wrapping_mul(self.total_messages_received);
        sum ^= self.queue_size as u64;
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices[i] {
                sum ^= device.identity.device_id.wrapping_mul(device.total_commands_executed);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.device_count as u64).wrapping_mul(self.security_alerts);
        sum ^= self.total_messages_sent.wrapping_mul(self.total_messages_received);
        sum ^= self.queue_size as u64;
        for i in 0..self.device_count {
            if let Some(ref device) = self.devices[i] {
                sum ^= device.identity.device_id.wrapping_mul(device.total_commands_executed);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct ManagerStatus {
    pub manager_id: u64,
    pub total_devices: usize,
    pub healthy_devices: usize,
    pub maintenance_required: usize,
    pub offline_devices: usize,
    pub pending_commands: usize,
    pub security_alerts: u64,
    pub network_health_score: f64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub last_update_ns: u64,
}

impl ManagerStatus {
    pub fn operational_readiness(&self) -> f64 {
        let device_readiness = self.healthy_devices as f64 / self.total_devices.max(1) as f64;
        let queue_readiness = if self.pending_commands < 100 { 1.0 } else { 0.5 };
        let security_readiness = if self.security_alerts == 0 { 1.0 } else { 0.7 };
        (device_readiness * 0.5 + queue_readiness * 0.3 + security_readiness * 0.2).min(1.0)
    }
}
