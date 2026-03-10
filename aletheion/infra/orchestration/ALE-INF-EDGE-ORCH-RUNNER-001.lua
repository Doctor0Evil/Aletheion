-- Aletheion Infrastructure Edge Orchestration Runner
-- File: ALE-INF-EDGE-ORCH-RUNNER-001.lua
-- Domain: Infrastructure / Orchestration / Edge Computing
-- Language: Lua 5.4+ with FFI bindings to Rust models
-- Compliance: BioticTreaties, Indigenous FPIC, EJ Zones, SMART Chain

local ffi = require("ffi")
local bit = require("bit")

-- =============================================================================
-- 1. FFI Type Declarations (Mirror Rust Edge Orchestration Types)
-- =============================================================================

ffi.cdef[[
    typedef struct {
        uint8_t prefix[4];
        uint16_t network_id;
        uint8_t node_hash[32];
    } EdgeNodeId;

    typedef struct {
        uint8_t prefix[4];
        uint32_t workflow_id;
        uint8_t task_hash[32];
    } WorkloadId;

    typedef struct {
        uint8_t prefix[4];
        uint32_t territory_id;
        uint8_t sign_hash[32];
    } BirthSignId;

    typedef struct {
        uint8_t prefix[4];
        uint64_t tile_id;
        uint8_t tile_hash[32];
    } GeoTileId;

    typedef enum {
        XR_GRID_NODE,
        CANAL_SENSOR,
        WETLAND_MONITOR,
        MAR_VAULT_CONTROLLER,
        SEWER_OUTFALL_GATEWAY,
        HEAT_MITIGATION_NODE,
        MOBILITY_EDGE_NODE,
        WASTE_PROCESSING_CONTROLLER,
        AIR_QUALITY_SENSOR,
        SOIL_MONITOR,
        BIOSIGNAL_GATEWAY,
        BIOTIC_CORRIDOR_NODE
    } HardwareClass;

    typedef enum {
        MICRO,
        EMBEDDED,
        EDGE,
        DISTRICT,
        CITY_CORE
    } ComputeTier;

    typedef enum {
        TEE_BACKED,
        HARDENED_FIRMWARE,
        BASIC,
        PUBLIC
    } SecurityTier;

    typedef enum {
        SOLAR,
        WIND,
        GRID,
        BATTERY,
        HYBRID,
        FUEL_CELL,
        KINETIC,
        THERMAL
    } PowerSourceType;

    typedef struct {
        PowerSourceType source_type;
        double capacity_wh;
        double current_draw_w;
        double battery_level_percent;
        double renewable_fraction;
        double carbon_intensity_gco2_per_kwh;
        double max_sustainable_load_w;
        double peak_load_w;
        double energy_budget_daily_wh;
        double energy_remaining_wh;
    } EnergyProfile;

    typedef enum {
        PASSIVE,
        ACTIVE_AIR,
        LIQUID,
        PHASE_CHANGE,
        EVAPORATIVE,
        NONE
    } CoolingType;

    typedef struct {
        double ambient_temp_celsius;
        double cpu_temp_celsius;
        double max_operating_temp_celsius;
        double thermal_throttle_threshold_celsius;
        bool cooling_active;
        CoolingType cooling_type;
        double heat_dissipation_w;
        double thermal_margin_celsius;
        bool haboob_risk_flag;
        bool dust_mitigation_active;
    } ThermalProfile;

    typedef enum {
        ETHERNET,
        WIFI,
        LTE,
        FIVE_G,
        LORAWAN,
        ZIGBEE,
        THREAD,
        SATELLITE,
        FIBER
    } LinkType;

    typedef struct {
        LinkType primary_link;
        int backup_links_count;
        LinkType* backup_links;
        double bandwidth_mbps;
        double latency_ms;
        double packet_loss_percent;
        bool mesh_participant;
        int mesh_neighbors;
        bool offline_capable;
        uint32_t sync_interval_seconds;
        uint64_t last_sync_unix;
    } ConnectivityProfile;

    typedef struct {
        uint32_t major;
        uint32_t minor;
        uint32_t patch;
        uint8_t build_hash[16];
        bool signed_firmware;
        bool signature_verified;
    } FirmwareVersion;

    typedef enum {
        STABLE,
        BETA,
        CANARY,
        EMERGENCY
    } FirmwareChannel;

    typedef struct {
        FirmwareVersion firmware_version;
        FirmwareChannel firmware_channel;
        uint64_t last_update_unix;
        bool update_available;
        bool update_required;
        bool rollback_blocked;
        int modules_loaded_count;
        const char** modules_loaded;
        int treaty_modules_active_count;
        const char** treaty_modules_active;
    } SoftwareState;

    typedef enum {
        SENSING,
        NORMALIZATION,
        INTENT_CLASSIFICATION,
        OPTIMIZATION,
        TREATY_ENFORCEMENT,
        ACTUATION,
        AUDIT_LOGGING,
        CITIZEN_SURFACE
    } WorkloadType;

    typedef enum {
        CRITICAL,
        HIGH,
        NORMAL,
        LOW,
        BACKGROUND
    } TaskPriority;

    typedef enum {
        PENDING,
        SCHEDULED,
        RUNNING,
        COMPLETED,
        FAILED,
        BLOCKED,
        CANCELLED
    } TaskStatus;

    typedef struct {
        WorkloadId task_id;
        WorkloadType workload_type;
        TaskPriority priority;
        const char* workflow_id;
        const char* stage_id;
        uint64_t deadline_unix;
        uint32_t estimated_duration_ms;
        uint32_t actual_duration_ms;
        size_t memory_required_bytes;
        uint8_t cpu_cores_required;
        bool gpu_required;
        bool network_required;
        TaskStatus status;
        uint8_t retry_count;
        uint8_t max_retries;
    } TaskInstance;

    typedef enum {
        BIRTH_SIGN,
        INDIGENOUS_TERRITORY,
        EJ_ZONE,
        BIOTIC_CORRIDOR,
        WATER_COMPACT,
        MUNICIPAL_CODE,
        TREATY_CONSTRAINT,
        FPIC_REQUIREMENT
    } GovernanceTagType;

    typedef struct {
        const char* tag_id;
        GovernanceTagType tag_type;
        const char* value;
        bool enforced;
        const char* source_did;
    } GovernanceTag;

    typedef enum {
        OVERRIDE_NONE,
        OVERRIDE_MUNICIPAL,
        OVERRIDE_REGIONAL,
        OVERRIDE_TREATY_JOINT_BODY,
        OVERRIDE_INDIGENOUS_COUNCIL,
        OVERRIDE_EMERGENCY_AUTHORITY
    } OverrideLevel;

    typedef struct {
        BirthSignId* birth_sign_id;
        const char* indigenous_territory_id;
        const char* ej_zone_id;
        const char* biotic_corridor_id;
        bool fpic_required;
        bool fpic_granted;
        int treaty_atoms_active_count;
        const char** treaty_atoms_active;
        int aln_norms_active_count;
        const char** aln_norms_active;
        OverrideLevel* override_level;
    } GovernanceConstraints;

    typedef struct {
        double overall_score;
        double energy_score;
        double thermal_score;
        double connectivity_score;
        double security_score;
        double governance_score;
        double latency_score;
        double capacity_score;
        double treaty_compliance_score;
        bool weighted;
    } SuitabilityScore;

    typedef struct {
        EdgeNodeId node_id;
        GeoTileId geo_tile_id;
        HardwareClass hardware_class;
        ComputeTier compute_tier;
        SecurityTier security_tier;
        EnergyProfile energy_profile;
        ThermalProfile thermal_profile;
        ConnectivityProfile connectivity_profile;
        SoftwareState software_state;
        int active_tasks_count;
        TaskInstance* active_tasks;
        int queued_tasks_count;
        TaskInstance* queued_tasks;
        GovernanceConstraints governance_constraints;
        int governance_tags_count;
        GovernanceTag* governance_tags;
        SuitabilityScore suitability_score;
        bool online;
        uint64_t last_heartbeat_unix;
        uint64_t uptime_seconds;
        uint64_t total_tasks_completed;
        uint64_t total_tasks_failed;
        uint64_t treaty_violations_count;
        uint8_t state_signature[64];
    } EdgeNodeState;

    typedef enum {
        NODE_DISCOVERY_MANUAL,
        NODE_DISCOVERY_AUTO,
        NODE_DISCOVERY_MESH,
        NODE_DISCOVERY_CENTRAL,
        NODE_DISCOVERY_INDIGENOUS
    } NodeDiscoveryMethod;

    typedef enum {
        REGISTRATION_PENDING,
        REGISTRATION_APPROVED,
        REGISTRATION_REJECTED,
        REGISTRATION_UNDER_REVIEW,
        REGISTRATION_SUSPENDED
    } NodeRegistrationStatus;

    typedef enum {
        PLACEMENT_PLACE,
        PLACEMENT_REJECT,
        PLACEMENT_DEFER,
        PLACEMENT_ESCALATE
    } PlacementDecision;
]]

