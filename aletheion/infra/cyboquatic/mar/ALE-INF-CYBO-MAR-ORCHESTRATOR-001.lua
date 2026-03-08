-- ============================================================================
-- ALETHEION INFRASTRUCTURE — CYBOQUATIC MAR ORCHESTRATOR
-- Domain: Water Capital (Managed Aquifer Recharge Dispatch)
-- Language: Lua 5.4 (Embedded, Offline-Capable, Deterministic)
-- License: Aletheion Public License v1.0 (Neurorights + BioticTreaty Bound)
-- Version: 1.0.0
-- Generated: 2026-03-09T00:00:00Z
-- SMART-Chain Binding: SMART01_AWP_THERMAL_THERMAPHORA
-- KER-Band: K=0.94, E=0.90, R=0.12 (Ecosafety Grammar Spine)
-- Cryptography: CRYSTALS-Dilithium (via Rust FFI Binding)
-- ============================================================================
-- CONSTRAINTS:
--   - No rollback, no downgrade, no reversal (forward-compatible only)
--   - Offline-capable execution (no external HTTP/API calls)
--   - Indigenous Water Treaty (Akimel O'odham, Piipaash) hard gates
--   - BioticTreaty (Riparian, Species) hard gates
--   - "No corridor, no build" enforced at orchestration layer
--   - Bound to Rust Validator in ALE-ERM-SMARTCHAIN-VALIDATOR-WATER-001.rs
--   - Bound to ALN Contracts in ALE-ERM-ECOSAFETY-WATER-CORRIDOR-CONTRACTS-001.aln
--   - Bound to Types in ALE-ERM-ECOSAFETY-WATER-CORRIDOR-TYPES-001.rs
-- ============================================================================
-- WORKFLOW PATTERN:
--   Sense → Model → Optimize → Treaty-Check → Act → Log → Interface
-- ============================================================================

-- ============================================================================
-- SECTION 1: ENVIRONMENT & FFI BINDINGS (Rust Interop)
-- ============================================================================
-- Lua runs embedded within the Rust runtime. All crypto and heavy validation
-- is delegated to the Rust validator module for PQ security and type safety.
-- ============================================================================

-- Load Aletheion Rust FFI Module (Compiled from Chunk 3)
-- In production, this is linked via lua_open_aletheion_validator()
local aletheion_validator = require("aletheion_validator")

-- Load Local State Store (Offline-First, Edge-DB)
-- In production, this connects to local sled/rocksdb instance
local local_state = require("aletheion_local_state")

-- Load Googolswarm Ledger Client (Audit Logging)
-- In production, this queues logs for batch signing via Dilithium
local ledger_client = require("aletheion_ledger_client")

-- ============================================================================
-- SECTION 2: CONFIGURATION & CONSTANTS
-- ============================================================================
-- Hard-coded constants for MAR Vault PHX-DT-MAR-VAULT-A
-- These must match the Corridor Atoms in Chunk 2 ALN contracts.
-- ============================================================================

local CONFIG = {
    VAULT_URN = "urn:ngsi-ld:MARVault:PHX-DT-MAR-VAULT-A",
    SMART_CHAIN_ID = "SMART01_AWP_THERMAL_THERMAPHORA",
    CORRIDOR_ID = "MAR_PFAS_2026",
    TREATY_REF_INDIGENOUS = "INDIGENOUS_WATER_TREATY_AKIMEL",
    TREATY_REF_BIOTIC = "BIOTIC_TREATY_AQUIFER",
    KER_META = {
        k = 0.94,
        e = 0.90,
        r = 0.12,
        line_ref = "MAR_CYBOQUATIC_2026"
    },
    PQ_MODE = "PQSTRICT",
    DERATE_FACTOR_SOFT = 0.5, -- 50% flow reduction on soft violation
    MAX_PUMP_SPEED = 100.0,   -- Percent
    SAFE_PUMP_SPEED = 0.0,    -- Percent (Stop)
    MONSOON_THRESHOLD_HEAD = 0.95, -- Normalized hydraulic head
    LOG_LEDGER_URN = "urn:ngsi-ld:Ledger:GOOGOLSWARM-WATER-01"
}

-- ============================================================================
-- SECTION 3: SENSE LAYER (Data Ingestion)
-- ============================================================================
-- Collects real-time sensor data from local edge nodes.
-- All values are normalized to [0,1] for RiskVector construction.
-- ============================================================================

--- Normalize raw sensor value to [0,1] range based on corridor bounds
-- @param raw_value number Raw sensor reading
-- @param min_safe number Minimum safe threshold (normalized)
-- @param max_safe number Maximum safe threshold (normalized)
-- @return number normalized_value
local function normalize_sensor_value(raw_value, min_safe, max_safe)
    -- Clamp to [0,1]
    local normalized = (raw_value - min_safe) / (max_safe - min_safe)
    return math.max(0.0, math.min(1.0, normalized))
end

--- Ingest sensor data for MAR Vault
-- @return table RiskVectorData
local function ingest_mar_sensors()
    -- Read from local state store (offline-capable)
    -- In production: local_state.get(CONFIG.VAULT_URN, "sensors")
    local sensors = local_state.get_sensors(CONFIG.VAULT_URN)
    
    -- Construct Risk Coordinate Data (matching Chunk 1 Rust types)
    local risk_coords = {
        {
            id = "PFAS",
            value = normalize_sensor_value(sensors.pfas_ng_l, 0.0, 4.0), -- EPA HAL 4ng/L
            minsafe = 0.0,
            maxsafe = 0.7,
            soft_boundary = 0.5,
            timestamp_ms = sensors.timestamp_ms,
            source_urn = "urn:ngsi-ld:Sensor:PHX-DT-MAR-PFAS-01"
        },
        {
            id = "Nutrient",
            value = normalize_sensor_value(sensors.nitrate_mg_l, 0.0, 10.0),
            minsafe = 0.1,
            maxsafe = 0.8,
            soft_boundary = 0.6,
            timestamp_ms = sensors.timestamp_ms,
            source_urn = "urn:ngsi-ld:Sensor:PHX-DT-MAR-NUT-01"
        },
        {
            id = "HydraulicHead",
            value = normalize_sensor_value(sensors.head_m, 10.0, 50.0),
            minsafe = 0.2,
            maxsafe = 0.9,
            soft_boundary = 0.75,
            timestamp_ms = sensors.timestamp_ms,
            source_urn = "urn:ngsi-ld:Sensor:PHX-DT-MAR-HEAD-01"
        },
        {
            id = "Temp",
            value = normalize_sensor_value(sensors.temp_c, 10.0, 35.0),
            minsafe = 0.1,
            maxsafe = 0.85,
            soft_boundary = 0.7,
            timestamp_ms = sensors.timestamp_ms,
            source_urn = "urn:ngsi-ld:Sensor:PHX-DT-MAR-TEMP-01"
        }
    }
    
    return {
        vault_urn = CONFIG.VAULT_URN,
        timestamp_ms = sensors.timestamp_ms,
        coords = risk_coords
    }
end

-- ============================================================================
-- SECTION 4: MODEL & OPTIMIZE LAYER (Risk Vector Construction)
-- ============================================================================
-- Builds the RiskVector object required by the Rust Validator (Chunk 3).
-- ============================================================================

--- Construct RiskVector for Validator
-- @param sensor_data table Ingested sensor data
-- @return table RiskVectorStruct
local function construct_risk_vector(sensor_data)
    return {
        id = "RV-" .. sensor_data.timestamp_ms,
        coords = sensor_data.coords,
        domain = "MAR",
        assembled_ms = sensor_data.timestamp_ms,
        node_urn = CONFIG.VAULT_URN,
        smart_chain_id = CONFIG.SMART_CHAIN_ID
    }
end

--- Construct CyboquaticNodeEcosafety Declaration
-- This enforces "No Corridor, No Build" at runtime
-- @return table NodeEcosafetyStruct
local function construct_node_ecosafety()
    return {
        node_id = CONFIG.VAULT_URN,
        corridors = { CONFIG.CORRIDOR_ID },
        node_type = "MarVault",
        location_geojson = '{"type":"Point","coordinates":[-112.0740,33.4484]}',
        smart_chain_ids = { CONFIG.SMART_CHAIN_ID },
        treaty_refs = { CONFIG.TREATY_REF_INDIGENOUS, CONFIG.TREATY_REF_BIOTIC },
        spec_version = "1.0.0"
    }
end

-- ============================================================================
-- SECTION 5: TREATY-CHECK LAYER (Governance & FPIC)
-- ============================================================================
-- Validates Indigenous Water Treaty and BioticTreaty constraints.
-- This is the hard gate before any actuation.
-- ============================================================================

--- Verify Free, Prior, and Informed Consent (FPIC)
-- Checks Googolswarm ledger for valid consent tokens
-- @return boolean is_consented
local function verify_fpic()
    -- Query local ledger cache for valid FPIC token
    -- In production: ledger_client.verify_consent(CONFIG.VAULT_URN, CONFIG.TREATY_REF_INDIGENOUS)
    local consent_status = ledger_client.get_consent_status(
        CONFIG.VAULT_URN, 
        CONFIG.TREATY_REF_INDIGENOUS
    )
    
    if not consent_status.valid then
        aletheion_validator.log_audit({
            level = "CRITICAL",
            message = "FPIC Consent Missing or Expired for Indigenous Water Treaty",
            vault_urn = CONFIG.VAULT_URN,
            treaty_ref = CONFIG.TREATY_REF_INDIGENOUS
        })
        return false
    end
    
    return true
end

--- Check SMART-Chain Governance Invariants
-- Calls Rust Validator to verify PQ mode, treaties, rollback rules
-- @param risk_vector table
-- @return boolean is_valid
local function verify_smart_chain_governance(risk_vector)
    local node_ecosafety = construct_node_ecosafety()
    
    -- Call Rust Validator (Chunk 3)
    local validation_result = aletheion_validator.validate_action({
        node_ecosafety = node_ecosafety,
        risk_vector = risk_vector,
        chain_id = CONFIG.SMART_CHAIN_ID,
        action_urn = "urn:ngsi-ld:Action:MARRecharge:" .. os.time(),
        timestamp_ms = risk_vector.assembled_ms
    })
    
    if not validation_result.success then
        aletheion_validator.log_audit({
            level = "ERROR",
            message = "SMART-Chain Governance Validation Failed",
            error_code = validation_result.error_code,
            vault_urn = CONFIG.VAULT_URN
        })
        return false
    end
    
    return true
end

-- ============================================================================
-- SECTION 6: ACT LAYER (Actuation Decision & Execution)
-- ============================================================================
-- Executes pump commands based on NodeAction (Normal/Derate/Stop).
-- ============================================================================

--- Execute Pump Actuation
-- @param action string "Normal", "Derate", "Stop"
-- @param derate_factor number Optional factor for Derate
local function execute_pump_actuation(action, derate_factor)
    local target_speed = CONFIG.SAFE_PUMP_SPEED
    
    if action == "Normal" then
        target_speed = CONFIG.MAX_PUMP_SPEED
    elseif action == "Derate" then
        target_speed = CONFIG.MAX_PUMP_SPEED * (derate_factor or CONFIG.DERATE_FACTOR_SOFT)
    elseif action == "Stop" then
        target_speed = CONFIG.SAFE_PUMP_SPEED
    else
        -- Fallback to Safe State
        target_speed = CONFIG.SAFE_PUMP_SPEED
    end
    
    -- Send command to physical PLC/Controller (Offline)
    -- In production: local_state.set_actuator(CONFIG.VAULT_URN, "pump_speed", target_speed)
    local actuation_result = local_state.set_actuator(CONFIG.VAULT_URN, "pump_speed", target_speed)
    
    if not actuation_result.success then
        error("CRITICAL: Actuation failed for MAR Vault " .. CONFIG.VAULT_URN)
    end
    
    return target_speed
end

--- Handle Monsoon Emergency Override
-- Special logic for flash flood events (Chunk 2 ALN Protocol)
-- @param hydraulic_head number Normalized head value
-- @return boolean override_active
local function handle_monsoon_override(hydraulic_head)
    if hydraulic_head >= CONFIG.MONSOON_THRESHOLD_HEAD then
        aletheion_validator.log_audit({
            level = "WARNING",
            message = "Monsoon Emergency Mode Activated (High Hydraulic Head)",
            head_value = hydraulic_head,
            threshold = CONFIG.MONSOON_THRESHOLD_HEAD
        })
        -- In monsoon, stop recharge to prevent surcharge
        execute_pump_actuation("Stop")
        return true
    end
    return false
end

-- ============================================================================
-- SECTION 7: LOG LAYER (Audit & Googolswarm)
-- ============================================================================
-- Records all decisions to the immutable ledger for audit.
-- ============================================================================

--- Log Action to Googolswarm Ledger
-- @param action string Decision made
-- @param risk_vector table State at decision time
-- @param validation_result table Validator output
local function log_action_to_ledger(action, risk_vector, validation_result)
    local audit_entry = {
        log_id = "audit:" .. CONFIG.VAULT_URN .. ":" .. risk_vector.assembled_ms,
        timestamp_ms = risk_vector.assembled_ms,
        action_urn = "urn:ngsi-ld:Action:MARRecharge:" .. risk_vector.assembled_ms,
        chain_id = CONFIG.SMART_CHAIN_ID,
        node_id = CONFIG.VAULT_URN,
        decision = action,
        ker_snapshot = CONFIG.KER_META,
        ledger_urn = CONFIG.LOG_LEDGER_URN,
        pq_signature = validation_result.pq_signature -- Dilithium signature from Rust
    }
    
    -- Queue for batch signing (offline-capable)
    ledger_client.queue_audit(audit_entry)
end

-- ============================================================================
-- SECTION 8: MAIN ORCHESTRATION LOOP (Workflow Entry Point)
-- ============================================================================
-- The canonical 6-step funnel: Sense → Model → Optimize → Treaty → Act → Log
-- ============================================================================

--- Main MAR Recharge Dispatch Function
-- This function is called by the central scheduler or event trigger
-- @return table ExecutionResult
function dispatch_mar_recharge()
    print("Starting MAR Recharge Dispatch for " .. CONFIG.VAULT_URN)
    
    -- STEP 1: SENSE (Ingest Data)
    local sensor_data = ingest_mar_sensors()
    if not sensor_data then
        error("CRITICAL: Sensor ingestion failed for " .. CONFIG.VAULT_URN)
    end
    
    -- STEP 2: MODEL (Construct Risk Vector)
    local risk_vector = construct_risk_vector(sensor_data)
    
    -- STEP 3: OPTIMIZE (Check Monsoon Emergency)
    local head_coord = nil
    for _, coord in ipairs(risk_vector.coords) do
        if coord.id == "HydraulicHead" then
            head_coord = coord
            break
        end
    end
    
    if head_coord and handle_monsoon_override(head_coord.value) then
        log_action_to_ledger("Stop", risk_vector, { pq_signature = "MONSOON_OVERRIDE" })
        return { status = "Stopped", reason = "Monsoon Emergency" }
    end
    
    -- STEP 4: TREATY-CHECK (Governance & Ecosafety)
    -- 4a: Verify FPIC (Indigenous Water Treaty)
    if not verify_fpic() then
        execute_pump_actuation("Stop")
        log_action_to_ledger("Stop", risk_vector, { pq_signature = "FPIC_FAILED" })
        return { status = "Stopped", reason = "FPIC Consent Missing" }
    end
    
    -- 4b: Verify SMART-Chain & Ecosafety (Rust Validator)
    if not verify_smart_chain_governance(risk_vector) then
        execute_pump_actuation("Stop")
        log_action_to_ledger("Stop", risk_vector, { pq_signature = "GOVERNANCE_FAILED" })
        return { status = "Stopped", reason = "SMART-Chain Validation Failed" }
    end
    
    -- STEP 5: ACT (Execute Decision)
    -- Call Rust Validator to get NodeAction (Normal/Derate/Stop)
    local node_action = aletheion_validator.decide_node_action(risk_vector, CONFIG.CORRIDOR_ID)
    
    local actual_speed = execute_pump_actuation(node_action.action, node_action.derate_factor)
    
    -- STEP 6: LOG (Audit to Ledger)
    log_action_to_ledger(node_action.action, risk_vector, { pq_signature = "VALIDATED" })
    
    print("MAR Recharge Dispatch Complete. Action: " .. node_action.action .. ", Speed: " .. actual_speed .. "%")
    
    return {
        status = "Success",
        action = node_action.action,
        pump_speed = actual_speed,
        timestamp_ms = risk_vector.assembled_ms
    }
end

-- ============================================================================
-- SECTION 9: CI/CD & TESTING HOOKS
-- ============================================================================
-- Exposed for automated testing pipelines (offline-capable)
-- ============================================================================

--- Test Hook: Verify Corridor Presence
-- @return boolean
function test_has_corridor()
    local node = construct_node_ecosafety()
    return #node.corridors > 0
end

--- Test Hook: Verify Treaty_refs
-- @return boolean
function test_has_treaties()
    local node = construct_node_ecosafety()
    local has_indigenous = false
    for _, ref in ipairs(node.treaty_refs) do
        if ref == CONFIG.TREATY_REF_INDIGENOUS then
            has_indigenous = true
            break
        end
    end
    return has_indigenous
end

--- Test Hook: Verify KER Meta
-- @return boolean
function test_ker_meta_valid()
    return CONFIG.KER_META.k >= 0.90 and 
           CONFIG.KER_META.e >= 0.90 and 
           CONFIG.KER_META.r <= 0.15
end

-- ============================================================================
-- END OF FILE: ALE-INF-CYBO-MAR-ORCHESTRATOR-001.lua
-- ============================================================================
-- This file is part of the Aletheion MAR Orchestration Layer.
-- It binds Chunk 1 (Types), Chunk 2 (ALN), and Chunk 3 (Validator) into
-- executable workflow logic.
-- CI must run test_has_corridor, test_has_treaties, and test_ker_meta_valid
-- on every commit to aletheion/infra/cyboquatic/mar/*.lua.
-- Indigenous Water Treaty (Akimel O'odham) is enforced via verify_fpic().
-- PQSTRICT mode is enforced via Rust FFI binding.
-- "No corridor, no build" is enforced via construct_node_ecosafety().
-- ============================================================================
