-- Aletheion Multi-Hazard Emergency Coordination Center v20260310
-- License: BioticTreaty_v3
-- Compliance: NIMS_ICS_FEMA_Arizona_Emergency_Management_2026_BioticTreaty_v3

local EMERGENCY_COORDINATION_VERSION = 20260310
local MAX_EMERGENCY_OPERATIONS = 256
local MAX_RESOURCE_INVENTORIES = 4096
local MAX_DEPLOYED_UNITS = 16384
local MAX_SHELTER_LOCATIONS = 1024
local INCIDENT_COMMAND_SYSTEM_VERSION = "2026-AZ"

local EmergencyType = {
    HEAT = 1, FLOOD = 2, DUST_STORM = 3, EARTHQUAKE = 4, WILDFIRE = 5,
    HAZMAT = 6, MEDICAL = 7, TERRORIST = 8, CIVIL_UNREST = 9, UTILITY = 10,
    TRANSPORTATION = 11, STRUCTURAL = 12, WEATHER = 13, MULTI_HAZARD = 14
}

local EmergencyLevel = {
    NORMAL = 0, LEVEL_1 = 1, LEVEL_2 = 2, LEVEL_3 = 3, LEVEL_4 = 4, LEVEL_5 = 5
}

local ResourceCategory = {
    PERSONNEL = 1, EQUIPMENT = 2, VEHICLES = 3, MEDICAL = 4,
    SHELTER = 5, FOOD_WATER = 6, COMMUNICATIONS = 7, POWER = 8,
    TRANSPORTATION = 9, SPECIALIZED = 10
}

local EmergencyOperation = {}
EmergencyOperation.__index = EmergencyOperation

function EmergencyOperation:new(operation_id, emergency_type, level, title)
    local self = setmetatable({}, EmergencyOperation)
    self.operation_id = operation_id or 0
    self.emergency_type = emergency_type or 1
    self.emergency_level = level or 0
    self.title = title or ""
    self.status = "PLANNING"
    self.activated_at_ns = 0
    self.deactivated_at_ns = 0
    self.incident_commander = ""
    self.operations_chief = ""
    self.planning_chief = ""
    self.logistics_chief = ""
    self.finance_chief = ""
    self.public_information_officer = ""
    self.safety_officer = ""
    self.liaison_officer = ""
    self.affected_zones = {}
    self.affected_zone_count = 0
    self.resource_requests = {}
    self.resource_request_count = 0
    self.deployed_units = {}
    self.deployed_unit_count = 0
    self.casualties_reported = 0
    self.fatalities_reported = 0
    self.structures_damaged = 0
    self.economic_loss_usd = 0.0
    self.evacuation_ordered = false
    self.evacuation_count = 0
    self.shelters_opened = 0
    self.last_situation_report_ns = 0
    return self
end

function EmergencyOperation:activate(now_ns, incident_commander)
    if self.status ~= "PLANNING" then return false end
    self.status = "ACTIVE"
    self.activated_at_ns = now_ns
    self.incident_commander = incident_commander
    self.last_situation_report_ns = now_ns
    return true
end

function EmergencyOperation:deactivate(now_ns)
    if self.status ~= "ACTIVE" then return false end
    self.status = "DEACTIVATED"
    self.deactivated_at_ns = now_ns
    return true
end

function EmergencyOperation:add_resource_request(request)
    if self.resource_request_count >= 100 then return false end
    self.resource_requests[self.resource_request_count + 1] = request
    self.resource_request_count = self.resource_request_count + 1
    return true
end

function EmergencyOperation:deploy_unit(unit)
    if self.deployed_unit_count >= 500 then return false end
    self.deployed_units[self.deployed_unit_count + 1] = unit
    self.deployed_unit_count = self.deployed_unit_count + 1
    return true
end

function EmergencyOperation:duration_hours(now_ns)
    local end_time = self.deactivated_at_ns > 0 and self.deactivated_at_ns or now_ns
    return (end_time - self.activated_at_ns) / 3600000000000
end

local ResourceInventory = {}
ResourceInventory.__index = ResourceInventory

function ResourceInventory:new(inventory_id, category, name, quantity)
    local self = setmetatable({}, ResourceInventory)
    self.inventory_id = inventory_id or 0
    self.category = category or 1
    self.name = name or ""
    self.total_quantity = quantity or 0
    self.available_quantity = quantity or 0
    self.deployed_quantity = 0
    self.maintenance_quantity = 0
    self.location = ""
    self.condition = "OPERATIONAL"
    self.last_inspection_ns = 0
    self.next_maintenance_ns = 0
    self.operational = true
    self.emergency_ready = true
    return self
end

function ResourceInventory:deploy(quantity)
    if quantity > self.available_quantity then return false end
    self.available_quantity = self.available_quantity - quantity
    self.deployed_quantity = self.deployed_quantity + quantity
    return true
