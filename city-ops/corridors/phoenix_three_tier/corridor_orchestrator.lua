-- Aletheion :: Phoenix Three-Tier Corridor Orchestrator
-- Language: Lua
--
-- Purpose:
--   Bind the Phoenix 3-Tier Corridor Framework (State–Service–Governance)
--   to executable runtime operations without granting superpowers to any
--   human or augmented-citizen. All authority is delegated to machine-
--   enforceable rules, treaties, and ALN contracts referenced via
--   corridor policies.
--
--   This orchestrator:
--     - Registers corridors and their tiers.
--     - Routes workflow intents from ERM / governance into specific corridors.
--     - Enforces no-build / no-gate policies and maintenance windows.
--     - Coordinates conflict resolution between service-level actions (e.g.
--       AWP routing, sewer relief, heat-mitigation) while preserving safety,
--       BioticTreaty constraints, neurorights, and KER bands.
--
--   This file is intentionally self-contained for offline-capable deployment
--   on edge nodes in Phoenix.

----------------------------
-- Core Type Declarations --
----------------------------

local CorridorOrchestrator = {}
CorridorOrchestrator.__index = CorridorOrchestrator

-- tier: "STATE" | "SERVICE" | "GOVERNANCE"
-- status: "ACTIVE" | "MAINTENANCE" | "LOCKED"
-- corridor_id: globally-unique corridor identifier
-- canonical fields explicitly avoid personal identifiers
local function new_corridor(corridor_id, tier, attrs)
  return {
    id = corridor_id,
    tier = tier,
    status = "ACTIVE",
    attrs = attrs or {},
    policies = {
      no_build = attrs and attrs.no_build or false,
      no_gate  = attrs and attrs.no_gate or false,
      ker_min  = attrs and attrs.ker_min or 0.0,
      ker_max  = attrs and attrs.ker_max or 1.0,
      biotic_treaty_id = attrs and attrs.biotic_treaty_id or nil,
      neuro_safe = attrs and attrs.neuro_safe or true
    },
    maintenance = {
      windows = {}, -- list of { start_ts, end_ts, reason }
      active = false
    },
    bindings = {
      -- workflow bindings: { [workflow_id] = true }
      workflows = {},
      -- physical / logical segments, e.g. canal sections, streets
      segments = attrs and attrs.segments or {}
    }
  }
end

-------------------------
-- Utility / Time Mock --
-------------------------

local function now_ts()
  -- Placeholder for node-local monotonic time (seconds)
  -- In production, bind to a trusted time source or HW clock.
  return os.time()
end

local function overlaps(a_start, a_end, b_start, b_end)
  return a_start <= b_end and b_start <= a_end
end

---------------------------
-- KER & Treaty Checks  --
---------------------------

local function ker_within_band(ker_value, ker_min, ker_max)
  if ker_value == nil then
    return false, "KER_UNKNOWN"
  end
  if ker_value < ker_min then
    return false, "KER_TOO_LOW"
  end
  if ker_value > ker_max then
    return false, "KER_TOO_HIGH"
  end
  return true, "KER_OK"
end

-- treaty_check_fn is injected at runtime; here is the interface:
-- treaty_check_fn(treaty_id, corridor_id, intent) -> (ok:boolean, code:string)
local function treaty_allows_intent(treaty_check_fn, treaty_id, corridor_id, intent)
  if not treaty_id then
    return true, "TREATY_NOT_BOUND"
  end
  if not treaty_check_fn then
    -- fail-closed: if treaty is specified but no checker is wired, block
    return false, "TREATY_CHECKER_MISSING"
  end
  local ok, code = treaty_check_fn(treaty_id, corridor_id, intent)
  if ok then
    return true, code or "TREATY_OK"
  end
  return false, code or "TREATY_DENY"
end

---------------------------
-- Maintenance Handling  --
---------------------------

local function is_in_maintenance(corridor)
  if corridor.status == "MAINTENANCE" or corridor.status == "LOCKED" then
    return true
  end
  local ts = now_ts()
  for _, win in ipairs(corridor.maintenance.windows) do
    if overlaps(win.start_ts, win.end_ts, ts, ts) then
      return true
    end
  end
  return false
end

local function add_maintenance_window(corridor, start_ts, end_ts, reason)
  table.insert(corridor.maintenance.windows, {
    start_ts = start_ts,
    end_ts = end_ts,
    reason = reason or "UNSPECIFIED"
  })
end

-----------------------
-- Intent Definition --
-----------------------

