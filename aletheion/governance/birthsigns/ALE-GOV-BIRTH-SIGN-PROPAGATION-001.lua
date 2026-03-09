-- Role: Jurisdictional signature propagation glue for Aletheion.
--       Ensures every governed record carries a non-empty birthSignId from S1
--       (ingest) through S6 (actuation), and rejects governed actions that
--       drop or fabricate this context.[file:2][file:5]

local M = {}

----------------------------------------------------------------------
-- 1. Shared configuration and dependencies
----------------------------------------------------------------------

-- Reserved metadata key for the Birth-Sign identifier.
M.BIRTHSIGN_KEY = "birthSignId"

-- Minimal geospatial context client contract:
--   get_birth_sign_id(lat, lon, t) -> string|nil
-- Implementation lives in infra; here we call via injected adapter.[file:2]
local geo = require("aletheion.governance.birthsigns.geo_context_adapter_001")

-- Minimal compliance runtime contract:
--   validate_birthsign_envelope(envelope) -> ok:boolean, err?:string
-- Envelope is expected to include birthSignIds, ALN norms, DIDs, decision info.[file:2]
local compliance = require("aletheion.governance.runtime.compliance_bridge_001")

----------------------------------------------------------------------
-- 2. S1 ingest: attach birthSignId
----------------------------------------------------------------------

--- Attach a Birth-Sign ID to an incoming event based on location and time.
-- @param ev   table  Incoming event with at least { lat, lon, ts }.
-- @return table|nil, string|nil  Enriched event or nil, error message.
function M.attach_at_ingest(ev)
  if type(ev) ~= "table" then
    return nil, "ingest: event must be table"
  end

  -- Preserve existing ID if already attached (e.g., upstream edge).[file:2]
  if ev[M.BIRTHSIGN_KEY] ~= nil and ev[M.BIRTHSIGN_KEY] ~= "" then
    return ev, nil
  end

  local lat, lon, ts = ev.lat, ev.lon, ev.ts
  if type(lat) ~= "number" or type(lon) ~= "number" or ts == nil then
    return nil, "ingest: missing lat/lon/ts for birthSign lookup"
  end

  local id, gerr = geo.get_birth_sign_id(lat, lon, ts)
  if not id or id == "" then
    return nil, ("ingest: no Birth-Sign for lat=%.6f lon=%.6f"):format(lat, lon)
  end

  ev[M.BIRTHSIGN_KEY] = id
  return ev, nil
end

----------------------------------------------------------------------
-- 3. S2 state-model update: preserve birthSignId
----------------------------------------------------------------------

--- Copy birthSignId from an event into the state-model record.
-- @param ev   table  Ingested event with birthSignId.
-- @param rec  table  Existing or new state-model record.
-- @return table|nil, string|nil
function M.propagate_to_state(ev, rec)
  if type(ev) ~= "table" or type(rec) ~= "table" then
    return nil, "state: ev and rec must be tables"
  end

  local id = ev[M.BIRTHSIGN_KEY]
  if not id or id == "" then
    return nil, "state: missing birthSignId on event"
  end

  rec[M.BIRTHSIGN_KEY] = id
  return rec, nil
end

----------------------------------------------------------------------
-- 4. S3/S4: prepare optimization input with birthSignIds
----------------------------------------------------------------------

--- Extract the set of Birth-Sign IDs from a list of state records.
-- @param records table[]  List of state records touching land/water/air/etc.
-- @return table  { [id] = true, ... }
function M.collect_birthsign_ids(records)
  local out = {}
  if type(records) ~= "table" then
    return out
  end
  for _, r in ipairs(records) do
    if type(r) == "table" then
      local id = r[M.BIRTHSIGN_KEY]
      if id and id ~= "" then
        out[id] = true
      end
    end
  end
  return out
end

