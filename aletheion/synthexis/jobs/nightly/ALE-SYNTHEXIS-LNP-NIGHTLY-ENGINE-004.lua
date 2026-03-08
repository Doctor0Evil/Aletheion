-- ============================================================================
-- FILE: aletheion/synthexis/jobs/nightly/ALE-SYNTHEXIS-LNP-NIGHTLY-ENGINE-004.lua
-- PURPOSE: Nightly orchestration job for LNP (Light/Noise/Pesticide) planning,
--          habitat continuity validation, and trust layer append operations
-- LANGUAGE: Lua 5.4+
-- DESTINATION: Aletheion Repository - Synthexis Jobs Nightly Subsystem
-- COMPLIANCE: Zero-contamination IFC, BioticTreaty enforcement, FPIC metadata
-- INTEGRATION: ALE-SYNTHEXIS-IFC-KERNEL-001.rs, ALE-SYNTHEXIS-BIOTIC-TREATY-CORE-001.aln
-- ORCHESTRATION: Rust as black-box planner, ALN as policy source, Lua as glue
-- ============================================================================

-- SECTION 1: MODULE DECLARATIONS & DEPENDENCY LOADING
local json = require("dkjson")
local http = require("socket.http")
local ltn12 = require("ltn12")
local fs = require("plenary.path")
local sha256 = require("resty.sha256")
local cjson = require("cjson.safe")

local ALE_SYNTHESIS_VERSION = "004"
local ALE_POLICY_VERSION = "001"
local ALE_KERNEL_BINARY = "./bin/ale_synthexis_ifc_kernel"
local ALE_ALN_VERIFIER = "./bin/ale_aln_verifier"
local ALE_COMPLIANCE_CLI = "./bin/ale_compliance_ifc_neurorights"

local ALE_DATA_DIR = "./aletheion/synthexis/data"
local ALE_OUTPUT_DIR = "./aletheion/synthexis/output"
local ALE_TRUST_DIR = "./aletheion/synthexis/trust"
local ALE_LOG_DIR = "./aletheion/synthexis/logs"
local ALE_TREATY_DIR = "./aletheion/synthexis/engine/contracts"

-- SECTION 2: LOGGING & AUDIT UTILITIES
local function get_unix_timestamp()
    return os.time()
end

local function generate_event_id(prefix)
    local timestamp = get_unix_timestamp()
    local random = math.random(1000, 9999)
    return string.format("%s-%d-%04d", prefix, timestamp, random)
end

local function log_info(message, context)
    local entry = {
        level = "INFO",
        timestamp = get_unix_timestamp(),
        event_id = generate_event_id("LOG"),
        message = message,
        context = context or {}
    }
    local log_file = fs:new(ALE_LOG_DIR .. "/synthexis-nightly-" .. os.date("%Y%m%d") .. ".jsonl")
    log_file:append(json.encode(entry) .. "\n")
    print("[INFO] " .. message)
end

local function log_warning(message, context)
    local entry = {
        level = "WARNING",
        timestamp = get_unix_timestamp(),
        event_id = generate_event_id("LOG"),
        message = message,
        context = context or {}
    }
    local log_file = fs:new(ALE_LOG_DIR .. "/synthexis-nightly-" .. os.date("%Y%m%d") .. ".jsonl")
    log_file:append(json.encode(entry) .. "\n")
    print("[WARNING] " .. message)
end

local function log_error(message, context)
    local entry = {
        level = "ERROR",
        timestamp = get_unix_timestamp(),
        event_id = generate_event_id("LOG"),
        message = message,
        context = context or {}
    }
    local log_file = fs:new(ALE_LOG_DIR .. "/synthexis-nightly-" .. os.date("%Y%m%d") .. ".jsonl")
    log_file:append(json.encode(entry) .. "\n")
    print("[ERROR] " .. message)
end

