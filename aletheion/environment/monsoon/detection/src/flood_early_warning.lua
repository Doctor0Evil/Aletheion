-- Aletheion Monsoon Flash Flood Early Warning System v20260310
-- License: BioticTreaty_v3
-- Compliance: Neurorights_v1, ISO14851, Phoenix_Monsoon_Protocol_2026

local FLOOD_WARNING_VERSION = 20260310
local MAX_SENSOR_NODES = 512
local MAX_WATERSHED_ZONES = 128
local ALERT_BROADCAST_INTERVAL_S = 300

local RiskCoordinate = {}
RiskCoordinate.__index = RiskCoordinate

function RiskCoordinate:new(id, rx, threshold, timestamp_ns)
    local self = setmetatable({}, RiskCoordinate)
    self.id = id or 0
    self.rx = rx or 0.0
    self.threshold = threshold or 1.0
    self.timestamp_ns = timestamp_ns or 0
    return self
end

function RiskCoordinate:violated()
    return self.rx > self.threshold
end

local SensorNode = {}
SensorNode.__index = SensorNode

function SensorNode:new(node_id, lat, lon)
    local self = setmetatable({}, SensorNode)
    self.node_id = node_id or 0
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.elevation_m = 0.0
    self.rainfall_mm_h = 0.0
    self.water_level_m = 0.0
    self.flow_velocity_ms = 0.0
    self.soil_moisture_pct = 0.0
    self.battery_pct = 100.0
    self.signal_strength_dbm = -50
    self.last_transmission_ns = 0
    self.active = true
    self.calibrated = false
    return self
end

function SensorNode:update_readings(rainfall, water_level, flow_vel, soil_moist, now_ns)
    self.rainfall_mm_h = rainfall or 0.0
    self.water_level_m = water_level or 0.0
    self.flow_velocity_ms = flow_vel or 0.0
    self.soil_moisture_pct = soil_moist or 0.0
    self.last_transmission_ns = now_ns
end

function SensorNode:compute_flood_risk(baseline_water_level)
    local risk = 0.0
    if self.rainfall_mm_h > 50.0 then risk = risk + 0.3 end
    if self.rainfall_mm_h > 100.0 then risk = risk + 0.2 end
    if baseline_water_level > 0 then
        local water_rise = (self.water_level_m - baseline_water_level) / baseline_water_level
        if water_rise > 0.5 then risk = risk + 0.3 end
        if water_rise > 1.0 then risk = risk + 0.2 end
    end
    if self.flow_velocity_ms > 2.0 then risk = risk + 0.2 end
    if self.soil_moisture_pct > 80.0 then risk = risk + 0.1 end
    return math.min(risk, 1.0)
end

function SensorNode:is_operational(now_ns)
    local elapsed_s = (now_ns - self.last_transmission_ns) / 1000000000
    return self.active and self.calibrated and 
           self.battery_pct > 10.0 and elapsed_s < 3600
end

local WatershedZone = {}
WatershedZone.__index = WatershedZone

function WatershedZone:new(zone_id, name)
    local self = setmetatable({}, WatershedZone)
    self.zone_id = zone_id or 0
    self.name = name or ""
    self.sensor_nodes = {}
    self.node_count = 0
    self.baseline_water_level_m = 0.0
    self.flood_threshold_m = 0.0
    self.evacuation_route = ""
    self.shelter_location = ""
    self.current_risk_rx = 0.0
    self.alert_level = 0
    self.last_alert_ns = 0
    return self
end

function WatershedZone:add_sensor_node(node)
    if self.node_count >= 64 then return false end
    self.sensor_nodes[self.node_count + 1] = node
    self.node_count = self.node_count + 1
    return true
end

function WatershedZone:compute_aggregate_risk(now_ns)
    local total_risk = 0.0
    local active_nodes = 0
    for i = 1, self.node_count do
        local node = self.sensor_nodes[i]
        if node:is_operational(now_ns) then
            total_risk = total_risk + node:compute_flood_risk(self.baseline_water_level_m)
            active_nodes = active_nodes + 1
        end
    end
    if active_nodes == 0 then return 0.0 end
    self.current_risk_rx = total_risk / active_nodes
    return self.current_risk_rx
end

function WatershedZone:determine_alert_level()
    if self.current_risk_rx >= 0.8 then
        self.alert_level = 4
        return "CRITICAL_EVACUATE"
    elseif self.current_risk_rx >= 0.6 then
        self.alert_level = 3
        return "SEVERE_WARNING"
    elseif self.current_risk_rx >= 0.4 then
        self.alert_level = 2
        return "MODERATE_ALERT"
    elseif self.current_risk_rx >= 0.2 then
        self.alert_level = 1
        return "LOW_ADVISORY"
    else
        self.alert_level = 0
        return "NORMAL"
    end
end

local FloodEarlyWarningSystem = {}
FloodEarlyWarningSystem.__index = FloodEarlyWarningSystem

