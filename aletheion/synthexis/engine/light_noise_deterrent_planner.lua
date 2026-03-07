-- Aletheion :: Synthexis :: LightNoiseDeterrentPlanner
-- Path: aletheion/synthexis/engine/light_noise_deterrent_planner.lua
-- Role: Compute non-lethal light/noise deterrent envelopes that respect BioticTreaties and human comfort

local LightNoiseDeterrentPlanner = {}

-- core type hints (informal, for tooling and documentation)
-- SpeciesAgent = {
--   species_id = "string",
--   light_tolerance = { light_averse = boolean, max_ALAN_lux = number, spectral_sensitivity_bands = { {min_nm, max_nm} } },
--   noise_tolerance = { max_night_dB = number, impulsive_noise_penalty = number, continuous_noise_penalty = number },
--   seasonal_sensitivities = { active_seasons = { "string" }, critical_dark_corridor_periods = { {start_ts, end_ts} } },
--   priority_weights = { conservation_priority = number, ecosystem_service_weight = number }
-- }
--
-- BioticTreaty = {
--   treaty_id = "string",
--   species = { SpeciesAgent, ... },
--   scope = {
--     spatial_segments = { "segment_id", ... },
--     temporal_windows = { {start_ts, end_ts, recurrence = "string"}, ... }
--   },
--   infra_domains = {
--     lighting = {
--       max_lux = number,
--       permitted_spectra = { {min_nm, max_nm} },
--       shielding_required = boolean
--     },
--     noise = {
--       max_dB_by_timeband = { {band = "night"|"day"|"evening", max_dB = number} }
--     }
--   },
--   enforcement_mode = "hard"|"advisory"
-- }

-- base_infrastructure_state = {
--   luminaires = {
--      [lum_id] = {
--        segment_id = "string",
--        capabilities = { dimmable = boolean, spectra = { {min_nm,max_nm} }, max_lux = number, can_strobe = boolean },
--        current_schedule = { {start_ts, end_ts, lux, spectra_band, mode} }
--      }, ...
--   },
--   sound_emitters = {
--      [emitter_id] = {
--        segment_id = "string",
--        capabilities = { max_dB = number, tonal_profiles = { "broadband","narrowband","modulated" } },
--        current_schedule = { {start_ts, end_ts, dB, profile, purpose} }
--      }, ...
--   }
-- }
--
-- human_comfort_targets = {
--   safety_min_lux_by_segment = { [segment_id] = number },
--   max_annoyance_dB_by_band = { {band = "night"|"day"|"evening", max_dB = number} }
-- }
--
-- species_activity_forecast = {
--   -- probability that species is actively using a segment at time-band
--   [segment_id] = {
--      [species_id] = { night = number, day = number, evening = number }
--   }
-- }

---------------------------------------------------------------------------
-- Utility helpers
---------------------------------------------------------------------------

local function band_for_time(local_hour)
  if local_hour >= 22 or local_hour < 6 then return "night"
  elseif local_hour >= 6 and local_hour < 18 then return "day"
  else return "evening" end
end

local function ts_to_hour(ts)
  return os.date("*t", ts).hour
end

local function clamp(v, lo, hi)
  if v < lo then return lo end
  if v > hi then return hi end
  return v
end

local function overlaps(a_start, a_end, b_start, b_end)
  return not (a_end <= b_start or b_end <= a_start)
end

local function treaty_applies_to_segment_and_time(treaty, segment_id, ts)
  local spatial_ok = false
  for _, seg in ipairs(treaty.scope.spatial_segments or {}) do
    if seg == segment_id then spatial_ok = true break end
  end
  if not spatial_ok then return false end
  local windows = treaty.scope.temporal_windows or {}
  if #windows == 0 then return true end
  for _, w in ipairs(windows) do
    if overlaps(w.start_ts, w.end_ts, ts, ts + 1) then
      return true
    end
  end
  return false
end

local function band_limit_from_treaties(treaties, segment_id, band)
  local max_lux = math.huge
  local allowed_spectra = {}
  local max_dB = math.huge
  for _, t in ipairs(treaties) do
    if treaty_applies_to_segment_and_time(t, segment_id, os.time()) then
      local L = t.infra_domains and t.infra_domains.lighting or nil
      local N = t.infra_domains and t.infra_domains.noise or nil
      if L and L.max_lux and L.max_lux < max_lux then
        max_lux = L.max_lux
      end
      if L and L.permitted_spectra then
        for _, sb in ipairs(L.permitted_spectra) do
          table.insert(allowed_spectra, sb)
        end
      end
      if N and N.max_dB_by_timeband then
        for _, nb in ipairs(N.max_dB_by_timeband) do
          if nb.band == band and nb.max_dB < max_dB then
            max_dB = nb.max_dB
          end
        end
      end
    end
  end
  if max_lux == math.huge then max_lux = nil end
  if max_dB == math.huge then max_dB = nil end
  return max_lux, allowed_spectra, max_dB
end

local function safest_spectrum(cap_spectra, allowed_spectra, species_list)
  -- strategy: choose spectrum band present in both capability and allowed_spectra
  -- that minimally overlaps sensitive bands of high-priority species
  if not cap_spectra or #cap_spectra == 0 then return nil end
  local candidates = {}
  if allowed_spectra and #allowed_spectra > 0 then
    for _, c in ipairs(cap_spectra) do
      for _, a in ipairs(allowed_spectra) do
        local mn = math.max(c[1], a[1])
        local mx = math.min(c[2], a[2])
        if mn < mx then
          table.insert(candidates, {mn, mx})
        end
      end
    end
  else
    for _, c in ipairs(cap_spectra) do
      table.insert(candidates, {c[1], c[2]})
    end
  end
  if #candidates == 0 then return nil end
  local best_idx, best_penalty = 1, math.huge
  for i, band in ipairs(candidates) do
    local penalty = 0
    for _, sp in ipairs(species_list) do
      local sens = sp.light_tolerance and sp.light_tolerance.spectral_sensitivity_bands or {}
      local weight = (sp.priority_weights and sp.priority_weights.conservation_priority or 1)
      for _, sb in ipairs(sens) do
        local mn = math.max(band[1], sb[1])
        local mx = math.min(band[2], sb[2])
        if mn < mx then
          penalty = penalty + weight * (mx - mn)
        end
      end
    end
    if penalty < best_penalty then
      best_penalty = penalty
      best_idx = i
    end
  end
  return candidates[best_idx]
