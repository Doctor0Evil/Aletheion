-- Aletheion Community Safety Network Coordinator v20260310
-- License: BioticTreaty_v3
-- Compliance: Phoenix_Public_Safety_Protocol_2026_Arizona_Statutes_Privacy_Laws

local SAFETY_COORDINATOR_VERSION = 20260310
local MAX_SAFETY_ZONES = 1024
local MAX_INCIDENT_REPORTS = 65536
local MAX_RESPONDER_UNITS = 8192
local MAX_COMMUNITY_MEMBERS = 131072
local ALERT_BROADCAST_INTERVAL_S = 120

local IncidentSeverity = {
    LOW = 1, MODERATE = 2, ELEVATED = 3, HIGH = 4, CRITICAL = 5
}

local IncidentType = {
    MEDICAL = 1, FIRE = 2, CRIME = 3, TRAFFIC = 4, HAZARD = 5,
    MISSING_PERSON = 6, DOMESTIC = 7, MENTAL_HEALTH = 8, SUBSTANCE = 9,
    NATURAL_DISASTER = 10, UTILITY = 11, ANIMAL = 12
}

local ResponderType = {
    POLICE = 1, FIRE = 2, EMS = 3, COMMUNITY_SAFETY = 4,
    MENTAL_HEALTH = 5, SOCIAL_WORKER = 6, UTILITY = 7, ANIMAL_CONTROL = 8
}

local SafetyZone = {}
SafetyZone.__index = SafetyZone

function SafetyZone:new(zone_id, zone_name, neighborhood, lat, lon)
    local self = setmetatable({}, SafetyZone)
    self.zone_id = zone_id or 0
    self.zone_name = zone_name or ""
    self.neighborhood = neighborhood or ""
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.area_km2 = 0.0
    self.population = 0
    self.community_members = {}
    self.member_count = 0
    self.incident_count_ytd = 0
    self.crime_rate_per_1000 = 0.0
    self.response_time_avg_s = 0.0
    self.safety_score = 100.0
    self.last_incident_ns = 0
    self.active_alerts = 0
    return self
end

function SafetyZone:add_community_member(member_id)
    if self.member_count >= 10000 then return false end
    self.community_members[self.member_count + 1] = member_id
    self.member_count = self.member_count + 1
    return true
end

function SafetyZone:compute_safety_score(now_ns)
    local score = 100.0
    score = score - (self.crime_rate_per_1000 * 2.0)
    score = score - (self.active_alerts * 5.0)
    if self.response_time_avg_s > 600 then score = score - 10.0
    elseif self.response_time_avg_s > 300 then score = score - 5.0 end
    if now_ns - self.last_incident_ns < 86400000000000 then score = score - 10.0 end
    self.safety_score = math.max(0.0, math.min(100.0, score))
    return self.safety_score
end

local IncidentReport = {}
IncidentReport.__index = IncidentReport

function IncidentReport:new(report_id, incident_type, severity, lat, lon, timestamp_ns)
    local self = setmetatable({}, IncidentReport)
    self.report_id = report_id or 0
    self.incident_type = incident_type or 1
    self.severity = severity or 1
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.timestamp_ns = timestamp_ns or 0
    self.status = "OPEN"
    self.safety_zone_id = 0
    self.reporter_id = 0
    self.anonymous = false
    self.assigned_responders = {}
    self.responder_count = 0
    self.resolution_ns = 0
    self.resolution_time_s = 0
    self.follow_up_required = false
    self.community_notification_sent = false
    return self
end

function IncidentReport:assign_responder(responder_id, responder_type)
    if self.responder_count >= 8 then return false end
    self.assigned_responders[self.responder_count + 1] = {
        responder_id = responder_id,
        responder_type = responder_type,
        assigned_ns = os.time() * 1000000000
    }
    self.responder_count = self.responder_count + 1
    return true
end

function IncidentReport:close_incident(resolution_ns)
    self.status = "CLOSED"
    self.resolution_ns = resolution_ns
    self.resolution_time_s = (resolution_ns - self.timestamp_ns) / 1000000000
end

local ResponderUnit = {}
ResponderUnit.__index = ResponderUnit

function ResponderUnit:new(unit_id, responder_type, unit_number, lat, lon)
    local self = setmetatable({}, ResponderUnit)
    self.unit_id = unit_id or 0
    self.responder_type = responder_type or 1
    self.unit_number = unit_number or ""
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.status = "AVAILABLE"
    self.current_incident_id = 0
    self.crew_count = 2
    self.last_communication_ns = 0
    self.total_calls_responded = 0
    self.on_duty_since_ns = 0
    self.certification_level = 3
    self.community_trained = false
    return self
