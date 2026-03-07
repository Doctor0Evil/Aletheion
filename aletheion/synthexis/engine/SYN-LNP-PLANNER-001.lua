-- Aletheion Synthexis Light/Noise/Pesticide Planner v1
-- Lua workflow: aligns human comfort targets with BioticTreaty constraints.

local json = require("cjson.safe")

local Planner = {}

-- human_targets: map block_id -> { night_lux_target, max_night_dba, cooling_priority }
-- treaties: list of BioticTreaty objects (de-serialized from ALN model)[file:1]
-- returns: table { lighting = {block_id -> cfg}, noise = {...}, pesticide = {...} }

function Planner.plan(human_targets, treaties)
    local lighting_cfg = {}
    local noise_cfg = {}
    local pesticide_cfg = {}

    for _, treaty in ipairs(treaties or {}) do
        for _, block in ipairs(treaty.scope.blocks or {}) do
            local human = human_targets[block] or {}

            -- Lighting: respect most restrictive lux constraint across treaties
            local min_lux = human.night_lux_target or 10.0
            local allowed_spectra = nil
            local require_shielding = false

            for _, lc in ipairs(treaty.lightingRules or {}) do
                if lc.maxLux and lc.maxLux < min_lux then
                    min_lux = lc.maxLux
                end
                if lc.allowedSpectra and #lc.allowedSpectra > 0 then
                    allowed_spectra = lc.allowedSpectra
                end
                if lc.requireShielding then
                    require_shielding = true
                end
            end

            local lcfg = lighting_cfg[block] or {}
            lcfg.max_night_lux = min_lux
            if allowed_spectra then
                lcfg.allowed_spectra = allowed_spectra
            end
            lcfg.require_shielding = require_shielding or lcfg.require_shielding or false
            lighting_cfg[block] = lcfg

            -- Noise: combine max dBA windows
            local nc_list = {}
            for _, nr in ipairs(treaty.noiseRules or {}) do
                for _, pair in ipairs(nr.maxDbAByTime or {}) do
                    table.insert(nc_list, pair)
                end
            end
            if #nc_list > 0 then
                noise_cfg[block] = noise_cfg[block] or {}
                noise_cfg[block].max_dba_windows = nc_list
            end

            -- Pesticides: combine blackout windows + buffers
            local blackout = {}
            local max_buffer = 0.0
            local banned_classes = {}

            for _, cr in ipairs(treaty.chemicalRules or {}) do
                for _, w in ipairs(cr.blackoutWindows or {}) do
                    table.insert(blackout, w)
                end
                if cr.noSprayBuffersM and cr.noSprayBuffersM > max_buffer then
                    max_buffer = cr.noSprayBuffersM
                end
                for _, cls in ipairs(cr.bannedClasses or {}) do
                    banned_classes[cls] = true
                end
            end

            if #blackout > 0 or max_buffer > 0.0 or next(banned_classes) ~= nil then
                pesticide_cfg[block] = pesticide_cfg[block] or {}
                pesticide_cfg[block].blackout_windows = blackout
                pesticide_cfg[block].buffer_m = max_buffer
                pesticide_cfg[block].banned_classes = {}
                for cls, _ in pairs(banned_classes) do
                    table.insert(pesticide_cfg[block].banned_classes, cls)
                end
            end
        end
    end

    return {
        lighting = lighting_cfg,
        noise = noise_cfg,
        pesticide = pesticide_cfg,
    }
end

function Planner.plan_to_json(human_targets, treaties)
    local result = Planner.plan(human_targets, treaties)
    return json.encode(result)
end

return Planner