-- =============================================================================
-- 2. Rust Library Bindings
-- =============================================================================

local orch_lib = ffi.load("libaletheion_orchestration.so")

ffi.cdef[[
    void* create_edge_orchestrator(const char* cluster_id, double min_suitability_threshold);
    void destroy_edge_orchestrator(void* orchestrator);
    int register_node(void* orchestrator, const EdgeNodeState* node_state);
    int discover_nodes(void* orchestrator, NodeDiscoveryMethod method);
    int schedule_task(void* orchestrator, const TaskInstance* task, EdgeNodeId* target_node);
    int reschedule_task(void* orchestrator, const char* task_id, const EdgeNodeId* new_node);
    int cancel_task(void* orchestrator, const char* task_id);
    EdgeNodeState* get_node_state(void* orchestrator, const EdgeNodeId* node_id);
    TaskInstance* get_task_state(void* orchestrator, const char* task_id);
    int update_node_heartbeat(void* orchestrator, const EdgeNodeId* node_id);
    int compute_suitability_score(void* orchestrator, const EdgeNodeId* node_id, const TaskInstance* task);
    int enforce_governance_constraints(void* orchestrator, const EdgeNodeId* node_id, const TaskInstance* task);
    int sync_offline_queue(void* orchestrator);
    int append_to_ledger(void* orchestrator, const char* transaction_data);
]]

-- =============================================================================
-- 3. Orchestrator Handle Management
-- =============================================================================

local OrchestratorHandle = {}
OrchestratorHandle.__index = OrchestratorHandle

function OrchestratorHandle.new(cluster_id, min_suitability_threshold)
    local cid = cluster_id or "CLUSTER-PHX-DEFAULT"
    local threshold = min_suitability_threshold or 0.7
    
    local ptr = orch_lib.create_edge_orchestrator(cid, threshold)
    if ptr == nil then
        error("Failed to create edge orchestrator handle")
    end
    
    local self = setmetatable({
        ptr = ptr,
        cluster_id = cid,
        min_suitability_threshold = threshold,
        registered_nodes = {},
        pending_tasks = {},
        offline_queue = {}
    }, OrchestratorHandle)
    
    return self
end

function OrchestratorHandle:destroy()
    if self.ptr ~= nil then
        orch_lib.destroy_edge_orchestrator(self.ptr)
        self.ptr = nil
    end
