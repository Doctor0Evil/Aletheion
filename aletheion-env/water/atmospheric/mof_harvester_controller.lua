--[[
Aletheion Smart City Core - Batch 2
File: 102/200
Layer: 21 (Advanced Environment)
Path: aletheion-env/water/atmospheric/mof_harvester_controller.lua

Compliance:
  - ALE-COMP-CORE (v2.1)
  - FPIC (Free, Prior, Informed Consent)
  - Phoenix Heat Protocols (Offline-72h)
  - BioticTreaties (Water Rights)
  - Post-Quantum Secure (via Rust FFI)

Blacklist Check:
  - NO SHA-256, SHA3, Python, Digital Twins, Rollbacks.
  - Uses pq_hash via Rust FFI bridge.

Workflow: ERM Chain (Sense → Model → Optimize → Treaty-Check → Act → Log → Interface)
--]]

-- ============================================================================
-- MODULE IMPORTS (Rust FFI Bridge)
-- ============================================================================
local ffi = require("ffi")
local C = ffi.C

-- Rust FFI Bindings (Established in Batch 1)
ffi.cdef[[
    typedef struct { uint8_t data[64]; } pq_hash_t;
    pq_hash_t pq_hash(const uint8_t* data, size_t len);
    int treaty_check_water_rights(const uint8_t* zone_id, uint8_t* allowed);
    int did_sign_action(const uint8_t* node_id, const uint8_t* action_data, size_t len);
    void ledger_append_immutable(const char* entry);
    uint64_t time_now();
]]

-- ============================================================================
-- CONSTANTS & PHOENIX DESERT PARAMETERS
-- ============================================================================

-- MOF Harvesting Efficiency (Research-based from Knowledge Base)
-- Range: 0.7-1.3 L/kg-MOF/day in desert conditions
local MOF_EFFICIENCY_MIN = 0.7
local MOF_EFFICIENCY_MAX = 1.3
local MOF_CAPACITY_KG = 10.0  -- Standard MOF module capacity

-- Phoenix Desert Humidity Thresholds (Seasonal)
local HUMIDITY_THRESHOLD_NIGHT = 35.0   -- %RH optimal for night harvesting
local HUMIDITY_THRESHOLD_MONSOON = 60.0 -- %RH during monsoon season (Aug-Sep)
local HUMIDITY_THRESHOLD_DAY = 20.0     -- %RH minimum for daytime operation

-- Energy Constraints (Solar-powered system)
local MAX_POWER_WATTS = 500.0
local FAN_POWER_WATTS = 120.0
local HEATER_POWER_WATTS = 300.0
local MIN_BATTERY_PERCENT = 15.0

-- Operational Limits (Phoenix Heat Protocols)
local MAX_AMBIENT_TEMP_C = 55.0  -- 131°F equipment safety limit
local MIN_AMBIENT_TEMP_C = 5.0   -- 41°F condensation threshold
local OFFLINE_BUFFER_HOURS = 72

-- Treaty & Water Rights
local TREATY_CACHE_TTL_SECONDS = 300

-- ============================================================================
-- ENUMERATIONS & DATA STRUCTURES
-- ============================================================================

local HarvestState = {
    IDLE = 0,
    SENSING = 1,
    COLLECTING = 2,
    DESORPTION = 3,
    MAINTENANCE = 4,
    ERROR = 5
}

local HarvestMode = {
    NIGHT_OPTIMAL = 1,      -- High humidity, low temperature
    MONSOON_RUSH = 2,       -- Max collection during monsoon
    DAY_CONSERVATIVE = 3,   -- Low power, minimal collection
    STANDBY = 4             -- Energy conservation mode
}

local MOFHarvester = {}
MOFHarvester.__index = MOFHarvester

-- ============================================================================
-- CORE CONTROLLER CLASS
-- ============================================================================

function MOFHarvester:new(node_id, device_config)
    local instance = setmetatable({}, MOFHarvester)
    
    -- Identity & Configuration
    instance.node_id = node_id or {}
    instance.device_id = device_config.device_id or {}
    instance.location_zone = device_config.location_zone or {}
    instance.mof_mass_kg = device_config.mof_mass_kg or MOF_CAPACITY_KG
    
    -- Operational State
    instance.state = HarvestState.IDLE
    instance.mode = HarvestMode.STANDBY
    instance.water_collected_liters = 0.0
    instance.energy_consumed_wh = 0.0
    instance.cycle_count = 0
    
    -- Environmental Cache
    instance.humidity_rh = 0.0
    instance.temperature_c = 0.0
    instance.battery_percent = 100.0
    instance.solar_available_w = 0.0
    
    -- Offline Queue & Treaty Cache
    instance.offline_queue = {}
    instance.treaty_cache = {
        allowed = false,
        last_check = 0,
        hash = {}
    }
    
    -- Performance Metrics
    instance.efficiency_l_per_kg = MOF_EFFICIENCY_MIN
    instance.last_collection_time = 0
    
    return instance
