-- Nightly LightNoisePesticidePlanner runner wiring species activity, treaties, and envelopes.

local json = require("cjson.safe")

local M = {}

local function load_species_activity_forecast(date_iso)
  -- Placeholder: in real deployment, pull from ERM state model APIs.
  return {
    date = date_iso,
    corridors = {
      { id = "bat-corridor-001", nocturnal = true, peak_start = "20:30", peak_end = "05:00" },
      { id = "pollinator-corridor-007", diurnal = true, peak_start = "06:00", peak_end = "18:00" },
    }
  }
end

local function load_biotic_treaties_for_city()
  return {
    {
      id = "BT-BATS-ALAN-001",
      corridor_id = "bat-corridor-001",
      max_lux_night = 5.0,
      spectrum_allowed = { "warm_white" },
    },
    {
      id = "BT-POLLINATOR-PESTICIDE-001",
      corridor_id = "pollinator-corridor-007",
      pesticide_nospray_hours = { "06:00-18:00" },
    }
  }
end

local function load_human_comfort_targets()
  return {
    min_path_lux = 3.0,
    min_square_lux = 8.0,
    max_night_noise_db = 45.0,
  }
end

local function load_base_infrastructure_state()
  return {
    luminaires = {
      { id = "L-001", segment = "seg-main-ave-01", corridor_id = "bat-corridor-001", spectrum = "cool_white" },
      { id = "L-002", segment = "seg-park-07", corridor_id = "pollinator-corridor-007", spectrum = "warm_white" },
    },
    noise_sources = {
      { id = "NS-001", kind = "traffic", segment = "seg-main-ave-01", base_db = 60.0 },
    },
    pesticide_regimes = {
      { id = "P-001", corridor_id = "pollinator-corridor-007", schedule = { "02:00-03:00", "10:00-11:00" } },
    }
  }
end

local function call_native_planner(input_tbl)
  local ok, ffi = pcall(require, "ffi")
  if not ok then
    return nil, "ffi_not_available"
  end

  ffi.cdef[[
    const char* alethion_synthexis_lnp_compute(const char* json_input);
  ]]

  local lib = ffi.load("alethion_synthexis_lnp_core_001")
  local payload = json.encode(input_tbl)
  local cstr = ffi.new("char[?]", #payload + 1, payload)
  local out_c = lib.alethion_synthexis_lnp_compute(cstr)
  if out_c == nil then
    return nil, "native_null"
  end
  local out_json = ffi.string(out_c)
  local out_tbl, err = json.decode(out_json)
  if not out_tbl then
    return nil, "decode_error:" .. tostring(err)
  end
  return out_tbl, nil
end

function M.run_nightly_planner(date_iso)
  local species = load_species_activity_forecast(date_iso)
  local treaties = load_biotic_treaties_for_city()
  local comfort = load_human_comfort_targets()
  local infra   = load_base_infrastructure_state()

  local input_tbl = {
    date = date_iso,
    species_activity = species,
    treaties = treaties,
    human_comfort_targets = comfort,
    base_infrastructure_state = infra,
  }

  local result, err = call_native_planner(input_tbl)
  if not result then
    return {
      status = "error",
      error  = err,
      date   = date_iso,
    }
  end

  return {
    status = "ok",
    date   = date_iso,
    envelopes  = result.envelopes or {},
    violations = result.violations or {},
    meta       = result.meta or {},
  }
end

return M