end

local function deterrent_intensity_for_segment(species_activity, band, base_max_dB, human_max_dB)
  -- non-lethal deterrence: stay WELL below hearing-damage & annoyance thresholds
  -- and modulate intensity only in segments with low sensitive-activity
  local activity = 0
  for _, v in pairs(species_activity or {}) do
    activity = math.max(activity, v[band] or 0)
  end
  local max_dB = math.min(base_max_dB or 65, human_max_dB or 60)
  if activity >= 0.7 then
    return 0 -- no deterrent: corridor is too sensitive
  elseif activity >= 0.3 then
    return clamp(max_dB * 0.25, 0, max_dB)
  else
    return clamp(max_dB * 0.5, 0, max_dB)
  end
end

---------------------------------------------------------------------------
-- Main planner
---------------------------------------------------------------------------

-- plan_deterrent_envelopes(
--   human_comfort_targets,
--   treaties,
--   base_infrastructure_state,
--   species_activity_forecast
-- ) -> { lighting = {...}, sound = {...}, violations = {...} }
function LightNoiseDeterrentPlanner.plan_deterrent_envelopes(human_comfort_targets, treaties, base_infrastructure_state, species_activity_forecast)
  local out = { lighting = {}, sound = {}, violations = {} }

  -- index species from treaties
  local all_species = {}
  for _, t in ipairs(treaties or {}) do
    for _, sp in ipairs(t.species or {}) do
      table.insert(all_species, sp)
    end
  end

  -- LUMINAIRES: choose spectra & lux envelopes that repel pests but protect sensitive species
  for lum_id, lum in pairs(base_infrastructure_state.luminaires or {}) do
    local seg = lum.segment_id
    local seg_forecast = species_activity_forecast[seg] or {}
    out.lighting[lum_id] = { schedule = {} }

    -- basic: define three bands (day/evening/night)
    for _, band in ipairs({ "day", "evening", "night" }) do
      local sample_ts = os.time({year=2026, month=3, day=6, hour=(band=="day" and 12 or band=="evening" and 19 or 1)})
      local lux_limit, allowed_spectra, _ = band_limit_from_treaties(treaties, seg, band)
      local safety_min = (human_comfort_targets.safety_min_lux_by_segment or {})[seg] or 0
      local cap_max = lum.capabilities.max_lux or 50
      local target_lux
      if band == "night" then
        local max_from_treaty = lux_limit or cap_max
        target_lux = clamp(math.min(max_from_treaty, cap_max), 0, 5 + safety_min) -- keep very low at night
      else
        target_lux = clamp(math.min(lux_limit or cap_max, cap_max), safety_min, cap_max)
      end
      local spec_band = safest_spectrum(lum.capabilities.spectra, allowed_spectra, all_species)
      table.insert(out.lighting[lum_id].schedule, {
        band = band,
        start_ts = sample_ts,
        end_ts = sample_ts + 4 * 3600,
        lux = target_lux,
        spectra_band = spec_band
      })
      if lux_limit and target_lux > lux_limit + 1e-6 then
        table.insert(out.violations, {
          type = "lighting_over_treaty_limit",
          luminaire_id = lum_id,
          band = band,
          target_lux = target_lux,
          treaty_limit = lux_limit
        })
      end
    end
  end

  -- SOUND EMITTERS: non-lethal deterrence envelopes
  for emitter_id, em in pairs(base_infrastructure_state.sound_emitters or {}) do
    local seg = em.segment_id
    local seg_forecast = species_activity_forecast[seg] or {}
    out.sound[emitter_id] = { schedule = {} }
    for _, band in ipairs({ "day", "evening", "night" }) do
      local human_band_limit = nil
      for _, hb in ipairs(human_comfort_targets.max_annoyance_dB_by_band or {}) do
        if hb.band == band then human_band_limit = hb.max_dB break end
      end
      local _, _, treaty_max_dB = band_limit_from_treaties(treaties, seg, band)
      local base_max = em.capabilities.max_dB or 70
      local max_allowed = base_max
      if human_band_limit then
        max_allowed = math.min(max_allowed, human_band_limit)
      end
      if treaty_max_dB then
        max_allowed = math.min(max_allowed, treaty_max_dB)
      end
      local det_dB = deterrent_intensity_for_segment(seg_forecast, band, max_allowed, human_band_limit or max_allowed)
      local purpose = det_dB > 0 and "remote_nonlethal_deterrent" or "off"
      table.insert(out.sound[emitter_id].schedule, {
        band = band,
        dB = det_dB,
        profile = det_dB > 0 and "broadband" or "none",
        purpose = purpose
      })
      if treaty_max_dB and det_dB > treaty_max_dB + 1e-6 then
        table.insert(out.violations, {
          type = "noise_over_treaty_limit",
          emitter_id = emitter_id,
          band = band,
          target_dB = det_dB,
          treaty_limit = treaty_max_dB
        })
      end
    end
  end

  return out
end

return LightNoiseDeterrentPlanner