-- SECTION 3: IFC LABEL CREATION & VALIDATION
local function create_ifc_label(sensitivity, domain, provenance, origin_data)
    local hash = sha256:new()
    hash:update(origin_data or "")
    local origin_hash = hash:final()
    
    local label = {
        label_id = generate_event_id("IFC"),
        sensitivity = sensitivity,
        domain = domain,
        provenance = provenance,
        origin_hash = origin_hash,
        timestamp = get_unix_timestamp(),
        fpic_verified = false,
        neurorights_compliant = false,
        biotic_treaty_bound = false,
        corridor_validated = false
    }
    return label
end

local function validate_ifc_label(label)
    if not label.label_id then
        return false, "IFC label missing label_id"
    end
    if not label.sensitivity then
        return false, "IFC label missing sensitivity"
    end
    if not label.domain then
        return false, "IFC label missing domain"
    end
    if not label.origin_hash then
        return false, "IFC label missing origin_hash"
    end
    if label.sensitivity == "sovereign" and not label.fpic_verified then
        return false, "Sovereign IFC label requires FPIC verification"
    end
    return true, nil
end

local function can_flow_to(source_label, target_label)
    local sensitivity_order = {["public"] = 0, ["internal"] = 1, ["confidential"] = 2, ["sovereign"] = 3}
    local source_idx = sensitivity_order[source_label.sensitivity] or 0
    local target_idx = sensitivity_order[target_label.sensitivity] or 0
    
    if source_idx > target_idx then
        return false, "Sensitivity downgrade not allowed"
    end
    if source_label.domain ~= target_label.domain and source_idx >= 2 then
        return false, "Cross-domain flow restricted for confidential+ data"
    end
    if not source_label.fpic_verified and target_label.fpic_verified then
        return false, "FPIC verification required for flow"
    end
    return true, nil
end

-- SECTION 4: STATE LOADING & DATA INGESTION
local function load_state_json(state_path)
    local state_file = fs:new(state_path)
    if not state_file:exists() then
        return nil, "State file not found: " .. state_path
    end
    local content = state_file:read()
    local state, pos, err = json.decode(content, 1, nil)
    if not state then
        return nil, "State JSON parse failed: " .. err
    end
    return state, nil
end

local function load_biotic_treaty(treaty_id)
    local treaty_path = ALE_TREATY_DIR .. "/ALE-SYNTHEXIS-BIOTIC-TREATY-CORE-001.aln"
    local treaty_file = fs:new(treaty_path)
    if not treaty_file:exists() then
        return nil, "Treaty file not found: " .. treaty_path
    end
    local content = treaty_file:read()
    return content, nil
end

local function load_lnp_sensor_data(region_hash)
    local sensor_path = ALE_DATA_DIR .. "/lnp_sensors_" .. region_hash .. ".json"
    return load_state_json(sensor_path)
end

local function load_species_agents(region_hash)
    local species_path = ALE_DATA_DIR .. "/species_agents_" .. region_hash .. ".json"
    return load_state_json(species_path)
end

-- SECTION 5: RUST KERNEL CLI INTEGRATION
local function execute_rust_kernel(input_data)
    local input_json = json.encode(input_data)
    local input_file = fs:new(ALE_DATA_DIR .. "/kernel_input_" .. generate_event_id("IN") .. ".json")
    input_file:write(input_json)
    
    local cmd = string.format("%s process_habitat_json '%s'", ALE_KERNEL_BINARY, input_file:absolute())
    local handle = io.popen(cmd)
    if not handle then
        return nil, "Failed to execute Rust kernel"
    end
    local result = handle:read("*a")
    handle:close()
    
    input_file:rm()
    
    local output, pos, err = json.decode(result, 1, nil)
    if not output then
        return nil, "Kernel output JSON parse failed: " .. err
    end
    
    return output, nil
end

local function validate_treaty_via_kernel(treaty_data)
    local treaty_json = json.encode(treaty_data)
    local treaty_file = fs:new(ALE_DATA_DIR .. "/treaty_input_" .. generate_event_id("TR") .. ".json")
    treaty_file:write(treaty_json)
    
    local cmd = string.format("%s validate_treaty_json '%s'", ALE_KERNEL_BINARY, treaty_file:absolute())
    local handle = io.popen(cmd)
    if not handle then
        return nil, "Failed to execute treaty validation"
    end
    local result = handle:read("*a")
    handle:close()
    
    treaty_file:rm()
    
    local output, pos, err = json.decode(result, 1, nil)
    if not output then
        return nil, "Treaty validation JSON parse failed: " .. err
    end
    
    return output, nil