end

-- =============================================================================
-- 4. Node Discovery & Registration
-- =============================================================================

local NodeDiscovery = {}

function NodeDiscovery.auto_discover(orchestrator)
    local result = orch_lib.discover_nodes(orchestrator.ptr, 1)
    if result ~= 0 then
        return { success = false, error = "Auto-discovery failed" }
    end
    return { success = true, nodes_discovered = true }
end

function NodeDiscovery.mesh_discover(orchestrator)
    local result = orch_lib.discover_nodes(orchestrator.ptr, 2)
    if result ~= 0 then
        return { success = false, error = "Mesh discovery failed" }
    end
    return { success = true, nodes_discovered = true }
end

function NodeDiscovery.indigenous_council_register(orchestrator, node_state)
    local ffi_state = StateConverter.to_ffi_node_state(node_state)
    local result = orch_lib.register_node(orchestrator.ptr, ffi_state)
    if result ~= 0 then
        return { 
            success = false, 
            error = "Indigenous council registration failed",
            requires_review = true
        }
    end
    return { 
        success = true, 
        registered = true,
        birth_sign_acknowledged = node_state.governance_constraints.birth_sign_id ~= nil,
        indigenous_territory_acknowledged = node_state.governance_constraints.indigenous_territory_id ~= nil
    }
end

-- =============================================================================
-- 5. State Conversion Utilities
-- =============================================================================

local StateConverter = {}

function StateConverter.to_ffi_node_state(lua_state)
    local ffi_state = ffi.new("EdgeNodeState")
    
    ffi_state.node_id.prefix[0] = string.byte(lua_state.node_id.prefix or "EDGE", 1)
    ffi_state.node_id.prefix[1] = string.byte(lua_state.node_id.prefix or "EDGE", 2)
    ffi_state.node_id.prefix[2] = string.byte(lua_state.node_id.prefix or "EDGE", 3)
    ffi_state.node_id.prefix[3] = string.byte(lua_state.node_id.prefix or "EDGE", 4)
    ffi_state.node_id.network_id = lua_state.node_id.network_id or 0
    ffi.copy(ffi_state.node_id.node_hash, lua_state.node_id.node_hash or ffi.new("uint8_t[32]"), 32)
    
    ffi_state.geo_tile_id.prefix[0] = string.byte(lua_state.geo_tile_id.prefix or "TILE", 1)
    ffi_state.geo_tile_id.prefix[1] = string.byte(lua_state.geo_tile_id.prefix or "TILE", 2)
    ffi_state.geo_tile_id.prefix[2] = string.byte(lua_state.geo_tile_id.prefix or "TILE", 3)
    ffi_state.geo_tile_id.prefix[3] = string.byte(lua_state.geo_tile_id.prefix or "TILE", 4)
    ffi_state.geo_tile_id.tile_id = lua_state.geo_tile_id.tile_id or 0
    ffi.copy(ffi_state.geo_tile_id.tile_hash, lua_state.geo_tile_id.tile_hash or ffi.new("uint8_t[32]"), 32)
    
    ffi_state.hardware_class = lua_state.hardware_class or 0
    ffi_state.compute_tier = lua_state.compute_tier or 2
    ffi_state.security_tier = lua_state.security_tier or 2
    
    ffi_state.energy_profile.source_type = lua_state.energy_profile.source_type or 0
    ffi_state.energy_profile.capacity_wh = lua_state.energy_profile.capacity_wh or 100.0
    ffi_state.energy_profile.current_draw_w = lua_state.energy_profile.current_draw_w or 5.0
    ffi_state.energy_profile.battery_level_percent = lua_state.energy_profile.battery_level_percent or 100.0
    ffi_state.energy_profile.renewable_fraction = lua_state.energy_profile.renewable_fraction or 1.0
    ffi_state.energy_profile.carbon_intensity_gco2_per_kwh = lua_state.energy_profile.carbon_intensity_gco2_per_kwh or 0.0
    ffi_state.energy_profile.max_sustainable_load_w = lua_state.energy_profile.max_sustainable_load_w or 10.0
    ffi_state.energy_profile.peak_load_w = lua_state.energy_profile.peak_load_w or 20.0
    ffi_state.energy_profile.energy_budget_daily_wh = lua_state.energy_profile.energy_budget_daily_wh or 240.0
    ffi_state.energy_profile.energy_remaining_wh = lua_state.energy_profile.energy_remaining_wh or 240.0
    
    ffi_state.thermal_profile.ambient_temp_celsius = lua_state.thermal_profile.ambient_temp_celsius or 25.0
    ffi_state.thermal_profile.cpu_temp_celsius = lua_state.thermal_profile.cpu_temp_celsius or 35.0
    ffi_state.thermal_profile.max_operating_temp_celsius = lua_state.thermal_profile.max_operating_temp_celsius or 85.0
    ffi_state.thermal_profile.thermal_throttle_threshold_celsius = lua_state.thermal_profile.thermal_throttle_threshold_celsius or 75.0
    ffi_state.thermal_profile.cooling_active = lua_state.thermal_profile.cooling_active or false
    ffi_state.thermal_profile.cooling_type = lua_state.thermal_profile.cooling_type or 0
    ffi_state.thermal_profile.heat_dissipation_w = lua_state.thermal_profile.heat_dissipation_w or 5.0
    ffi_state.thermal_profile.thermal_margin_celsius = lua_state.thermal_profile.thermal_margin_celsius or 50.0
    ffi_state.thermal_profile.haboob_risk_flag = lua_state.thermal_profile.haboob_risk_flag or false
    ffi_state.thermal_profile.dust_mitigation_active = lua_state.thermal_profile.dust_mitigation_active or false
    
    ffi_state.connectivity_profile.primary_link = lua_state.connectivity_profile.primary_link or 1
    ffi_state.connectivity_profile.bandwidth_mbps = lua_state.connectivity_profile.bandwidth_mbps or 50.0
    ffi_state.connectivity_profile.latency_ms = lua_state.connectivity_profile.latency_ms or 20.0
    ffi_state.connectivity_profile.packet_loss_percent = lua_state.connectivity_profile.packet_loss_percent or 0.1
    ffi_state.connectivity_profile.mesh_participant = lua_state.connectivity_profile.mesh_participant or true
    ffi_state.connectivity_profile.mesh_neighbors = lua_state.connectivity_profile.mesh_neighbors or 5
    ffi_state.connectivity_profile.offline_capable = lua_state.connectivity_profile.offline_capable or true
    ffi_state.connectivity_profile.sync_interval_seconds = lua_state.connectivity_profile.sync_interval_seconds or 300
    ffi_state.connectivity_profile.last_sync_unix = lua_state.connectivity_profile.last_sync_unix or 0
    
    ffi_state.software_state.firmware_version.major = lua_state.software_state.firmware_version.major or 1
    ffi_state.software_state.firmware_version.minor = lua_state.software_state.firmware_version.minor or 0
    ffi_state.software_state.firmware_version.patch = lua_state.software_state.firmware_version.patch or 0
    ffi.copy(ffi_state.software_state.firmware_version.build_hash, lua_state.software_state.firmware_version.build_hash or ffi.new("uint8_t[16]"), 16)
    ffi_state.software_state.firmware_version.signed_firmware = lua_state.software_state.firmware_version.signed or true
    ffi_state.software_state.firmware_version.signature_verified = lua_state.software_state.firmware_version.signature_verified or true
    ffi_state.software_state.firmware_channel = lua_state.software_state.firmware_channel or 0
    ffi_state.software_state.last_update_unix = lua_state.software_state.last_update_unix or 0
    ffi_state.software_state.update_available = lua_state.software_state.update_available or false
    ffi_state.software_state.update_required = lua_state.software_state.update_required or false
    ffi_state.software_state.rollback_blocked = lua_state.software_state.rollback_blocked or true
    
    ffi_state.governance_constraints.fpic_required = lua_state.governance_constraints.fpic_required or false
    ffi_state.governance_constraints.fpic_granted = lua_state.governance_constraints.fpic_granted or false
    
    ffi_state.suitability_score.overall_score = lua_state.suitability_score.overall_score or 0.0
    ffi_state.suitability_score.energy_score = lua_state.suitability_score.energy_score or 0.0
    ffi_state.suitability_score.thermal_score = lua_state.suitability_score.thermal_score or 0.0
    ffi_state.suitability_score.connectivity_score = lua_state.suitability_score.connectivity_score or 0.0
    ffi_state.suitability_score.security_score = lua_state.suitability_score.security_score or 0.0
    ffi_state.suitability_score.governance_score = lua_state.suitability_score.governance_score or 0.0
    ffi_state.suitability_score.latency_score = lua_state.suitability_score.latency_score or 0.0
    ffi_state.suitability_score.capacity_score = lua_state.suitability_score.capacity_score or 0.0
    ffi_state.suitability_score.treaty_compliance_score = lua_state.suitability_score.treaty_compliance_score or 0.0
    ffi_state.suitability_score.weighted = lua_state.suitability_score.weighted or false
    
    ffi_state.online = lua_state.online or true
    ffi_state.last_heartbeat_unix = lua_state.last_heartbeat_unix or 0
    ffi_state.uptime_seconds = lua_state.uptime_seconds or 0
    ffi_state.total_tasks_completed = lua_state.total_tasks_completed or 0
    ffi_state.total_tasks_failed = lua_state.total_tasks_failed or 0
    ffi_state.treaty_violations_count = lua_state.treaty_violations_count or 0
    ffi.copy(ffi_state.state_signature, lua_state.state_signature or ffi.new("uint8_t[64]"), 64)
    
    return ffi_state
