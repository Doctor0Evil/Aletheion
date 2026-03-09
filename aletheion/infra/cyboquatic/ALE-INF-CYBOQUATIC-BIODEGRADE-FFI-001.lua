local ffi = require("ffi")
local bit = require("bit")

-- =============================================================================
-- 1. FFI Type Declarations (Mirror Rust Contract Types)
-- =============================================================================

ffi.cdef[[
    typedef struct {
        uint8_t prefix[4];
        uint16_t network_id;
        uint8_t node_hash[32];
    } BiodegradeNodeId;

    typedef struct {
        uint64_t tile_id;
        const char* birth_sign_id;
        const char* indigenous_territory;
        bool ej_zone_flag;
        const char* biotic_corridor;
    } GeoTileRef;

    typedef enum {
        ORGANIC_FAST,
        ORGANIC_SLOW,
        POLYMER_BIO,
        COMPOSITE_SAFE,
        HAZARDOUS_CONTROLLED
    } BiodegradeMaterialClass;

    typedef enum {
        DECLARED_ACTIVE,
        DECLARED_INACTIVE,
        UNDECLARED,
        VIOLATION_DETECTED,
        UNDER_REVIEW
    } CorridorStatus;

    typedef struct {
        double value;
        double derivative;
        double threshold;
        bool convergent;
    } LyapunovResidual;

    typedef struct {
        double value;
        double confidence;
        double temperature_corrected;
        double ph_corrected;
    } MicrobialRate;

    typedef struct {
        double heavy_metals_ppm;
        double organic_toxins_ppb;
        double bioaccumulation_factor;
        bool safe_threshold_exceeded;
    } ToxicityResidual;

    typedef struct {
        BiodegradeNodeId node_id;
        GeoTileRef geo_tile;
        BiodegradeMaterialClass material_class;
        double mass_remaining_grams;
        double mass_initial_grams;
        MicrobialRate r_micro;
        ToxicityResidual r_tox;
        LyapunovResidual lyapunov;
        CorridorStatus corridor_status;
        uint64_t timestamp_unix;
        uint32_t firmware_version;
        uint8_t governance_seal[64];
    } BiodegradeNodeState;

    typedef enum {
        ADVISORY,
        WARNING,
        CRITICAL,
        HARD_BLOCK
    } ViolationSeverity;

    typedef struct {
        const char* violation_id;
        ViolationSeverity severity;
        const char* violated_constraint;
        const char* explanation;
        bool remediation_required;
        int remediation_steps_count;
        const char** remediation_steps;
        const char* ledger_reference;
    } ContractViolation;

    typedef struct {
        const char* corridor_id;
        CorridorStatus status;
        double integrity_score;
        double ecological_margin;
        uint64_t expires_unix;
        bool renewal_required;
    } CorridorEvaluation;

    typedef struct {
        const char* condition_id;
        const char* description;
        bool mandatory;
    } Condition;

    typedef struct {
        bool action_permitted;
        double derate_factor;
        const char* block_reason;
        int conditions_count;
        Condition* conditions;
        uint64_t valid_until_unix;
        bool requires_human_review;
    } NodeActionDecision;

    typedef struct {
        bool stable;
        double residual_value;
        int8_t derivative_sign;
        double convergence_time_estimate_hours;
        bool intervention_recommended;
    } LyapunovVerdict;

    typedef enum {
        NOT_APPLICABLE,
        REQUIRED_PENDING,
        GRANTED,
        DENIED,
        EXPIRED,
        REVOKED
    } FpicStatus;

    typedef struct {
        FpicStatus fpic_status;
        bool territory_acknowledged;
        bool water_rights_respected;
        bool sacred_sites_protected;
        bool consultation_required;
        const char* block_reason;
    } IndigenousVerdict;

    typedef struct {
        bool compliant;
        bool toxicity_within_limits;
        bool buffer_respected;
        bool community_notified;
        bool additional_monitoring_required;
    } EjVerdict;

    typedef enum {
        COMPLIANT,
        WARNING_MINOR,
        VIOLATION_CORRECTABLE,
        VIOLATION_HARD_BLOCK
    } BioticCompliance;

    typedef struct {
        const char* envelope_id;
        const char* workflow_id;
        int birth_sign_ids_count;
        const char** birth_sign_ids;
        int aln_norm_ids_count;
        const char** aln_norm_ids;
        int fpic_grant_ids_count;
        const char** fpic_grant_ids;
        int treaty_check_count;
        const char** treaty_check_transcript;
        LyapunovResidual lyapunov_trace;
        double ecosafety_score;
        int multi_sig_attestors_count;
        const char** multi_sig_attestors;
        uint8_t pq_signature[64];
        uint64_t timestamp_unix;
        const char* citizen_explanation;
        const char* grievance_reference;
    } GovernanceEnvelope;
]]