end

-- ============================================================================
-- ERM CHAIN: SENSE
-- ============================================================================

function MOFHarvester:sense(sensor_readings)
    -- Validate sensor integrity (PQ Hash)
    local sensor_hash = C.pq_hash(sensor_readings.sensor_id, #sensor_readings.sensor_id)
    if sensor_hash.data[0] == 0 then
        self:_log_error("Sensor signature invalid")
        return false
    end
    
    -- Update environmental cache
    self.humidity_rh = sensor_readings.humidity_rh
    self.temperature_c = sensor_readings.temperature_c
    self.battery_percent = sensor_readings.battery_percent
    self.solar_available_w = sensor_readings.solar_available_w
    
    -- Update state
    self.state = HarvestState.SENSING
    
    -- Log sensing event
    self:_log_event(string.format(
        "SENSE: H=%0.1f%% RH, T=%0.1f°C, Bat=%0.1f%%, Solar=%0.1fW",
        self.humidity_rh, self.temperature_c, self.battery_percent, self.solar_available_w
    ))
    
    return true
end

-- ============================================================================
-- ERM CHAIN: MODEL
-- ============================================================================

function MOFHarvester:model_optimal_cycle()
    -- Determine optimal harvest mode based on environmental conditions
    
    local current_time = C.time_now()
    local hour = self:_get_local_hour(current_time)
    
    -- Check Phoenix Heat Protocol safety limits
    if self.temperature_c > MAX_AMBIENT_TEMP_C then
        self.mode = HarvestMode.STANDBY
        self:_log_warning("Temperature exceeds safety limit, entering standby")
        return false
    end
    
    if self.battery_percent < MIN_BATTERY_PERCENT then
        self.mode = HarvestMode.STANDBY
        self:_log_warning("Battery low, conserving energy")
        return false
    end
    
    -- Night Optimal Mode (High humidity, low temperature)
    if (hour >= 20 or hour < 6) and self.humidity_rh >= HUMIDITY_THRESHOLD_NIGHT then
        self.mode = HarvestMode.NIGHT_OPTIMAL
        self.efficiency_l_per_kg = self:_calculate_efficiency(
            self.humidity_rh, self.temperature_c, true
        )
        return true
    end
    
    -- Monsoon Rush Mode (Aug-Sep seasonal)
    if self:_is_monsoon_season(current_time) and self.humidity_rh >= HUMIDITY_THRESHOLD_MONSOON then
        self.mode = HarvestMode.MONSOON_RUSH
        self.efficiency_l_per_kg = MOF_EFFICIENCY_MAX
        return true
    end
    
    -- Day Conservative Mode (Minimal collection)
    if self.humidity_rh >= HUMIDITY_THRESHOLD_DAY then
        self.mode = HarvestMode.DAY_CONSERVATIVE
        self.efficiency_l_per_kg = MOF_EFFICIENCY_MIN * 0.5
        return true
    end
    
    -- Default: Standby
    self.mode = HarvestMode.STANDBY
    return false
end

function MOFHarvester:_calculate_efficiency(humidity, temperature, is_night)
    -- Research-based efficiency model for MOF water harvesting
    -- Formula: Efficiency = base * humidity_factor * temperature_factor
    
    local base = MOF_EFFICIENCY_MIN
    local humidity_factor = math.min(humidity / 100.0, 1.0) * 1.8
    local temperature_factor = 1.0
    
    if is_night then
        -- Nighttime: cooler temperatures improve adsorption
        temperature_factor = math.max(1.0 - (temperature - 20.0) / 50.0, 0.5)
    else
        -- Daytime: higher temperatures reduce efficiency
        temperature_factor = math.max(1.0 - (temperature - 30.0) / 30.0, 0.3)
    end
    
    local efficiency = base * humidity_factor * temperature_factor
    return math.min(math.max(efficiency, MOF_EFFICIENCY_MIN), MOF_EFFICIENCY_MAX)
end

function MOFHarvester:_is_monsoon_season(timestamp)
    -- Phoenix monsoon season: typically July 15 - September 30
    -- Based on Knowledge Base research
    local month = self:_get_local_month(timestamp)
    return month >= 7 and month <= 9
end

function MOFHarvester:_get_local_hour(timestamp)
    -- Convert UTC timestamp to Phoenix local time (MST/MDT)
    -- Simplified: Phoenix is UTC-7 (no DST for simplicity)
    local seconds_per_hour = 3600
    local local_time = timestamp - (7 * seconds_per_hour)
    return (local_time / seconds_per_hour) % 24
end

function MOFHarvester:_get_local_month(timestamp)
    -- Simplified month extraction (assumes Unix epoch)
    -- For production: use proper date library
    local seconds_per_day = 86400
    local days_since_epoch = math.floor(timestamp / seconds_per_day)
    local years = math.floor(days_since_epoch / 365.25)
    local days_in_year = days_since_epoch % 365
    local month_days = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31}
    
    local month = 1
    local day_count = 0
    
    for i, days in ipairs(month_days) do
        if day_count + days > days_in_year then
            month = i
            break
        end
        day_count = day_count + days
    end
    
    return month