end

function ResourceInventory:return_from_deployment(quantity)
    if quantity > self.deployed_quantity then return false end
    self.deployed_quantity = self.deployed_quantity - quantity
    self.available_quantity = self.available_quantity + quantity
    return true
end

function ResourceInventory:availability_ratio()
    if self.total_quantity == 0 then return 0.0 end
    return self.available_quantity / self.total_quantity
end

local ShelterLocation = {}
ShelterLocation.__index = ShelterLocation

function ShelterLocation:new(shelter_id, name, latitude, longitude, capacity)
    local self = setmetatable({}, ShelterLocation)
    self.shelter_id = shelter_id or 0
    self.name = name or ""
    self.latitude = latitude or 0.0
    self.longitude = longitude or 0.0
    self.capacity = capacity or 0
    self.current_occupancy = 0
    self.status = "CLOSED"
    self.operational = true
    self.accessibility_compliant = true
    self.medical_support_available = false
    self.pet_friendly = false
    self.power_backup_available = false
    self.water_supply_days = 0
    self.food_supply_days = 0
    self.staff_count = 0
    self.last_inspection_ns = 0
    self.opened_at_ns = 0
    self.closed_at_ns = 0
    return self
end

function ShelterLocation:open(now_ns)
    if not self.operational then return false end
    self.status = "OPEN"
    self.opened_at_ns = now_ns
    return true
end

function ShelterLocation:close(now_ns)
    self.status = "CLOSED"
    self.closed_at_ns = now_ns
    self.current_occupancy = 0
    return true
end

function ShelterLocation:admit_occupants(count)
    local available = self.capacity - self.current_occupancy
    if count > available then return false end
    self.current_occupancy = self.current_occupancy + count
    return true
end

function ShelterLocation:occupancy_ratio()
    if self.capacity == 0 then return 0.0 end
    return self.current_occupancy / self.capacity
end

local EmergencyCoordinationCenter = {}
EmergencyCoordinationCenter.__index = EmergencyCoordinationCenter

function EmergencyCoordinationCenter:new(center_id, city_code, region)
    local self = setmetatable({}, EmergencyCoordinationCenter)
    self.center_id = center_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.emergency_operations = {}
    self.operation_count = 0
    self.resource_inventories = {}
    self.inventory_count = 0
    self.shelter_locations = {}
    self.shelter_count = 0
    self.active_operations = 0
    self.total_operations_ytd = 0
    self.total_resources_deployed = 0
    self.total_shelter_days = 0
    self.average_response_time_min = 0.0
    self.last_major_incident_ns = 0
    self.readiness_level = EmergencyLevel.NORMAL
    return self
end

function EmergencyCoordinationCenter:register_emergency_operation(operation)
    if self.operation_count >= MAX_EMERGENCY_OPERATIONS then return false, "OPERATION_LIMIT" end
    self.emergency_operations[self.operation_count + 1] = operation
    self.operation_count = self.operation_count + 1
    self.total_operations_ytd = self.total_operations_ytd + 1
    if operation.status == "ACTIVE" then
        self.active_operations = self.active_operations + 1
    end
    return true, "OK"
end

function EmergencyCoordinationCenter:register_resource_inventory(inventory)
    if self.inventory_count >= MAX_RESOURCE_INVENTORIES then return false, "INVENTORY_LIMIT" end
    self.resource_inventories[self.inventory_count + 1] = inventory
    self.inventory_count = self.inventory_count + 1
    return true, "OK"
end

function EmergencyCoordinationCenter:register_shelter_location(shelter)
    if self.shelter_count >= MAX_SHELTER_LOCATIONS then return false, "SHELTER_LIMIT" end
    self.shelter_locations[self.shelter_count + 1] = shelter
    self.shelter_count = self.shelter_count + 1
    return true, "OK"
end

function EmergencyCoordinationCenter:activate_operation(operation_id, incident_commander, now_ns)
    for i = 1, self.operation_count do
        if self.emergency_operations[i].operation_id == operation_id then
            local success = self.emergency_operations[i]:activate(now_ns, incident_commander)
            if success then
                self.active_operations = self.active_operations + 1
                if self.emergency_operations[i].emergency_level >= 4 then
                    self.last_major_incident_ns = now_ns
                end
            end
            return success
        end
    end
    return false
end