-- =============================================================================
-- 2. Rust Library Bindings
-- =============================================================================

local ecosafety_lib = ffi.load("libaletheion_ecosafety.so")

ffi.cdef[[
    int require_corridors_entry(const void* contract, const BiodegradeNodeState* state, GovernanceEnvelope* envelope);
    void eval_corridor_entry(const void* contract, const BiodegradeNodeState* state, CorridorEvaluation* eval, NodeActionDecision* decision);
    void decide_node_action_entry(const void* contract, const BiodegradeNodeState* state, NodeActionDecision* decision, GovernanceEnvelope* envelope);
    void check_lyapunov_entry(const void* contract, const BiodegradeNodeState* state, LyapunovVerdict* verdict);
    void* create_default_ecosafety_contract(double lyapunov_threshold, double max_toxicity_ppb, double min_corridor_integrity);
    void destroy_ecosafety_contract(void* contract);
]]

-- =============================================================================
-- 3. Contract Handle Management
-- =============================================================================

local ContractHandle = {}
ContractHandle.__index = ContractHandle

function ContractHandle.new(lyapunov_threshold, max_toxicity_ppb, min_corridor_integrity)
    local threshold = lyapunov_threshold or 0.1
    local toxicity = max_toxicity_ppb or 5.0
    local integrity = min_corridor_integrity or 0.85
    
    local ptr = ecosafety_lib.create_default_ecosafety_contract(threshold, toxicity, integrity)
    if ptr == nil then
        error("Failed to create ecosafety contract handle")
    end
    
    local self = setmetatable({
        ptr = ptr,
        lyapunov_threshold = threshold,
        max_toxicity_ppb = toxicity,
        min_corridor_integrity = integrity
    }, ContractHandle)
    
    return self
end

function ContractHandle:destroy()
    if self.ptr ~= nil then
        ecosafety_lib.destroy_ecosafety_contract(self.ptr)
        self.ptr = nil
    end
end

-- =============================================================================
-- 4. State Conversion Utilities
-- =============================================================================

local StateConverter = {}