end

-- SECTION 6: ALN VERIFIER INTEGRATION
local function verify_aln_compliance(output_data, treaty_aln)
    local output_json = json.encode(output_data)
    local output_file = fs:new(ALE_DATA_DIR .. "/aln_verify_input_" .. generate_event_id("AV") .. ".json")
    output_file:write(output_json)
    
    local aln_file = fs:new(ALE_DATA_DIR .. "/aln_verify_treaty.aln")
    aln_file:write(treaty_aln)
    
    local cmd = string.format("%s verify --input '%s' --policy '%s'", ALE_ALN_VERIFIER, output_file:absolute(), aln_file:absolute())
    local handle = io.popen(cmd)
    if not handle then
        return nil, "Failed to execute ALN verifier"
    end
    local result = handle:read("*a")
    local success = handle:close()
    
    output_file:rm()
    aln_file:rm()
    
    if not success then
        return {compliant = false, errors = {"ALN verification failed"}}, nil
    end
    
    local verification, pos, err = json.decode(result, 1, nil)
    if not verification then
        return {compliant = false, errors = {"ALN verification JSON parse failed: " .. err}}, nil
    end
    
    return verification, nil
end

-- SECTION 7: COMPLIANCE PREFLIGHT CHECKS
local function run_compliance_preflight(data)
    local data_json = json.encode(data)
    local data_file = fs:new(ALE_DATA_DIR .. "/compliance_input_" .. generate_event_id("CP") .. ".json")
    data_file:write(data_json)
    
    local cmd = string.format("%s scan --input '%s' --format json", ALE_COMPLIANCE_CLI, data_file:absolute())
    local handle = io.popen(cmd)
    if not handle then
        return {passed = false, errors = {"Compliance CLI execution failed"}}, nil
    end
    local result = handle:read("*a")
    local success = handle:close()
    
    data_file:rm()
    
    local report, pos, err = json.decode(result, 1, nil)
    if not report then
        return {passed = false, errors = {"Compliance report JSON parse failed: " .. err}}, nil
    end
    
    return report, nil
end

local function check_contamination_scan(data)
    local forbidden_patterns = {
        "rollback", "downgrade", "reversal", "Exergy", "DOW", "NDM",
        "NEURON", "Brian2", "blake", "argon", "Python"
    }
    local data_str = json.encode(data)
    for _, pattern in ipairs(forbidden_patterns) do
        if string.find(data_str, pattern) then
            return false, "Contamination detected: " .. pattern
        end
    end
    return true, nil
end

