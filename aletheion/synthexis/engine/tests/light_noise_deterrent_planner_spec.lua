-- Aletheion :: Synthexis :: LightNoiseDeterrentPlanner Spec
-- Path: aletheion/synthexis/engine/tests/light_noise_deterrent_planner_spec.lua
-- Purpose: Scenario tests for bat-sensitive riparian corridor and pollinator-rich streetscape
-- Context: Bats avoid lit, impervious urban corridors and prefer dark, vegetated routes near water [web:98][web:106][web:109][web:111].
--          Pollinators in Phoenix depend on native desert plants and pesticide-free, chemically gentle practices [web:93][web:94][web:97][web:99][web:96][web:110][web:112].

local planner = require("aletheion.synthexis.engine.light_noise_deterrent_planner")

local Spec = {}

local function approx_equal(a, b, eps)
  eps = eps or 1e-6
  return math.abs(a - b) <= eps
end

local function assert_true(cond, msg)
  if not cond then
    error(msg or "assertion failed", 2)
  end
end

local function assert_false(cond, msg)
  if cond then
    error(msg or "assertion failed (expected false)", 2)
  end
end

local function assert_between(v, lo, hi, msg)
  if v < lo or v > hi then
    error((msg or "value out of range") .. string.format(" (got %.3f, expected [%.3f, %.3f])", v, lo, hi), 2)
  end
end

---------------------------------------------------------------------------
-- Shared fixtures
---------------------------------------------------------------------------

local human_comfort_targets = {
  safety_min_lux_by_segment = {
    riparian_corridor = 1.0,     -- minimal safety lighting along dark water-adjacent trails
    pollinator_street = 5.0      -- brighter streetscape for human navigation
  },
  max_annoyance_dB_by_band = {
    { band = "night",   max_dB = 50.0 },
    { band = "evening", max_dB = 55.0 },
    { band = "day",     max_dB = 60.0 }
  }
}

-- Greater mouse-eared style bat proxy: strongly light-averse, uses dark riparian corridors, avoids lit, impervious roads [web:98][web:106][web:109][web:111].
local bat_agent = {
  species_id = "myotis_proxy_bat",
  light_tolerance = {
    light_averse = true,
    max_ALAN_lux = 2.0,
    spectral_sensitivity_bands = { {400, 520} } -- avoid blue/green dominant street lighting [web:98][web:101]
  },
  noise_tolerance = {
    max_night_dB = 45.0,
    impulsive_noise_penalty = 2.0,
    continuous_noise_penalty = 1.0
  },
  seasonal_sensitivities = {
    active_seasons = { "spring", "summer", "fall" },
    critical_dark_corridor_periods = { {0, 86400 * 365} }
  },
  priority_weights = {
    conservation_priority = 3.0,
    ecosystem_service_weight = 2.0
  }
}

-- Pollinator guild: native bees & butterflies in Phoenix depending on native plant streetscapes [web:93][web:94][web:97][web:96][web:99][web:110][web:112].
local pollinator_agent = {
  species_id = "phoenix_pollinator_guild",
  light_tolerance = {
    light_averse = false,
    max_ALAN_lux = 10.0,
    spectral_sensitivity_bands = { {350, 450}, {500, 650} }
  },
  noise_tolerance = {
    max_night_dB = 55.0,
    impulsive_noise_penalty = 1.0,
    continuous_noise_penalty = 0.5
  },
  seasonal_sensitivities = {
    active_seasons = { "spring", "summer", "fall" },
    critical_dark_corridor_periods = {}
  },
  priority_weights = {
    conservation_priority = 2.0,
    ecosystem_service_weight = 3.0
  }
}

-- Treaties: one for strictly dark riparian corridor, one for gentle pollinator streetscape.
local treaties = {
  {
    treaty_id = "riparian_dark_corridor_bat_protection",
    species = { bat_agent },
    scope = {
      spatial_segments = { "riparian_corridor" },
      temporal_windows = {
        { start_ts = 0, end_ts = 86400 * 365, recurrence = "always" }
      }
    },
    infra_domains = {
      lighting = {
        max_lux = 2.0,
        permitted_spectra = { {560, 620} }, -- warm amber band, low blue content [web:101][web:98]
        shielding_required = true
      },
      noise = {
        max_dB_by_timeband = {
          { band = "night",   max_dB = 45.0 },
          { band = "evening", max_dB = 50.0 },
          { band = "day",     max_dB = 60.0 }
        }
      }
    },
    enforcement_mode = "hard"
  },
  {
    treaty_id = "pollinator_friendly_streetscape",
    species = { pollinator_agent },
    scope = {
      spatial_segments = { "pollinator_street" },
      temporal_windows = {
        { start_ts = 0, end_ts = 86400 * 365, recurrence = "always" }
      }
    },
    infra_domains = {
      lighting = {
        max_lux = 15.0,
        permitted_spectra = { {500, 650} } -- favor warm spectrum compatible with pollinator plants and human comfort [web:96][web:110]
      },
      noise = {
        max_dB_by_timeband = {
          { band = "night",   max_dB = 50.0 },
          { band = "evening", max_dB = 55.0 },
          { band = "day",     max_dB = 60.0 }
        }
      }
    },
    enforcement_mode = "hard"
  }
}