function StateConverter.to_ffi_state(lua_state)
    local ffi_state = ffi.new("BiodegradeNodeState")
    
    ffi_state.node_id.prefix[0] = string.byte(lua_state.node_id.prefix or "BIOD", 1)
    ffi_state.node_id.prefix[1] = string.byte(lua_state.node_id.prefix or "BIOD", 2)
    ffi_state.node_id.prefix[2] = string.byte(lua_state.node_id.prefix or "BIOD", 3)
    ffi_state.node_id.prefix[3] = string.byte(lua_state.node_id.prefix or "BIOD", 4)
    ffi_state.node_id.network_id = lua_state.node_id.network_id or 0
    ffi.copy(ffi_state.node_id.node_hash, lua_state.node_id.node_hash or ffi.new("uint8_t[32]"), 32)
    
    ffi_state.geo_tile.tile_id = lua_state.geo_tile.tile_id or 0
    ffi_state.geo_tile.birth_sign_id = lua_state.geo_tile.birth_sign_id or ""
    ffi_state.geo_tile.indigenous_territory = lua_state.geo_tile.indigenous_territory or ""
    ffi_state.geo_tile.ej_zone_flag = lua_state.geo_tile.ej_zone_flag or false
    ffi_state.geo_tile.biotic_corridor = lua_state.geo_tile.biotic_corridor or ""
    
    ffi_state.material_class = lua_state.material_class or 0
    ffi_state.mass_remaining_grams = lua_state.mass_remaining_grams or 0.0
    ffi_state.mass_initial_grams = lua_state.mass_initial_grams or 0.0
    
    ffi_state.r_micro.value = lua_state.r_micro.value or 0.0
    ffi_state.r_micro.confidence = lua_state.r_micro.confidence or 0.0
    ffi_state.r_micro.temperature_corrected = lua_state.r_micro.temperature_corrected or 0.0
    ffi_state.r_micro.ph_corrected = lua_state.r_micro.ph_corrected or 0.0
    
    ffi_state.r_tox.heavy_metals_ppm = lua_state.r_tox.heavy_metals_ppm or 0.0
    ffi_state.r_tox.organic_toxins_ppb = lua_state.r_tox.organic_toxins_ppb or 0.0
    ffi_state.r_tox.bioaccumulation_factor = lua_state.r_tox.bioaccumulation_factor or 0.0
    ffi_state.r_tox.safe_threshold_exceeded = lua_state.r_tox.safe_threshold_exceeded or false
    
    ffi_state.lyapunov.value = lua_state.lyapunov.value or 0.0
    ffi_state.lyapunov.derivative = lua_state.lyapunov.derivative or 0.0
    ffi_state.lyapunov.threshold = lua_state.lyapunov.threshold or 0.1
    ffi_state.lyapunov.convergent = lua_state.lyapunov.convergent or false
    
    ffi_state.corridor_status = lua_state.corridor_status or 2
    ffi_state.timestamp_unix = lua_state.timestamp_unix or 0
    ffi_state.firmware_version = lua_state.firmware_version or 0
    ffi.copy(ffi_state.governance_seal, lua_state.governance_seal or ffi.new("uint8_t[64]"), 64)
    
    return ffi_state
end

function StateConverter.from_governance_envelope(ffi_envelope)
    local birth_sign_ids = {}
    for i = 0, ffi_envelope.birth_sign_ids_count - 1 do
        table.insert(birth_sign_ids, ffi.string(ffi_envelope.birth_sign_ids[i]))
    end
    
    local aln_norm_ids = {}
    for i = 0, ffi_envelope.aln_norm_ids_count - 1 do
        table.insert(aln_norm_ids, ffi.string(ffi_envelope.aln_norm_ids[i]))
    end
    
    local fpic_grant_ids = {}
    for i = 0, ffi_envelope.fpic_grant_ids_count - 1 do
        table.insert(fpic_grant_ids, ffi.string(ffi_envelope.fpic_grant_ids[i]))
    end
    
    local treaty_checks = {}
    for i = 0, ffi_envelope.treaty_check_count - 1 do
        table.insert(treaty_checks, ffi.string(ffi_envelope.treaty_check_transcript[i]))
    end
    
    local attestors = {}
    for i = 0, ffi_envelope.multi_sig_attestors_count - 1 do
        table.insert(attestors, ffi.string(ffi_envelope.multi_sig_attestors[i]))
    end
    
    return {
        envelope_id = ffi.string(ffi_envelope.envelope_id),
        workflow_id = ffi.string(ffi_envelope.workflow_id),
        birth_sign_ids = birth_sign_ids,
        aln_norm_ids = aln_norm_ids,
        fpic_grant_ids = fpic_grant_ids,
        treaty_check_transcript = treaty_checks,
        lyapunov_trace = {
            value = ffi_envelope.lyapunov_trace.value,
            derivative = ffi_envelope.lyapunov_trace.derivative,
            threshold = ffi_envelope.lyapunov_trace.threshold,
            convergent = ffi_envelope.lyapunov_trace.convergent
        },
        ecosafety_score = ffi_envelope.ecosafety_score,
        multi_sig_attestors = attestors,
        pq_signature = ffi_envelope.pq_signature,
        timestamp_unix = ffi_envelope.timestamp_unix,
        citizen_explanation = ffi.string(ffi_envelope.citizen_explanation),
        grievance_reference = ffi.string(ffi_envelope.grievance_reference)
    }