end

function ResponderUnit:is_available()
    return self.status == "AVAILABLE"
end

function ResponderUnit:dispatch_to_incident(incident_id, dispatch_ns)
    if not self:is_available() then return false end
    self.status = "DISPATCHED"
    self.current_incident_id = incident_id
    self.last_communication_ns = dispatch_ns
    return true
end

function ResponderUnit:complete_call(completion_ns)
    self.status = "AVAILABLE"
    self.current_incident_id = 0
    self.total_calls_responded = self.total_calls_responded + 1
    self.last_communication_ns = completion_ns
end

local CommunitySafetyNetwork = {}
CommunitySafetyNetwork.__index = CommunitySafetyNetwork

function CommunitySafetyNetwork:new(network_id, city_code, region)
    local self = setmetatable({}, CommunitySafetyNetwork)
    self.network_id = network_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.safety_zones = {}
    self.zone_count = 0
    self.incident_reports = {}
    self.report_count = 0
    self.responder_units = {}
    self.responder_count = 0
    self.community_members = 0
    self.active_incidents = 0
    self.total_incidents_ytd = 0
    self.average_response_time_s = 0.0
    self.last_broadcast_ns = 0
    self.system_status = "NORMAL"
    self.alert_level = 0
    return self
end

function CommunitySafetyNetwork:register_safety_zone(zone)
    if self.zone_count >= MAX_SAFETY_ZONES then return false, "ZONE_LIMIT" end
    self.safety_zones[self.zone_count + 1] = zone
    self.zone_count = self.zone_count + 1
    self.community_members = self.community_members + zone.member_count
    return true, "OK"
end

function CommunitySafetyNetwork:register_incident_report(report)
    if self.report_count >= MAX_INCIDENT_REPORTS then return false, "REPORT_LIMIT" end
    self.incident_reports[self.report_count + 1] = report
    self.report_count = self.report_count + 1
    if report.status == "OPEN" then
        self.active_incidents = self.active_incidents + 1
        self.total_incidents_ytd = self.total_incidents_ytd + 1
    end
    return true, "OK"
end

function CommunitySafetyNetwork:register_responder_unit(unit)
    if self.responder_count >= MAX_RESPONDER_UNITS then return false, "UNIT_LIMIT" end
    self.responder_units[self.responder_count + 1] = unit
    self.responder_count = self.responder_count + 1
    return true, "OK"
end

function CommunitySafetyNetwork:compute_distance(lat1, lon1, lat2, lon2)
    local earth_radius_km = 6371.0
    local d_lat = math.rad(lat2 - lat1)
    local d_lon = math.rad(lon2 - lon1)
    local a = math.sin(d_lat / 2) * math.sin(d_lat / 2) +
              math.cos(math.rad(lat1)) * math.cos(math.rad(lat2)) *
              math.sin(d_lon / 2) * math.sin(d_lon / 2)
    local c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return earth_radius_km * c
end

function CommunitySafetyNetwork:find_nearest_responders(incident, count, now_ns)
    local candidates = {}
    local candidate_count = 0
    for i = 1, self.responder_count do
        local unit = self.responder_units[i]
        if unit:is_available() then
            local distance = self:compute_distance(
                incident.latitude, incident.longitude,
                unit.latitude, unit.longitude
            )
            local type_match = true
            if incident.incident_type == IncidentType.MEDICAL and 
               unit.responder_type ~= ResponderType.EMS then
                type_match = false
            end
            if incident.incident_type == IncidentType.FIRE and 
               unit.responder_type ~= ResponderType.FIRE then
                type_match = false
            end
            if type_match then
                candidates[candidate_count + 1] = {unit = unit, distance = distance}
                candidate_count = candidate_count + 1
            end
        end
    end
    table.sort(candidates, function(a, b) return a.distance < b.distance end)
    local result = {}
    for i = 1, math.min(count, candidate_count) do
        result[i] = candidates[i].unit
    end
    return result
end

function CommunitySafetyNetwork:dispatch_responders(report_id, now_ns)
    local report = nil
    for i = 1, self.report_count do
        if self.incident_reports[i].report_id == report_id then
            report = self.incident_reports[i]
            break
        end
    end
    if not report or report.status ~= "OPEN" then return false, "REPORT_NOT_FOUND" end
    local responder_count_needed = report.severity * 2
    local responders = self:find_nearest_responders(report, responder_count_needed, now_ns)
    if #responders == 0 then return false, "NO_AVAILABLE_RESPONDERS" end
    for i, unit in ipairs(responders) do
        if unit:dispatch_to_incident(report_id, now_ns) then
            report:assign_responder(unit.unit_id, unit.responder_type)
        end
    end
    return true, "DISPATCHED"
