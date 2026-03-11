local Bridge = {}

local registry = {}
local metrics = {}

local function metric_row(id)
    local row = metrics[id]
    if not row then
        row = { ok = 0, warn = 0, err = 0 }
        metrics[id] = row
    end
    return row
end

function Bridge.register(id, fn)
    registry[id] = fn
end

function Bridge.call(id, zone, epoch, payload)
    local fn = registry[id]
    if not fn then
        return false, "missing"
    end
    local r = metric_row(id)
    local ok, w = fn(zone, epoch, payload)
    if ok then
        r.ok = r.ok + 1
    elseif w then
        r.warn = r.warn + 1
    else
        r.err = r.err + 1
    end
    return ok, w
end

function Bridge.snapshot()
    return metrics
end

return Bridge