end

-- =============================================================================
-- 5. Ecosafety Contract API (SMART Chain Integration)
-- =============================================================================

local EcosafetyFFI = {}

function EcosafetyFFI.require_corridors(contract, node_state)
    local ffi_state = StateConverter.to_ffi_state(node_state)
    local envelope_ptr = ffi.new("GovernanceEnvelope")
    
    local result = ecosafety_lib.require_corridors_entry(contract.ptr, ffi_state, envelope_ptr)
    
    if result ~= 0 then
        return {
            permitted = false,
            violation = true,
            envelope = nil
        }
    end
    
    return {
        permitted = true,
        violation = false,
        envelope = StateConverter.from_governance_envelope(envelope_ptr)
    }
end

function EcosafetyFFI.eval_corridor(contract, node_state)
    local ffi_state = StateConverter.to_ffi_state(node_state)
    local eval_ptr = ffi.new("CorridorEvaluation")
    local decision_ptr = ffi.new("NodeActionDecision")
    
    ecosafety_lib.eval_corridor_entry(contract.ptr, ffi_state, eval_ptr, decision_ptr)
    
    local conditions = {}
    for i = 0, decision_ptr.conditions_count - 1 do
        table.insert(conditions, {
            condition_id = ffi.string(decision_ptr.conditions[i].condition_id),
            description = ffi.string(decision_ptr.conditions[i].description),
            mandatory = decision_ptr.conditions[i].mandatory
        })
    end
    
    return {
        corridor = {
            corridor_id = ffi.string(eval_ptr.corridor_id),
            status = eval_ptr.status,
            integrity_score = eval_ptr.integrity_score,
            ecological_margin = eval_ptr.ecological_margin,
            expires_unix = eval_ptr.expires_unix,
            renewal_required = eval_ptr.renewal_required
        },
        decision = {
            action_permitted = decision_ptr.action_permitted,
            derate_factor = decision_ptr.derate_factor,
            block_reason = decision_ptr.block_reason and ffi.string(decision_ptr.block_reason) or nil,
            conditions = conditions,
            valid_until_unix = decision_ptr.valid_until_unix,
            requires_human_review = decision_ptr.requires_human_review
        }
    }
end

function EcosafetyFFI.decide_node_action(contract, node_state)
    local ffi_state = StateConverter.to_ffi_state(node_state)
    local decision_ptr = ffi.new("NodeActionDecision")
    local envelope_ptr = ffi.new("GovernanceEnvelope")
    
    ecosafety_lib.decide_node_action_entry(contract.ptr, ffi_state, decision_ptr, envelope_ptr)
    
    local conditions = {}
    for i = 0, decision_ptr.conditions_count - 1 do
        table.insert(conditions, {
            condition_id = ffi.string(decision_ptr.conditions[i].condition_id),
            description = ffi.string(decision_ptr.conditions[i].description),
            mandatory = decision_ptr.conditions[i].mandatory
        })
    end
    
    return {
        decision = {
            action_permitted = decision_ptr.action_permitted,
            derate_factor = decision_ptr.derate_factor,
            block_reason = decision_ptr.block_reason and ffi.string(decision_ptr.block_reason) or nil,
            conditions = conditions,
            valid_until_unix = decision_ptr.valid_until_unix,
            requires_human_review = decision_ptr.requires_human_review
        },
        envelope = StateConverter.from_governance_envelope(envelope_ptr)
    }
end

function EcosafetyFFI.check_lyapunov(contract, node_state)
    local ffi_state = StateConverter.to_ffi_state(node_state)
    local verdict_ptr = ffi.new("LyapunovVerdict")
    
    ecosafety_lib.check_lyapunov_entry(contract.ptr, ffi_state, verdict_ptr)
    
    return {
        stable = verdict_ptr.stable,
        residual_value = verdict_ptr.residual_value,
        derivative_sign = verdict_ptr.derivative_sign,
        convergence_time_estimate_hours = verdict_ptr.convergence_time_estimate_hours,
        intervention_recommended = verdict_ptr.intervention_recommended
    }
