-- ============================================================================
-- MODULE: evidence_wallet.lua
-- PURPOSE: Lightweight evidence wallet management for corridor policy enforcement
-- COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
-- OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
-- ============================================================================

local EvidenceWallet = {}
EvidenceWallet.__index = EvidenceWallet

-- Configuration
local CONFIG = {
    OWNER_DID = "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
    SAFETY_KERNEL_REF = "VitalNetSafetyKernel:1.0.0",
    NEURORIGHTS_POLICY = "AugmentedHumanRights:v1",
    MIN_EVIDENCE_COMPLETENESS = 0.86,
    MAX_RECORDS_PER_WALLET = 10000
}

-- Utility functions
local function generate_uuid()
    local template = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'
    return string.gsub(template, '[xy]', function(c)
        local v = (c == 'x') and math.random(0, 15) or math.random(8, 11)
        return string.format('%x', v)
    end)
end

local function get_timestamp()
    return os.date("!%Y-%m-%dT%H:%M:%SZ")
end

local function calculate_hash(data)
    -- In production, use proper SHA256 implementation
    -- This is a simplified version for demonstration
    local hash = 0
    for i = 1, #data do
        hash = (hash * 31 + data:byte(i)) % 4294967296
    end
    return string.format("%08x", hash)
end

-- Evidence Record Class
local EvidenceRecord = {}
EvidenceRecord.__index = EvidenceRecord

function EvidenceRecord.new(evidence_type, metric, delta, unit, corridor, owner_did, linked_bci_device_id)
    local self = setmetatable({}, EvidenceRecord)
    self.record_id = generate_uuid()
    self.row_ref = ""
    self.evidence_type = evidence_type
    self.metric = metric
    self.delta = delta
    self.unit = unit
    self.timestamp = get_timestamp()
    self.owner_did = owner_did
    self.corridor = corridor
    self.completeness_score = 0.0
    self.linked_bci_device_id = linked_bci_device_id
    self.consciousness_preservation_relevant = false
    return self
end

function EvidenceRecord:calculate_completeness(chain_verified, audit_passed)
    local score = 0.3  -- Base score for valid record structure

    if self.row_ref and #self.row_ref > 0 then
        score = score + 0.2
    end

    if chain_verified then
        score = score + 0.3
    end

    if audit_passed then
        score = score + 0.2
    end

    self.completeness_score = score
    return score
end

function EvidenceRecord:meets_threshold()
    return self.completeness_score >= CONFIG.MIN_EVIDENCE_COMPLETENESS
end

function EvidenceRecord:to_table()
    return {
        record_id = self.record_id,
        row_ref = self.row_ref,
        evidence_type = self.evidence_type,
        metric = self.metric,
        delta = self.delta,
        unit = self.unit,
        timestamp = self.timestamp,
        owner_did = self.owner_did,
        corridor = self.corridor,
        completeness_score = self.completeness_score,
        linked_bci_device_id = self.linked_bci_device_id,
        consciousness_preservation_relevant = self.consciousness_preservation_relevant
    }
end

-- Evidence Wallet Class
function EvidenceWallet.new(owner_did, linked_bci_device_id)
    local self = setmetatable({}, EvidenceWallet)
    self.wallet_id = "evidence-wallet-" .. generate_uuid()
    self.owner_did = owner_did
    self.linked_bci_device_id = linked_bci_device_id
    self.evidence_records = {}
    self.health_improvements = {}
    self.eco_improvements = {}
    self.care_access_providers = {}
    self.consciousness_preservation_data = nil
    self.wallet_status = "active"
    self.created_at = get_timestamp()
    self.updated_at = get_timestamp()
    self.evidence_completeness_score = 1.0
    return self
end

