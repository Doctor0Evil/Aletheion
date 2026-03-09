-- Aletheion Medical Emergency Response Router v20260310
-- License: BioticTreaty_v3
-- Compliance: HIPAA_1996_GDPR_2018_Neurorights_v1_Arizona_EMS_Protocols_2026

local MEDICAL_ROUTER_VERSION = 20260310
local MAX_MEDICAL_FACILITIES = 256
local MAX_EMERGENCY_VEHICLES = 1024
local MAX_ACTIVE_INCIDENTS = 512
local TRIAGE_BROADCAST_INTERVAL_S = 30

local MedicalFacilityType = {
    TRAUMA_CENTER_1 = 1, TRAUMA_CENTER_2 = 2, TRAUMA_CENTER_3 = 3,
    EMERGENCY_DEPT = 4, URGENT_CARE = 5, SPECIALTY_CLINIC = 6,
    PSYCHIATRIC = 7, PEDIATRIC = 8, BURN_CENTER = 9, STROKE_CENTER = 10
}

local VehicleType = {
    AMBULANCE_BLS = 1, AMBULANCE_ALS = 2, PARAMEDIC_UNIT = 3,
    AIR_MEDICAL = 4, MOBILE_ICU = 5, PSYCHIATRIC_CRISIS = 6,
    HAZMAT_MEDICAL = 7, MASS_CASUALTY_UNIT = 8
}

local VehicleStatus = {
    AVAILABLE = 1, DISPATCHED = 2, ON_SCENE = 3, TRANSPORTING = 4,
    AT_FACILITY = 5, OFF_DUTY = 6, MAINTENANCE = 7, OUT_OF_SERVICE = 8
}

local TriageCategory = {
    IMMEDIATE_RED = 1, DELAYED_YELLOW = 2, MINOR_GREEN = 3, EXPECTANT_BLACK = 4
}

local MedicalFacility = {}
MedicalFacility.__index = MedicalFacility

function MedicalFacility:new(facility_id, name, facility_type, lat, lon)
    local self = setmetatable({}, MedicalFacility)
    self.facility_id = facility_id or 0
    self.name = name or ""
    self.facility_type = facility_type or 4
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.total_beds = 0
    self.available_beds = 0
    self.icu_beds = 0
    self.available_icu_beds = 0
    self.er_capacity = 0
    self.current_er_patients = 0
    self.trauma_level = 0
    self.specialties = {}
    self.operational = true
    self.diversion_status = false
    self.helipad_available = false
    self.last_update_ns = 0
    return self
end

function MedicalFacility:capacity_utilization()
    if self.total_beds == 0 then return 0.0 end
    return (self.total_beds - self.available_beds) / self.total_beds
end

function MedicalFacility:icu_utilization()
    if self.icu_beds == 0 then return 0.0 end
    return (self.icu_beds - self.available_icu_beds) / self.icu_beds
end

function MedicalFacility:can_accept_patient(acuity_level)
    if not self.operational or self.diversion_status then return false end
    if acuity_level == 1 and self.trauma_level < 2 then return false end
    if self:capacity_utilization() > 0.95 then return false end
    return true
end

function MedicalFacility:update_capacity(available_beds, available_icu, diversion, now_ns)
    self.available_beds = available_beds or self.available_beds
    self.available_icu_beds = available_icu or self.available_icu_beds
    self.diversion_status = diversion or self.diversion_status
    self.last_update_ns = now_ns
end

local EmergencyVehicle = {}
EmergencyVehicle.__index = EmergencyVehicle

function EmergencyVehicle:new(vehicle_id, vehicle_type, unit_number, lat, lon)
    local self = setmetatable({}, EmergencyVehicle)
    self.vehicle_id = vehicle_id or 0
    self.vehicle_type = vehicle_type or 1
    self.unit_number = unit_number or ""
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.status = VehicleStatus.AVAILABLE
    self.current_incident_id = 0
    self.crew_count = 2
    self.paramedic_on_board = false
    self.physician_on_board = false
    self.battery_pct = 100.0
    self.fuel_pct = 100.0
    self.oxygen_level_pct = 100.0
    self.medical_supplies_level_pct = 100.0
    self.last_communication_ns = 0
    self.total_calls_responded = 0
    self.on_duty_since_ns = 0
    return self