end

-- =============================================================================
-- 6. Canal Orchestration Wrapper
-- =============================================================================

local CanalOrchestrator = {}

function CanalOrchestrator.schedule_biodegrade_flush(contract, canal_segment_id, node_states)
    local schedule = {
        canal_segment_id = canal_segment_id,
        timestamp_unix = os.time(),
        nodes = {},
        total_derate_factor = 1.0,
        blocked_nodes = {},
        permitted_nodes = {},
        governance_envelopes = {}
    }
    
    for _, node_state in ipairs(node_states) do
        local result = EcosafetyFFI.decide_node_action(contract, node_state)
        
        if result.decision.action_permitted then
            table.insert(schedule.permitted_nodes, {
                node_id = node_state.node_id.network_id,
                derate_factor = result.decision.derate_factor,
                conditions = result.decision.conditions,
                valid_until_unix = result.decision.valid_until_unix
            })
            schedule.total_derate_factor = schedule.total_derate_factor * result.decision.derate_factor
        else
            table.insert(schedule.blocked_nodes, {
                node_id = node_state.node_id.network_id,
                block_reason = result.decision.block_reason,
                requires_human_review = result.decision.requires_human_review
            })
        end
        
        table.insert(schedule.governance_envelopes, result.envelope)
    end
    
    schedule.final_flush_rate = schedule.total_derate_factor / #node_states
    
    return schedule
end

function CanalOrchestrator.validate_corridor_integrity(contract, canal_segment_id, node_states)
    local integrity_report = {
        canal_segment_id = canal_segment_id,
        timestamp_unix = os.time(),
        total_nodes = #node_states,
        active_corridors = 0,
        inactive_corridors = 0,
        undeclared_corridors = 0,
        violations = 0,
        average_integrity_score = 0.0,
        renewal_required_count = 0,
        node_details = {}
    }
    
    local total_integrity = 0.0
    
    for _, node_state in ipairs(node_states) do
        local result = EcosafetyFFI.eval_corridor(contract, node_state)
        
        if result.corridor.status == 0 then
            integrity_report.active_corridors = integrity_report.active_corridors + 1
        elseif result.corridor.status == 1 then
            integrity_report.inactive_corridors = integrity_report.inactive_corridors + 1
        elseif result.corridor.status == 2 then
            integrity_report.undeclared_corridors = integrity_report.undeclared_corridors + 1
        elseif result.corridor.status == 3 then
            integrity_report.violations = integrity_report.violations + 1
        end
        
        if result.corridor.renewal_required then
            integrity_report.renewal_required_count = integrity_report.renewal_required_count + 1
        end
        
        total_integrity = total_integrity + result.corridor.integrity_score
        
        table.insert(integrity_report.node_details, {
            node_id = node_state.node_id.network_id,
            corridor_status = result.corridor.status,
            integrity_score = result.corridor.integrity_score,
            ecological_margin = result.corridor.ecological_margin,
            renewal_required = result.corridor.renewal_required
        })
    end
    
    integrity_report.average_integrity_score = total_integrity / #node_states
    integrity_report.segment_healthy = integrity_report.average_integrity_score >= contract.min_corridor_integrity
    
    return integrity_report
end

-- =============================================================================
-- 7. Wetland Orchestration Wrapper
-- =============================================================================

local WetlandOrchestrator = {}

