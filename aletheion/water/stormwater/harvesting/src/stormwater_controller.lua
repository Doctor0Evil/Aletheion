-- Aletheion Stormwater Harvesting Network Controller v20260310
-- License: BioticTreaty_v3
-- Compliance: Phoenix_Monsoon_Protocol_2026_Arizona_Water_Laws_Indigenous_Water_Rights

local STORMWATER_CONTROLLER_VERSION = 20260310
local MAX_CATCHMENT_AREAS = 1024
local MAX_STORAGE_TANKS = 512
local MAX_CONVEYANCE_CHANNELS = 2048
local MAX_TREATMENT_FACILITIES = 128
local MONSOON_SEASON_START_MONTH = 6
local MONSOON_SEASON_END_MONTH = 9

local WaterQualityGrade = {
    EXCELLENT = 0, GOOD = 1, ACCEPTABLE = 2, MARGINAL = 3, POOR = 4, HAZARDOUS = 5
}

local CatchmentType = {
    ROOFTOP = 1, PAVEMENT = 2, PARK = 3, ROAD = 4,
    DESERT_LAND = 5, AGRICULTURAL = 6, INDUSTRIAL = 7
}

local CatchmentArea = {}
CatchmentArea.__index = CatchmentArea

function CatchmentArea:new(area_id, catchment_type, area_m2, lat, lon)
    local self = setmetatable({}, CatchmentArea)
    self.area_id = area_id or 0
    self.catchment_type = catchment_type or 1
    self.area_m2 = area_m2 or 0.0
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.runoff_coefficient = 0.0
    self.first_flush_diverter = false
    self.pre_treatment = false
    self.contamination_risk = 0.0
    self.last_cleaning_ns = 0
    self.operational = true
    self.cumulative_capture_l = 0.0
    return self
end

function CatchmentArea:compute_runoff_coefficient()
    if self.catchment_type == 1 then self.runoff_coefficient = 0.85
    elseif self.catchment_type == 2 then self.runoff_coefficient = 0.75
    elseif self.catchment_type == 3 then self.runoff_coefficient = 0.30
    elseif self.catchment_type == 4 then self.runoff_coefficient = 0.70
    elseif self.catchment_type == 5 then self.runoff_coefficient = 0.15
    elseif self.catchment_type == 6 then self.runoff_coefficient = 0.40
    elseif self.catchment_type == 7 then self.runoff_coefficient = 0.65
    end
    if self.catchment_type == 4 or self.catchment_type == 7 then
        self.contamination_risk = 0.6
    elseif self.catchment_type == 1 or self.catchment_type == 3 then
        self.contamination_risk = 0.2
    else
        self.contamination_risk = 0.4
    end
    return self.runoff_coefficient
end

function CatchmentArea:estimate_capture_mm(rainfall_mm)
    local capture_l = rainfall_mm * self.area_m2 * self.runoff_coefficient
    if self.first_flush_diverter then
        capture_l = capture_l * 0.92
    end
    self.cumulative_capture_l = self.cumulative_capture_l + capture_l
    return capture_l
end

local StorageTank = {}
StorageTank.__index = StorageTank

function StorageTank:new(tank_id, capacity_l, lat, lon)
    local self = setmetatable({}, StorageTank)
    self.tank_id = tank_id or 0
    self.capacity_l = capacity_l or 0.0
    self.current_volume_l = 0.0
    self.latitude = lat or 0.0
    self.longitude = lon or 0.0
    self.material = "concrete"
    self.covered = true
    self.filtration_system = true
    self.uv_disinfection = false
    self.water_quality_grade = WaterQualityGrade.EXCELLENT
    self.last_inspection_ns = 0
    self.last_cleaning_ns = 0
    self.operational = true
    self.connected_areas = {}
    self.connected_area_count = 0
    return self
end

function StorageTank:fill_volume(volume_l)
    local available = self.capacity_l - self.current_volume_l
    local actual_fill = math.min(volume_l, available)
    self.current_volume_l = self.current_volume_l + actual_fill
    return actual_fill, volume_l - actual_fill
end

function StorageTank:draw_volume(volume_l)
    local actual_draw = math.min(volume_l, self.current_volume_l)
    self.current_volume_l = self.current_volume_l - actual_draw
    return actual_draw
end

function StorageTank:fill_percentage()
    if self.capacity_l == 0 then return 0.0 end
    return self.current_volume_l / self.capacity_l * 100.0
end

local ConveyanceChannel = {}
ConveyanceChannel.__index = ConveyanceChannel