end

function EmergencyVehicle:is_available()
    return self.status == VehicleStatus.AVAILABLE and
           self.battery_pct > 20.0 and
           self.fuel_pct > 30.0 and
           self.oxygen_level_pct > 50.0
end

function EmergencyVehicle:dispatch_to_incident(incident_id, dispatch_ns)
    if not self:is_available() then return false end
    self.status = VehicleStatus.DISPATCHED
    self.current_incident_id = incident_id
    self.last_communication_ns = dispatch_ns
    return true
end

function EmergencyVehicle:arrive_on_scene(arrival_ns)
    self.status = VehicleStatus.ON_SCENE
    self.last_communication_ns = arrival_ns
end

function EmergencyVehicle:begin_transport(transport_ns)
    self.status = VehicleStatus.TRANSPORTING
    self.last_communication_ns = transport_ns
end

function EmergencyVehicle:complete_call(completion_ns)
    self.status = VehicleStatus.AVAILABLE
    self.current_incident_id = 0
    self.total_calls_responded = self.total_calls_responded + 1
    self.last_communication_ns = completion_ns
end

local MedicalIncident = {}
MedicalIncident.__index = MedicalIncident

function MedicalIncident:new(incident_id, incident_type, severity, lat, lon, timestamp_ns)
    local self = setmetatable({}, MedicalIncident)
    self.incident_id = incident_id or 0
    self.incident_type = incident_type or 1
    self.severity = severity or 1
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.timestamp_ns = timestamp_ns or 0
    self.status = "OPEN"
    self.patient_count = 0
    self.triage_categories = {[1] = 0, [2] = 0, [3] = 0, [4] = 0}
    self.assigned_vehicles = {}
    self.assigned_vehicle_count = 0
    self.transport_destination_id = 0
    self.estimated_arrival_ns = 0
    self.actual_arrival_ns = 0
    self.notes = ""
    self.last_update_ns = timestamp_ns
    return self
end

function MedicalIncident:add_patient(triage_category)
    self.patient_count = self.patient_count + 1
    self.triage_categories[triage_category] = self.triage_categories[triage_category] + 1
end

function MedicalIncident:assign_vehicle(vehicle_id, assignment_ns)
    if self.assigned_vehicle_count >= 4 then return false end
    self.assigned_vehicles[self.assigned_vehicle_count + 1] = {
        vehicle_id = vehicle_id,
        assigned_ns = assignment_ns
    }
    self.assigned_vehicle_count = self.assigned_vehicle_count + 1
    return true
end

function MedicalIncident:compute_response_time_ns()
    if self.assigned_vehicle_count == 0 then return 0 end
    return self.assigned_vehicles[1].assigned_ns - self.timestamp_ns
end

function MedicalIncident:get_priority_score()
    local priority = self.severity * 10
    priority = priority + self.triage_categories[1] * 5
    priority = priority + self.triage_categories[2] * 3
    priority = priority + (self.patient_count > 5 and 10 or 0)
    return priority
end

local MedicalEmergencyRouter = {}
MedicalEmergencyRouter.__index = MedicalEmergencyRouter

function MedicalEmergencyRouter:new(router_id, city_code, region)
    local self = setmetatable({}, MedicalEmergencyRouter)
    self.router_id = router_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.facilities = {}
    self.facility_count = 0
    self.vehicles = {}
    self.vehicle_count = 0
    self.incidents = {}
    self.incident_count = 0
    self.active_incidents = 0
    self.total_calls_ytd = 0
    self.total_patients_transported = 0
    self.average_response_time_s = 0.0
    self.last_broadcast_ns = 0
    self.system_status = "NORMAL"
    self.alert_level = 0
    return self
end