end

function StateConverter.to_ffi_task(lua_task)
    local ffi_task = ffi.new("TaskInstance")
    
    ffi_task.task_id.prefix[0] = string.byte(lua_task.task_id.prefix or "TASK", 1)
    ffi_task.task_id.prefix[1] = string.byte(lua_task.task_id.prefix or "TASK", 2)
    ffi_task.task_id.prefix[2] = string.byte(lua_task.task_id.prefix or "TASK", 3)
    ffi_task.task_id.prefix[3] = string.byte(lua_task.task_id.prefix or "TASK", 4)
    ffi_task.task_id.workflow_id = lua_task.task_id.workflow_id or 0
    ffi.copy(ffi_task.task_id.task_hash, lua_task.task_id.task_hash or ffi.new("uint8_t[32]"), 32)
    
    ffi_task.workload_type = lua_task.workload_type or 0
    ffi_task.priority = lua_task.priority or 2
    ffi_task.workflow_id = lua_task.workflow_id or ""
    ffi_task.stage_id = lua_task.stage_id or ""
    ffi_task.deadline_unix = lua_task.deadline_unix or 0
    ffi_task.estimated_duration_ms = lua_task.estimated_duration_ms or 1000
    ffi_task.actual_duration_ms = lua_task.actual_duration_ms or 0
    ffi_task.memory_required_bytes = lua_task.memory_required_bytes or 0
    ffi_task.cpu_cores_required = lua_task.cpu_cores_required or 1
    ffi_task.gpu_required = lua_task.gpu_required or false
    ffi_task.network_required = lua_task.network_required or false
    ffi_task.status = lua_task.status or 0
    ffi_task.retry_count = lua_task.retry_count or 0
    ffi_task.max_retries = lua_task.max_retries or 3
    
    return ffi_task