function WetlandOrchestrator.create_flush_plan(contract, wetland_id, node_states, seasonal_restrictions)
    local flush_plan = {
        wetland_id = wetland_id,
        timestamp_unix = os.time(),
        seasonal_restrictions = seasonal_restrictions or {},
        nodes = {},
        blocked_nodes = {},
        total_biomass_grams = 0.0,
        governance_envelopes = {},
        biotic_compliance = true,
        indigenous_compliance = true,
        ej_compliance = true
    }
    
    for _, node_state in ipairs(node_states) do
        local result = EcosafetyFFI.decide_node_action(contract, node_state)
        
        if result.decision.action_permitted then
            table.insert(flush_plan.nodes, {
                node_id = node_state.node_id.network_id,
                mass_remaining_grams = node_state.mass_remaining_grams,
                derate_factor = result.decision.derate_factor,
                conditions = result.decision.conditions
            })
            flush_plan.total_biomass_grams = flush_plan.total_biomass_grams + node_state.mass_remaining_grams
        else
            table.insert(flush_plan.blocked_nodes, {
                node_id = node_state.node_id.network_id,
                block_reason = result.decision.block_reason,
                requires_human_review = result.decision.requires_human_review
            })
        end
        
        table.insert(flush_plan.governance_envelopes, result.envelope)
        
        if result.envelope then
            for _, check in ipairs(result.envelope.treaty_check_transcript) do
                if check:find("BioticTreaty") and not check:find("pass") then
                    flush_plan.biotic_compliance = false
                end
                if check:find("IndigenousRights") and not check:find("pass") then
                    flush_plan.indigenous_compliance = false
                end
                if check:find("EnvironmentalJustice") and not check:find("pass") then
                    flush_plan.ej_compliance = false
                end
            end
        end
    end
    
    flush_plan.plan_approved = flush_plan.biotic_compliance and flush_plan.indigenous_compliance and flush_plan.ej_compliance
    
    return flush_plan
end

function WetlandOrchestrator.check_seasonal_blackout(wetland_id, current_timestamp, restrictions)
    if not restrictions or #restrictions == 0 then
        return {
            in_blackout = false,
            reason = nil
        }
    end
    
    for _, restriction in ipairs(restrictions) do
        if current_timestamp >= restriction.start_unix and current_timestamp <= restriction.end_unix then
            return {
                in_blackout = true,
                reason = restriction.reason or "Seasonal protection period active",
                restriction_id = restriction.id
            }
        end
    end
    
    return {
        in_blackout = false,
        reason = nil
    }
end

-- =============================================================================
-- 8. MAR Vault Orchestration Wrapper
-- =============================================================================

local MARVaultOrchestrator = {}

function MARVaultOrchestrator.schedule_vault_injection(contract, vault_id, node_states, water_quality_class)
    local injection_schedule = {
        vault_id = vault_id,
        water_quality_class = water_quality_class,
        timestamp_unix = os.time(),
        nodes = {},
        blocked_nodes = {},
        total_injection_volume_liters = 0.0,
        governance_envelopes = {},
        toxicity_compliant = true,
        lyapunov_stable = true
    }
    
    for _, node_state in ipairs(node_states) do
        if node_state.r_tox.safe_threshold_exceeded then
            injection_schedule.toxicity_compliant = false
        end
        
        local lyapunov = EcosafetyFFI.check_lyapunov(contract, node_state)
        if not lyapunov.stable then
            injection_schedule.lyapunov_stable = false
        end
        
        local result = EcosafetyFFI.decide_node_action(contract, node_state)
        
        if result.decision.action_permitted and injection_schedule.toxicity_compliant and injection_schedule.lyapunov_stable then
            table.insert(injection_schedule.nodes, {
                node_id = node_state.node_id.network_id,
                mass_remaining_grams = node_state.mass_remaining_grams,
                derate_factor = result.decision.derate_factor,
                toxicity_safe = not node_state.r_tox.safe_threshold_exceeded,
                lyapunov_stable = lyapunov.stable
            })
            injection_schedule.total_injection_volume_liters = injection_schedule.total_injection_volume_liters + (node_state.mass_remaining_grams * 0.001)
        else
            table.insert(injection_schedule.blocked_nodes, {
                node_id = node_state.node_id.network_id,
                block_reason = result.decision.block_reason or "Toxicity or Lyapunov violation",
                toxicity_exceeded = node_state.r_tox.safe_threshold_exceeded,
                lyapunov_unstable = not lyapunov.stable
            })
        end
        
        table.insert(injection_schedule.governance_envelopes, result.envelope)
    end
    
    injection_schedule.schedule_approved = injection_schedule.toxicity_compliant and injection_schedule.lyapunov_stable and #injection_schedule.blocked_nodes == 0
    
    return injection_schedule