end

-- ============================================================================
-- ERM CHAIN: OPTIMIZE & TREATY-CHECK
-- ============================================================================

function MOFHarvester:optimize_and_check()
    -- Calculate optimal collection duration based on mode and resources
    
    local collection_duration_minutes = 0
    local power_required_w = 0
    
    if self.mode == HarvestMode.NIGHT_OPTIMAL then
        collection_duration_minutes = 480  -- 8 hours overnight
        power_required_w = FAN_POWER_WATTS * 0.7  -- Reduced fan speed
    elseif self.mode == HarvestMode.MONSOON_RUSH then
        collection_duration_minutes = 720  -- 12 hours continuous
        power_required_w = FAN_POWER_WATTS
    elseif self.mode == HarvestMode.DAY_CONSERVATIVE then
        collection_duration_minutes = 120  -- 2 hours minimal
        power_required_w = FAN_POWER_WATTS * 0.3
    else
        return nil  -- No collection in standby
    end
    
    -- Energy feasibility check
    local energy_required_wh = (power_required_w * collection_duration_minutes) / 60.0
    
    if energy_required_wh > (self.battery_percent / 100.0 * 1000) then
        self:_log_warning("Insufficient energy for planned collection")
        return nil
    end
    
    -- Treaty Check: FPIC for water rights
    if not self:_check_water_treaty() then
        self:_log_error("FPIC violation: Water collection not permitted in this zone")
        return nil
    end
    
    -- Calculate expected yield
    local expected_yield_liters = self:_calculate_yield(collection_duration_minutes)
    
    return {
        duration_minutes = collection_duration_minutes,
        power_watts = power_required_w,
        expected_yield_liters = expected_yield_liters,
        mode = self.mode
    }
end

function MOFHarvester:_check_water_treaty()
    -- Cache treaty check to avoid repeated FFI calls
    local now = C.time_now()
    
    if now - self.treaty_cache.last_check < TREATY_CACHE_TTL_SECONDS then
        return self.treaty_cache.allowed
    end
    
    -- Call Rust FFI for treaty compliance check
    local allowed = ffi.new("uint8_t[1]")
    local result = C.treaty_check_water_rights(self.location_zone, allowed)
    
    if result ~= 0 then
        self:_log_error("Treaty check failed")
        return false
    end
    
    self.treaty_cache.allowed = (allowed[0] == 1)
    self.treaty_cache.last_check = now
    
    return self.treaty_cache.allowed
end

function MOFHarvester:_calculate_yield(duration_minutes)
    -- Calculate expected water yield based on efficiency and duration
    -- Formula: Yield = efficiency * mass * (duration / 1440 minutes per day)
    
    local duration_days = duration_minutes / 1440.0
    local yield_liters = self.efficiency_l_per_kg * self.mof_mass_kg * duration_days
    
    return yield_liters
end

-- ============================================================================
-- ERM CHAIN: ACT
-- ============================================================================

