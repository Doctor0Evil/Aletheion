-- ALETHEION_HABOOB_EDGE_CONTROLLER_V1.0.0
-- LICENSE: BioticTreaty_Compliant_AGPLv3
-- ECO_IMPACT: K=0.88 | E=0.87 | R=0.18
-- CHAIN: ERM (Sense → Act)
-- CONSTRAINTS: Offline-Capable, Low-Power, Dust-Mitigation
-- HARDWARE: ESP32, PM2.5 Sensor, HVAC Interface

-- --- CONFIGURATION ---
local CONFIG = {
    PM25_CRITICAL = 55.0,      -- ug/m3 (Unhealthy)
    PM25_HAZARDOUS = 150.0,    -- ug/m3 (Health Alert)
    SAMPLE_INTERVAL_MS = 5000, -- 5 seconds
    HVAC_SHUTDOWN_THRESHOLD = 100.0 -- Protect internal systems
}

-- --- STATE ---
local state = {
    current_pm25 = 0.0,
    hvac_active = true,
    alert_level = "GREEN", -- GREEN, YELLOW, RED
    last_sync = 0
}

-- --- SENSE FUNCTION ---
function read_sensor()
    -- Interface with physical PM2.5 sensor (e.g., PMS5003)
    -- Returns float ug/m3
    local val = sensor.read_pm25() 
    return val or 0.0
end

-- --- ACT FUNCTION (ERM: ACT) ---
function mitigate_dust(pm25)
    if pm25 > CONFIG.PM25_HAZARDOUS then
        state.alert_level = "RED"
        if state.hvac_active then
            hvac.close_intakes() -- Prevent dust ingress
            state.hvac_active = false
            log_event("HVAC_SHUTDOWN_HABOUB_DETECTED")
        end
        citizen_alert("HAZARDOUS_AIR_QUALITY_SHELTER_IN_PLACE")
    elseif pm25 > CONFIG.PM25_CRITICAL then
        state.alert_level = "YELLOW"
        hvac.reduce_intakes(50) -- Partial closure
        citizen_alert("HIGH_DUST_LIMIT_OUTDOOR_ACTIVITY")
    else
        state.alert_level = "GREEN"
        if not state.hvac_active then
            hvac.open_intakes() -- Restore normal operation
            state.hvac_active = true
        end
    end
end

-- --- MAIN LOOP (Offline Capable) ---
function loop()
    while true do
        local pm25 = read_sensor()
        state.current_pm25 = pm25
        
        -- Local Decision Making (No Cloud Required)
        mitigate_dust(pm25)
        
        -- Queue data for later sync when network available
        data_log.append({
            ts = os.time(),
            pm25 = pm25,
            alert = state.alert_level
        })

        tmr.delay(CONFIG.SAMPLE_INTERVAL_MS)
    end
end

-- --- INITIALIZATION ---
print("Aletheion Haboob Controller Initialized")
print("Territory: Phoenix_Akimel_O'odham")
loop()