-- intent_type examples:
--   "WATER_REROUTE", "SEWER_RELIEF", "HEAT_MITIGATION",
--   "DEVICE_INSTALL", "SURFACE_MODIFY"
--
-- scope may reference:
--   - corridor_id(s)
--   - segments
--   - spatial bounding box
--
-- metadata should *never* contain PII; only operational context.
local function new_intent(intent_id, source, intent_type, scope, metadata)
  return {
    id = intent_id,
    source = source,           -- "ERM", "GOVERNANCE", "SERVICE_NODE"
    intent_type = intent_type,
    scope = scope or {},
    metadata = metadata or {}
  }
end

------------------------------
-- Orchestrator Construction --
------------------------------

function CorridorOrchestrator.new(config)
  local self = setmetatable({}, CorridorOrchestrator)
  self.corridors = {}
  self.index_by_tier = { STATE = {}, SERVICE = {}, GOVERNANCE = {} }

  -- external hooks (must be wired at deployment time)
  self.hooks = {
    treaty_check = config and config.treaty_check or nil,
    ker_query = config and config.ker_query or nil,
    log_sink = config and config.log_sink or nil,
    dispatch_intent = config and config.dispatch_intent or nil
  }

  return self
end

local function log(orchestrator, level, msg, fields)
  local sink = orchestrator.hooks.log_sink
  if not sink then return end
  sink({
    ts = now_ts(),
    level = level,
    msg = msg,
    fields = fields or {}
  })
end

---------------------------
-- Corridor Registration --
---------------------------

function CorridorOrchestrator:register_corridor(corridor_id, tier, attrs)
  if self.corridors[corridor_id] ~= nil then
    return false, "CORRIDOR_ALREADY_EXISTS"
  end
  if tier ~= "STATE" and tier ~= "SERVICE" and tier ~= "GOVERNANCE" then
    return false, "TIER_INVALID"
  end
  local c = new_corridor(corridor_id, tier, attrs)
  self.corridors[corridor_id] = c
  table.insert(self.index_by_tier[tier], corridor_id)
  log(self, "INFO", "corridor_registered", { id = corridor_id, tier = tier })
  return true, "OK"
end

function CorridorOrchestrator:bind_workflow(corridor_id, workflow_id)
  local c = self.corridors[corridor_id]
  if not c then
    return false, "CORRIDOR_NOT_FOUND"
  end
  c.bindings.workflows[workflow_id] = true
  log(self, "INFO", "workflow_bound", { corridor_id = corridor_id, workflow_id = workflow_id })
  return true, "OK"
end

function CorridorOrchestrator:set_status(corridor_id, status)
  local c = self.corridors[corridor_id]
  if not c then
    return false, "CORRIDOR_NOT_FOUND"
  end
  if status ~= "ACTIVE" and status ~= "MAINTENANCE" and status ~= "LOCKED" then
    return false, "STATUS_INVALID"
  end
  c.status = status
  log(self, "INFO", "corridor_status_changed", { id = corridor_id, status = status })
  return true, "OK"
end

function CorridorOrchestrator:add_maintenance(corridor_id, start_ts, end_ts, reason)
  local c = self.corridors[corridor_id]
  if not c then
    return false, "CORRIDOR_NOT_FOUND"
  end
  add_maintenance_window(c, start_ts, end_ts, reason)
  log(self, "INFO", "maintenance_added", {
    id = corridor_id,
    start_ts = start_ts,
    end_ts = end_ts,
    reason = reason
  })
  return true, "OK"
end

-------------------------
-- Policy Guard Rails  --
-------------------------

function CorridorOrchestrator:check_policies(corridor, intent)
  -- No-build / no-gate protection
  if corridor.policies.no_build and intent.intent_type == "DEVICE_INSTALL" then
    return false, "NO_BUILD_ZONE"
  end
  if corridor.policies.no_gate and intent.intent_type == "SURFACE_MODIFY" then
    return false, "NO_GATE_ZONE"
  end

  -- KER constraints
  local ker_query = self.hooks.ker_query
  if ker_query then
    local ker_value = ker_query(corridor.id, intent)
    local ok, code = ker_within_band(
      ker_value,
      corridor.policies.ker_min,
      corridor.policies.ker_max
    )
    if not ok then
      return false, code
    end
  end

  -- Treaty constraints
  local t_ok, t_code = treaty_allows_intent(
    self.hooks.treaty_check,
    corridor.policies.biotic_treaty_id,
    corridor.id,
    intent
  )
  if not t_ok then
    return false, t_code
  end

  -- Neuro-safety constraint (high-level guard)
  if corridor.policies.neuro_safe == false then
    -- block any intent classified as neuro-affecting
    if intent.metadata and intent.metadata.neuro_affecting == true then
      return false, "NEURO_UNSAFE_CORRIDOR"
    end
  end

  return true, "POLICY_OK"
end

-------------------------------
-- Intent Routing & Matching --
-------------------------------

