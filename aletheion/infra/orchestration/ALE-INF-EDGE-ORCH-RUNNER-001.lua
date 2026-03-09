-- Role: Edge orchestration runner for Aletheion.
--       Discovers nodes, evaluates energy + thermal + security posture,
--       respects BirthSigns and ALN constraints, and schedules/migrates tasks.[file:2][file:1]
--
-- ERM Layers: L0 Infra, L1 Edge Sensing, L2 State Modeling, L4 Optimization.
-- Languages: Lua only in this runner; heavy logic resides in Rust/ALN engines.

local M = {}

---------------------------------------------------------------------
-- External dependencies (to be implemented in their own modules)[file:2]
---------------------------------------------------------------------

-- Node registry: returns a list of edge node descriptors with profiles.
-- Expected shape (conceptual):
--   {
--     node_id = "node-001",
--     hardware_class = "router" | "appliance" | "mcu" | "vehicle",
--     energy_profile = { mode = "grid" | "solar", idle_watts = 5.0, peak_watts = 20.0 },
--     thermal_profile = { current_c = 45.0, max_c = 80.0 },
--     connectivity = { kind = "wifi" | "ethernet" | "cellular", score = 0.8 },
--     security_profile = { tier = "TeeBacked" | "HardenedFirmwareOnly" | "Basic" },
--     coverage_birthsign_ids = { "BS-TILE-XYZ", ... }
--   }
local node_registry = require("aletheion.infra.orchestration.edge_node_registry_001")

-- Task registry: returns desired task instances and governance tags.
-- Expected shape:
--   {
--     task_id = "ERM-WATER-ALLOCATION-DAILY",
--     domain = "water",
--     priority = 0.9,
--     required_security = "TeeBacked" | "HardenedFirmwareOnly" | "Basic",
--     required_domains = { "Water" },
--     affected_birthsign_ids = { "BS-TILE-XYZ", ... },
--     energy_budget_wh = 50.0
--   }
local task_registry = require("aletheion.infra.orchestration.task_catalog_001")

-- Governance engine: evaluates if a task may run on a node for given BirthSigns.
-- Must internally consult BirthSign model + ALN norms and return:
--   { allowed = true/false, reason = "..." }
local gov_engine = require("aletheion.governance.birthsigns.edge_placement_guard_001")

-- Trust client: appends governed orchestration decisions to Googolswarm.
-- Exposed API:
--   trust_client.append_orchestration_decision(envelope_tbl) -> bool
local trust_client = require("aletheion.trust.client.orchestration_append_001")

---------------------------------------------------------------------
-- Local helpers
---------------------------------------------------------------------

local function log_info(...)
  io.stderr:write("[EDGE-ORCH][INFO] ", table.concat({ ... }, " "), "\n")
end

local function log_warn(...)
  io.stderr:write("[EDGE-ORCH][WARN] ", table.concat({ ... }, " "), "\n")
end

local function log_error(...)
  io.stderr:write("[EDGE-ORCH][ERROR] ", table.concat({ ... }, " "), "\n")
end

-- Compute a numeric suitability score for placing a task on a node.
-- This is a heuristic that combines:
--   - Security tier vs. required_security
--   - Thermal headroom
--   - Connectivity score
--   - Energy mode (solar-friendly if daytime / config-supplied flag)
local function compute_suitability(node, task, opts)
  opts = opts or {}
  local base = 0.0

  -- Security match.
  local tier = node.security_profile and node.security_profile.tier or "Basic"
  local required = task.required_security or "Basic"
  if required == "TeeBacked" then
    if tier == "TeeBacked" then
      base = base + 0.5
    else
      return -1.0 -- cannot host
    end
  elseif required == "HardenedFirmwareOnly" then
    if tier == "TeeBacked" or tier == "HardenedFirmwareOnly" then
      base = base + 0.4
    else
      return -1.0
    end
  else -- Basic
    base = base + 0.3
  end

  -- Thermal headroom.
  local tp = node.thermal_profile or {}
  local cur = tp.current_c or 40.0
  local maxc = tp.max_c or 80.0
  local headroom = math.max(0.0, (maxc - cur) / math.max(1.0, maxc))
  base = base + 0.3 * headroom

  -- Connectivity.
  local conn = node.connectivity or {}
  local cscore = conn.score or 0.5
  base = base + 0.2 * cscore

  -- Energy mode: prefer solar if we’re in a “solar-preferred” window.
  local ep = node.energy_profile or {}
  local mode = ep.mode or "grid"
  if opts.solar_preferred and mode == "solar" then
    base = base + 0.2
  end

  return base
end

-- Build a governed decision envelope for orchestration.
-- This is a Lua-side mirror that must correspond to ALE-TRUST-GOVERNED-DECISION-TX-001.aln.[file:2]
local function build_orchestration_envelope(args)
  -- args:
  --   tx_id, workflow_id, workflow_stage, domains, birthsign_ids,
  --   node_id, task_id, allowed, gov_reason
  return {
    txId = args.tx_id,
    workflowId = args.workflow_id or "EDGE-ORCH-001",
    workflowStage = args.workflow_stage or "Placement",
    domains = args.domains or { "Infra", "Energy", "Governance" },
    birthSignIds = args.birthsign_ids or {},
    appliedAlnNorms = args.applied_aln_norms or {},
    appliedBioticTreaties = args.applied_biotic_treaties or {},
    appliedMicroTreaties = args.applied_micro_treaties or {},
    subjectDid = nil,
    operatorDid = nil,
    nodeProfileId = node_registry.node_profile_id_for(args.node_id),
    inputsHash = "",  -- can be filled by Rust side if needed
    outputsHash = "",
    evaluation = {
      birthSignId = args.primary_birthsign_id or "",
      constraintMode = "HighPenalty",
      outcomes = {},
      fpicStatus = "NotRequired"
    },
    outcome = args.allowed and "Approved" or "Rejected",
    explanation = args.gov_reason or "",
    tags = {
      node_id = args.node_id,
      task_id = args.task_id
    }
  }