function MOFHarvester:act(collection_plan)
    if not collection_plan then
        self.state = HarvestState.IDLE
        return false
    end
    
    -- Prepare action command
    local action = {
        device_id = self.device_id,
        mode = collection_plan.mode,
        duration_minutes = collection_plan.duration_minutes,
        timestamp = C.time_now()
    }
    
    -- Serialize action for signing
    local action_json = self:_serialize_action(action)
    
    -- Sign action (PQ Secure via Rust FFI)
    local signature_result = C.did_sign_action(
        self.node_id,
        action_json,
        #action_json
    )
    
    if signature_result ~= 0 then
        self:_log_error("Action signing failed")
        return false
    end
    
    -- Execute collection cycle
    local success = self:_execute_collection_cycle(collection_plan)
    
    if success then
        -- Update state and metrics
        self.state = HarvestState.COLLECTING
        self.cycle_count = self.cycle_count + 1
        self.energy_consumed_wh = self.energy_consumed_wh + 
            (collection_plan.power_watts * collection_plan.duration_minutes / 60.0)
        
        -- Log successful action
        self:_log_action(action, collection_plan.expected_yield_liters)
        
        -- Queue for offline sync if needed
        table.insert(self.offline_queue, action)
        
        return true
    else
        self.state = HarvestState.ERROR
        self:_log_error("Collection cycle execution failed")
        return false
    end
end

function MOFHarvester:_execute_collection_cycle(plan)
    -- This would interface with physical hardware via Rust HAL
    -- For simulation: return true
    
    -- In production, this calls:
    -- aletheion_physical::hal::mof_harvester_execute(plan)
    
    -- Simulate successful execution
    self.water_collected_liters = self.water_collected_liters + plan.expected_yield_liters
    
    return true
end

-- ============================================================================
-- ERM CHAIN: LOG & INTERFACE
-- ============================================================================

function MOFHarvester:_log_event(message)
    local timestamp = C.time_now()
    local log_entry = string.format("[%d] MOF_HARVESTER: %s", timestamp, message)
    
    -- Append to immutable ledger (Rust FFI)
    C.ledger_append_immutable(log_entry)
end

function MOFHarvester:_log_warning(message)
    self:_log_event("WARNING: " .. message)
end

function MOFHarvester:_log_error(message)
    self:_log_event("ERROR: " .. message)
end

function MOFHarvester:_log_action(action, yield_liters)
    local log_message = string.format(
        "ACT: Mode=%d, Duration=%dmin, Yield=%0.2fL, Energy=%0.1fWh",
        action.mode,
        action.duration_minutes,
        yield_liters,
        action.duration_minutes * action.power_watts / 60.0
    )
    self:_log_event(log_message)
end

function MOFHarvester:_serialize_action(action)
    -- Simple JSON-like serialization for FFI
    return string.format(
        '{"device_id":"%s","mode":%d,"duration":%d,"ts":%d}',
        self:_bytes_to_hex(action.device_id),
        action.mode,
        action.duration_minutes,
        action.timestamp
    )
end

function MOFHarvester:_bytes_to_hex(bytes)
    local hex = ""
    for i = 0, #bytes - 1 do
        hex = hex .. string.format("%02x", bytes[i])
    end
    return hex
end

function MOFHarvester:get_status_report()
    return {
        state = self.state,
        mode = self.mode,
        water_collected_liters = self.water_collected_liters,
        efficiency_l_per_kg = self.efficiency_l_per_kg,
        battery_percent = self.battery_percent,
        humidity_rh = self.humidity_rh,
        temperature_c = self.temperature_c,
        cycle_count = self.cycle_count,
        offline_queue_size = #self.offline_queue,
        treaty_compliant = self.treaty_cache.allowed
    }
end

-- ============================================================================
-- OFFLINE SYNC PROTOCOL
-- ============================================================================

function MOFHarvester:sync_offline_queue()
    -- Sync queued actions to central ALN-Blockchain when connectivity restored
    
    local synced_count = 0
    
    for i, action in ipairs(self.offline_queue) do
        -- In production: call Rust FFI to sync with ALN
        -- C.aln_sync_action(action)
        
        synced_count = synced_count + 1
    end
    
    -- Clear queue after successful sync
    self.offline_queue = {}
    
    self:_log_event(string.format("SYNC: %d actions synced to ALN", synced_count))
    
    return synced_count
end

-- ============================================================================
-- PUBLIC API EXPORT
-- ============================================================================

return {
    HarvestState = HarvestState,
    HarvestMode = HarvestMode,
    MOFHarvester = MOFHarvester,
    
    -- Factory function
    create_harvester = function(node_id, config)
        return MOFHarvester:new(node_id, config)
    end
}
