-- Thermaphora MicroclimateFieldDesigner v1
-- Goal: propose shade/evap/cool-material mixes per block under simple constraints.

local Designer = {}

-- baseline_field: MicroclimateField-like table
-- constraints: { max_tree_water_lpd, max_mister_water_lpd, max_albedo_change, target_delta_c }
-- returns: { interventions = { trees = n, shade_structures = n, cool_pavement_frac, misting_level01 }, expected_temp_drop_c }

function Designer.design(baseline_field, constraints)
    local air = baseline_field.air_temp_c or 40.0
    local shade = baseline_field.shade_fraction01 or 0.1
    local canopy = baseline_field.tree_canopy_fraction01 or 0.05
    local evap_cap = baseline_field.evap_cooling_capacity01 or 0.0

    local target_drop = constraints.target_delta_c or 3.0

    local shade_gap = math.max(0.0, 0.6 - shade)
    local trees_gap = math.max(0.0, 0.25 - canopy)

    local trees = math.floor(trees_gap * 10.0)
    local shade_structures = math.floor(shade_gap * 8.0)

    local tree_water = trees * 200.0
    if tree_water > (constraints.max_tree_water_lpd or tree_water) then
        local scale = (constraints.max_tree_water_lpd or tree_water) / tree_water
        trees = math.floor(trees * scale)
    end

    local mist_level = math.min(1.0, 0.5 + 0.5 * (1.0 - shade))
    local mister_water = mist_level * 500.0
    if mister_water > (constraints.max_mister_water_lpd or mister_water) then
        local scale = (constraints.max_mister_water_lpd or mister_water) / mister_water
        mist_level = mist_level * scale
    end

    local cool_frac = 0.0
    if not baseline_field.has_cool_pavement then
        cool_frac = math.min(0.5, constraints.max_albedo_change or 0.3)
    end

    local delta_c = 0.0
    delta_c = delta_c + 4.0 * math.min(0.6, shade + shade_gap)
    delta_c = delta_c + 2.0 * math.min(0.3, canopy + trees_gap)
    if cool_frac > 0 then
        delta_c = delta_c + 1.0 * cool_frac
    end
    delta_c = delta_c + 2.0 * mist_level * (1.0 - baseline_field.relative_humidity01)

    delta_c = math.min(delta_c, target_drop + 1.5)

    return {
        interventions = {
            trees = trees,
            shade_structures = shade_structures,
            cool_pavement_frac = cool_frac,
            misting_level01 = mist_level,
        },
        expected_temp_drop_c = delta_c,
    }
end

return Designer