end

---------------------------------------------------------------------
-- Core scheduling loop
---------------------------------------------------------------------

--- Plan placements for all tasks on currently visible nodes.
-- @param opts table: { solar_preferred = bool, dry_run = bool }
-- @return table: { placements = { { task_id, node_id, score }... } }
function M.plan(opts)
  opts = opts or {}
  local nodes = node_registry.list_nodes()
  local tasks = task_registry.list_tasks()
  local placements = {}

  if not nodes or #nodes == 0 then
    log_warn("No nodes available for scheduling")
    return { placements = {} }
  end
  if not tasks or #tasks == 0 then
    log_info("No tasks to schedule")
    return { placements = {} }
  end

  for _, task in ipairs(tasks) do
    local best_node = nil
    local best_score = -1.0
    local best_gov = nil

    for _, node in ipairs(nodes) do
      local score = compute_suitability(node, task, opts)
      if score > 0.0 then
        -- Governance check: may this task run on this node for its BirthSigns?
        local g = gov_engine.evaluate_placement(node, task)
        if g and g.allowed then
          if score > best_score then
            best_score = score
            best_node = node
            best_gov = g
          end
        else
          log_info("Governance denied placement",
                   "task=" .. task.task_id,
                   "node=" .. (node.node_id or "unknown"),
                   "reason=" .. (g and g.reason or "unknown"))
        end
      end
    end

    if best_node then
      log_info("Planned placement",
               "task=" .. task.task_id,
               "node=" .. best_node.node_id,
               string.format("score=%.3f", best_score))
      table.insert(placements, {
        task_id = task.task_id,
        node_id = best_node.node_id,
        score   = best_score,
        gov     = best_gov
      })
    else
      log_warn("No suitable node found for task", task.task_id)
    end
  end

  return { placements = placements }
end

--- Apply planned placements: instruct nodes to launch/migrate tasks and append
-- governed decisions to the trust layer.
-- @param opts table: { solar_preferred = bool, dry_run = bool }
-- @return boolean success
function M.apply(opts)
  opts = opts or {}
  local dry_run = opts.dry_run or false
  local plan = M.plan(opts)
  local ok_all = true

  for _, p in ipairs(plan.placements) do
    local node_id = p.node_id
    local task_id = p.task_id
    local gov = p.gov or {}

    local bs_ids = gov.birthsign_ids or {}
    local env = build_orchestration_envelope({
      tx_id = trust_client.new_tx_id(),
      workflow_id = "EDGE-ORCH-001",
      workflow_stage = "Placement",
      domains = { "Infra", "Energy", "Governance" },
      birthsign_ids = bs_ids,
      node_id = node_id,
      task_id = task_id,
      allowed = true,
      gov_reason = gov.reason or ""
    })

    if dry_run then
      log_info("DRY-RUN placement",
               "task=" .. task_id,
               "node=" .. node_id,
               "bs=" .. table.concat(bs_ids, ","))
    else
      -- 1) Send command to node to start/migrate task.
      local launch_ok, launch_err = node_registry.launch_task_on_node(node_id, task_id)
      if not launch_ok then
        ok_all = false
        log_error("Failed to launch task",
                  "task=" .. task_id,
                  "node=" .. node_id,
                  "err=" .. tostring(launch_err))
      end

      -- 2) Append governed orchestration decision to trust layer.
      local trust_ok = trust_client.append_orchestration_decision(env)
      if not trust_ok then
        ok_all = false
        log_error("Failed to append orchestration decision to trust layer",
                  "task=" .. task_id,
                  "node=" .. node_id)
      end
    end
  end

  return ok_all
end

---------------------------------------------------------------------
-- CLI entrypoint
---------------------------------------------------------------------

-- Usage:
--   lua ALE-INF-EDGE-ORCH-RUNNER-001.lua plan
--   lua ALE-INF-EDGE-ORCH-RUNNER-001.lua apply
if arg and arg[0] and arg[0]:match("ALE%-INF%-EDGE%-ORCH%-RUNNER%-001") then
  local mode = arg[1] or "plan"
  local solar_preferred = (os.date("*t").hour or 12) >= 9 and (os.date("*t").hour or 12) <= 16
  local opts = { solar_preferred = solar_preferred, dry_run = (mode == "plan") }

  if mode == "plan" then
    local plan = M.plan(opts)
    log_info("Planned placements:", tostring(#plan.placements))
    os.exit(0)
  elseif mode == "apply" then
    local ok = M.apply(opts)
    os.exit(ok and 0 or 1)
  else
    log_error("Unknown mode:", mode)
    os.exit(1)
  end
end

return M