function MedicalEmergencyRouter:register_facility(facility)
    if self.facility_count >= MAX_MEDICAL_FACILITIES then return false, "FACILITY_LIMIT" end
    self.facilities[self.facility_count + 1] = facility
    self.facility_count = self.facility_count + 1
    return true, "OK"
end

function MedicalEmergencyRouter:register_vehicle(vehicle)
    if self.vehicle_count >= MAX_EMERGENCY_VEHICLES then return false, "VEHICLE_LIMIT" end
    self.vehicles[self.vehicle_count + 1] = vehicle
    self.vehicle_count = self.vehicle_count + 1
    return true, "OK"
end

function MedicalEmergencyRouter:register_incident(incident)
    if self.incident_count >= MAX_ACTIVE_INCIDENTS then return false, "INCIDENT_LIMIT" end
    self.incidents[self.incident_count + 1] = incident
    self.incident_count = self.incident_count + 1
    if incident.status == "OPEN" then
        self.active_incidents = self.active_incidents + 1
        self.total_calls_ytd = self.total_calls_ytd + 1
    end
    return true, "OK"
end

function MedicalEmergencyRouter:compute_distance(lat1, lon1, lat2, lon2)
    local earth_radius_km = 6371.0
    local d_lat = math.rad(lat2 - lat1)
    local d_lon = math.rad(lon2 - lon1)
    local a = math.sin(d_lat / 2) * math.sin(d_lat / 2) +
              math.cos(math.rad(lat1)) * math.cos(math.rad(lat2)) *
              math.sin(d_lon / 2) * math.sin(d_lon / 2)
    local c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    return earth_radius_km * c
end

function MedicalEmergencyRouter:find_nearest_available_vehicles(incident, count, now_ns)
    local candidates = {}
    local candidate_count = 0
    for i = 1, self.vehicle_count do
        local vehicle = self.vehicles[i]
        if vehicle:is_available() then
            local distance = self:compute_distance(
                incident.latitude, incident.longitude,
                vehicle.latitude, vehicle.longitude
            )
            local acuity_match = true
            if incident.severity == 1 and vehicle.vehicle_type ~= VehicleType.PARAMEDIC_UNIT then
                acuity_match = false
            end
            if acuity_match then
                candidates[candidate_count + 1] = {vehicle = vehicle, distance = distance}
                candidate_count = candidate_count + 1
            end
        end
    end
    table.sort(candidates, function(a, b) return a.distance < b.distance end)
    local result = {}
    for i = 1, math.min(count, candidate_count) do
        result[i] = candidates[i].vehicle
    end
    return result
end

function MedicalEmergencyRouter:find_best_facility(incident, vehicle, now_ns)
    local best_facility = nil
    local best_score = -1
    for i = 1, self.facility_count do
        local facility = self.facilities[i]
        if facility:can_accept_patient(incident.severity) then
            local distance = self:compute_distance(
                vehicle.latitude, vehicle.longitude,
                facility.latitude, facility.longitude
            )
            local score = 1000 - distance - (facility:capacity_utilization() * 500)
            if incident.severity == 1 and facility.trauma_level >= 2 then
                score = score + 200
            end
            if score > best_score then
                best_score = score
                best_facility = facility
            end
        end
    end
    return best_facility
end

function MedicalEmergencyRouter:dispatch_vehicles(incident_id, now_ns)
    local incident = nil
    for i = 1, self.incident_count do
        if self.incidents[i].incident_id == incident_id then
            incident = self.incidents[i]
            break
        end
    end
    if not incident or incident.status ~= "OPEN" then return false, "INCIDENT_NOT_FOUND" end
    local vehicle_count_needed = math.ceil(incident.patient_count / 2) + incident.triage_categories[1]
    local vehicles = self:find_nearest_available_vehicles(incident, vehicle_count_needed, now_ns)
    if #vehicles == 0 then return false, "NO_AVAILABLE_VEHICLES" end
    for i, vehicle in ipairs(vehicles) do
        if vehicle:dispatch_to_incident(incident_id, now_ns) then
            incident:assign_vehicle(vehicle.vehicle_id, now_ns)
        end
    end
    return true, "DISPATCHED"