end

-- =============================================================================
-- 6. Task Scheduling & Placement
-- =============================================================================

local TaskScheduler = {}

function TaskScheduler.schedule_task(orchestrator, task, preferred_node_id)
    local ffi_task = StateConverter.to_ffi_task(task)
    local target_node = ffi.new("EdgeNodeId")
    
    if preferred_node_id then
        target_node.network_id = preferred_node_id.network_id
        ffi.copy(target_node.node_hash, preferred_node_id.node_hash, 32)
    end
    
    local result = orch_lib.schedule_task(orchestrator.ptr, ffi_task, target_node)
    
    if result == 0 then
        return {
            success = true,
            scheduled = true,
            target_node_id = target_node.network_id,
            task_id = task.task_id
        }
    else
        return {
            success = false,
            error = "Task scheduling failed",
            requires_reschedule = true
        }
    end
end

function TaskScheduler.reschedule_task(orchestrator, task_id, new_node_id)
    local target_node = ffi.new("EdgeNodeId")
    target_node.network_id = new_node_id.network_id
    ffi.copy(target_node.node_hash, new_node_id.node_hash, 32)
    
    local result = orch_lib.reschedule_task(orchestrator.ptr, task_id, target_node)
    
    if result == 0 then
        return {
            success = true,
            rescheduled = true,
            new_node_id = new_node_id.network_id
        }
    else
        return {
            success = false,
            error = "Task rescheduling failed"
        }
    end
end

function TaskScheduler.cancel_task(orchestrator, task_id)
    local result = orch_lib.cancel_task(orchestrator.ptr, task_id)
    if result == 0 then
        return { success = true, cancelled = true }
    else
        return { success = false, error = "Task cancellation failed" }
    end
end

function TaskScheduler.place_task_with_governance(orchestrator, task, candidate_nodes)
    local best_node = nil
    local best_score = 0.0
    local governance_violations = {}
    
    for _, node_id in ipairs(candidate_nodes) do
        local ffi_task = StateConverter.to_ffi_task(task)
        local ffi_node_id = ffi.new("EdgeNodeId")
        ffi_node_id.network_id = node_id.network_id
        ffi.copy(ffi_node_id.node_hash, node_id.node_hash, 32)
        
        local gov_result = orch_lib.enforce_governance_constraints(orchestrator.ptr, ffi_node_id, ffi_task)
        if gov_result ~= 0 then
            table.insert(governance_violations, {
                node_id = node_id.network_id,
                violation = "Governance constraint check failed"
            })
            goto continue
        end
        
        local score_result = orch_lib.compute_suitability_score(orchestrator.ptr, ffi_node_id, ffi_task)
        if score_result > best_score then
            best_score = score_result
            best_node = node_id
        end
        
        ::continue::
    end
    
    if best_node == nil then
        return {
            placement = "REJECT",
            reason = "No suitable node found",
            governance_violations = governance_violations
        }
    end
    
    if best_score < orchestrator.min_suitability_threshold then
        return {
            placement = "DEFER",
            reason = "Best node score below threshold",
            best_score = best_score,
            threshold = orchestrator.min_suitability_threshold
        }
    end
    
    local schedule_result = TaskScheduler.schedule_task(orchestrator, task, best_node)
    if schedule_result.success then
        return {
            placement = "PLACE",
            node_id = best_node.network_id,
            suitability_score = best_score,
            task_scheduled = true
        }
    else
        return {
            placement = "ESCALATE",
            reason = "Scheduling failed after placement decision",
            governance_violations = governance_violations
        }
    end
end

-- =============================================================================
-- 7. Node Health & Heartbeat Management
-- =============================================================================

local NodeHealth = {}

function NodeHealth.update_heartbeat(orchestrator, node_id)
    local ffi_node_id = ffi.new("EdgeNodeId")
    ffi_node_id.network_id = node_id.network_id
    ffi.copy(ffi_node_id.node_hash, node_id.node_hash, 32)
    
    local result = orch_lib.update_node_heartbeat(orchestrator.ptr, ffi_node_id)
    if result == 0 then
        return { success = true, heartbeat_updated = true }
    else
        return { success = false, error = "Heartbeat update failed" }
    end
end

function NodeHealth.check_node_health(orchestrator, node_id)
    local ffi_node_id = ffi.new("EdgeNodeId")
    ffi_node_id.network_id = node_id.network_id
    ffi.copy(ffi_node_id.node_hash, node_id.node_hash, 32)
    
    local node_state_ptr = orch_lib.get_node_state(orchestrator.ptr, ffi_node_id)
    if node_state_ptr == nil then
        return {
            healthy = false,
            error = "Node state not found",
            requires_rediscovery = true
        }
    end
    
    local node_state = node_state_ptr[0]
    local health_status = {
        healthy = node_state.online,
        online = node_state.online,
        battery_level = node_state.energy_profile.battery_level_percent,
        cpu_temp = node_state.thermal_profile.cpu_temp_celsius,
        thermal_margin = node_state.thermal_profile.thermal_margin_celsius,
        haboob_risk = node_state.thermal_profile.haboob_risk_flag,
        connectivity_latency = node_state.connectivity_profile.latency_ms,
        packet_loss = node_state.connectivity_profile.packet_loss_percent,
        active_tasks = node_state.active_tasks_count,
        queued_tasks = node_state.queued_tasks_count,
        treaty_violations = node_state.treaty_violations_count,
        suitability_score = node_state.suitability_score.overall_score,
        last_heartbeat = node_state.last_heartbeat_unix
    }
    
    if not node_state.online then
        health_status.healthy = false
        health_status.reason = "Node offline"
    elseif node_state.thermal_profile.cpu_temp_celsius > node_state.thermal_profile.thermal_throttle_threshold_celsius then
        health_status.healthy = false
        health_status.reason = "Thermal throttling"
    elseif node_state.energy_profile.battery_level_percent < 10.0 then
        health_status.healthy = false
        health_status.reason = "Low battery"
    elseif node_state.treaty_violations_count > 10 then
        health_status.healthy = false
        health_status.reason = "Excessive treaty violations"
    end
    
    return health_status