local base_infrastructure_state = {
  luminaires = {
    riparian_light_1 = {
      segment_id = "riparian_corridor",
      capabilities = {
        dimmable = true,
        spectra = { {400, 520}, {560, 620} }, -- has both blueish and amber; treaties should force amber [web:98][web:101]
        max_lux = 30.0,
        can_strobe = false
      },
      current_schedule = {}
    },
    pollinator_street_light_1 = {
      segment_id = "pollinator_street",
      capabilities = {
        dimmable = true,
        spectra = { {500, 650} },
        max_lux = 40.0,
        can_strobe = false
      },
      current_schedule = {}
    }
  },
  sound_emitters = {
    riparian_speaker_1 = {
      segment_id = "riparian_corridor",
      capabilities = {
        max_dB = 65.0,
        tonal_profiles = { "broadband", "narrowband" }
      },
      current_schedule = {}
    },
    pollinator_street_speaker_1 = {
      segment_id = "pollinator_street",
      capabilities = {
        max_dB = 70.0,
        tonal_profiles = { "broadband", "narrowband", "modulated" }
      },
      current_schedule = {}
    }
  }
}

-- Species activity: bats highly active at night in riparian corridor, pollinators active day/evening on streetscape.
local species_activity_forecast = {
  riparian_corridor = {
    myotis_proxy_bat = { night = 0.9, evening = 0.6, day = 0.1 },
    phoenix_pollinator_guild = { night = 0.0, evening = 0.1, day = 0.2 }
  },
  pollinator_street = {
    myotis_proxy_bat = { night = 0.2, evening = 0.1, day = 0.0 },
    phoenix_pollinator_guild = { night = 0.1, evening = 0.7, day = 0.9 }
  }
}

---------------------------------------------------------------------------
-- Test 1: Bat-sensitive riparian corridor stays dark & quiet
---------------------------------------------------------------------------

function Spec.test_riparian_corridor_bat_protection()
  local result = planner.plan_deterrent_envelopes(
    human_comfort_targets,
    treaties,
    base_infrastructure_state,
    species_activity_forecast
  )

  local riparian_light = result.lighting["riparian_light_1"]
  assert_true(riparian_light ~= nil, "missing riparian_light_1 schedule")

  local bands_seen = {}
  for _, slot in ipairs(riparian_light.schedule) do
    bands_seen[slot.band] = true
    if slot.band == "night" then
      -- Night lux should be very low to protect dark flight corridor [web:98][web:106][web:109][web:111].
      assert_between(slot.lux, 0.0, 3.0, "night lux too high on riparian corridor")
      assert_true(slot.spectra_band ~= nil, "night spectrum must be set")
      assert_between(slot.spectra_band[1], 550, 570, "night spectrum lower bound should be amber-compatible")
      assert_between(slot.spectra_band[2], 600, 630, "night spectrum upper bound should be amber-compatible")
    elseif slot.band == "evening" then
      assert_between(slot.lux, 0.0, 5.0, "evening lux too high on riparian corridor")
    elseif slot.band == "day" then
      assert_between(slot.lux, human_comfort_targets.safety_min_lux_by_segment.riparian_corridor, 10.0, "daytime lux should respect safety but not exceed treaty limits too much")
    end
  end
  assert_true(bands_seen.day and bands_seen.evening and bands_seen.night, "missing one or more bands for riparian_light_1")

  local riparian_sound = result.sound["riparian_speaker_1"]
  assert_true(riparian_sound ~= nil, "missing riparian_speaker_1 schedule")

  local bands_sound = {}
  for _, slot in ipairs(riparian_sound.schedule) do
    bands_sound[slot.band] = true
    if slot.band == "night" then
      -- With bat activity 0.9 at night, deterrent should be fully suppressed (0 dB) [web:98][web:106][web:109][web:111].
      assert_true(approx_equal(slot.dB, 0.0), "night deterrent should be off in bat corridor")
      assert_true(slot.purpose == "off", "night deterrent purpose must be off")
    elseif slot.band == "evening" then
      -- Moderate bat activity; deterrent must be very low or zero.
      assert_between(slot.dB, 0.0, 15.0, "evening deterrent too loud in bat corridor")
    elseif slot.band == "day" then
      -- Minimal bat activity; some gentle deterrent may be allowed but must still respect treaty caps.
      assert_between(slot.dB, 0.0, 22.5, "day deterrent too loud in bat corridor")
    end
  end
  assert_true(bands_sound.day and bands_sound.evening and bands_sound.night, "missing one or more bands for riparian_speaker_1")

  for _, v in ipairs(result.violations) do
    assert_false(v.type == "lighting_over_treaty_limit" and v.luminaire_id == "riparian_light_1",
      "riparian_light_1 must not exceed bat treaty lux limits at any band")
    assert_false(v.type == "noise_over_treaty_limit" and v.emitter_id == "riparian_speaker_1",
      "riparian_speaker_1 must not exceed bat treaty noise limits at any band")
  end