end

function MedicalEmergencyRouter:compute_average_response_time()
    local total_time = 0
    local count = 0
    for i = 1, self.incident_count do
        local incident = self.incidents[i]
        if incident.status == "CLOSED" and incident.assigned_vehicle_count > 0 then
            local response_time_ns = incident:compute_response_time_ns()
            total_time = total_time + response_time_ns
            count = count + 1
        end
    end
    if count == 0 then return 0.0 end
    self.average_response_time_s = (total_time / count) / 1000000000
    return self.average_response_time_s
end

function MedicalEmergencyRouter:determine_system_status(now_ns)
    local critical_incidents = 0
    for i = 1, self.incident_count do
        if self.incidents[i].status == "OPEN" and self.incidents[i].severity >= 4 then
            critical_incidents = critical_incidents + 1
        end
    end
    local available_vehicles = 0
    for i = 1, self.vehicle_count do
        if self.vehicles[i]:is_available() then
            available_vehicles = available_vehicles + 1
        end
    end
    local vehicle_ratio = available_vehicles / math.max(self.active_incidents * 2, 1)
    local facility_capacity = 0
    for i = 1, self.facility_count do
        if self.facilities[i]:can_accept_patient(1) then
            facility_capacity = facility_capacity + 1
        end
    end
    if critical_incidents > 5 or vehicle_ratio < 0.5 or facility_capacity == 0 then
        self.system_status = "CRITICAL"
        self.alert_level = 5
    elseif critical_incidents > 2 or vehicle_ratio < 0.7 then
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

function MedicalEmergencyRouter:generate_situation_report(now_ns)
    self:compute_average_response_time()
    self:determine_system_status(now_ns)
    local available_vehicles = 0
    for i = 1, self.vehicle_count do
        if self.vehicles[i]:is_available() then
            available_vehicles = available_vehicles + 1
        end
    end
    local available_facilities = 0
    for i = 1, self.facility_count do
        if self.facilities[i]:can_accept_patient(1) then
            available_facilities = available_facilities + 1
        end
    end
    return {
        router_id = self.router_id,
        city_code = self.city_code,
        report_timestamp_ns = now_ns,
        system_status = self.system_status,
        alert_level = self.alert_level,
        active_incidents = self.active_incidents,
        total_calls_ytd = self.total_calls_ytd,
        total_vehicles = self.vehicle_count,
        available_vehicles = available_vehicles,
        total_facilities = self.facility_count,
        available_facilities = available_facilities,
        average_response_time_s = self.average_response_time_s,
        total_patients_transported = self.total_patients_transported,
        last_broadcast_ns = self.last_broadcast_ns
    }
end

function MedicalEmergencyRouter:compute_readiness_score()
    local available_vehicles = 0
    for i = 1, self.vehicle_count do
        if self.vehicles[i]:is_available() then
            available_vehicles = available_vehicles + 1
        end
    end
    local vehicle_readiness = available_vehicles / math.max(self.vehicle_count, 1)
    local response_time_score = 1.0
    if self.average_response_time_s > 0 then
        if self.average_response_time_s < 480 then response_time_score = 1.0
        elseif self.average_response_time_s < 720 then response_time_score = 0.8
        elseif self.average_response_time_s < 960 then response_time_score = 0.6
        else response_time_score = 0.4 end
    end
    local incident_load_penalty = math.min(self.active_incidents * 0.02, 0.3)
    return (vehicle_readiness * 0.5 + response_time_score * 0.3 + (1.0 - incident_load_penalty) * 0.2)
end

return {
    MedicalEmergencyRouter = MedicalEmergencyRouter,
    MedicalFacility = MedicalFacility,
    EmergencyVehicle = EmergencyVehicle,
    MedicalIncident = MedicalIncident,
    MedicalFacilityType = MedicalFacilityType,
    VehicleType = VehicleType,
    VehicleStatus = VehicleStatus,
    TriageCategory = TriageCategory,
    VERSION = MEDICAL_ROUTER_VERSION,
}