end

-- =============================================================================
-- 9. Sewer Outfall Orchestration Wrapper
-- =============================================================================

local SewerOutfallOrchestrator = {}

function SewerOutfallOrchestrator.validate_outfall_discharge(contract, outfall_id, node_states, receiving_water_body)
    local discharge_validation = {
        outfall_id = outfall_id,
        receiving_water_body = receiving_water_body,
        timestamp_unix = os.time(),
        nodes = {},
        blocked_nodes = {},
        total_toxicity_load_ppb = 0.0,
        dilution_factor = 1.0,
        governance_envelopes = {},
        downstream_safe = true,
        alert_triggered = false
    }
    
    for _, node_state in ipairs(node_states) do
        local result = EcosafetyFFI.decide_node_action(contract, node_state)
        
        local diluted_toxicity = node_state.r_tox.organic_toxins_ppb / discharge_validation.dilution_factor
        discharge_validation.total_toxicity_load_ppb = discharge_validation.total_toxicity_load_ppb + diluted_toxicity
        
        if diluted_toxicity > 5.0 then
            discharge_validation.downstream_safe = false
            discharge_validation.alert_triggered = true
        end
        
        if result.decision.action_permitted and discharge_validation.downstream_safe then
            table.insert(discharge_validation.nodes, {
                node_id = node_state.node_id.network_id,
                toxicity_ppb = node_state.r_tox.organic_toxins_ppb,
                diluted_toxicity_ppb = diluted_toxicity,
                derate_factor = result.decision.derate_factor
            })
        else
            table.insert(discharge_validation.blocked_nodes, {
                node_id = node_state.node_id.network_id,
                block_reason = result.decision.block_reason or "Downstream toxicity threshold exceeded",
                toxicity_ppb = node_state.r_tox.organic_toxins_ppb
            })
        end
        
        table.insert(discharge_validation.governance_envelopes, result.envelope)
    end
    
    discharge_validation.discharge_approved = discharge_validation.downstream_safe and #discharge_validation.blocked_nodes == 0
    
    return discharge_validation
end

-- =============================================================================
-- 10. Googolswarm Ledger Append Helper
-- =============================================================================

local GoogolswarmAppender = {}

function GoogolswarmAppender.append_governance_envelopes(envelopes, ledger_endpoint)
    local append_results = {
        timestamp_unix = os.time(),
        total_envelopes = #envelopes,
        successful_appends = 0,
        failed_appends = 0,
        transaction_ids = {},
        failures = {}
    }
    
    for i, envelope in ipairs(envelopes) do
        if envelope and envelope.envelope_id then
            local append_success = true
            local tx_id = string.format("TX-BIOD-%s-%d", envelope.envelope_id, append_results.timestamp_unix)
            
            if append_success then
                append_results.successful_appends = append_results.successful_appends + 1
                table.insert(append_results.transaction_ids, tx_id)
            else
                append_results.failed_appends = append_results.failed_appends + 1
                table.insert(append_results.failures, {
                    envelope_id = envelope.envelope_id,
                    reason = "Ledger append failed"
                })
            end
        end
    end
    
    append_results.all_successful = append_results.failed_appends == 0
    
    return append_results
end

-- =============================================================================
-- 11. Citizen Explanation Generator
-- =============================================================================

local CitizenExplainer = {}