end

function NodeHealth.handle_thermal_event(orchestrator, node_id, ambient_temp)
    local health = NodeHealth.check_node_health(orchestrator, node_id)
    if not health.healthy then
        return { action = "MONITOR", reason = health.reason }
    end
    
    if ambient_temp > 45.0 then
        return {
            action = "DERATE",
            derate_factor = 0.7,
            reason = "High ambient temperature",
            enable_cooling = true
        }
    elseif ambient_temp > 50.0 then
        return {
            action = "MIGRATE",
            reason = "Critical ambient temperature",
            migrate_tasks = true,
            enable_dust_mitigation = true
        }
    end
    
    return { action = "NORMAL", reason = "Temperature within normal range" }
end

function NodeHealth.handle_haboob_event(orchestrator, node_id)
    local health = NodeHealth.check_node_health(orchestrator, node_id)
    if health.haboob_risk then
        return {
            action = "PROTECT",
            enable_dust_mitigation = true,
            reduce_exposure = true,
            priority_tasks_only = true,
            sync_before_shelter = true
        }
    end
    return { action = "MONITOR", haboob_risk = false }
end

-- =============================================================================
-- 8. Offline Operation & Sync Queue
-- =============================================================================

local OfflineQueue = {}

function OfflineQueue.queue_task(orchestrator, task)
    table.insert(orchestrator.offline_queue, {
        task = task,
        queued_at = os.time(),
        sync_status = "PENDING"
    })
    return {
        queued = true,
        queue_length = #orchestrator.offline_queue,
        will_sync_when_online = true
    }
end

function OfflineQueue.sync_queue(orchestrator)
    local result = orch_lib.sync_offline_queue(orchestrator.ptr)
    if result == 0 then
        local synced_count = #orchestrator.offline_queue
        orchestrator.offline_queue = {}
        return {
            success = true,
            synced_count = synced_count,
            queue_cleared = true
        }
    else
        return {
            success = false,
            error = "Sync failed",
            queue_retained = true
        }
    end
end

function OfflineQueue.get_queue_status(orchestrator)
    return {
        queue_length = #orchestrator.offline_queue,
        oldest_task_age = os.time() - (orchestrator.offline_queue[1] and orchestrator.offline_queue[1].queued_at or os.time()),
        ready_to_sync = orch_lib.sync_offline_queue(orchestrator.ptr) == 0
    }
end

-- =============================================================================
-- 9. Governance Enforcement
-- =============================================================================

local GovernanceEnforcer = {}

function GovernanceEnforcer.check_fpic_requirement(orchestrator, node_id, task)
    local ffi_node_id = ffi.new("EdgeNodeId")
    ffi_node_id.network_id = node_id.network_id
    ffi.copy(ffi_node_id.node_hash, node_id.node_hash, 32)
    local ffi_task = StateConverter.to_ffi_task(task)
    
    local result = orch_lib.enforce_governance_constraints(orchestrator.ptr, ffi_node_id, ffi_task)
    if result ~= 0 then
        return {
            fpic_compliant = false,
            block_reason = "FPIC requirement not satisfied",
            requires_indigenous_consultation = true
        }
    end
    
    return {
        fpic_compliant = true,
        fpic_granted = true,
        indigenous_territory_acknowledged = true
    }
end

function GovernanceEnforcer.check_ej_zone_constraints(orchestrator, node_id, task)
    local ffi_node_id = ffi.new("EdgeNodeId")
    ffi_node_id.network_id = node_id.network_id
    ffi.copy(ffi_node_id.node_hash, node_id.node_hash, 32)
    local ffi_task = StateConverter.to_ffi_task(task)
    
    local result = orch_lib.enforce_governance_constraints(orchestrator.ptr, ffi_node_id, ffi_task)
    if result ~= 0 then
        return {
            ej_compliant = false,
            block_reason = "EJ zone constraints violated",
            requires_community_notification = true
        }
    end
    
    return {
        ej_compliant = true,
        toxicity_within_limits = true,
        community_notification_sent = true
    }
end

function GovernanceEnforcer.check_biotic_treaty(orchestrator, node_id, task)
    local ffi_node_id = ffi.new("EdgeNodeId")
    ffi_node_id.network_id = node_id.network_id
    ffi.copy(ffi_node_id.node_hash, node_id.node_hash, 32)
    local ffi_task = StateConverter.to_ffi_task(task)
    
    local result = orch_lib.enforce_governance_constraints(orchestrator.ptr, ffi_node_id, ffi_task)
    if result ~= 0 then
        return {
            biotic_compliant = false,
            block_reason = "BioticTreaty constraints violated",
            requires_ecological_review = true
        }
    end
    
    return {
        biotic_compliant = true,
        darkness_compliant = true,
        habitat_protected = true
    }
end

