-- Birth-Sign propagation workflow glue for Aletheion.
-- Ensures every governed event carries a birthSignId from ingest through
-- state-model, optimization, governance preflight, and actuation.

local json = require("cjson.safe")

local M = {}

-- Abstract geospatial gateway. In production this will call a dedicated
-- Birth-Sign service; here we expose a pluggable function.
local geo_gateway = {}

--- Configure the geospatial lookup callback.
-- fn signature: fn(lat, lon, timestamp_iso) -> birthSignId | nil, err
function M.set_geo_lookup(fn)
  geo_gateway.lookup = fn
end

--- Attach a Birth-Sign to a raw ingest event (S1 -> S2).
-- event: table with at least { lat, lon, ts } fields.
function M.attach_at_ingest(event)
  if not geo_gateway.lookup then
    return nil, "birthsign_geo_lookup_not_configured"
  end
  if not event.lat or not event.lon or not event.ts then
    return nil, "birthsign_missing_spatiotemporal_fields"
  end

  local bs_id, err = geo_gateway.lookup(event.lat, event.lon, event.ts)
  if not bs_id then
    return nil, "birthsign_lookup_failed:" .. tostring(err or "nil")
  end

  event.birthSignId = bs_id
  return event
end

--- Validate that a pipeline message carries a non-empty birthSignId.
-- This is called at S2, S4, S5, and S6 boundaries.
function M.require_birthsign(msg, stage_name)
  if not msg.birthSignId or msg.birthSignId == "" then
    return false,
      string.format("birthsign_missing_for_stage:%s", stage_name or "unknown")
  end
  return true
end

--- Propagate Birth-Sign from one message to another when deriving state.
-- If dst already has a birthSignId, it is left unchanged.
function M.propagate(src, dst)
  if (not dst.birthSignId or dst.birthSignId == "") and src.birthSignId then
    dst.birthSignId = src.birthSignId
  end
  return dst
end

--- Wrap an optimizer request, enforcing Birth-Sign presence.
function M.wrap_optimizer_request(state_snapshot, optimize_fn)
  local ok, err = M.require_birthsign(state_snapshot, "S4_OPTIMIZATION_INPUT")
  if not ok then
    return nil, err
  end
  return optimize_fn(state_snapshot)
end

--- Wrap a governance preflight call, enforcing Birth-Sign presence.
function M.wrap_governance_check(action_plan, check_fn)
  local ok, err = M.require_birthsign(action_plan, "S5_GOVERNANCE_PREFLIGHT")
  if not ok then
    return nil, err
  end
  return check_fn(action_plan)
end

--- Wrap an actuation call, enforcing Birth-Sign presence.
function M.wrap_actuation(command, actuate_fn)
  local ok, err = M.require_birthsign(command, "S6_ACTUATION")
  if not ok then
    return nil, err
  end
  return actuate_fn(command)
end

--- Utility: serialize a pipeline message for audit logs with Birth-Sign.
function M.to_audit_json(msg)
  local payload = {
    birthSignId = msg.birthSignId or "",
    workflowId = msg.workflowId or "",
    stage = msg.stage or "",
    ts = msg.ts or "",
  }
  local encoded, err = json.encode(payload)
  if not encoded then
    return nil, "birthsign_audit_encode_failed:" .. tostring(err or "nil")
  end
  return encoded
end

return M