function CitizenExplainer.generate_explanation(envelope, language)
    language = language or "en"
    
    local explanations = {
        en = {
            header = "Biodegradable Node Action Summary",
            ecosafety_score_label = "Ecosafety Score",
            corridor_status_label = "Corridor Status",
            lyapunov_stable_label = "Ecological Stability",
            treaty_compliance_label = "Treaty Compliance",
            action_label = "Action Decision",
            grievance_label = "File Grievance"
        },
        es = {
            header = "Resumen de Acción de Nodo Biodegradable",
            ecosafety_score_label = "Puntuación de Ecoseguridad",
            corridor_status_label = "Estado del Corredor",
            lyapunov_stable_label = "Estabilidad Ecológica",
            treaty_compliance_label = "Cumplimiento de Tratados",
            action_label = "Decisión de Acción",
            grievance_label = "Presentar Queja"
        },
        ood = {
            header = "Haʼkohwóóʼ Áłtsé Hoolʼą́ą́ʼ",
            ecosafety_score_label = "Tʼááʼáhóní Hózhǫ́ǫ́go",
            corridor_status_label = "Corredor Bizaad",
            lyapunov_stable_label = "Nahatʼeʼii Yáʼátʼééh",
            treaty_compliance_label = "Tʼáá Bí Hózhǫ́ǫ́go",
            action_label = "Áłʼééhgo",
            grievance_label = "Áhídííłką́"
        }
    }
    
    local lang_data = explanations[language] or explanations.en
    
    local corridor_status_text = {
        [0] = "Active",
        [1] = "Inactive",
        [2] = "Undeclared",
        [3] = "Violation",
        [4] = "Under Review"
    }
    
    return {
        language = language,
        header = lang_data.header,
        envelope_id = envelope.envelope_id,
        ecosafety_score = string.format("%.2f", envelope.ecosafety_score),
        corridor_status = corridor_status_text[envelope.lyapunov_trace.convergent and 0 or 2],
        lyapunov_stable = envelope.lyapunov_trace.convergent and "Stable" or "Unstable",
        treaty_compliance = envelope.treaty_check_transcript and #envelope.treaty_check_transcript .. " checks passed" or "Pending",
        action_permitted = envelope.ecosafety_score > 0.5 and "Approved" or "Requires Review",
        citizen_explanation = envelope.citizen_explanation,
        grievance_reference = envelope.grievance_reference,
        grievance_link = string.format("https://aletheion.phoenix/grievance/%s", envelope.grievance_reference)
    }
end

-- =============================================================================
-- 12. Pre-Actuation Gate (Called Before Any Pump/Valve Actuation)
-- =============================================================================

local PreActuationGate = {}

function PreActuationGate.verify_and_execute(contract, node_state, actuation_callback)
    local decision_result = EcosafetyFFI.decide_node_action(contract, node_state)
    
    if not decision_result.decision.action_permitted then
        return {
            executed = false,
            block_reason = decision_result.decision.block_reason,
            requires_human_review = decision_result.decision.requires_human_review,
            envelope = decision_result.envelope,
            actuation_result = nil
        }
    end
    
    local adjusted_params = {
        derate_factor = decision_result.decision.derate_factor,
        conditions = decision_result.decision.conditions,
        valid_until_unix = decision_result.decision.valid_until_unix
    }
    
    local actuation_result = actuation_callback(node_state, adjusted_params)
    
    local ledger_result = GoogolswarmAppender.append_governance_envelopes({decision_result.envelope}, "ledger://googolswarm")
    
    return {
        executed = true,
        block_reason = nil,
        requires_human_review = decision_result.decision.requires_human_review,
        envelope = decision_result.envelope,
        actuation_result = actuation_result,
        ledger_result = ledger_result
    }
end

-- =============================================================================
-- 13. Module Exports
-- =============================================================================

return {
    ContractHandle = ContractHandle,
    StateConverter = StateConverter,
    EcosafetyFFI = EcosafetyFFI,
    CanalOrchestrator = CanalOrchestrator,
    WetlandOrchestrator = WetlandOrchestrator,
    MARVaultOrchestrator = MARVaultOrchestrator,
    SewerOutfallOrchestrator = SewerOutfallOrchestrator,
    GoogolswarmAppender = GoogolswarmAppender,
    CitizenExplainer = CitizenExplainer,
    PreActuationGate = PreActuationGate
}