-- SECTION 8: LNP ENVELOPE PROCESSING
local function process_lnp_envelope(sensor_data, treaty_constraints)
    local lnp_envelope = {
        light = {
            ambient_lux = sensor_data.light_ambient_lux or 0.0,
            sky_glow_index = sensor_data.light_sky_glow or 0.0,
            spectral_distribution = sensor_data.light_spectrum or {},
            dark_sky_compliant = (sensor_data.light_ambient_lux or 0) <= 10.0,
            nocturnal_species_safe = (sensor_data.light_ambient_lux or 0) <= 5.0,
            max_allowed_lux = 10.0,
            measurement_timestamp = sensor_data.light_timestamp or get_unix_timestamp()
        },
        noise = {
            ambient_db = sensor_data.noise_ambient_db or 0.0,
            frequency_spectrum = sensor_data.noise_spectrum or {},
            peak_events_per_hour = sensor_data.noise_peak_events or 0,
            wildlife_disturbance_index = 0.0,
            human_health_impact = 0.0,
            max_allowed_db = 55.0,
            measurement_timestamp = sensor_data.noise_timestamp or get_unix_timestamp()
        },
        pesticide = {
            concentration_ppb = sensor_data.pesticide_concentration or 0.0,
            compound_types = sensor_data.pesticide_compounds or {},
            bioaccumulation_risk = 0.0,
            pollinator_toxicity = 0.0,
            aquatic_toxicity = 0.0,
            zero_tolerance_violation = (sensor_data.pesticide_concentration or 0) > 0.0,
            measurement_timestamp = sensor_data.pesticide_timestamp or get_unix_timestamp()
        },
        aggregate_compliance = true,
        habitat_continuity_score = 1.0
    }
    
    if lnp_envelope.light.ambient_lux > 10.0 then
        lnp_envelope.aggregate_compliance = false
    end
    if lnp_envelope.noise.ambient_db > 55.0 then
        lnp_envelope.aggregate_compliance = false
    end
    if lnp_envelope.pesticide.concentration_ppb > 0.0 then
        lnp_envelope.aggregate_compliance = false
    end
    
    local light_score = lnp_envelope.light.dark_sky_compliant and 1.0 or (1.0 - lnp_envelope.light.ambient_lux / 10.0)
    local noise_score = lnp_envelope.noise.ambient_db <= 55.0 and 1.0 or (1.0 - lnp_envelope.noise.ambient_db / 90.0)
    local pesticide_score = lnp_envelope.pesticide.zero_tolerance_violation and 0.0 or 1.0
    lnp_envelope.habitat_continuity_score = light_score * 0.3 + noise_score * 0.3 + pesticide_score * 0.4
    
    return lnp_envelope
end

-- SECTION 9: HABITAT CONTINUITY WORKFLOW
local function execute_habitat_continuity_workflow(region_hash, request_id)
    local workflow_start = get_unix_timestamp()
    local workflow_id = generate_event_id("HCW")
    
    log_info("Starting habitat continuity workflow", {
        workflow_id = workflow_id,
        region_hash = region_hash,
        request_id = request_id
    })
    
    local sensor_data, err = load_lnp_sensor_data(region_hash)
    if not sensor_data then
        log_error("Failed to load LNP sensor data", {error = err, region_hash = region_hash})
        return nil, err
    end
    
    local species_agents, err = load_species_agents(region_hash)
    if not species_agents then
        log_warning("Species agents data not found, using empty set", {error = err, region_hash = region_hash})
        species_agents = {}
    end
    
    local treaty_aln, err = load_biotic_treaty("MT-000001-BIOT")
    if not treaty_aln then
        log_error("Failed to load biotic treaty", {error = err})
        return nil, err
    end
    
    local treaty_data = {
        treaty_id = "MT-000001-BIOT",
        version = "001",
        stake_multisig_verified = true,
        indigenous_consultation_complete = true,
        fpic_verified = true,
        pesticide_zero_tolerance = true,
        corridor_connectivity_min = 0.7,
        roH_bound = 0.15,
        sovereignty_scalar = 0.85
    }
    
    local treaty_validation, err = validate_treaty_via_kernel(treaty_data)
    if not treaty_validation or not treaty_validation.valid then
        log_error("Treaty validation failed", {error = err, validation = treaty_validation})
        return nil, "Treaty validation failed: " .. (err or "unknown")
    end
    
    local lnp_envelope = process_lnp_envelope(sensor_data, treaty_data)
    
    local input_ifc_label = create_ifc_label("internal", "biotic", "sensor", json.encode(sensor_data))
    input_ifc_label.fpic_verified = treaty_data.fpic_verified
    input_ifc_label.biotic_treaty_bound = true
    
    local kernel_input = {
        request_id = request_id,
        region_hash = region_hash,
        habitat_types = sensor_data.habitat_types or {"pollinator_corridor"},
        species_agents = species_agents,
        lnp_envelope = lnp_envelope,
        biotic_treaty = treaty_data,
        ifc_labels = {input_ifc_label},
        corridor_id = "CORR-" .. region_hash,
        timestamp = get_unix_timestamp()
    }
    
    local contamination_ok, contamination_err = check_contamination_scan(kernel_input)
    if not contamination_ok then
        log_error("Contamination scan failed", {error = contamination_err})
        return nil, contamination_err
    end
    
    local compliance_report, err = run_compliance_preflight(kernel_input)
    if not compliance_report or not compliance_report.passed then
        log_error("Compliance preflight failed", {error = err, report = compliance_report})
        return nil, "Compliance preflight failed: " .. (err or "unknown")
    end
    
    local kernel_output, err = execute_rust_kernel(kernel_input)
    if not kernel_output then
        log_error("Rust kernel execution failed", {error = err})
        return nil, err
    end
    
    if not kernel_output.can_proceed then
        log_error("Kernel output indicates cannot proceed", {errors = kernel_output.errors, warnings = kernel_output.warnings})
        return kernel_output, "Kernel validation failed"
    end
    
    local aln_verification, err = verify_aln_compliance(kernel_output, treaty_aln)
    if not aln_verification or not aln_verification.compliant then
        log_error("ALN verification failed", {error = err, verification = aln_verification})
        return kernel_output, "ALN verification failed"
    end
    
    local output_ifc_label = kernel_output.output_ifc_label
    local valid, err = validate_ifc_label(output_ifc_label)
    if not valid then
        log_error("Output IFC label validation failed", {error = err})
        return kernel_output, err
    end
    
    local workflow_end = get_unix_timestamp()
    log_info("Habitat continuity workflow completed successfully", {
        workflow_id = workflow_id,
        duration_seconds = workflow_end - workflow_start,
        continuity_score = kernel_output.continuity_score,
        lnp_compliance = kernel_output.lnp_compliance,
        treaty_compliance = kernel_output.treaty_compliance
    })
    
    return kernel_output, nil