-- resolve candidate corridors based on intent scope and tier preference
function CorridorOrchestrator:resolve_corridors(intent)
  local candidates = {}

  -- Explicit corridor IDs in scope override everything else.
  if intent.scope and intent.scope.corridor_ids then
    for _, id in ipairs(intent.scope.corridor_ids) do
      local c = self.corridors[id]
      if c then
        table.insert(candidates, c)
      end
    end
  else
    -- Fallback: match by tier + segments.
    local pref_tier = intent.scope and intent.scope.preferred_tier
    local tiers = {}

    if pref_tier and self.index_by_tier[pref_tier] then
      tiers = { pref_tier }
    else
      tiers = { "SERVICE", "STATE", "GOVERNANCE" }
    end

    for _, tier in ipairs(tiers) do
      for _, cid in ipairs(self.index_by_tier[tier]) do
        local c = self.corridors[cid]
        if c then
          if not intent.scope or not intent.scope.segment_ids then
            table.insert(candidates, c)
          else
            local seg_index = {}
            for _, seg in ipairs(c.bindings.segments) do
              seg_index[seg] = true
            end
            for _, seg in ipairs(intent.scope.segment_ids) do
              if seg_index[seg] then
                table.insert(candidates, c)
                break
              end
            end
          end
        end
      end
    end
  end

  return candidates
end

-----------------------------------
-- Core Intent Orchestration API --
-----------------------------------

-- evaluate an intent across all matching corridors,
-- returning a structured decision set without superpowers.
function CorridorOrchestrator:evaluate_intent(intent)
  local results = {
    intent_id = intent.id,
    decisions = {}
  }

  local corridors = self:resolve_corridors(intent)
  if #corridors == 0 then
    log(self, "WARN", "intent_no_corridor_match", { intent_id = intent.id })
    results.global_status = "NO_CORRIDOR_MATCH"
    return results
  end

  local any_approved = false

  for _, c in ipairs(corridors) do
    local decision = {
      corridor_id = c.id,
      tier = c.tier,
      status = "PENDING"
    }

    if is_in_maintenance(c) then
      decision.status = "REJECTED"
      decision.code = "IN_MAINTENANCE"
      table.insert(results.decisions, decision)
      goto continue
    end

    local p_ok, p_code = self:check_policies(c, intent)
    if not p_ok then
      decision.status = "REJECTED"
      decision.code = p_code
      table.insert(results.decisions, decision)
      goto continue
    end

    -- All checks passed: mark as APPROVED.
    decision.status = "APPROVED"
    decision.code = "OK"
    any_approved = true
    table.insert(results.decisions, decision)

    ::continue::
  end

  results.global_status = any_approved and "APPROVED_PARTIAL" or "REJECTED_ALL"
  log(self, "INFO", "intent_evaluated", {
    intent_id = intent.id,
    global_status = results.global_status
  })
  return results
end

-- dispatch approved intents to lower-level services / devices.
function CorridorOrchestrator:dispatch_intent(intent, evaluation)
  local dispatcher = self.hooks.dispatch_intent
  if not dispatcher then
    log(self, "WARN", "dispatch_missing", { intent_id = intent.id })
    return false, "DISPATCHER_MISSING"
  end

  local approved_corridors = {}
  for _, d in ipairs(evaluation.decisions) do
    if d.status == "APPROVED" then
      table.insert(approved_corridors, d.corridor_id)
    end
  end

  if #approved_corridors == 0 then
    return false, "NO_APPROVED_CORRIDORS"
  end

  local ok, code = dispatcher(intent, approved_corridors)
  if ok then
    log(self, "INFO", "intent_dispatched", {
      intent_id = intent.id,
      corridors = approved_corridors
    })
  else
    log(self, "ERROR", "intent_dispatch_failed", {
      intent_id = intent.id,
      corridors = approved_corridors,
      code = code
    })
  end
  return ok, code
end

----------------------
-- Public Factories --
----------------------

function CorridorOrchestrator.new_intent(intent_id, source, intent_type, scope, metadata)
  return new_intent(intent_id, source, intent_type, scope, metadata)
end

------------------------
-- Example Wiring Note --
------------------------
-- This module is designed to be required() by a higher-level runtime:
--
-- local orchestrator = CorridorOrchestrator.new({
--   treaty_check = function(treaty_id, corridor_id, intent) ... end,
--   ker_query = function(corridor_id, intent) ... end,
--   log_sink = function(entry) ... end,
--   dispatch_intent = function(intent, corridor_ids) ... end
-- })
--
-- That runtime can be an ALN/Rust bridge, a C++ daemon, or a
-- lightweight Lua VM on embedded hardware, as long as it respects
-- the non-superpower, treaty-bound design.

return CorridorOrchestrator