end

---------------------------------------------------------------------------
-- Test 2: Pollinator-rich streetscape balances human safety & non-lethal deterrence
---------------------------------------------------------------------------

function Spec.test_pollinator_streetscape_deterrence()
  local result = planner.plan_deterrent_envelopes(
    human_comfort_targets,
    treaties,
    base_infrastructure_state,
    species_activity_forecast
  )

  local street_light = result.lighting["pollinator_street_light_1"]
  assert_true(street_light ~= nil, "missing pollinator_street_light_1 schedule")

  local bands_seen = {}
  for _, slot in ipairs(street_light.schedule) do
    bands_seen[slot.band] = true
    if slot.band == "day" then
      -- Daytime: high pollinator activity but also human navigation; lux should meet safety and stay under treaty cap [web:93][web:94][web:97][web:96][web:99][web:110][web:112].
      assert_between(slot.lux, human_comfort_targets.safety_min_lux_by_segment.pollinator_street, 15.0,
        "day lux on pollinator_street must respect safety and treaty max")
    elseif slot.band == "evening" then
      -- Evening: pollinators still active; keep moderate lux and avoid harsh blue.
      assert_between(slot.lux, human_comfort_targets.safety_min_lux_by_segment.pollinator_street, 15.0,
        "evening lux on pollinator_street too high")
    elseif slot.band == "night" then
      -- Night: pollinator activity low; dim to minimal necessary safety.
      assert_between(slot.lux, 0.0, 10.0, "night lux on pollinator_street should be modest")
    end
    if slot.spectra_band then
      assert_between(slot.spectra_band[1], 500, 520, "spectra lower bound should align with warm pollinator-friendly band")
      assert_between(slot.spectra_band[2], 620, 660, "spectra upper bound should align with warm pollinator-friendly band")
    end
  end
  assert_true(bands_seen.day and bands_seen.evening and bands_seen.night, "missing one or more bands for pollinator_street_light_1")

  local street_sound = result.sound["pollinator_street_speaker_1"]
  assert_true(street_sound ~= nil, "missing pollinator_street_speaker_1 schedule")

  local bands_sound = {}
  for _, slot in ipairs(street_sound.schedule) do
    bands_sound[slot.band] = true
    if slot.band == "day" then
      -- High pollinator activity and humans; deterrent must be gentle and below annoyance caps [web:93][web:94][web:97][web:96][web:99][web:110][web:112].
      assert_between(slot.dB, 0.0, 30.0, "day deterrent too loud on pollinator_street")
    elseif slot.band == "evening" then
      assert_between(slot.dB, 0.0, 25.0, "evening deterrent too loud on pollinator_street")
    elseif slot.band == "night" then
      assert_between(slot.dB, 0.0, 20.0, "night deterrent too loud on pollinator_street")
    end
  end
  assert_true(bands_sound.day and bands_sound.evening and bands_sound.night, "missing one or more bands for pollinator_street_speaker_1")

  for _, v in ipairs(result.violations) do
    assert_false(v.type == "lighting_over_treaty_limit" and v.luminaire_id == "pollinator_street_light_1",
      "pollinator_street_light_1 must not exceed pollinator treaty lux limits")
    assert_false(v.type == "noise_over_treaty_limit" and v.emitter_id == "pollinator_street_speaker_1",
      "pollinator_street_speaker_1 must not exceed pollinator treaty noise limits")
  end
end

---------------------------------------------------------------------------
-- Minimal runner (if executed directly)
---------------------------------------------------------------------------

if ... == nil then
  local tests = {
    "test_riparian_corridor_bat_protection",
    "test_pollinator_streetscape_deterrence"
  }
  for _, name in ipairs(tests) do
    io.write(string.format("[Synthexis Spec] running %s ... ", name))
    local ok, err = pcall(Spec[name])
    if ok then
      io.write("OK\n")
    else
      io.write("FAIL\n")
      io.write(err .. "\n")
      os.exit(1)
    end
  end
  os.exit(0)
end

return Spec