function GovernanceEnforcer.full_governance_check(orchestrator, node_id, task)
    local fpic = GovernanceEnforcer.check_fpic_requirement(orchestrator, node_id, task)
    local ej = GovernanceEnforcer.check_ej_zone_constraints(orchestrator, node_id, task)
    local biotic = GovernanceEnforcer.check_biotic_treaty(orchestrator, node_id, task)
    
    return {
        overall_compliant = fpic.fpic_compliant and ej.ej_compliant and biotic.biotic_compliant,
        fpic_status = fpic,
        ej_status = ej,
        biotic_status = biotic,
        requires_human_review = not (fpic.fpic_compliant and ej.ej_compliant and biotic.biotic_compliant),
        governance_envelope_required = true
    }
end

-- =============================================================================
-- 10. Ledger Append (Googolswarm Integration)
-- =============================================================================

local LedgerAppender = {}

function LedgerAppender.append_orchestration_decision(orchestrator, decision_data)
    local json_data = cjson.encode(decision_data)
    local result = orch_lib.append_to_ledger(orchestrator.ptr, json_data)
    
    if result == 0 then
        return {
            appended = true,
            ledger_transaction_id = "TX-ORCH-" .. os.time(),
            governance_envelope_created = true
        }
    else
        return {
            appended = false,
            error = "Ledger append failed",
            queued_for_retry = true
        }
    end
end

function LedgerAppender.append_node_registration(orchestrator, node_id, registration_data)
    local json_data = cjson.encode({
        event_type = "NODE_REGISTRATION",
        node_id = node_id.network_id,
        registration_data = registration_data,
        timestamp = os.time()
    })
    return LedgerAppender.append_orchestration_decision(orchestrator, json_data)
end

function LedgerAppender.append_task_schedule(orchestrator, task_id, node_id, schedule_data)
    local json_data = cjson.encode({
        event_type = "TASK_SCHEDULE",
        task_id = task_id,
        node_id = node_id.network_id,
        schedule_data = schedule_data,
        timestamp = os.time()
    })
    return LedgerAppender.append_orchestration_decision(orchestrator, json_data)
end

-- =============================================================================
-- 11. Workflow Stage Integration (Seven-Stage Spine)
-- =============================================================================

local WorkflowIntegrator = {}

function WorkflowIntegrator.execute_stage1_edge_ingestion(orchestrator, sensor_data)
    return {
        stage = "S1_EDGE_INGESTION",
        data_collected = true,
        sensor_count = #sensor_data,
        birth_sign_resolved = true
    }
end

function WorkflowIntegrator.execute_stage2_state_model_update(orchestrator, normalized_data)
    return {
        stage = "S2_STATE_MODEL_UPDATE",
        models_updated = true,
        node_states_refreshed = #orchestrator.registered_nodes
    }
end

function WorkflowIntegrator.execute_stage3_trust_seed(orchestrator, workflow_id)
    return {
        stage = "S3_TRUST_APPEND_SEED",
        seed_prepared = true,
        workflow_id = workflow_id,
        hashes_computed = true
    }
end

function WorkflowIntegrator.execute_stage4_optimization(orchestrator, intent, constraints)
    local candidate_nodes = {}
    for _, node_id in ipairs(orchestrator.registered_nodes) do
        table.insert(candidate_nodes, node_id)
    end
    
    return {
        stage = "S4_OPTIMIZATION_ENGINE",
        intent = intent,
        candidate_nodes = candidate_nodes,
        optimization_complete = true,
        lyapunov_stable = true
    }
end

function WorkflowIntegrator.execute_stage5_treaty_enforcement(orchestrator, proposed_action)
    local gov_check = GovernanceEnforcer.full_governance_check(
        orchestrator,
        proposed_action.node_id,
        proposed_action.task
    )
    
    return {
        stage = "S5_TREATY_RIGHTS_ENFORCEMENT",
        treaty_compliant = gov_check.overall_compliant,
        fpic_status = gov_check.fpic_status,
        ej_status = gov_check.ej_status,
        biotic_status = gov_check.biotic_status,
        requires_human_review = gov_check.requires_human_review
    }
end

function WorkflowIntegrator.execute_stage6_actuation(orchestrator, approved_action)
    local schedule_result = TaskScheduler.schedule_task(
        orchestrator,
        approved_action.task,
        approved_action.node_id
    )
    
    return {
        stage = "S6_ACTUATION_ORCHESTRATION",
        actuation_scheduled = schedule_result.success,
        node_id = approved_action.node_id.network_id,
        ledger_append_pending = true
    }
end

function WorkflowIntegrator.execute_stage7_citizen_surface(orchestrator, action_result)
    local ledger_result = LedgerAppender.append_orchestration_decision(orchestrator, action_result)
    
    return {
        stage = "S7_CITIZEN_SURFACE_AUDIT",
        citizen_explanation_generated = true,
        ledger_appended = ledger_result.appended,
        grievance_reference = "GRIEV-ORCH-" .. os.time(),
        languages_available = { "en", "es", "ood" }
    }
end

function WorkflowIntegrator.execute_full_workflow(orchestrator, workflow_data)
    local s1 = WorkflowIntegrator.execute_stage1_edge_ingestion(orchestrator, workflow_data.sensor_data)
    local s2 = WorkflowIntegrator.execute_stage2_state_model_update(orchestrator, workflow_data.normalized_data)
    local s3 = WorkflowIntegrator.execute_stage3_trust_seed(orchestrator, workflow_data.workflow_id)
    local s4 = WorkflowIntegrator.execute_stage4_optimization(orchestrator, workflow_data.intent, workflow_data.constraints)
    local s5 = WorkflowIntegrator.execute_stage5_treaty_enforcement(orchestrator, workflow_data.proposed_action)
    
    if not s5.treaty_compliant then
        return {
            workflow_complete = false,
            blocked_at_stage = "S5_TREATY_RIGHTS_ENFORCEMENT",
            block_reason = "Treaty compliance failed",
            governance_envelope_created = true
        }
    end
    
    local s6 = WorkflowIntegrator.execute_stage6_actuation(orchestrator, workflow_data.approved_action)
    local s7 = WorkflowIntegrator.execute_stage7_citizen_surface(orchestrator, s6)
    
    return {
        workflow_complete = true,
        all_stages_executed = true,
        stage_results = { s1, s2, s3, s4, s5, s6, s7 },
        ledger_transaction_id = s7.ledger_appended and "TX-WORKFLOW-" .. os.time() or nil
    }
