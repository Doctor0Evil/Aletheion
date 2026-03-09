-- Aletheion Emergency Response Coordination System v20260310
-- License: BioticTreaty_v3
-- Compliance: Neurorights_v1, Phoenix_Emergency_Protocol_2026, NIMS_ICS

local EMERGENCY_COORDINATOR_VERSION = 20260310
local MAX_INCIDENTS = 512
local MAX_RESPONDERS = 4096
local MAX_RESOURCES = 2048
local ALERT_BROADCAST_INTERVAL_S = 60

local IncidentSeverity = {
    LOW = 1, MODERATE = 2, HIGH = 3, SEVERE = 4, CRITICAL = 5
}

local IncidentType = {
    MEDICAL = 1, FIRE = 2, HAZMAT = 3, RESCUE = 4,
    FLOOD = 5, HEAT = 6, DUST_STORM = 7, TRAFFIC = 8,
    UTILITY = 9, SECURITY = 10, EVACUATION = 11
}

local ResponderType = {
    PARAMEDIC = 1, FIREFIGHTER = 2, POLICE = 3, HAZMAT_TEAM = 4,
    SEARCH_RESCUE = 5, UTILITY_WORKER = 6, TRANSPORT = 7, COMMAND = 8
}

local ResponderStatus = {
    AVAILABLE = 1, DISPATCHED = 2, ON_SCENE = 3, TRANSPORTING = 4,
    STANDBY = 5, OFF_DUTY = 6, MAINTENANCE = 7
}

local Incident = {}
Incident.__index = Incident

function Incident:new(incident_id, incident_type, severity, lat, lon, timestamp_ns)
    local self = setmetatable({}, Incident)
    self.incident_id = incident_id or 0
    self.incident_type = incident_type or 1
    self.severity = severity or 1
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.timestamp_ns = timestamp_ns or 0
    self.status = "OPEN"
    self.assigned_responders = {}
    self.assigned_responder_count = 0
    self.estimated_resolution_ns = 0
    self.actual_resolution_ns = 0
    self.casualties = 0
    self.transport_required = false
    self.evacuation_zone = ""
    self.hazmat_material = ""
    self.notes = ""
    self.last_update_ns = timestamp_ns
    return self
end

function Incident:assign_responder(responder_id, responder_type)
    if self.assigned_responder_count >= 16 then return false end
    self.assigned_responders[self.assigned_responder_count + 1] = {
        responder_id = responder_id,
        responder_type = responder_type,
        assigned_ns = os.time() * 1000000000
    }
    self.assigned_responder_count = self.assigned_responder_count + 1
    return true
end

function Incident:close_incident(resolution_ns, casualties)
    self.status = "CLOSED"
    self.actual_resolution_ns = resolution_ns
    self.casualties = casualties or 0
    self.last_update_ns = resolution_ns
end

function Incident:compute_response_time_ns()
    if self.assigned_responder_count == 0 then return 0 end
    local first_assignment_ns = self.assigned_responders[1].assigned_ns
    return first_assignment_ns - self.timestamp_ns
end

function Incident:get_priority_score()
    return self.severity * 10 + (self.casualties > 0 and 5 or 0) + 
           (self.transport_required and 3 or 0) + (self.evacuation_zone ~= "" and 5 or 0)
end

local Responder = {}
Responder.__index = Responder

function Responder:new(responder_id, responder_type, unit_id, lat, lon)
    local self = setmetatable({}, Responder)
    self.responder_id = responder_id or 0
    self.responder_type = responder_type or 1
    self.unit_id = unit_id or ""
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.status = ResponderStatus.AVAILABLE
    self.current_incident_id = 0
    self.last_communication_ns = os.time() * 1000000000
    self.battery_pct = 100.0
    self.fuel_pct = 100.0
    self.crew_count = 1
    self.specializations = {}
    self.total_incidents_responded = 0
    self.on_duty_since_ns = 0
    return self
end

function Responder:is_available()
    return self.status == ResponderStatus.AVAILABLE and 
           self.battery_pct > 20.0 and 
           (self.fuel_pct > 30.0 or self.responder_type ~= ResponderType.TRANSPORT)