--- Inject birthSignIds into an optimization request payload.
-- @param opt_req table  Optimization input (domain-specific fields).
-- @param state_records table[]  Records whose context is being optimized.
-- @return table|nil, string|nil
function M.prepare_optimization_request(opt_req, state_records)
  if type(opt_req) ~= "table" then
    return nil, "opt: request must be table"
  end
  local ids = M.collect_birthsign_ids(state_records)
  local list = {}
  for k, _ in pairs(ids) do
    list[#list + 1] = k
  end
  if #list == 0 then
    return nil, "opt: no birthSignIds for governed optimization"
  end
  opt_req.birthSignIds = list
  return opt_req, nil
end

----------------------------------------------------------------------
-- 5. S5: governance pre-flight wrapper
----------------------------------------------------------------------

--- Build a minimal governed decision envelope for compliance runtime.[file:2]
-- @param workflow_id  string
-- @param stage        string  e.g. "Optimization" or "Actuation"
-- @param domains      string[]  e.g. {"Water","Mobility"}
-- @param birth_ids    string[]
-- @param aln_norms    string[]
-- @param did_subject  string|nil
-- @param did_operator string|nil
-- @return table
function M.build_governed_envelope(workflow_id, stage, domains,
                                   birth_ids, aln_norms,
                                   did_subject, did_operator)
  return {
    workflowId     = workflow_id,
    workflowStage  = stage,
    domains        = domains or {},
    birthSignIds   = birth_ids or {},
    appliedAlnNorms = aln_norms or {},
    subjectDid     = did_subject,
    operatorDid    = did_operator,
  }
end

--- Run governance pre-flight; enforce non-empty birthSignIds.
-- @param envelope table  As built by build_governed_envelope plus domain data.
-- @return boolean, string|nil
function M.governance_preflight(envelope)
  if type(envelope) ~= "table" then
    return false, "gov: envelope must be table"
  end
  local ids = envelope.birthSignIds
  if type(ids) ~= "table" or #ids == 0 then
    return false, "gov: empty birthSignIds for governed decision"
  end
  local ok, err = compliance.validate_birthsign_envelope(envelope)
  if not ok then
    return false, err or "gov: compliance runtime rejected envelope"
  end
  return true, nil
end

----------------------------------------------------------------------
-- 6. S6: actuation-level guard
----------------------------------------------------------------------

--- Guard a device/service command with Birth-Sign propagation checks.
-- @param cmd   table  Actuation command, MUST carry birthSignId.
-- @param env   table  Governed envelope already validated by governance_preflight.
-- @return table|nil, string|nil  Command if allowed, error otherwise.
function M.guard_actuation(cmd, env)
  if type(cmd) ~= "table" then
    return nil, "act: command must be table"
  end
  if type(env) ~= "table" then
    return nil, "act: envelope must be table"
  end

  local id = cmd[M.BIRTHSIGN_KEY]
  if not id or id == "" then
    return nil, "act: missing birthSignId on command"
  end

  -- Ensure the command’s Birth-Sign is one of the envelope’s IDs.
  local allowed = false
  if type(env.birthSignIds) == "table" then
    for _, bid in ipairs(env.birthSignIds) do
      if bid == id then
        allowed = true
        break
      end
    end
  end
  if not allowed then
    return nil, "act: command birthSignId not in governed envelope set"
  end

  return cmd, nil
end

----------------------------------------------------------------------
-- 7. Utility: bulk propagation for pipeline stages
----------------------------------------------------------------------

--- Propagate birthSignId from a source record to a list of derived records.
-- @param src   table    Source with birthSignId.
-- @param list  table[]  Derived records to tag.
-- @return table[]|nil, string|nil
function M.propagate_to_many(src, list)
  if type(src) ~= "table" or type(list) ~= "table" then
    return nil, "prop: src and list must be tables"
  end
  local id = src[M.BIRTHSIGN_KEY]
  if not id or id == "" then
    return nil, "prop: missing birthSignId on source"
  end
  for _, r in ipairs(list) do
    if type(r) == "table" then
      r[M.BIRTHSIGN_KEY] = id
    end
  end
  return list, nil
end

return M