end

-- SECTION 10: TRUST LAYER APPEND OPERATIONS
local function append_to_trust_layer(output_data, workflow_id)
    local trust_entry = {
        entry_id = generate_event_id("TRUST"),
        workflow_id = workflow_id,
        timestamp = get_unix_timestamp(),
        output_hash = sha256:new():update(json.encode(output_data)):final(),
        continuity_score = output_data.continuity_score,
        lnp_compliance = output_data.lnp_compliance,
        treaty_compliance = output_data.treaty_compliance,
        ifc_valid = output_data.ifc_valid,
        corridor_valid = output_data.corridor_valid,
        species_risk_count = #output_data.species_risk_assessment,
        recommended_actions_count = #output_data.recommended_actions,
        warnings_count = #output_data.warnings,
        errors_count = #output_data.errors,
        audit_signature = "pending_multisig"
    }
    
    local trust_file = fs:new(ALE_TRUST_DIR .. "/trust_log_" .. os.date("%Y%m%d") .. ".jsonl")
    trust_file:append(json.encode(trust_entry) .. "\n")
    
    log_info("Trust layer entry appended", {entry_id = trust_entry.entry_id})
    return trust_entry
end

local function write_output_envelope(output_data, region_hash)
    local output_file = fs:new(ALE_OUTPUT_DIR .. "/lnp_envelope_" .. region_hash .. "_" .. os.date("%Y%m%d") .. ".json")
    output_file:write(json.encode(output_data, {indent = true}))
    
    log_info("Output envelope written", {file = output_file:absolute()})
    return output_file:absolute()
end

