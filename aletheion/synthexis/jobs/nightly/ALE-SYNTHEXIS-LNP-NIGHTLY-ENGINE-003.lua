-- Nightly Synthexis LightNoisePesticide (LNP) planner execution for Phoenix.
--
-- ERM Layers: L2 (species + state), L4 (planning), L3 (logging).
--
-- Responsibilities:
--  - Load species-activity forecasts and human comfort thresholds
--  - Call LightNoisePesticidePlanner engine
--  - Emit recommended operating envelopes per block/corridor
--  - Append violations and deltas to the trust layer (via external client)
--
-- This script is designed to be invoked by a nightly scheduler (e.g., cron,
-- Kubernetes CronJob, or a GitHub Actions workflow calling into the runtime).

local json = require("cjson.safe")

-- -------- External dependencies (to be implemented in other modules) --------

-- Synthexis engine bindings (implemented in ALN/Rust):
--   aletheion/synthexis/engine/LightNoisePesticidePlanner.aln / .so
local lnp_engine = require("alethion.synthexis.engine.lnp_binding_core_001")

-- Species-activity provider:
--   loads precomputed probability distributions for bats, pollinators, etc.
local species_provider = require("alethion.synthexis.data.species_activity_provider_001")

-- Human comfort and safety thresholds:
local comfort_provider = require("alethion.synthexis.data.human_comfort_thresholds_001")

-- Trust-layer client (Googolswarm-compatible append):
local trust_client = require("alethion.trust.client.light_noise_pesticide_append_001")

-- Simple logging abstraction (can be wired to structured logging later).
local function log(level, msg, ctx)
  local line = {
    ts  = os.date("!%Y-%m-%dT%H:%M:%SZ"),
    lvl = level,
    msg = msg,
    ctx = ctx or {}
  }
  io.stdout:write(json.encode(line) .. "\n")
end

-- -------- Core nightly workflow --------

local M = {}

-- Nightly job entrypoint.
-- `target_date` is the local Phoenix date for which to compute envelopes, e.g. "2026-03-08".
function M.run_nightly(target_date)
  target_date = target_date or os.date("%Y-%m-%d")

  log("info", "Starting Synthexis LNP nightly run", { date = target_date })

  -- 1. Load species-activity forecasts (bats, pollinators, desert fauna/flora).
  local species_ok, species_activity = species_provider.load_activity_for_date(target_date)
  if not species_ok then
    log("error", "Failed to load species activity", { date = target_date })
    return false
  end

  -- 2. Load human comfort and safety thresholds.
  local comfort_ok, comfort = comfort_provider.load_thresholds_for_date(target_date)
  if not comfort_ok then
    log("error", "Failed to load human comfort thresholds", { date = target_date })
    return false
  end

  -- 3. Prepare base infrastructure state snapshot (lighting, noise sources, pesticides).
  -- This should be a compact representation consumed by the LNP engine.
  local state_ok, base_state = M.load_base_infrastructure_state(target_date)
  if not state_ok then
    log("error", "Failed to load base infrastructure state", { date = target_date })
    return false
  end

  -- 4. Call LNP planner to compute operating envelopes and violation reports.
  local planner_input = {
    date = target_date,
    base_state = base_state,
    species_activity = species_activity,
    comfort_thresholds = comfort,
  }

  local envelopes, violations, planner_meta = lnp_engine.compute_envelopes(planner_input)
  if not envelopes then
    log("error", "LNP planner returned no envelopes", { date = target_date })
    return false
  end

  log("info", "LNP planner completed", {
    date = target_date,
    blocks = #envelopes,
    violations = #violations
  })

  -- 5. Persist recommendations to filesystem or configuration store.
  local persist_ok = M.persist_envelopes(target_date, envelopes)
  if not persist_ok then
    log("error", "Failed to persist envelopes", { date = target_date })
  end

  -- 6. Append violation reports to the trust layer for auditability.
  local trust_ok = trust_client.append_violations(target_date, violations, planner_meta)
  if not trust_ok then
    log("error", "Failed to append violations to trust layer", { date = target_date })
  end

  log("info", "Synthexis LNP nightly run completed", {
    date = target_date,
    persisted = persist_ok,
    trust_logged = trust_ok
  })

  return persist_ok and trust_ok
end

-- -------- Helper functions (state + persistence) --------

-- Placeholder for integrating with the city-state model / infra registries.
-- In v1, this might load from a JSON export produced by L2 state modeling.
function M.load_base_infrastructure_state(target_date)
  -- Example path: /var/lib/alethion/state/light_noise_pesticide/<date>.json
  local path = string.format(
    "/var/lib/alethion/state/light_noise_pesticide/%s.json",
    target_date
  )
  local fh, err = io.open(path, "r")
  if not fh then
    log("error", "Cannot open base state file", { path = path, err = tostring(err) })
    return false, nil
  end
  local content = fh:read("*a")
  fh:close()

  local data, decode_err = json.decode(content)
  if not data then
    log("error", "Failed to decode base state JSON", { path = path, err = tostring(decode_err) })
    return false, nil
  end

  return true, data
end

-- Persist computed envelopes as JSON for downstream consumers (lighting controllers,
-- noise enforcement logic, pesticide schedulers).
function M.persist_envelopes(target_date, envelopes)
  local path = string.format(
    "/var/lib/alethion/synthexis/envelopes/%s_light_noise_pesticide.json",
    target_date
  )

  local ok, encoded = pcall(json.encode, envelopes)
  if not ok then
    log("error", "Failed to encode envelopes to JSON", { date = target_date })
    return false
  end

  local fh, err = io.open(path, "w")
  if not fh then
    log("error", "Cannot open envelope output file", { path = path, err = tostring(err) })
    return false
  end

  fh:write(encoded)
  fh:close()

  log("info", "Persisted envelopes", { path = path })
  return true
end

-- CLI entrypoint for running from the command line:
--   lua ALE-SYNTHEXIS-LNP-NIGHTLY-ENGINE-003.lua 2026-03-08
if arg and arg[0] and string.match(arg[0], "ALE%-SYNTHEXIS%-LNP%-NIGHTLY%-ENGINE%-003") then
  local target_date = arg[1] or os.date("%Y-%m-%d")
  local ok = M.run_nightly(target_date)
  os.exit(ok and 0 or 1)
end

return M