end

function CommunitySafetyNetwork:compute_average_response_time()
    local total_time = 0
    local count = 0
    for i = 1, self.report_count do
        local report = self.incident_reports[i]
        if report.status == "CLOSED" and report.responder_count > 0 then
            total_time = total_time + report.resolution_time_s
            count = count + 1
        end
    end
    if count == 0 then return 0.0 end
    self.average_response_time_s = total_time / count
    return self.average_response_time_s
end

function CommunitySafetyNetwork:determine_system_status(now_ns)
    local critical_incidents = 0
    for i = 1, self.report_count do
        if self.incident_reports[i].status == "OPEN" and 
           self.incident_reports[i].severity >= 4 then
            critical_incidents = critical_incidents + 1
        end
    end
    local available_units = 0
    for i = 1, self.responder_count do
        if self.responder_units[i]:is_available() then
            available_units = available_units + 1
        end
    end
    local unit_ratio = available_units / math.max(self.active_incidents * 2, 1)
    if critical_incidents > 10 or unit_ratio < 0.5 then
        self.system_status = "CRITICAL"
        self.alert_level = 5
    elseif critical_incidents > 5 or unit_ratio < 0.7 then
        self.system_status = "ELEVATED"
        self.alert_level = 4
    elseif self.active_incidents > 50 then
        self.system_status = "HIGH"
        self.alert_level = 3
    elseif self.active_incidents > 20 then
        self.system_status = "MODERATE"
        self.alert_level = 2
    else
        self.system_status = "NORMAL"
        self.alert_level = 1
    end
    return self.system_status, self.alert_level
end

function CommunitySafetyNetwork:generate_situation_report(now_ns)
    self:compute_average_response_time()
    self:determine_system_status(now_ns)
    local available_units = 0
    for i = 1, self.responder_count do
        if self.responder_units[i]:is_available() then
            available_units = available_units + 1
        end
    end
    local zone_safety_scores = {}
    for i = 1, self.zone_count do
        self.safety_zones[i]:compute_safety_score(now_ns)
        zone_safety_scores[i] = self.safety_zones[i].safety_score
    end
    local avg_zone_safety = 0.0
    for i = 1, #zone_safety_scores do
        avg_zone_safety = avg_zone_safety + zone_safety_scores[i]
    end
    if self.zone_count > 0 then avg_zone_safety = avg_zone_safety / self.zone_count end
    return {
        network_id = self.network_id,
        city_code = self.city_code,
        report_timestamp_ns = now_ns,
        system_status = self.system_status,
        alert_level = self.alert_level,
        active_incidents = self.active_incidents,
        total_incidents_ytd = self.total_incidents_ytd,
        total_responder_units = self.responder_count,
        available_units = available_units,
        total_safety_zones = self.zone_count,
        average_zone_safety_score = avg_zone_safety,
        community_members = self.community_members,
        average_response_time_s = self.average_response_time_s,
        last_broadcast_ns = self.last_broadcast_ns
    }
end

function CommunitySafetyNetwork:compute_readiness_score()
    local available_units = 0
    for i = 1, self.responder_count do
        if self.responder_units[i]:is_available() then
            available_units = available_units + 1
        end
    end
    local unit_readiness = available_units / math.max(self.responder_count, 1)
    local response_time_score = 1.0
    if self.average_response_time_s > 0 then
        if self.average_response_time_s < 300 then response_time_score = 1.0
        elseif self.average_response_time_s < 600 then response_time_score = 0.8
        elseif self.average_response_time_s < 900 then response_time_score = 0.6
        else response_time_score = 0.4 end
    end
    local incident_load_penalty = math.min(self.active_incidents * 0.01, 0.3)
    return (unit_readiness * 0.5 + response_time_score * 0.3 + (1.0 - incident_load_penalty) * 0.2)
end

return {
    CommunitySafetyNetwork = CommunitySafetyNetwork,
    SafetyZone = SafetyZone,
    IncidentReport = IncidentReport,
    ResponderUnit = ResponderUnit,
    IncidentSeverity = IncidentSeverity,
    IncidentType = IncidentType,
    ResponderType = ResponderType,
    VERSION = SAFETY_COORDINATOR_VERSION,
}