function ConveyanceChannel:new(channel_id, channel_type, length_m, capacity_l_s)
    local self = setmetatable({}, ConveyanceChannel)
    self.channel_id = channel_id or 0
    self.channel_type = channel_type or 1
    self.length_m = length_m or 0.0
    self.capacity_l_s = capacity_l_s or 0.0
    self.current_flow_l_s = 0.0
    self.sediment_accumulation_pct = 0.0
    self.blockage_detected = false
    self.last_maintenance_ns = 0
    self.operational = true
    self.source_id = 0
    self.destination_id = 0
    return self
end

function ConveyanceChannel:flow_capacity_ratio()
    if self.capacity_l_s == 0 then return 0.0 end
    return self.current_flow_l_s / self.capacity_l_s
end

function ConveyanceChannel:requires_maintenance()
    return self.sediment_accumulation_pct > 30.0 or self.blockage_detected
end

local StormwaterHarvestingNetwork = {}
StormwaterHarvestingNetwork.__index = StormwaterHarvestingNetwork

function StormwaterHarvestingNetwork:new(network_id, city_code, watershed_name)
    local self = setmetatable({}, StormwaterHarvestingNetwork)
    self.network_id = network_id or 0
    self.city_code = city_code or ""
    self.watershed_name = watershed_name or ""
    self.catchment_areas = {}
    self.catchment_count = 0
    self.storage_tanks = {}
    self.tank_count = 0
    self.conveyance_channels = {}
    self.channel_count = 0
    self.treatment_facilities = {}
    self.facility_count = 0
    self.total_capture_capacity_l = 0.0
    self.total_storage_capacity_l = 0.0
    self.current_stored_volume_l = 0.0
    self.cumulative_annual_capture_l = 0.0
    self.monsoon_season_active = false
    self.current_rainfall_mm_h = 0.0
    self.flood_risk_level = 0
    self.last_rainfall_event_ns = 0
    return self
end

function StormwaterHarvestingNetwork:register_catchment_area(area)
    if self.catchment_count >= MAX_CATCHMENT_AREAS then return false end
    area:compute_runoff_coefficient()
    self.catchment_areas[self.catchment_count + 1] = area
    self.catchment_count = self.catchment_count + 1
    self.total_capture_capacity_l = self.total_capture_capacity_l + 
                                    area.area_m2 * area.runoff_coefficient * 50.0
    return true
end

function StormwaterHarvestingNetwork:register_storage_tank(tank)
    if self.tank_count >= MAX_STORAGE_TANKS then return false end
    self.storage_tanks[self.tank_count + 1] = tank
    self.tank_count = self.tank_count + 1
    self.total_storage_capacity_l = self.total_storage_capacity_l + tank.capacity_l
    return true
end

function StormwaterHarvestingNetwork:register_conveyance_channel(channel)
    if self.channel_count >= MAX_CONVEYANCE_CHANNELS then return false end
    self.conveyance_channels[self.channel_count + 1] = channel
    self.channel_count = self.channel_count + 1
    return true
end

function StormwaterHarvestingNetwork:is_monsoon_season(month)
    return month >= MONSOON_SEASON_START_MONTH and month <= MONSOON_SEASON_END_MONTH
end

function StormwaterHarvestingNetwork:process_rainfall_event(rainfall_mm_h, duration_h, now_ns, month)
    self.monsoon_season_active = self:is_monsoon_season(month)
    self.current_rainfall_mm_h = rainfall_mm_h
    self.last_rainfall_event_ns = now_ns
    local total_captured = 0.0
    local total_overflow = 0.0
    for i = 1, self.catchment_count do
        local area = self.catchment_areas[i]
        if area.operational then
            local captured, overflow = area:estimate_capture_mm(rainfall_mm_h * duration_h)
            total_captured = total_captured + captured
            total_overflow = total_overflow + overflow
        end
    end
    for i = 1, self.tank_count do
        local tank = self.storage_tanks[i]
        if tank.operational then
            local allocated = total_captured / self.tank_count
            local filled, remaining = tank:fill_volume(allocated)
            total_overflow = total_overflow + remaining
        end
    end
    self.current_stored_volume_l = 0.0
    for i = 1, self.tank_count do
        self.current_stored_volume_l = self.current_stored_volume_l + 
                                       self.storage_tanks[i].current_volume_l
    end
    self.cumulative_annual_capture_l = self.cumulative_annual_capture_l + total_captured
    if rainfall_mm_h > 25.0 then
        self.flood_risk_level = 3
    elseif rainfall_mm_h > 15.0 then
        self.flood_risk_level = 2
    elseif rainfall_mm_h > 5.0 then
        self.flood_risk_level = 1
    else
        self.flood_risk_level = 0
    end
    return total_captured, total_overflow
