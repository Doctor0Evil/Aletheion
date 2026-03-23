-- filepath: runtime/lua/citymesh/ci_preflight/eco_ci_guard.lua

local M = {}

local function check_ecosafety_corridors(node_spec)
    if not node_spec.ecosafety_corridor_id then
        return false, "no corridor, no build"
    end
    return true, "corridor declared"
end

local function check_risk_coordinates(node_spec)
    local coords = node_spec.risk_coordinates or {}
    local required = {
        "r_degrade",
        "r_residualmass",
        "r_microplastics",
        "r_tox_acute",
        "r_tox_chronic",
        "r_shear",
        "r_habitatload",
    }
    for _, key in ipairs(required) do
        local v = coords[key]
        if v == nil or v < 0.0 or v > 1.0 then
            return false, "missing or out-of-range risk coordinate: " .. key
        end
    end
    return true, "risk coordinates ok"
end

function M.preflight(node_spec)
    local ok, msg = check_ecosafety_corridors(node_spec)
    if not ok then
        return false, msg
    end
    local ok2, msg2 = check_risk_coordinates(node_spec)
    if not ok2 then
        return false, msg2
    end
    return true, "ecosafety CI preflight passed"
end

return M