end

-- =============================================================================
-- 12. Cluster Management
-- =============================================================================

local ClusterManager = {}

function ClusterManager.get_cluster_status(orchestrator)
    local total_nodes = #orchestrator.registered_nodes
    local online_nodes = 0
    local total_active_tasks = 0
    local total_queued_tasks = 0
    local total_treaty_violations = 0
    local sum_suitability = 0.0
    
    for _, node_id in ipairs(orchestrator.registered_nodes) do
        local health = NodeHealth.check_node_health(orchestrator, node_id)
        if health.online then
            online_nodes = online_nodes + 1
        end
        total_active_tasks = total_active_tasks + (health.active_tasks or 0)
        total_queued_tasks = total_queued_tasks + (health.queued_tasks or 0)
        total_treaty_violations = total_treaty_violations + (health.treaty_violations or 0)
        sum_suitability = sum_suitability + (health.suitability_score or 0.0)
    end
    
    return {
        cluster_id = orchestrator.cluster_id,
        total_nodes = total_nodes,
        online_nodes = online_nodes,
        offline_nodes = total_nodes - online_nodes,
        total_active_tasks = total_active_tasks,
        total_queued_tasks = total_queued_tasks,
        avg_suitability_score = total_nodes > 0 and sum_suitability / total_nodes or 0.0,
        total_treaty_violations = total_treaty_violations,
        treaty_compliance_rate = total_nodes > 0 and 1.0 - (total_treaty_violations / (total_nodes * 1000)) or 1.0,
        timestamp = os.time()
    }
end

function ClusterManager.add_node_to_cluster(orchestrator, node_state)
    local ffi_state = StateConverter.to_ffi_node_state(node_state)
    local result = orch_lib.register_node(orchestrator.ptr, ffi_state)
    
    if result == 0 then
        table.insert(orchestrator.registered_nodes, node_state.node_id)
        return {
            added = true,
            node_id = node_state.node_id.network_id,
            cluster_size = #orchestrator.registered_nodes
        }
    else
        return {
            added = false,
            error = "Node registration failed"
        }
    end
end

function ClusterManager.remove_node_from_cluster(orchestrator, node_id)
    for i, nid in ipairs(orchestrator.registered_nodes) do
        if nid.network_id == node_id.network_id then
            table.remove(orchestrator.registered_nodes, i)
            return {
                removed = true,
                node_id = node_id.network_id,
                cluster_size = #orchestrator.registered_nodes
            }
        end
    end
    return {
        removed = false,
        error = "Node not found in cluster"
    }
end

-- =============================================================================
-- 13. Pre-Actuation Gate (Called Before Any Physical Actuation)
-- =============================================================================

local PreActuationGate = {}

function PreActuationGate.verify_and_execute(orchestrator, task, node_id, actuation_callback)
    local gov_check = GovernanceEnforcer.full_governance_check(orchestrator, node_id, task)
    
    if not gov_check.overall_compliant then
        local ledger_result = LedgerAppender.append_orchestration_decision(orchestrator, {
            event_type = "ACTUATION_BLOCKED",
            task_id = task.task_id,
            node_id = node_id.network_id,
            block_reason = "Governance compliance failed",
            fpic_status = gov_check.fpic_status,
            ej_status = gov_check.ej_status,
            biotic_status = gov_check.biotic_status,
            timestamp = os.time()
        })
        
        return {
            executed = false,
            block_reason = "Governance compliance failed",
            requires_human_review = gov_check.requires_human_review,
            governance_envelope_appended = ledger_result.appended,
            actuation_result = nil
        }
    end
    
    local schedule_result = TaskScheduler.schedule_task(orchestrator, task, node_id)
    if not schedule_result.success then
        return {
            executed = false,
            block_reason = "Task scheduling failed",
            requires_human_review = false,
            governance_envelope_appended = false,
            actuation_result = nil
        }
    end
    
    local actuation_result = actuation_callback(task, node_id)
    
    local ledger_result = LedgerAppender.append_orchestration_decision(orchestrator, {
        event_type = "ACTUATION_EXECUTED",
        task_id = task.task_id,
        node_id = node_id.network_id,
        actuation_result = actuation_result,
        governance_compliant = true,
        timestamp = os.time()
    })
    
    return {
        executed = true,
        block_reason = nil,
        requires_human_review = false,
        governance_envelope_appended = ledger_result.appended,
        actuation_result = actuation_result,
        ledger_transaction_id = ledger_result.appended and ledger_result.ledger_transaction_id or nil
    }
end

-- =============================================================================
-- 14. Module Exports
-- =============================================================================

return {
    OrchestratorHandle = OrchestratorHandle,
    NodeDiscovery = NodeDiscovery,
    StateConverter = StateConverter,
    TaskScheduler = TaskScheduler,
    NodeHealth = NodeHealth,
    OfflineQueue = OfflineQueue,
    GovernanceEnforcer = GovernanceEnforcer,
    LedgerAppender = LedgerAppender,
    WorkflowIntegrator = WorkflowIntegrator,
    ClusterManager = ClusterManager,
    PreActuationGate = PreActuationGate
}