function EmergencyCoordinationCenter:deploy_resource(operation_id, inventory_id, quantity, now_ns)
    local operation = nil
    local inventory = nil
    for i = 1, self.operation_count do
        if self.emergency_operations[i].operation_id == operation_id then
            operation = self.emergency_operations[i]
            break
        end
    end
    for i = 1, self.inventory_count do
        if self.resource_inventories[i].inventory_id == inventory_id then
            inventory = self.resource_inventories[i]
            break
        end
    end
    if not operation or not inventory then return false end
    if not inventory:deploy(quantity) then return false end
    operation:add_resource_request({
        inventory_id = inventory_id,
        quantity = quantity,
        requested_ns = now_ns,
        fulfilled_ns = now_ns,
        status = "FULFILLED"
    })
    self.total_resources_deployed = self.total_resources_deployed + quantity
    return true
end

function EmergencyCoordinationCenter:open_shelter(shelter_id, now_ns)
    for i = 1, self.shelter_count do
        if self.shelter_locations[i].shelter_id == shelter_id then
            return self.shelter_locations[i]:open(now_ns)
        end
    end
    return false
end

function EmergencyCoordinationCenter:compute_resource_readiness()
    local total_available = 0
    local total_capacity = 0
    for i = 1, self.inventory_count do
        local inv = self.resource_inventories[i]
        if inv.operational and inv.emergency_ready then
            total_available = total_available + inv.available_quantity
            total_capacity = total_capacity + inv.total_quantity
        end
    end
    if total_capacity == 0 then return 0.0 end
    return total_available / total_capacity
end

function EmergencyCoordinationCenter:compute_shelter_capacity()
    local total_capacity = 0
    local total_occupancy = 0
    local open_shelters = 0
    for i = 1, self.shelter_count do
        local shelter = self.shelter_locations[i]
        if shelter.status == "OPEN" then
            open_shelters = open_shelters + 1
            total_capacity = total_capacity + shelter.capacity
            total_occupancy = total_occupancy + shelter.current_occupancy
        end
    end
    return total_capacity, total_occupancy, open_shelters
end

function EmergencyCoordinationCenter:determine_readiness_level(now_ns)
    local resource_readiness = self:compute_resource_readiness()
    local active_ops = self.active_operations
    if active_ops >= 5 or resource_readiness < 0.5 then
        self.readiness_level = EmergencyLevel.LEVEL_5
    elseif active_ops >= 3 or resource_readiness < 0.6 then
        self.readiness_level = EmergencyLevel.LEVEL_4
    elseif active_ops >= 2 or resource_readiness < 0.7 then
        self.readiness_level = EmergencyLevel.LEVEL_3
    elseif active_ops >= 1 or resource_readiness < 0.8 then
        self.readiness_level = EmergencyLevel.LEVEL_2
    elseif resource_readiness < 0.9 then
        self.readiness_level = EmergencyLevel.LEVEL_1
    else
        self.readiness_level = EmergencyLevel.NORMAL
    end
    return self.readiness_level
end

function EmergencyCoordinationCenter:generate_situation_report(now_ns)
    self:determine_readiness_level(now_ns)
    local total_capacity, total_occupancy, open_shelters = self:compute_shelter_capacity()
    local resource_readiness = self:compute_resource_readiness()
    return {
        center_id = self.center_id,
        city_code = self.city_code,
        region = self.region,
        report_timestamp_ns = now_ns,
        readiness_level = self.readiness_level,
        active_operations = self.active_operations,
        total_operations_ytd = self.total_operations_ytd,
        total_resource_inventories = self.inventory_count,
        resource_readiness = resource_readiness,
        total_resources_deployed = self.total_resources_deployed,
        total_shelters = self.shelter_count,
        open_shelters = open_shelters,
        total_shelter_capacity = total_capacity,
        current_shelter_occupancy = total_occupancy,
        total_shelter_days = self.total_shelter_days,
        average_response_time_min = self.average_response_time_min,
        last_major_incident_ns = self.last_major_incident_ns,
        last_report_ns = now_ns
    }
end

function EmergencyCoordinationCenter:compute_emergency_resilience_index(now_ns)
    local resource_readiness = self:compute_resource_readiness()
    local _, occupancy, open_shelters = self:compute_shelter_capacity()
    local shelter_readiness = open_shelters / self.shelter_count.max(1)
    local operational_readiness = 1.0 - (self.active_operations / 10.0).math.min(1.0)
    local response_factor = self.average_response_time_min < 30 and 1.0 or (30.0 / self.average_response_time_min).math.min(1.0)
    return (resource_readiness * 0.35 + shelter_readiness * 0.25 + 
            operational_readiness * 0.20 + response_factor * 0.20)
end

return {
    EmergencyCoordinationCenter = EmergencyCoordinationCenter,
    EmergencyOperation = EmergencyOperation,
    ResourceInventory = ResourceInventory,
    ShelterLocation = ShelterLocation,
    EmergencyType = EmergencyType,
    EmergencyLevel = EmergencyLevel,
    ResourceCategory = ResourceCategory,
    VERSION = EMERGENCY_COORDINATION_VERSION,
}