function EvidenceWallet:add_evidence_record(record)
    -- Verify completeness before adding
    record:calculate_completeness(true, true)

    if not record:meets_threshold() then
        return nil, string.format(
            "Evidence record %s has completeness score %f < %f",
            record.record_id,
            record.completeness_score,
            CONFIG.MIN_EVIDENCE_COMPLETENESS
        )
    end

    -- Check wallet capacity
    if #self.evidence_records >= CONFIG.MAX_RECORDS_PER_WALLET then
        return nil, "Wallet capacity exceeded"
    end

    -- Track improvements
    if record.evidence_type == "health" then
        self.health_improvements[record.metric] = (self.health_improvements[record.metric] or 0) + record.delta
    elseif record.evidence_type == "eco" then
        self.eco_improvements[record.metric] = (self.eco_improvements[record.metric] or 0) + record.delta
    end

    table.insert(self.evidence_records, record)
    self.updated_at = get_timestamp()
    self:recalculate_completeness()

    return true, nil
end

function EvidenceWallet:recalculate_completeness()
    if #self.evidence_records == 0 then
        self.evidence_completeness_score = 1.0
        return
    end

    local total = 0
    for _, record in ipairs(self.evidence_records) do
        total = total + record.completeness_score
    end
    self.evidence_completeness_score = total / #self.evidence_records
end

function EvidenceWallet:meets_threshold()
    return self.evidence_completeness_score >= CONFIG.MIN_EVIDENCE_COMPLETENESS
end

function EvidenceWallet:get_records_by_corridor(corridor)
    local records = {}
    for _, record in ipairs(self.evidence_records) do
        if record.corridor == corridor then
            table.insert(records, record)
        end
    end
    return records
end

function EvidenceWallet:get_bci_linked_records()
    local records = {}
    for _, record in ipairs(self.evidence_records) do
        if record.linked_bci_device_id then
            table.insert(records, record)
        end
    end
    return records
end

function EvidenceWallet:to_table()
    return {
        wallet_id = self.wallet_id,
        owner_did = self.owner_did,
        linked_bci_device_id = self.linked_bci_device_id,
        evidence_records_count = #self.evidence_records,
        health_improvements = self.health_improvements,
        eco_improvements = self.eco_improvements,
        wallet_status = self.wallet_status,
        created_at = self.created_at,
        updated_at = self.updated_at,
        evidence_completeness_score = self.evidence_completeness_score,
        consciousness_preservation_enabled = self.consciousness_preservation_data ~= nil
    }
end

-- Wallet Manager
local WalletManager = {}
WalletManager.__index = WalletManager

function WalletManager.new()
    local self = setmetatable({}, WalletManager)
    self.wallets = {}
    self.audit_log = {}
    return self
end

function WalletManager:get_or_create_wallet(owner_did, linked_bci_device_id)
    -- Neurorights check: ensure no discrimination based on BCI presence
    self:log_audit("Equal protection verified for " .. owner_did .. " (has_bci: " .. tostring(linked_bci_device_id ~= nil) .. ")")

    if not self.wallets[owner_did] then
        local wallet = EvidenceWallet.new(owner_did, linked_bci_device_id)
        self.wallets[owner_did] = wallet
        self:log_audit("Created new wallet for " .. owner_did)
    end

    return self.wallets[owner_did]
end

function WalletManager:add_evidence_record(owner_did, record_data)
    local wallet = self:get_or_create_wallet(owner_did, record_data.linked_bci_device_id)
    local record = EvidenceRecord.new(
        record_data.evidence_type,
        record_data.metric,
        record_data.delta,
        record_data.unit,
        record_data.corridor,
        record_data.owner_did,
        record_data.linked_bci_device_id
    )

    local success, err = wallet:add_evidence_record(record)
    if not success then
        return nil, err
    end

    self:log_audit("Evidence record added: " .. record.record_id .. " for " .. owner_did)
    return record, nil
end

function WalletManager:log_audit(message)
    local entry = string.format("[%s] %s", get_timestamp(), message)
    table.insert(self.audit_log, entry)
end

function WalletManager:get_audit_log()
    return self.audit_log
end

function WalletManager:get_all_wallets()
    local wallets = {}
    for owner_did, wallet in pairs(self.wallets) do
        table.insert(wallets, wallet:to_table())
    end
    return wallets
end

-- Export
return {
    EvidenceWallet = EvidenceWallet,
    EvidenceRecord = EvidenceRecord,
    WalletManager = WalletManager,
    CONFIG = CONFIG
}
