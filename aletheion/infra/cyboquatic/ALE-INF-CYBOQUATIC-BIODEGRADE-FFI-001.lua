local cjson = require("cjson.safe")
local ffi = require("ffi")

ffi.cdef[[
const char* ale_ecosafety_biodegrade_safestep(const char* ctx_json);
]]

local lib = ffi.load("ale_erm_ecosafety_biodegrade")

local M = {}

local function call_safestep(ctx_tbl)
  local j, enc_err = cjson.encode(ctx_tbl)
  if not j then
    return nil, "encode_failed:" .. tostring(enc_err)
  end
  local c_in = ffi.new("const char*", j)
  local c_out = lib.ale_ecosafety_biodegrade_safestep(c_in)
  if c_out == nil then
    return nil, "native_null"
  end
  local out = ffi.string(c_out)
  local decoded, dec_err = cjson.decode(out)
  if not decoded then
    return nil, "decode_failed:" .. tostring(dec_err)
  end
  return decoded, nil
end

function M.guard_actuation(node_ctx, proposed_cmd)
  local ctx = {
    node = node_ctx,
    command = proposed_cmd,
  }
  local decision, err = call_safestep(ctx)
  if not decision then
    return false, { action = "STOP", reason = err }
  end
  if decision.action == "ALLOW" then
    return true, decision
  elseif decision.action == "DERATE" then
    return false, decision
  else
    return false, decision
  end
end

return M
