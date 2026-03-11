local Great_Functions_0121 = {}

local BLACKLIST_SEQUENCES = {
  "SHA3_512",
  "SHA3-512",
  "sha3_512",
  "sha3-512",
  "Sha3_512",
  "Sha3-512"
}

local BLACKLIST_TOKENS = {
  "SHA3",
  "sha3",
  "Keccak",
  "KECCAK",
  "ripemd160",
  "RIPEMD160",
  "argon",
  "ARGON"
}

local function line_has_blacklisted_content(line)
  local trimmed = line:gsub("^%s+", ""):gsub("%s+$", "")
  if trimmed == "" then
    return false
  end
  for _, seq in ipairs(BLACKLIST_SEQUENCES) do
    if string.find(trimmed, seq, 1, true) then
      return true
    end
  end
  local parts = {}
  for token in string.gmatch(trimmed, "[^%s%(%){},;:.]+") do
    table.insert(parts, token)
  end
  for _, token in ipairs(BLACKLIST_TOKENS) do
    for _, part in ipairs(parts) do
      if part == token then
        return true
      end
    end
  end
  return false
end

local function assess_per_line_quality(source)
  local report = {
    file_path = "core/vm/orchestration/lua/great_functions/gf_0121_cybercore_bridge.lua",
    lines = {},
    rejected = false,
    rejection_reason = nil
  }

  local index = 1
  for line in source:gmatch("([^\n]*)\n?") do
    if line == "" and index > #source then
      break
    end
    local trimmed = line:gsub("^%s+", ""):gsub("%s+$", "")
    local token_count = 0
    for _ in string.gmatch(trimmed, "%S+") do
      token_count = token_count + 1
    end
    local entropy = 0
    do
      local chars = {}
      for c in trimmed:gmatch(".") do
        chars[c] = true
      end
      local unique = 0
      for _ in pairs(chars) do
        unique = unique + 1
      end
      local length = #trimmed
      local base = unique * 3
      local penalty = 0
      if length > 120 then
        penalty = 10
      elseif length > 80 then
        penalty = 5
      end
      local score = base - penalty
      if score < 0 then
        score = 0
      end
      if score > 100 then
        score = 100
      end
      entropy = score
    end

    local branching = 0
    if trimmed ~= "" then
      if trimmed:find("if ") or trimmed:find(" if%(") then
        branching = branching + 15
      end
      if trimmed:find("when ") then
        branching = branching + 12
      end
      if trimmed:find("for ") or trimmed:find("while ") then
        branching = branching + 10
      end
      if trimmed:find("&&") or trimmed:find("||") then
        branching = branching + 8
      end
      if branching > 100 then
        branching = 100
      end
    end

    local density = 0
    if trimmed ~= "" and token_count > 0 then
      local length = #trimmed
      local d = (token_count / length) * 120
      if d < 0 then
        d = 0
      end
      if d > 100 then
        d = 100
      end
      density = math.floor(d + 0.5)
    end

    local blacklist_hits = 0
    if line_has_blacklisted_content(trimmed) then
      blacklist_hits = 1
      if report.rejected == false then
        report.rejected = true
        report.rejection_reason = "Blacklisted token in line " .. tostring(index)
      end
    end

    local quality_grade = 0
    if blacklist_hits == 0 then
      local base = (entropy * 2 + branching + density) / 4
      if base < 0 then
        base = 0
      end
      if base > 100 then
        base = 100
      end
      quality_grade = math.floor(base + 0.5)
    end

    table.insert(report.lines, {
      line_number = index,
      entropy_score = entropy,
      branching_score = branching,
      token_density_score = density,
      blacklist_hits = blacklist_hits,
      quality_grade = quality_grade
    })

    index = index + 1
    if index > 2048 then
      break
    end
  end

  return report
end

local function build_trace_token(tenant_id, node_id, invocation_id, great_function_id, language_name, language_version, language_dialect, issued_at_ms)
  return {
    tenant_id = tenant_id,
    node_id = node_id,
    invocation_id = invocation_id,
    great_function_id = great_function_id,
    language_tag = {
      name = language_name,
      version_hint = language_version,
      dialect = language_dialect
    },
    issued_at_ms = issued_at_ms
  }
end

local function build_cybercore_request(channel_id, mode, locale_id, payload, trace_token, soft_timeout_ms, hard_timeout_ms)
  return {
    channel_id = channel_id,
    mode = mode,
    locale_id = locale_id,
    payload_bytes = payload,
    trace_token = trace_token,
    soft_timeout_ms = soft_timeout_ms,
    hard_timeout_ms = hard_timeout_ms
  }
end

local function now_ms()
  return math.floor(os.clock() * 1000)
end

function Great_Functions_0121.describe()
  return {
    id = "GF_0121_LUA_CYBERCORE_BRIDGE",
    language = {
      name = "Lua",
      version_hint = "5.4",
      dialect = "Aletheion-VM"
    },
    latency_budget = {
      hard_limit_ms = 50,
      soft_limit_ms = 24,
      target_ms = 12
    },
    security_profile = {
      risk = "LOW",
      max_offline_ms = 120000,
      max_chain_depth = 5,
      allow_dynamic_eval = false
    }
  }
end

function Great_Functions_0121.verify_source(source)
  return assess_per_line_quality(source)
end

function Great_Functions_0121.invoke(vm_ctx, cybercore_client, mode, payload)
  local cfg = Great_Functions_0121.describe()
  local issued_at = now_ms()

  local trace = build_trace_token(
    vm_ctx.tenant_id or "default",
    vm_ctx.node_id or "lua-node",
    vm_ctx.next_invocation_id or 121,
    cfg.id,
    cfg.language.name,
    cfg.language.version_hint,
    cfg.language.dialect,
    issued_at
  )

  local req = build_cybercore_request(
    "lua-cybercore-bridge",
    mode or "COMMAND",
    vm_ctx.locale_id or "en_US",
    payload or "",
    trace,
    cfg.latency_budget.soft_limit_ms,
    cfg.latency_budget.hard_limit_ms
  )

  local start = now_ms()
  local resp = cybercore_client.invoke(req)
  local elapsed = now_ms() - start

  local soft_hit = elapsed > cfg.latency_budget.soft_limit_ms
  local hard_hit = elapsed > cfg.latency_budget.hard_limit_ms

  return {
    channel_id = resp.channel_id or "lua-cybercore-bridge",
    status_code = resp.status_code or 500,
    payload_bytes = resp.payload_bytes or "",
    soft_deadline_hit = soft_hit or resp.soft_deadline_hit == true,
    hard_deadline_hit = hard_hit or resp.hard_deadline_hit == true
  }
end

return Great_Functions_0121
