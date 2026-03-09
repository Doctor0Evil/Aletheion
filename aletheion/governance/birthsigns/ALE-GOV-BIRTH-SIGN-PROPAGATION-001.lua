-- Aletheion Birth-Sign Jurisdictional Signature Propagation
-- Role:
--  - Attach BirthSignId to all ingested assets/events (S1 → S2).
--  - Ensure every message between Ingest, StateModel, Optimization, Governance, Actuation
--    carries a non-empty birthSignId field when domains touch land, water, air, biosignals, or citizens.
--  - Provide glue for ERM sync orchestrators and SMARTchain validators to enforce jurisdictional preconditions.
--
-- Assumptions:
--  - Rust model aletheion/governance/birthsigns/ALE-GOV-BIRTH-SIGN-MODEL-001.rs defines BirthSignId and domains.
--  - A geospatial context service is reachable via get_birth_sign_for_point(lat, lon, t).
--  - Messages are Lua tables with a reserved field `birthSignId` and a `domains` array of domain strings.
--  - Compliance preflight is handled by ALE-COMP-CORE-001.rs at CI/runtime, not reimplemented here.

local BirthSignPropagation = {}

----------------------------------------------------------------------
-- Domain and scope helpers (mirrors GovernanceDomain in Rust)
----------------------------------------------------------------------

local TERRITORIAL_DOMAINS = {
  Land = true,
  Water = true,
  Air = true,
  Materials = true,
  Mobility = true,
  Biosignals = true,
  Augmentation = true,
  Energy = true,
  Culture = true,
  Emergency = true,
}

-- Return true if any of the message domains require Birth-Sign binding.
local function domains_require_birthsign(domains)
  if type(domains) ~= "table" then
    return false
  end
  for _, d in ipairs(domains) do
    if TERRITORIAL_DOMAINS[d] then
      return true
    end
  end
  return false
end

----------------------------------------------------------------------
-- Geospatial context service adapter
-- (must be wired to real geospatial DB / service in deployment)
----------------------------------------------------------------------

local GeoContext = {}

-- Stub for actual geospatial lookup.
-- Expected to return a non-empty BirthSignId string for (lat, lon, timestamp),
-- or nil if no Birth-Sign is configured for that tile.
function GeoContext.get_birth_sign_for_point(lat, lon, timestamp)
  -- In production, this would query the geospatial context service:
  --  - Tile index / H3 / S2 / custom grid
  --  - Time-versioned BirthSign registry
  --  - Return tile-scoped BirthSignId (matches Rust BirthSignId(pub String))
  --
  -- Here we only expose the hook; implementation belongs in infra/geo stack.
  if type(lat) ~= "number" or type(lon) ~= "number" then
    return nil
  end
  -- Placeholder: caller must replace with real lookup.
  return nil
end

BirthSignPropagation.GeoContext = GeoContext

----------------------------------------------------------------------
-- Core propagation functions
----------------------------------------------------------------------

-- Attach a BirthSignId to a single ingested record if required by domains.
-- record:
--  {
--    assetId = "...",
--    deviceId = "...",
--    lat = number,
--    lon = number,
--    timestamp = number | string | os.time(),
--    domains = { "Water", "Energy", ... },
--    birthSignId = nil | string (will be set/validated),
--    ...
--  }
--
-- Returns:
--  record (mutated) and a status table:
--  {
--    attached = boolean,
--    reason = "ok" | "no_domains" | "no_birthsign" | "not_required",
--  }
function BirthSignPropagation.attach_birth_sign_for_ingest(record, geocontext)
  geocontext = geocontext or GeoContext

  if type(record) ~= "table" then
    return record, { attached = false, reason = "invalid_record" }
  end

  local domains = record.domains
  if not domains_require_birthsign(domains) then
    -- For non-territorial domains, do not force a Birth-Sign.
    return record, { attached = false, reason = "not_required" }
  end

  if type(record.birthSignId) == "string" and record.birthSignId ~= "" then
    -- Already tagged at the edge: accept and propagate.
    return record, { attached = false, reason = "already_present" }
  end

  local lat = record.lat
  local lon = record.lon
  local ts  = record.timestamp or os.time()

  local bsid = geocontext.get_birth_sign_for_point(lat, lon, ts)
  if type(bsid) ~= "string" or bsid == "" then
    -- Missing Birth-Sign is a governance error when domains require it.
    return record, { attached = false, reason = "no_birthsign" }
  end

  record.birthSignId = bsid
  return record, { attached = true, reason = "ok" }
end