end

function Responder:dispatch_to_incident(incident_id, dispatch_ns)
    if not self:is_available() then return false end
    self.status = ResponderStatus.DISPATCHED
    self.current_incident_id = incident_id
    self.last_communication_ns = dispatch_ns
    return true
end

function Responder:arrive_on_scene(arrival_ns)
    self.status = ResponderStatus.ON_SCENE
    self.last_communication_ns = arrival_ns
end

function Responder:complete_assignment(completion_ns)
    self.status = ResponderStatus.AVAILABLE
    self.current_incident_id = 0
    self.total_incidents_responded = self.total_incidents_responded + 1
    self.last_communication_ns = completion_ns
end

local EmergencyCoordinator = {}
EmergencyCoordinator.__index = EmergencyCoordinator

function EmergencyCoordinator:new(coordinator_id, city_code, jurisdiction)
    local self = setmetatable({}, EmergencyCoordinator)
    self.coordinator_id = coordinator_id or 0
    self.city_code = city_code or ""
    self.jurisdiction = jurisdiction or ""
    self.incidents = {}
    self.incident_count = 0
    self.responders = {}
    self.responder_count = 0
    self.resources = {}
    self.resource_count = 0
    self.active_incidents = 0
    self.total_incidents_ytd = 0
    self.total_responder_deployments = 0
    self.average_response_time_s = 0.0
    self.last_broadcast_ns = 0
    self.system_status = "NORMAL"
    self.alert_level = 0
    return self
end

function EmergencyCoordinator:register_incident(incident)
    if self.incident_count >= MAX_INCIDENTS then return false, "INCIDENT_LIMIT" end
    self.incidents[self.incident_count + 1] = incident
    self.incident_count = self.incident_count + 1
    if incident.status == "OPEN" then
        self.active_incidents = self.active_incidents + 1
        self.total_incidents_ytd = self.total_incidents_ytd + 1
    end
    return true, "OK"
end

function EmergencyCoordinator:register_responder(responder)
    if self.responder_count >= MAX_RESPONDERS then return false, "RESPONDER_LIMIT" end
    self.responders[self.responder_count + 1] = responder
    self.responder_count = self.responder_count + 1
    return true, "OK"
end

function EmergencyCoordinator:find_nearest_available_responders(incident, count, now_ns)
    local candidates = {}
    local candidate_count = 0
    for i = 1, self.responder_count do
        local responder = self.responders[i]
        if responder:is_available() then
            local distance = self:compute_distance(
                incident.latitude, incident.longitude,
                responder.latitude, responder.longitude
            )
            candidates[candidate_count + 1] = {responder = responder, distance = distance}
            candidate_count = candidate_count + 1
        end
    end
    table.sort(candidates, function(a, b) return a.distance < b.distance end)
    local result = {}
    for i = 1, math.min(count, candidate_count) do
        result[i] = candidates[i].responder
    end
    return result
end

function EmergencyCoordinator:compute_distance(lat1, lon1, lat2, lon2)
    local earth_radius_km = 6371.0
    local d_lat = math.rad(lat2 - lat1)
    local d_lon = math.rad(lon2 - lon1)
    local a = math.sin(d_lat / 2) * math.sin(d_lat / 2) +
              math.cos(math.rad(lat1)) * math.cos(math.rad(lat2)) *
              math.sin(d_lon / 2) * math.sin(d_lon / 2)
    local c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return earth_radius_km * c
end

function EmergencyCoordinator:dispatch_responders(incident_id, now_ns)
    local incident = nil
    for i = 1, self.incident_count do
        if self.incidents[i].incident_id == incident_id then
            incident = self.incidents[i]
            break
        end
    end
    if not incident or incident.status ~= "OPEN" then return false, "INCIDENT_NOT_FOUND" end
    local responder_count_needed = incident.severity * 2
    local responders = self:find_nearest_available_responders(incident, responder_count_needed, now_ns)
    if #responders == 0 then return false, "NO_AVAILABLE_RESPONDERS" end
    for i, responder in ipairs(responders) do
        if responder:dispatch_to_incident(incident_id, now_ns) then
            incident:assign_responder(responder.responder_id, responder.responder_type)
            self.total_responder_deployments = self.total_responder_deployments + 1
        end
    end
    return true, "DISPATCHED"