function FloodEarlyWarningSystem:new(system_id)
    local self = setmetatable({}, FloodEarlyWarningSystem)
    self.system_id = system_id or 0
    self.watershed_zones = {}
    self.zone_count = 0
    self.all_sensor_nodes = {}
    self.total_sensor_count = 0
    self.alert_history = {}
    self.alert_count = 0
    self.monsoon_season_active = false
    self.season_start_month = 6
    self.season_end_month = 9
    return self
end

function FloodEarlyWarningSystem:register_watershed_zone(zone)
    if self.zone_count >= MAX_WATERSHED_ZONES then return false end
    self.watershed_zones[self.zone_count + 1] = zone
    self.zone_count = self.zone_count + 1
    return true
end

function FloodEarlyWarningSystem:is_monsoon_season(month)
    return month >= self.season_start_month and month <= self.season_end_month
end

function FloodEarlyWarningSystem:process_all_sensors(now_ns, month)
    self.monsoon_season_active = self:is_monsoon_season(month)
    local critical_zones = {}
    local critical_count = 0
    for i = 1, self.zone_count do
        local zone = self.watershed_zones[i]
        local risk = zone:compute_aggregate_risk(now_ns)
        local alert = zone:determine_alert_level()
        if zone.alert_level >= 3 then
            critical_count = critical_count + 1
            critical_zones[critical_count] = zone.zone_id
        end
        if zone.alert_level > 0 and (now_ns - zone.last_alert_ns) > ALERT_BROADCAST_INTERVAL_S * 1000000000 then
            self.alert_count = self.alert_count + 1
            self.alert_history[self.alert_count] = {
                zone_id = zone.zone_id,
                alert_level = zone.alert_level,
                alert_type = alert,
                risk_rx = zone.current_risk_rx,
                timestamp_ns = now_ns,
                broadcast_sent = true
            }
            zone.last_alert_ns = now_ns
        end
    end
    return critical_zones, critical_count
end

function FloodEarlyWarningSystem:generate_evacuation_recommendations(critical_zones)
    local recommendations = {}
    local rec_count = 0
    for i = 1, #critical_zones do
        local zone_id = critical_zones[i]
        for j = 1, self.zone_count do
            local zone = self.watershed_zones[j]
            if zone.zone_id == zone_id then
                rec_count = rec_count + 1
                recommendations[rec_count] = {
                    zone_id = zone.zone_id,
                    zone_name = zone.name,
                    evacuation_route = zone.evacuation_route,
                    shelter_location = zone.shelter_location,
                    urgency = zone.alert_level >= 4 and "IMMEDIATE" or "SOON",
                    risk_level = zone.current_risk_rx
                }
                break
            end
        end
    end
    return recommendations, rec_count
end

function FloodEarlyWarningSystem:get_system_status(now_ns)
    local operational_sensors = 0
    local total_sensors = 0
    for i = 1, self.zone_count do
        local zone = self.watershed_zones[i]
        for j = 1, zone.node_count do
            total_sensors = total_sensors + 1
            if zone.sensor_nodes[j]:is_operational(now_ns) then
                operational_sensors = operational_sensors + 1
            end
        end
    end
    local avg_risk = 0.0
    for i = 1, self.zone_count do
        avg_risk = avg_risk + self.watershed_zones[i].current_risk_rx
    end
    if self.zone_count > 0 then avg_risk = avg_risk / self.zone_count end
    return {
        system_id = self.system_id,
        monsoon_active = self.monsoon_season_active,
        total_zones = self.zone_count,
        total_sensors = total_sensors,
        operational_sensors = operational_sensors,
        sensor_health_pct = operational_sensors / math.max(total_sensors, 1) * 100.0,
        average_risk_rx = avg_risk,
        total_alerts_issued = self.alert_count,
        last_update_ns = now_ns
    }
end

function FloodEarlyWarningSystem:compute_readiness_score()
    local sensor_health = 0.0
    local zone_coverage = 0.0
    local alert_response = 0.0
    local total_sensors = 0
    local operational_sensors = 0
    for i = 1, self.zone_count do
        local zone = self.watershed_zones[i]
        for j = 1, zone.node_count do
            total_sensors = total_sensors + 1
            if zone.sensor_nodes[j].active and zone.sensor_nodes[j].calibrated then
                operational_sensors = operational_sensors + 1
            end
        end
        if zone.evacuation_route ~= "" and zone.shelter_location ~= "" then
            zone_coverage = zone_coverage + 1
        end
    end
    sensor_health = operational_sensors / math.max(total_sensors, 1)
    zone_coverage = zone_coverage / math.max(self.zone_count, 1)
    if self.alert_count > 0 then
        local successful_broadcasts = 0
        for i = 1, self.alert_count do
            if self.alert_history[i].broadcast_sent then
                successful_broadcasts = successful_broadcasts + 1
            end
        end
        alert_response = successful_broadcasts / self.alert_count
    else
        alert_response = 1.0
    end
    return sensor_health * 0.4 + zone_coverage * 0.4 + alert_response * 0.2
end

return {
    FloodEarlyWarningSystem = FloodEarlyWarningSystem,
    WatershedZone = WatershedZone,
    SensorNode = SensorNode,
    RiskCoordinate = RiskCoordinate,
    VERSION = FLOOD_WARNING_VERSION,
}