-- Ensure that an internal workflow message carries a valid BirthSignId
-- if its domains require one. Intended for hops between:
--  S1 Ingest → S2 StateModel → S4 Optimization → S5 Governance → S6 Actuation.
--
-- msg:
--  {
--    workflowEventId = "...",
--    domains = { "Water", "Mobility", ... },
--    birthSignId = "..." | nil,
--    ...
--  }
--
-- Returns:
--  msg (unchanged) and a validation status:
--  {
--    valid = boolean,
--    reason = "ok" | "missing_birthsign" | "not_required",
--  }
function BirthSignPropagation.validate_message_birth_sign(msg)
  if type(msg) ~= "table" then
    return msg, { valid = false, reason = "invalid_message" }
  end

  local domains = msg.domains
  if not domains_require_birthsign(domains) then
    return msg, { valid = true, reason = "not_required" }
  end

  local bsid = msg.birthSignId
  if type(bsid) == "string" and bsid ~= "" then
    return msg, { valid = true, reason = "ok" }
  end

  return msg, { valid = false, reason = "missing_birthsign" }
end

----------------------------------------------------------------------
-- Batch helpers for ERM sync orchestrator
----------------------------------------------------------------------

-- Apply Birth-Sign attachment to a batch of ingest records.
-- Returns:
--  updated_records, stats
-- stats:
--  {
--    total = N,
--    required = n_required,
--    attached = n_attached,
--    already_present = n_already,
--    missing_birthsign = n_missing,
--    not_required = n_not_required,
--  }
function BirthSignPropagation.attach_for_ingest_batch(records, geocontext)
  geocontext = geocontext or GeoContext

  if type(records) ~= "table" then
    return records, {
      total = 0,
      required = 0,
      attached = 0,
      already_present = 0,
      missing_birthsign = 0,
      not_required = 0,
    }
  end

  local stats = {
    total = 0,
    required = 0,
    attached = 0,
    already_present = 0,
    missing_birthsign = 0,
    not_required = 0,
  }

  for idx, rec in ipairs(records) do
    stats.total = stats.total + 1

    local needs = domains_require_birthsign(rec and rec.domains)
    if not needs then
      stats.not_required = stats.not_required + 1
    else
      stats.required = stats.required + 1
    end

    local updated, status = BirthSignPropagation.attach_birth_sign_for_ingest(rec, geocontext)
    records[idx] = updated

    if status.reason == "ok" and status.attached then
      stats.attached = stats.attached + 1
    elseif status.reason == "already_present" then
      stats.already_present = stats.already_present + 1
    elseif status.reason == "no_birthsign" then
      stats.missing_birthsign = stats.missing_birthsign + 1
    end
  end

  return records, stats
end

-- Validate that every message in a batch carries a BirthSignId if required.
-- Intended to be called:
--  - Immediately before S4 Optimization runs.
--  - Immediately before S6 Actuation on any governed action.
--
-- Returns:
--  messages, stats
-- stats:
--  {
--    total = N,
--    required = n_required,
--    valid = n_valid,
--    missing_birthsign = n_missing,
--    not_required = n_not_required,
--  }
function BirthSignPropagation.validate_message_batch(messages)
  if type(messages) ~= "table" then
    return messages, {
      total = 0,
      required = 0,
      valid = 0,
      missing_birthsign = 0,
      not_required = 0,
    }
  end

  local stats = {
    total = 0,
    required = 0,
    valid = 0,
    missing_birthsign = 0,
    not_required = 0,
  }

  for idx, msg in ipairs(messages) do
    stats.total = stats.total + 1

    local needs = domains_require_birthsign(msg and msg.domains)
    if not needs then
      stats.not_required = stats.not_required + 1
      -- Treat as valid for governance purposes.
      stats.valid = stats.valid + 1
    else
      stats.required = stats.required + 1
      local _, status = BirthSignPropagation.validate_message_birth_sign(msg)
      if status.valid then
        stats.valid = stats.valid + 1
      elseif status.reason == "missing_birthsign" then
        stats.missing_birthsign = stats.missing_birthsign + 1
      end
    end
  end

  return messages, stats
end

----------------------------------------------------------------------
-- SMARTchain / compliance integration hooks
----------------------------------------------------------------------

-- Pre-optimization hook:
--  - Ensures that every candidate action payload includes a BirthSignId
--    when required by its domains.
--  - Returns:
--      ok = true  -> safe to proceed to optimization
--      ok = false -> must reject or repair before proceeding
function BirthSignPropagation.pre_optimization_guard(messages)
  local _, stats = BirthSignPropagation.validate_message_batch(messages)
  -- Optimization MUST NOT proceed if any required message lacks Birth-Sign.
  if stats.missing_birthsign > 0 then
    return false, stats
  end
  return true, stats
end

-- Pre-actuation hook:
--  - Same semantics as pre_optimization_guard, but intended for S6.
function BirthSignPropagation.pre_actuation_guard(messages)
  local _, stats = BirthSignPropagation.validate_message_batch(messages)
  if stats.missing_birthsign > 0 then
    return false, stats
  end
  return true, stats
end

return BirthSignPropagation