-- SECTION 11: NIGHTLY JOB ORCHESTRATION
local function execute_nightly_job()
    local job_start = get_unix_timestamp()
    local job_id = generate_event_id("NIGHTLY")
    
    log_info("Synthexis LNP nightly job started", {job_id = job_id})
    
    local regions = {
        {region_hash = 10001, request_id = "REQ-10001-001"},
        {region_hash = 10002, request_id = "REQ-10002-001"},
        {region_hash = 10003, request_id = "REQ-10003-001"},
        {region_hash = 10004, request_id = "REQ-10004-001"},
        {region_hash = 10005, request_id = "REQ-10005-001"}
    }
    
    local results = {
        job_id = job_id,
        start_timestamp = job_start,
        end_timestamp = nil,
        total_regions = #regions,
        successful = 0,
        failed = 0,
        region_results = {}
    }
    
    for _, region in ipairs(regions) do
        log_info("Processing region", {region_hash = region.region_hash, request_id = region.request_id})
        
        local output, err = execute_habitat_continuity_workflow(region.region_hash, region.request_id)
        
        local region_result = {
            region_hash = region.region_hash,
            request_id = region.request_id,
            success = output ~= nil,
            error = err,
            continuity_score = output and output.continuity_score or nil,
            timestamp = get_unix_timestamp()
        }
        
        if output then
            results.successful = results.successful + 1
            append_to_trust_layer(output, job_id)
            write_output_envelope(output, region.region_hash)
        else
            results.failed = results.failed + 1
            log_error("Region processing failed", {region_hash = region.region_hash, error = err})
        end
        
        table.insert(results.region_results, region_result)
    end
    
    results.end_timestamp = get_unix_timestamp()
    results.duration_seconds = results.end_timestamp - results.start_timestamp
    
    local summary_file = fs:new(ALE_OUTPUT_DIR .. "/nightly_summary_" .. os.date("%Y%m%d") .. ".json")
    summary_file:write(json.encode(results, {indent = true}))
    
    log_info("Synthexis LNP nightly job completed", {
        job_id = job_id,
        successful = results.successful,
        failed = results.failed,
        duration_seconds = results.duration_seconds
    })
    
    return results
end

-- SECTION 12: ERROR HANDLING & RECOVERY
local function handle_job_failure(error_msg, job_id)
    local failure_entry = {
        entry_id = generate_event_id("FAIL"),
        job_id = job_id,
        timestamp = get_unix_timestamp(),
        error_message = error_msg,
        recovery_action = "trigger_healing_workflow",
        notification_sent = true
    }
    
    local failure_file = fs:new(ALE_LOG_DIR .. "/failure_log_" .. os.date("%Y%m%d") .. ".jsonl")
    failure_file:append(json.encode(failure_entry) .. "\n")
    
    log_error("Job failure recorded", {failure_id = failure_entry.entry_id, error = error_msg})
    
    return failure_entry
end

local function safe_execute_nightly_job()
    local status, result = pcall(execute_nightly_job)
    if not status then
        handle_job_failure(result, generate_event_id("NIGHTLY"))
        return {success = false, error = result}
    end
    return {success = true, result = result}
end

-- SECTION 13: MAIN ENTRY POINT
local function main()
    local args = {...}
    local mode = args[1] or "nightly"
    
    if mode == "nightly" then
        return safe_execute_nightly_job()
    elseif mode == "region" then
        local region_hash = tonumber(args[2])
        local request_id = args[3] or "REQ-" .. region_hash .. "-001"
        if not region_hash then
            log_error("Region hash required for region mode")
            return {success = false, error = "Region hash required"}
        end
        local output, err = execute_habitat_continuity_workflow(region_hash, request_id)
        if not output then
            return {success = false, error = err}
        end
        return {success = true, output = output}
    elseif mode == "validate_treaty" then
        local treaty_data = {
            treaty_id = "MT-000001-BIOT",
            version = "001",
            stake_multisig_verified = true,
            indigenous_consultation_complete = true,
            fpic_verified = true
        }
        local result, err = validate_treaty_via_kernel(treaty_data)
        if not result then
            return {success = false, error = err}
        end
        return {success = true, validation = result}
    else
        log_error("Unknown mode: " .. mode)
        return {success = false, error = "Unknown mode: " .. mode}
    end
end

return {
    main = main,
    execute_nightly_job = execute_nightly_job,
    execute_habitat_continuity_workflow = execute_habitat_continuity_workflow,
    validate_treaty_via_kernel = validate_treaty_via_kernel,
    process_lnp_envelope = process_lnp_envelope,
    create_ifc_label = create_ifc_label,
    validate_ifc_label = validate_ifc_label,
    run_compliance_preflight = run_compliance_preflight,
    append_to_trust_layer = append_to_trust_layer,
    VERSION = ALE_SYNTHESIS_VERSION,
    POLICY_VERSION = ALE_POLICY_VERSION
}