end

function StormwaterHarvestingNetwork:compute_harvest_efficiency()
    if self.total_capture_capacity_l == 0 then return 0.0 end
    return self.cumulative_annual_capture_l / self.total_capture_capacity_l
end

function StormwaterHarvestingNetwork:compute_storage_utilization()
    if self.total_storage_capacity_l == 0 then return 0.0 end
    return self.current_stored_volume_l / self.total_storage_capacity_l * 100.0
end

function StormwaterHarvestingNetwork:identify_flood_risk_areas()
    local risk_areas = {}
    local count = 0
    for i = 1, self.channel_count do
        local channel = self.conveyance_channels[i]
        if channel:flow_capacity_ratio() > 0.8 or channel.blockage_detected then
            count = count + 1
            risk_areas[count] = {
                channel_id = channel.channel_id,
                flow_ratio = channel:flow_capacity_ratio(),
                blockage = channel.blockage_detected,
                risk_level = "HIGH"
            }
        end
    end
    return risk_areas, count
end

function StormwaterHarvestingNetwork:generate_distribution_plan(demand_l, priority_zones)
    local distribution = {}
    local count = 0
    local remaining = self.current_stored_volume_l
    for i = 1, self.tank_count do
        local tank = self.storage_tanks[i]
        if tank.operational and tank.current_volume_l > 0 and remaining > 0 then
            local allocation = math.min(tank.current_volume_l, demand_l / self.tank_count)
            count = count + 1
            distribution[count] = {
                tank_id = tank.tank_id,
                allocated_l = allocation,
                priority = priority_zones[i] or 1,
                quality_grade = tank.water_quality_grade
            }
            remaining = remaining - allocation
        end
    end
    return distribution, count
end

function StormwaterHarvestingNetwork:get_network_status(now_ns)
    local operational_tanks = 0
    local tanks_near_full = 0
    for i = 1, self.tank_count do
        if self.storage_tanks[i].operational then
            operational_tanks = operational_tanks + 1
            if self.storage_tanks[i]:fill_percentage() > 80.0 then
                tanks_near_full = tanks_near_full + 1
            end
        end
    end
    local channels_requiring_maintenance = 0
    for i = 1, self.channel_count do
        if self.conveyance_channels[i]:requires_maintenance() then
            channels_requiring_maintenance = channels_requiring_maintenance + 1
        end
    end
    return {
        network_id = self.network_id,
        city_code = self.city_code,
        watershed_name = self.watershed_name,
        monsoon_active = self.monsoon_season_active,
        total_catchment_areas = self.catchment_count,
        total_storage_tanks = self.tank_count,
        operational_tanks = operational_tanks,
        tanks_near_full = tanks_near_full,
        total_conveyance_channels = self.channel_count,
        channels_requiring_maintenance = channels_requiring_maintenance,
        total_capture_capacity_l = self.total_capture_capacity_l,
        total_storage_capacity_l = self.total_storage_capacity_l,
        current_stored_volume_l = self.current_stored_volume_l,
        storage_utilization_pct = self:compute_storage_utilization(),
        cumulative_annual_capture_l = self.cumulative_annual_capture_l,
        harvest_efficiency = self:compute_harvest_efficiency(),
        current_rainfall_mm_h = self.current_rainfall_mm_h,
        flood_risk_level = self.flood_risk_level,
        last_rainfall_event_ns = self.last_rainfall_event_ns,
        last_update_ns = now_ns
    }
end

function StormwaterHarvestingNetwork:compute_resilience_score()
    local storage_score = 1.0 - (self.flood_risk_level * 0.15)
    local capacity_score = self:compute_harvest_efficiency()
    local maintenance_score = 1.0 - (self.channel_count > 0 and 
                        self.identify_flood_risk_areas() / self.channel_count or 0)
    local monsoon_bonus = self.monsoon_season_active and 0.1 or 0.0
    return (storage_score * 0.35 + capacity_score * 0.35 + 
            maintenance_score * 0.2 + monsoon_bonus).math.min(1.0)
end

return {
    StormwaterHarvestingNetwork = StormwaterHarvestingNetwork,
    CatchmentArea = CatchmentArea,
    StorageTank = StorageTank,
    ConveyanceChannel = ConveyanceChannel,
    WaterQualityGrade = WaterQualityGrade,
    CatchmentType = CatchmentType,
    VERSION = STORMWATER_CONTROLLER_VERSION,
}
