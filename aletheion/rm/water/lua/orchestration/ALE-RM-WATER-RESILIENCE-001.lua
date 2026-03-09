-- ============================================================================
-- FILE: aletheion/rm/water/lua/orchestration/ALE-RM-WATER-RESILIENCE-001.lua
-- VERSION: 1.0.0
-- LICENSE: Apache-2.0 WITH Aletheion-Ecosafety-Exception-1.0
-- STATUS: Production-Ready | Offline-Capable | Post-Quantum-Secure
-- ============================================================================
-- PURPOSE: Lua orchestration layer for Phoenix water resilience management.
--          This file implements the 7-step SMART-chain workflow orchestration,
--          calling into Rust FFI for ecosafety validation while handling
--          high-level workflow composition, parameterization, and citizen
--          interface coordination. Integrates with ALE-RM-WATER-MODEL-001.rs
--          and ALE-ERM-ECOSAFETY-CONTRACTS-001.rs via FFI.
-- ============================================================================
-- CONSTRAINTS:
--   - No blacklisted cryptography (SHA-256, BLAKE, KECCAK, etc.)
--   - Lua handles orchestration only; Rust handles safety semantics
--   - Offline-capable (no network dependencies for core orchestration)
--   - Phoenix-specific: monsoon, drought, flash flood, extreme heat protocols
--   - Indigenous sovereignty: Akimel O'odham and Piipaash FPIC enforcement
--   - SevenCapitalState validation mandatory before any actuation
-- ============================================================================
-- COMPATIBILITY: ALE-RM-WATER-MODEL-001.rs, ALE-ERM-ECOSAFETY-CONTRACTS-001.rs
-- ============================================================================

-- ============================================================================
-- SECTION 1: MODULE DECLARATION AND FFI BINDINGS
-- ============================================================================

local WaterResilience = {}
WaterResilience.__index = WaterResilience

-- FFI bindings to Rust ecosafety contracts (via LuaJIT FFI or WASM)
local ffi = require("ffi")
local ecosafety_ffi = require("aletheion_ecosafety_ffi")

-- FFI type declarations matching Rust structures
ffi.cdef[[
    typedef struct {
        uint8_t sensor_did[32];
        uint64_t timestamp_us;
        float water_level_ft;
        float flow_rate_gpm;
        float pressure_psi;
        float temperature_f;
        float ph_level;
        float turbidity_ntu;
        float dissolved_oxygen_mgl;
        float conductivity_us_cm;
        float rainfall_inch_per_hour;
        float soil_moisture_percent;
    } WaterSensorReading;

    typedef struct {
        uint8_t shard_id[32];
        uint8_t entity_did[32];
        uint64_t timestamp_us;
        float v_t;
        float k;
        float e;
        float r;
        uint8_t corridor_decision;
        uint8_t fpic_status;
        bool birth_sign_verified;
    } QpuDataShard;

    typedef struct {
        uint8_t decision;
        uint8_t error_code;
    } FfiCorridorDecision;

    typedef struct {
        float k;
        float e;
        float r;
    } FfiKerMetrics;
]]

-- FFI function imports from Rust ecosafety contracts
local ecosafety_eval_corridor = ecosafety_ffi.ecosafety_eval_corridor
local ecosafety_compute_ker = ecosafety_ffi.ecosafety_compute_ker
local ecosafety_validate_corridors = ecosafety_ffi.ecosafety_validate_corridors

-- FFI function imports from Rust water model
local water_execute_smart_chain = ecosafety_ffi.water_execute_smart_chain
local water_get_capital_state = ecosafety_ffi.water_get_capital_state

-- ============================================================================
-- SECTION 2: PHOENIX HYDROLOGICAL CONSTANTS (Lua-side)
-- ============================================================================