end

function EmergencyCoordinator:compute_average_response_time()
    local total_time = 0
    local count = 0
    for i = 1, self.incident_count do
        local incident = self.incidents[i]
        if incident.status == "CLOSED" and incident.assigned_responder_count > 0 then
            local response_time_ns = incident:compute_response_time_ns()
            total_time = total_time + response_time_ns
            count = count + 1
        end
    end
    if count == 0 then return 0.0 end
    self.average_response_time_s = (total_time / count) / 1000000000
    return self.average_response_time_s
end

function EmergencyCoordinator:determine_system_status(now_ns)
    local critical_incidents = 0
    for i = 1, self.incident_count do
        if self.incidents[i].status == "OPEN" and self.incidents[i].severity >= 4 then
            critical_incidents = critical_incidents + 1
        end
    end
    local available_responders = 0
    for i = 1, self.responder_count do
        if self.responders[i]:is_available() then
            available_responders = available_responders + 1
        end
    end
    local responder_ratio = available_responders / math.max(self.active_incidents * 3, 1)
    if critical_incidents > 5 or responder_ratio < 0.5 then
        self.system_status = "CRITICAL"
        self.alert_level = 5
    elseif critical_incidents > 2 or responder_ratio < 0.7 then
        self.system_status = "ELEVATED"
        self.alert_level = 4
    elseif self.active_incidents > 20 then
        self.system_status = "HIGH"
        self.alert_level = 3
    elseif self.active_incidents > 10 then
        self.system_status = "MODERATE"
        self.alert_level = 2
    else
        self.system_status = "NORMAL"
        self.alert_level = 1
    end
    return self.system_status, self.alert_level
end

function EmergencyCoordinator:generate_situation_report(now_ns)
    self:compute_average_response_time()
    self:determine_system_status(now_ns)
    local report = {
        coordinator_id = self.coordinator_id,
        city_code = self.city_code,
        report_timestamp_ns = now_ns,
        system_status = self.system_status,
        alert_level = self.alert_level,
        active_incidents = self.active_incidents,
        total_incidents_ytd = self.total_incidents_ytd,
        total_responders = self.responder_count,
        available_responders = 0,
        dispatched_responders = 0,
        average_response_time_s = self.average_response_time_s,
        total_deployments = self.total_responder_deployments
    }
    for i = 1, self.responder_count do
        if self.responders[i].status == ResponderStatus.AVAILABLE then
            report.available_responders = report.available_responders + 1
        elseif self.responders[i].status == ResponderStatus.DISPATCHED or
               self.responders[i].status == ResponderStatus.ON_SCENE then
            report.dispatched_responders = report.dispatched_responders + 1
        end
    end
    return report
end

function EmergencyCoordinator:compute_readiness_score()
    local available_responders = 0
    for i = 1, self.responder_count do
        if self.responders[i]:is_available() then
            available_responders = available_responders + 1
        end
    end
    local responder_readiness = available_responders / math.max(self.responder_count, 1)
    local response_time_score = 1.0
    if self.average_response_time_s > 0 then
        if self.average_response_time_s < 300 then response_time_score = 1.0
        elseif self.average_response_time_s < 600 then response_time_score = 0.8
        elseif self.average_response_time_s < 900 then response_time_score = 0.6
        else response_time_score = 0.4 end
    end
    local incident_load_penalty = math.min(self.active_incidents * 0.02, 0.3)
    return (responder_readiness * 0.5 + response_time_score * 0.3 + (1.0 - incident_load_penalty) * 0.2)
end

return {
    EmergencyCoordinator = EmergencyCoordinator,
    Incident = Incident,
    Responder = Responder,
    IncidentSeverity = IncidentSeverity,
    IncidentType = IncidentType,
    ResponderType = ResponderType,
    ResponderStatus = ResponderStatus,
    VERSION = EMERGENCY_COORDINATOR_VERSION,
}
