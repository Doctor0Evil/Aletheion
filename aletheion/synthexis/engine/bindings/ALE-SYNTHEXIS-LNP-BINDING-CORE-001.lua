-- Lua binding wrapper around the LightNoisePesticidePlanner native library.
-- Exposes a single function: compute_envelopes(input_tbl) -> envelopes, violations, meta.

local json = require("cjson.safe")

local M = {}

-- Native FFI binding (Rust-compiled .so/.dll).
-- The concrete loader (path, name) can be adjusted per deployment.
local ffi = require("ffi")

ffi.cdef[[
  // Serialized JSON-in / JSON-out interface to keep ABI stable.
  const char* alethion_synthexis_lnp_compute(const char* json_input);
]]

local lib = ffi.load("alethion_synthexis_lnp_core_001")

--- Compute operating envelopes using the native LNP planner.
-- @param input_tbl Lua table matching the planner_input structure:
--   { date, base_state, species_activity, comfort_thresholds }
-- @return envelopes_tbl, violations_tbl, meta_tbl
function M.compute_envelopes(input_tbl)
  local input_json, enc_err = json.encode(input_tbl)
  if not input_json then
    return nil, nil, { error = "encode_failed", detail = enc_err }
  end

  local c_str = ffi.new("const char*", input_json)
  local out_c = lib.alethion_synthexis_lnp_compute(c_str)
  if out_c == nil then
    return nil, nil, { error = "native_null_response" }
  end

  local out_json = ffi.string(out_c)
  local out_tbl, dec_err = json.decode(out_json)
  if not out_tbl then
    return nil, nil, { error = "decode_failed", detail = dec_err }
  end

  local envelopes = out_tbl.envelopes or {}
  local violations = out_tbl.violations or {}
  local meta = out_tbl.meta or {}

  return envelopes, violations, meta
end

return M