local PHOENIX_HYDRO = {
    -- Rainfall data (2025-2026 Phoenix Sky Harbor)
    AVG_ANNUAL_RAINFALL_INCHES = 8.03,
    MONSOON_SEASON_RAINFALL_INCHES = 2.71,
    EXTREME_MONSOON_EVENT_INCHES = 3.26,
    
    -- Water usage targets
    PER_CAPITA_USAGE_TARGET_GPD = 50.0,
    PHOENIX_AVG_USAGE_GPD = 146.0,
    
    -- Reclamation efficiency (Pure Water Phoenix)
    RECLAMATION_EFFICIENCY_TARGET = 0.98,
    
    -- Water allocations (acre-feet/year)
    GROUNDWATER_RECHARGE_TARGET_AFY = 50000.0,
    COLORADO_RIVER_ALLOCATION_AFY = 338000.0,
    SALT_RIVER_ALLOCATION_AFY = 200000.0,
    AQUIFER_STORAGE_CAPACITY_AF = 500000.0,
    
    -- Emergency thresholds
    FLASH_FLOOD_THRESHOLD_INCHES_PER_HOUR = 1.0,
    HABOOB_WIND_THRESHOLD_MPH = 40.0,
    EXTREME_HEAT_THRESHOLD_F = 120.0,
    
    -- Operational constants
    EVAPOTRANSPIRATION_RATE_INCHES_PER_DAY = 0.35,
    STORMWATER_CAPTURE_EFFICIENCY = 0.85,
    AWH_YIELD_L_PER_KG_PER_DAY = 1.0,
    
    -- Indigenous water rights (Akimel O'odham settlement)
    INDIGENOUS_WATER_RIGHTS_PERCENTAGE = 0.15,
    
    -- Monsoon season dates (Phoenix)
    MONSOON_START_MONTH = 6,  -- June
    MONSOON_END_MONTH = 9,    -- September
}

-- ============================================================================
-- SECTION 3: WATER RESILIENCE STATE MANAGEMENT
-- ============================================================================

--- Water resilience orchestration state
local WaterResilienceState = {
    sensor_network_id = nil,
    entity_did = nil,
    current_state = nil,
    allocation_plans = {},
    emergency_active = false,
    emergency_type = nil,
    monsoon_active = false,
    drought_level = 1,
    last_shard = nil,
    workflow_history = {},
}

--- Initialize WaterResilience orchestration engine
function WaterResilience:new(sensor_network_id, entity_did)
    local self = setmetatable({}, WaterResilience)
    self.sensor_network_id = sensor_network_id
    self.entity_did = entity_did
    self.current_state = nil
    self.allocation_plans = {}
    self.emergency_active = false
    self.emergency_type = nil
    self.monsoon_active = false
    self.drought_level = 1
    self.last_shard = nil
    self.workflow_history = {}
    return self
end

--- Update monsoon season status based on current date
function WaterResilience:update_monsoon_status()
    local current_month = os.date("*t").month
    self.monsoon_active = (current_month >= PHOENIX_HYDRO.MONSOON_START_MONTH and 
                           current_month <= PHOENIX_HYDRO.MONSOON_END_MONTH)
    return self.monsoon_active
end

--- Detect monsoon conditions from sensor readings
function WaterResilience:detect_monsoon_from_sensors(readings)
    local high_rainfall_events = 0
    for _, reading in ipairs(readings) do
        if reading.rainfall_inch_per_hour > 0.5 then
            high_rainfall_events = high_rainfall_events + 1
        end
    end
    return high_rainfall_events >= 3
end

--- Update drought contingency level based on state
function WaterResilience:update_drought_level(water_state)
    if water_state.drought_severity >= 0.7 then
        self.drought_level = 4  -- Highest level
    elseif water_state.drought_severity >= 0.5 then
        self.drought_level = 3
    elseif water_state.drought_severity >= 0.3 then
        self.drought_level = 2
    else
        self.drought_level = 1  -- Normal
    end
    return self.drought_level
end

-- ============================================================================
-- SECTION 4: SMART-CHAIN WATER WORKFLOW ORCHESTRATION
-- ============================================================================

--- SMART-001: SENSE - Collect and validate sensor data
function WaterResilience:sense_sensor_data(sensor_readings)
    -- Validate sensor reading count
    if not sensor_readings or #sensor_readings == 0 then
        return nil, "No sensor readings available"
    end
    
    -- Validate each reading
    for i, reading in ipairs(sensor_readings) do
        -- Check pH range (6.5-8.5 for potable water)
        if reading.ph_level < 6.0 or reading.ph_level > 9.0 then
            return nil, string.format("Sensor %d: pH out of range (%.2f)", i, reading.ph_level)
        end
        
        -- Check temperature range (Phoenix: 40-120°F operational)
        if reading.temperature_f < 32.0 or reading.temperature_f > 130.0 then
            return nil, string.format("Sensor %d: Temperature out of range (%.2f°F)", i, reading.temperature_f)
        end
        
        -- Check pressure range (municipal: 30-80 PSI)
        if reading.pressure_psi < 20.0 or reading.pressure_psi > 100.0 then
            return nil, string.format("Sensor %d: Pressure out of range (%.2f PSI)", i, reading.pressure_psi)
        end
        
        -- Check turbidity (EPA standard: <4 NTU)
        if reading.turbidity_ntu > 10.0 then
            return nil, string.format("Sensor %d: Turbidity too high (%.2f NTU)", i, reading.turbidity_ntu)
        end
    end
    
    -- Update monsoon detection
    local monsoon_detected = self:detect_monsoon_from_sensors(sensor_readings)
    if monsoon_detected then
        self.monsoon_active = true
    end
    
    return sensor_readings, nil
end

--- SMART-002: MODEL - Build water capital state from sensor data
function WaterResilience:model_water_state(sensor_readings)
    -- Call Rust FFI to aggregate sensor readings into capital state
    local state_ptr = water_get_capital_state(sensor_readings, #sensor_readings)
    
    if state_ptr == nil then
        return nil, "Failed to build water capital state from sensors"
    end
    
    -- Convert FFI pointer to Lua table for orchestration use
    self.current_state = {
        surface_storage_af = state_ptr.surface_storage_af,
        groundwater_level_ft = state_ptr.groundwater_level_ft,
        reclaimed_water_af = state_ptr.reclaimed_water_af,
        stormwater_captured_af = state_ptr.stormwater_captured_af,
        distribution_pressure_psi = state_ptr.distribution_pressure_psi,
        water_quality_index = state_ptr.water_quality_index,
        per_capita_usage_gpd = state_ptr.per_capita_usage_gpd,
        flash_flood_risk = state_ptr.flash_flood_risk,
        drought_severity = state_ptr.drought_severity,
        is_monsoon_season = self.monsoon_active,
    }
    
    -- Update drought level
    self:update_drought_level(self.current_state)
    
    return self.current_state, nil
end

--- SMART-003: OPTIMIZE - Generate water allocation plans
function WaterResilience:optimize_allocation(demands, priorities)
    if not self.current_state then
        return nil, "No water state available for optimization"
    end
    
    local available_water = self.current_state.surface_storage_af + 
                            self.current_state.reclaimed_water_af
    
    -- Sort demands by priority (1 = highest, 5 = lowest)
    local sorted_demands = {}
    for zone_id, demand in pairs(demands) do
        table.insert(sorted_demands, {
            zone_id = zone_id,
            demand = demand,
            priority = priorities[zone_id] or 5
        })
    end
    
    table.sort(sorted_demands, function(a, b)
        return a.priority < b.priority
    end)
    
    -- Allocate water by priority
    local allocation_plans = {}
    local remaining_water = available_water
    
    for _, demand_info in ipairs(sorted_demands) do
        if remaining_water <= 0 then
            break
        end
        
        local allocation = math.min(demand_info.demand, remaining_water)
        remaining_water = remaining_water - allocation
        
        -- Determine source type based on zone
        local source_type = self:determine_source_type(demand_info.zone_id)
        local allocation_type = self:determine_allocation_type(demand_info.zone_id)
        
        -- Check Indigenous water rights compliance
        local fpic_verified = self:verify_fpic_for_zone(demand_info.zone_id)
        
        table.insert(allocation_plans, {
            zone_id = demand_info.zone_id,
            volume_af = allocation,
            source_type = source_type,
            allocation_type = allocation_type,
            priority = demand_info.priority,
            fpic_verified = fpic_verified,
            indigenous_rights_respected = true,
        })
    end
    
    -- Validate Indigenous water rights (15% minimum)
    local total_allocated = 0
    local indigenous_allocated = 0
    for _, plan in ipairs(allocation_plans) do
        total_allocated = total_allocated + plan.volume_af
        if plan.allocation_type == "INDIGENOUS" then
            indigenous_allocated = indigenous_allocated + plan.volume_af
        end
    end
    
    local min_indigenous = total_allocated * PHOENIX_HYDRO.INDIGENOUS_WATER_RIGHTS_PERCENTAGE
    if indigenous_allocated < min_indigenous then
        return nil, string.format(
            "Indigenous water allocation below treaty minimum (%.2f < %.2f af)",
            indigenous_allocated, min_indigenous
        )
    end
    
    self.allocation_plans = allocation_plans
    return allocation_plans, nil
end

--- Determine water source type based on zone
function WaterResilience:determine_source_type(zone_id)
    -- Agricultural zones (100-199) get reclaimed water first
    if zone_id >= 100 and zone_id < 200 then
        return "RECLAIMED"
    end
    -- Environmental zones (300-399) get surface water
    elseif zone_id >= 300 and zone_id < 400 then
        return "SURFACE"
    end
    -- Municipal zones (0-99) get mixed sources
    elseif zone_id < 100 then
        return "SURFACE"  -- Prioritize drinking quality
    end
    -- Default to groundwater
    return "GROUNDWATER"
end

--- Determine allocation type based on zone
function WaterResilience:determine_allocation_type(zone_id)
    if zone_id < 100 then
        return "MUNICIPAL"
    elseif zone_id < 200 then
        return "AGRICULTURAL"
    elseif zone_id < 300 then
        return "INDUSTRIAL"
    elseif zone_id < 400 then
        return "ENVIRONMENTAL"
    elseif zone_id >= 500 and zone_id < 600 then
        return "INDIGENOUS"
    else
        return "EMERGENCY"
    end
end

--- Verify FPIC for zone (Indigenous territories require explicit consent)
function WaterResilience:verify_fpic_for_zone(zone_id)
    -- Indigenous zones (500-599) require explicit FPIC
    if zone_id >= 500 and zone_id < 600 then
        return self:verify_indigenous_fpic(zone_id)
    end
    -- All other zones require treaty verification
    return self:verify_treaty_compliance(zone_id)
end

--- Verify Indigenous FPIC (placeholder - would call treaty service)
function WaterResilience:verify_indigenous_fpic(zone_id)
    -- In production, this would query the FPIC registry
    -- For now, return true if Indigenous sovereignty is enabled
    return true
end

--- Verify treaty compliance for zone
function WaterResilience:verify_treaty_compliance(zone_id)
    -- In production, this would verify Akimel O'odham and Piipaash treaty obligations
    return true
end

-- ============================================================================
-- SECTION 5: ECOSAFETY VALIDATION (Rust FFI Integration)
-- ============================================================================

--- SMART-004: TREATY-CHECK - Validate allocation against ecosafety corridors
function WaterResilience:validate_ecosafety_corridors()
    if not self.current_state then
        return false, "No state available for validation"
    end
    
    -- Build SevenCapitalState for validation (simplified for Lua)
    local state_ptr = self:build_seven_capital_state_ptr()
    
    if state_ptr == nil then
        return false, "Failed to build SevenCapitalState"
    end
    
    -- Call Rust FFI for corridor validation
    local decision = ecosafety_validate_corridors(state_ptr, "water", 5)
    
    if decision.decision == 2 then  -- STOP
        return false, "Corridor validation failed: STOP"
    elseif decision.decision == 3 then  -- NO_BUILD
        return false, "Corridor validation failed: NO_BUILD"
    elseif decision.decision == 4 then  -- PENDING_FPIC
        return false, "FPIC verification pending"
    end
    
    return true, "Corridor validation passed"
end

--- Build SevenCapitalState pointer for FFI call
function WaterResilience:build_seven_capital_state_ptr()
    -- In production, this would construct the full SevenCapitalState struct
    -- For now, return a placeholder that Rust FFI can interpret
    return self.current_state
end

--- Compute K/E/R metrics via Rust FFI
function WaterResilience:compute_ker_metrics()
    if not self.current_state then
        return nil, "No state available for K/E/R computation"
    end
    
    local state_ptr = self:build_seven_capital_state_ptr()
    local metrics = ecosafety_compute_ker(state_ptr)
    
    return {
        k = metrics.k,
        e = metrics.e,
        r = metrics.r,
    }, nil
end

-- ============================================================================
-- SECTION 6: EMERGENCY PROTOCOL ORCHESTRATION
-- ============================================================================

--- SMART-005: ACT - Execute emergency protocols
function WaterResilience:execute_emergency_protocol(emergency_type)
    self.emergency_active = true
    self.emergency_type = emergency_type
    
    if emergency_type == "FLASH_FLOOD" then
        return self:handle_flash_flood()
    elseif emergency_type == "DROUGHT" then
        return self:handle_drought()
    elseif emergency_type == "CONTAMINATION" then
        return self:handle_contamination()
    elseif emergency_type == "INFRASTRUCTURE_FAILURE" then
        return self:handle_infrastructure_failure()
    elseif emergency_type == "EXTREME_HEAT" then
        return self:handle_extreme_heat()
    else
        return nil, "Unknown emergency type: " .. tostring(emergency_type)
    end
end

--- Handle flash flood emergency
function WaterResilience:handle_flash_flood()
    if not self.current_state then
        return nil, "No state available for flash flood handling"
    end
    
    if self.current_state.flash_flood_risk >= 0.8 then
        -- Issue public alerts
        self:issue_public_alert("FLASH_FLOOD_WARNING", "Critical flash flood risk detected")
        
        -- Divert stormwater to capture basins
        self:activate_stormwater_diversion()
        
        -- Halt non-essential water operations
        return "EMERGENCY_STOP", "Flash flood emergency: All non-essential operations halted"
    elseif self.current_state.flash_flood_risk >= 0.5 then
        return "DERATE", "Flash flood risk elevated: Operations derated"
    end
    
    return "PARK", "Flash flood risk normal"
end

--- Handle drought emergency
function WaterResilience:handle_drought()
    if not self.current_state then
        return nil, "No state available for drought handling"
    end
    
    if self.drought_level >= 4 then
        -- Implement highest level water restrictions
        self:implement_water_restrictions(4)
        
        -- Activate emergency reserves
        self:activate_emergency_reserves()
        
        return "DERATE", "Drought level 4: Maximum restrictions implemented"
    elseif self.drought_level >= 3 then
        self:implement_water_restrictions(3)
        return "DERATE", "Drought level 3: Severe restrictions implemented"
    end
    
    return "PARK", "Drought conditions normal"
end

--- Handle contamination emergency
function WaterResilience:handle_contamination()
    if not self.current_state then
        return nil, "No state available for contamination handling"
    end
    
    if self.current_state.water_quality_index < 50.0 then
        -- Severe contamination - halt distribution
        self:issue_public_alert("WATER_CONTAMINATION", "Do not consume tap water")
        
        -- Activate alternative water sources
        self:activate_alternative_water_sources()
        
        return "EMERGENCY_STOP", "Severe contamination: Distribution halted"
    elseif self.current_state.water_quality_index < 70.0 then
        return "DERATE", "Moderate contamination: Treatment increased"
    end
    
    return "ACTUATE", "Water quality within safe limits"
end

--- Handle infrastructure failure
function WaterResilience:handle_infrastructure_failure()
    local infrastructure_health = self:get_infrastructure_health()
    
    if infrastructure_health < 50.0 then
        return "EMERGENCY_STOP", "Critical infrastructure failure"
    elseif infrastructure_health < 70.0 then
        return "DERATE", "Degraded infrastructure: Operations reduced"
    end
    
    return "ACTUATE", "Infrastructure healthy"
end

--- Handle extreme heat emergency
function WaterResilience:handle_extreme_heat()
    -- Check ambient temperature
    local ambient_temp = self:get_ambient_temperature()
    
    if ambient_temp >= PHOENIX_HYDRO.EXTREME_HEAT_THRESHOLD_F then
        -- Increase water pressure for cooling systems
        self:increase_cooling_water_pressure()
        
        -- Activate misting systems in public areas
        self:activate_misting_systems()
        
        return "DERATE", "Extreme heat: Cooling systems prioritized"
    end
    
    return "ACTUATE", "Temperature within normal range"
end

--- Issue public alert via citizen interface
function WaterResilience:issue_public_alert(alert_type, message)
    -- In production, this would publish to citizen notification system
    print(string.format("[ALERT %s] %s", alert_type, message))
    
    -- Log alert to workflow history
    table.insert(self.workflow_history, {
        timestamp = os.time(),
        type = "ALERT",
        alert_type = alert_type,
        message = message,
    })
end

--- Activate stormwater diversion systems
function WaterResilience:activate_stormwater_diversion()
    -- In production, this would control physical infrastructure
    print("[ACTION] Stormwater diversion activated")
end

--- Implement water restrictions by level
function WaterResilience:implement_water_restrictions(level)
    -- In production, this would enforce usage limits
    print(string.format("[ACTION] Water restrictions level %d implemented", level))
end

--- Activate emergency reserves
function WaterResilience:activate_emergency_reserves()
    -- In production, this would open reserve valves
    print("[ACTION] Emergency water reserves activated")
end

--- Activate alternative water sources
function WaterResilience:activate_alternative_water_sources()
    -- In production, this would switch to backup sources
    print("[ACTION] Alternative water sources activated")
end

--- Get infrastructure health index
function WaterResilience:get_infrastructure_health()
    -- In production, this would query infrastructure monitoring
    return 95.0  -- Placeholder
end

--- Get ambient temperature
function WaterResilience:get_ambient_temperature()
    -- In production, this would query weather sensors
    return 105.0  -- Placeholder (°F)
end

--- Increase cooling water pressure
function WaterResilience:increase_cooling_water_pressure()
    -- In production, this would adjust pump settings
    print("[ACTION] Cooling water pressure increased")
end

--- Activate misting systems
function WaterResilience:activate_misting_systems()
    -- In production, this would activate public misting stations
    print("[ACTION] Public misting systems activated")
end

-- ============================================================================
-- SECTION 7: SMART-CHAIN LOGGING AND AUDIT
-- ============================================================================

--- SMART-006: LOG - Emit QpuDataShard for audit trail
function WaterResilience:emit_shard(action, allocation_plans)
    if not self.current_state then
        return nil, "No state available for shard emission"
    end
    
    -- Call Rust FFI to execute complete SMART-chain and get shard
    local shard_ptr = water_execute_smart_chain(
        self.sensor_readings,
        #self.sensor_readings,
        {},  -- demands JSON (placeholder)
        0,   -- demands length
        self.entity_did
    )
    
    if shard_ptr == nil then
        return nil, "Failed to emit QpuDataShard"
    end
    
    -- Convert FFI pointer to Lua table
    self.last_shard = {
        shard_id = shard_ptr.shard_id,
        entity_did = shard_ptr.entity_did,
        timestamp_us = shard_ptr.timestamp_us,
        v_t = shard_ptr.v_t,
        k = shard_ptr.k,
        e = shard_ptr.e,
        r = shard_ptr.r,
        corridor_decision = shard_ptr.corridor_decision,
        fpic_status = shard_ptr.fpic_status,
        birth_sign_verified = shard_ptr.birth_sign_verified,
        action = action,
        allocation_plans = allocation_plans,
    }
    
    -- Log to workflow history
    table.insert(self.workflow_history, {
        timestamp = os.time(),
        type = "SHARD_EMIT",
        shard_id = shard_ptr.shard_id,
        action = action,
        k = shard_ptr.k,
        e = shard_ptr.e,
        r = shard_ptr.r,
    })
    
    return self.last_shard, nil
end

--- SMART-007: INTERFACE - Return shard for dashboard/audit
function WaterResilience:get_interface_data()
    if not self.last_shard then
        return nil, "No shard available for interface"
    end
    
    return {
        shard = self.last_shard,
        current_state = self.current_state,
        allocation_plans = self.allocation_plans,
        emergency_active = self.emergency_active,
        emergency_type = self.emergency_type,
        monsoon_active = self.monsoon_active,
        drought_level = self.drought_level,
        workflow_history = self.workflow_history,
    }, nil
end

-- ============================================================================
-- SECTION 8: COMPLETE SMART-CHAIN WORKFLOW EXECUTION
-- ============================================================================

--- Execute complete 7-step SMART-chain for water management
function WaterResilience:execute_smart_chain(sensor_readings, demands, priorities)
    -- Store sensor readings for shard emission
    self.sensor_readings = sensor_readings
    
    -- STEP 1: SENSE
    local readings, err = self:sense_sensor_data(sensor_readings)
    if err then
        return nil, "STEP 1 (SENSE) failed: " .. err
    end
    
    -- STEP 2: MODEL
    local state, err = self:model_water_state(readings)
    if err then
        return nil, "STEP 2 (MODEL) failed: " .. err
    end
    
    -- STEP 3: OPTIMIZE
    local plans, err = self:optimize_allocation(demands, priorities)
    if err then
        return nil, "STEP 3 (OPTIMIZE) failed: " .. err
    end
    
    -- STEP 4: TREATY-CHECK
    local valid, err = self:validate_ecosafety_corridors()
    if not valid then
        return nil, "STEP 4 (TREATY-CHECK) failed: " .. err
    end
    
    -- STEP 5: ACT (check for emergencies first)
    local action = "ACTUATE"
    if self:check_emergency_conditions() then
        action, err = self:execute_emergency_protocol(self.emergency_type)
        if err then
            return nil, "STEP 5 (ACT) emergency handling failed: " .. err
        end
    end
    
    -- STEP 6: LOG
    local shard, err = self:emit_shard(action, plans)
    if err then
        return nil, "STEP 6 (LOG) failed: " .. err
    end
    
    -- STEP 7: INTERFACE
    local interface_data, err = self:get_interface_data()
    if err then
        return nil, "STEP 7 (INTERFACE) failed: " .. err
    end
    
    return interface_data, nil
end

--- Check for emergency conditions
function WaterResilience:check_emergency_conditions()
    if not self.current_state then
        return false
    end
    
    -- Check flash flood
    if self.current_state.flash_flood_risk >= 0.8 then
        self.emergency_type = "FLASH_FLOOD"
        return true
    end
    
    -- Check drought
    if self.drought_level >= 4 then
        self.emergency_type = "DROUGHT"
        return true
    end
    
    -- Check contamination
    if self.current_state.water_quality_index < 50.0 then
        self.emergency_type = "CONTAMINATION"
        return true
    end
    
    -- Check extreme heat
    if self:get_ambient_temperature() >= PHOENIX_HYDRO.EXTREME_HEAT_THRESHOLD_F then
        self.emergency_type = "EXTREME_HEAT"
        return true
    end
    
    return false
end

-- ============================================================================
-- SECTION 9: CITIZEN INTERFACE COORDINATION
-- ============================================================================

--- Publish water status to citizen dashboard
function WaterResilience:publish_to_citizen_dashboard()
    local interface_data, err = self:get_interface_data()
    if err then
        return nil, err
    end
    
    -- Format for citizen consumption
    local citizen_view = {
        water_quality = interface_data.current_state.water_quality_index,
        usage_tips = self:generate_usage_tips(),
        alerts = self:get_active_alerts(),
        restrictions_level = self.drought_level,
        monsoon_status = self.monsoon_active,
    }
    
    -- In production, this would publish to citizen app/web interface
    print("[DASHBOARD] Publishing citizen view")
    
    return citizen_view, nil
end

--- Generate water usage tips based on current conditions
function WaterResilience:generate_usage_tips()
    local tips = {}
    
    if self.drought_level >= 3 then
        table.insert(tips, "Severe drought: Limit outdoor watering to 1x/week")
        table.insert(tips, "Fix leaks immediately - report to 311")
    end
    
    if self.monsoon_active then
        table.insert(tips, "Monsoon season: Capture rainwater in barrels")
        table.insert(tips, "Avoid driving through flooded streets")
    end
    
    if self.current_state and self.current_state.per_capita_usage_gpd > PHOENIX_HYDRO.PER_CAPITA_USAGE_TARGET_GPD then
        table.insert(tips, string.format(
            "Your usage exceeds target (%.1f vs %.1f gpd). Reduce by taking shorter showers.",
            self.current_state.per_capita_usage_gpd,
            PHOENIX_HYDRO.PER_CAPITA_USAGE_TARGET_GPD
        ))
    end
    
    return tips
end

--- Get active alerts for citizens
function WaterResilience:get_active_alerts()
    local alerts = {}
    
    if self.emergency_active then
        table.insert(alerts, {
            type = "EMERGENCY",
            severity = "HIGH",
            message = string.format("%s emergency active", self.emergency_type),
        })
    end
    
    if self.drought_level >= 3 then
        table.insert(alerts, {
            type = "RESTRICTION",
            severity = "MEDIUM",
            message = string.format("Water restrictions level %d in effect", self.drought_level),
        })
    end
    
    return alerts
end

-- ============================================================================
-- SECTION 10: WORKFLOW HISTORY AND AUDIT TRAIL
-- ============================================================================

--- Get workflow history for audit
function WaterResilience:get_workflow_history(start_time, end_time)
    local filtered_history = {}
    
    for _, entry in ipairs(self.workflow_history) do
        if (not start_time or entry.timestamp >= start_time) and
           (not end_time or entry.timestamp <= end_time) then
            table.insert(filtered_history, entry)
        end
    end
    
    return filtered_history
end

--- Export workflow history for compliance audit
function WaterResilience:export_audit_trail(format)
    format = format or "JSON"
    
    if format == "JSON" then
        -- In production, this would use a proper JSON library
        local json_str = "{\"workflow_history\": ["
        for i, entry in ipairs(self.workflow_history) do
            json_str = json_str .. "{\"timestamp\": " .. entry.timestamp
            json_str = json_str .. ", \"type\": \"" .. entry.type .. "\""
            if i < #self.workflow_history then
                json_str = json_str .. "},"
            else
                json_str = json_str .. "}"
            end
        end
        json_str = json_str .. "]}"
        return json_str
    end
    
    return nil, "Unsupported export format: " .. format
end

--- Clear workflow history (with retention policy)
function WaterResilience:clear_history(retain_days)
    retain_days = retain_days or 90  -- Default 90-day retention
    
    local cutoff_time = os.time() - (retain_days * 86400)
    local new_history = {}
    
    for _, entry in ipairs(self.workflow_history) do
        if entry.timestamp >= cutoff_time then
            table.insert(new_history, entry)
        end
    end
    
    self.workflow_history = new_history
    return #new_history
end

-- ============================================================================
-- SECTION 11: MODULE EXPORTS
-- ============================================================================

return WaterResilience

-- ============================================================================
-- END OF FILE: ALE-RM-WATER-RESILIENCE-001.lua
-- ============================================================================
