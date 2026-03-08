-- Purpose: Birth-Sign jurisdictional signature propagation and enforcement.
-- Layers: L1 Edge Ingest, L2 State Model, L4 Governance Preflight, L6 Actuation.
-- Domains: land, water, air, biosignals, citizens must always carry non-empty birthSignId.[file:1]

local BirthSignPropagation = {}

-- Domains for which Birth-Signs are mandatory.
local GOVERNED_DOMAINS = {
  land       = true,
  water      = true,
  air        = true,
  biosignals = true,
  citizens   = true,
}

----------------------------------------------------------------------
-- External dependencies (to be provided by existing services)
----------------------------------------------------------------------

-- Geospatial context service: given (lat, lon, time) returns a BirthSignId or nil.
local GeoContext = require("aletheion.geo.context")          -- birth_sign_for_point(lat, lon, t) -> string|nil
-- State model ingress: append events with governance metadata into L2 state model.
local StateModelIngress = require("aletheion.erm.state_ingress") -- ingest(events_with_birthsign) -> ()
-- Compliance logger: structured errors and audit events.
local ComplianceLog = require("aletheion.compliance.log")    -- violation(kind, details), info(kind, details)

----------------------------------------------------------------------
-- Utility helpers
----------------------------------------------------------------------

local function is_governed_domain(domain)
  return GOVERNED_DOMAINS[domain] == true
end

-- Ensure event has the standard governance envelope table.
local function ensure_governance_envelope(event)
  event.governance = event.governance or {}
  return event.governance
end

-- Extract spatial-temporal coordinates from an event.
local function extract_spatiotemporal(event)
  local loc = event.location or {}
  return loc.lat, loc.lon, event.timestamp or os.time()
end

-- Attach a birthSignId to a single event if required by its domain.
local function attach_birth_sign_to_event(event)
  local domain = event.domain
  if not is_governed_domain(domain) then
    return event -- no-op for non-governed domains
  end

  local g = ensure_governance_envelope(event)

  -- Respect existing, valid birthSignId if already present.
  if type(g.birthSignId) == "string" and g.birthSignId ~= "" then
    return event
  end

  local lat, lon, t = extract_spatiotemporal(event)
  if not lat or not lon then
    ComplianceLog.violation("birthsign_missing_location", {
      domain = domain,
      reason = "Cannot resolve Birth-Sign without lat/lon",
      event_id = event.id,
    })
    error("Birth-Sign propagation: missing location for governed domain " .. tostring(domain))
  end

  local birthSignId = GeoContext.birth_sign_for_point(lat, lon, t)
  if not birthSignId or birthSignId == "" then
    ComplianceLog.violation("birthsign_resolution_failed", {
      domain = domain,
      lat = lat,
      lon = lon,
      event_id = event.id,
    })
    error("Birth-Sign propagation: geocontext lookup failed for governed domain " .. tostring(domain))
  end

  g.birthSignId = birthSignId

  ComplianceLog.info("birthsign_attached", {
    domain = domain,
    birthSignId = birthSignId,
    event_id = event.id,
  })

  return event
end

----------------------------------------------------------------------
-- Public API: Ingest path
----------------------------------------------------------------------

-- Attach birthSignId and forward a batch of events into the state model.
-- Each event MUST include:
--   - event.domain (string)
--   - event.location.lat, event.location.lon (for governed domains)
--   - event.timestamp (optional, defaults to now)
-- A governance envelope is added/updated at event.governance.birthSignId.[file:1]
function BirthSignPropagation.ingest_with_birth_sign(events)
  if type(events) ~= "table" then
    error("BirthSignPropagation.ingest_with_birth_sign expects a table of events")
  end

  local enriched = {}
  for i, ev in ipairs(events) do
    enriched[i] = attach_birth_sign_to_event(ev)
  end

  StateModelIngress.ingest(enriched)
end

----------------------------------------------------------------------
-- Public API: Actuation guard
----------------------------------------------------------------------

-- Validate that any actuation touching governed domains carries a non-empty birthSignId.
-- actuation_request MUST include:
--   - actuation_request.domain (string)
--   - actuation_request.governance.birthSignId (string) for governed domains.
-- If validation fails, an error is raised and the caller MUST NOT perform actuation.[file:1]
function BirthSignPropagation.require_birth_sign_for_actuation(actuation_request)
  if type(actuation_request) ~= "table" then
    error("BirthSignPropagation.require_birth_sign_for_actuation expects a table")
  end

  local domain = actuation_request.domain
  if not is_governed_domain(domain) then
    return true -- Non-governed domains are not constrained here.
  end

  local g = actuation_request.governance or {}
  local birthSignId = g.birthSignId

  if type(birthSignId) ~= "string" or birthSignId == "" then
    ComplianceLog.violation("birthsign_missing_for_actuation", {
      domain = domain,
      actuation_id = actuation_request.id,
    })
    error("Birth-Sign enforcement: non-empty birthSignId required for actuation in domain " .. tostring(domain))
  end

  ComplianceLog.info("birthsign_verified_for_actuation", {
    domain = domain,
    actuation_id = actuation_request.id,
    birthSignId = birthSignId,
  })

  return true
end

----------------------------------------------------------------------
-- Convenience wrapper for orchestrators
----------------------------------------------------------------------

-- Orchestrator helper: one-shot pattern combining ingest and later actuation.
-- Example usage pattern in a workflow:
--   local events = collect_edge_events()
--   BirthSignPropagation.ingest_with_birth_sign(events)
--   ...
--   BirthSignPropagation.require_birth_sign_for_actuation(plan.actuation)
--   perform_actuation(plan.actuation)
function BirthSignPropagation.guard_and_actuate(actuation_request, perform_fn)
  BirthSignPropagation.require_birth_sign_for_actuation(actuation_request)
  return perform_fn(actuation_request)
end

return BirthSignPropagation
