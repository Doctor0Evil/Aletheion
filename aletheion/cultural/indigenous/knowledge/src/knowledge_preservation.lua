-- Aletheion Indigenous Knowledge Preservation System v20260310
-- License: BioticTreaty_v3
-- Compliance: UNDRIP_2007_NAGPRA_1990_Akimel_Oodham_Piipaash_Agreements_Arizona_Tribal_Laws

local KNOWLEDGE_PRESERVATION_VERSION = 20260310
local MAX_KNOWLEDGE_RECORDS = 131072
local MAX_KNOWLEDGE_KEEPERS = 8192
local MAX_CULTURAL_ARTIFACTS = 65536
local MAX_LANGUAGE_RECORDS = 32768
local MAX_TRADITIONAL_PRACTICES = 16384

local KnowledgeType = {
    ORAL_HISTORY = 1, TRADITIONAL_ECOLOGICAL = 2, MEDICINAL = 3, AGRICULTURAL = 4,
    CEREMONIAL = 5, LINGUISTIC = 6, ARTISTIC = 7, ASTRONOMICAL = 8,
    NAVIGATION = 9, CRAFT = 10, CULINARY = 11, SPIRITUAL = 12
}

local AccessLevel = {
    PUBLIC = 1, COMMUNITY = 2, RESTRICTED = 3, SACRED = 4, SECRET = 5
}

local Tribe = {
    AKIMEL_OODHAM = 1, PIIPAASH = 2, TOHONO_OODHAM = 3, APACHE = 4,
    NAVAJO = 5, HOPI = 6, YAVAPAI = 7, OTHER = 8
}

local KnowledgeRecord = {}
KnowledgeRecord.__index = KnowledgeRecord

function KnowledgeRecord:new(record_id, knowledge_type, title, tribe, access_level)
    local self = setmetatable({}, KnowledgeRecord)
    self.record_id = record_id or 0
    self.knowledge_type = knowledge_type or 1
    self.title = title or ""
    self.tribe = tribe or 1
    self.access_level = access_level or 2
    self.language = ""
    self.content_hash = ""
    self.media_references = {}
    self.media_count = 0
    self.knowledge_keeper_id = 0
    self.date_recorded_ns = 0
    self.date_verified_ns = 0
    self.tribal_consent_obtained = false
    self.consent_date_ns = 0
    self.cultural_protocol_followed = false
    self.restriction_notes = ""
    self.preservation_priority = 0
    self.last_accessed_ns = 0
    self.access_count = 0
    return self
end

function KnowledgeRecord:is_accessible(user_clearance, user_tribe, now_ns)
    if self.access_level == 1 then return true end
    if self.access_level == 2 and user_clearance >= 2 then return true end
    if self.access_level >= 3 and user_tribe == self.tribe and user_clearance >= 3 then return true end
    return false
end

function KnowledgeRecord:requires_tribal_consent()
    return self.access_level >= 3 or self.knowledge_type == 5 or self.knowledge_type == 12
end

local KnowledgeKeeper = {}
KnowledgeKeeper.__index = KnowledgeKeeper

function KnowledgeKeeper:new(keeper_id, name, tribe, specialization)
    local self = setmetatable({}, KnowledgeKeeper)
    self.keeper_id = keeper_id or 0
    self.name = name or ""
    self.tribe = tribe or 1
    self.specialization = specialization or {}
    self.certification_level = 1
    self.languages = {}
    self.language_count = 0
    self.records_contributed = 0
    self.records_verified = 0
    self.apprentices = {}
    self.apprentice_count = 0
    self.elder_status = false
    self.community_recognition = 0
    self.active = true
    self.consent_authority = false
    return self
end

function KnowledgeKeeper:can_verify(knowledge_type)
    return self.active and table.contains(self.specialization, knowledge_type)
end

function KnowledgeKeeper:add_apprentice(apprentice_id)
    if self.apprentice_count >= 10 then return false end
    self.apprentices[self.apprentice_count + 1] = apprentice_id
    self.apprentice_count = self.apprentice_count + 1
    return true
end

local CulturalArtifact = {}
CulturalArtifact.__index = CulturalArtifact

function CulturalArtifact:new(artifact_id, artifact_type, name, tribe, origin_date)
    local self = setmetatable({}, CulturalArtifact)
    self.artifact_id = artifact_id or 0
    self.artifact_type = artifact_type or 1
    self.name = name or ""
    self.tribe = tribe or 1
    self.origin_date = origin_date or ""
    self.current_location = ""
    self.custodian = ""
    self.condition = "unknown"
    self.preservation_needs = {}
    self.cultural_significance = ""
    self.tribal_ownership_claimed = false
    self.repatriation_status = "not_requested"
    self.digital_preservation = false
    self.access_restrictions = ""
    self.last_inventory_ns = 0
    return self
end

function CulturalArtifact:requires_repatriation()
    return self.tribal_ownership_claimed and self.repatriation_status == "pending"
end

local LanguageRecord = {}
LanguageRecord.__index = LanguageRecord

function LanguageRecord:new(record_id, language_name, tribe, record_type)
    local self = setmetatable({}, LanguageRecord)
    self.record_id = record_id or 0
    self.language_name = language_name or ""
    self.tribe = tribe or 1
    self.record_type = record_type or 1
    self.speaker_count = 0
    self.fluency_levels = {}
    self.teaching_materials = 0
    self.documentation_completeness = 0.0
    self.endangerment_level = 0
    self.revitalization_active = false
    self.last_updated_ns = 0
    return self
end

function LanguageRecord:compute_endangerment()
    if self.speaker_count < 10 then self.endangerment_level = 5
    elseif self.speaker_count < 100 then self.endangerment_level = 4
    elseif self.speaker_count < 1000 then self.endangerment_level = 3
    elseif self.speaker_count < 10000 then self.endangerment_level = 2
    else self.endangerment_level = 1 end
    return self.endangerment_level
end

local TraditionalPractice = {}
TraditionalPractice.__index = TraditionalPractice

function TraditionalPractice:new(practice_id, practice_name, tribe, practice_type)
    local self = setmetatable({}, TraditionalPractice)
    self.practice_id = practice_id or 0
    self.practice_name = practice_name or ""
    self.tribe = tribe or 1
    self.practice_type = practice_type or 1
    self.seasonal_timing = ""
    self.location_required = ""
    self.participants_required = 0
    self.materials_needed = {}
    self.cultural_protocols = {}
    self.knowledge_keepers = {}
    self.transmission_status = "active"
    self.youth_participation = 0
    self.documented = false
    self.protected = false
    return self
end

local IndigenousKnowledgeSystem = {}
IndigenousKnowledgeSystem.__index = IndigenousKnowledgeSystem

function IndigenousKnowledgeSystem:new(system_id, city_code, region)
    local self = setmetatable({}, IndigenousKnowledgeSystem)
    self.system_id = system_id or 0
    self.city_code = city_code or ""
    self.region = region or ""
    self.knowledge_records = {}
    self.record_count = 0
    self.knowledge_keepers = {}
    self.keeper_count = 0
    self.cultural_artifacts = {}
    self.artifact_count = 0
    self.language_records = {}
    self.language_count = 0
    self.traditional_practices = {}
    self.practice_count = 0
    self.total_records_accessible = 0
    self.tribal_consent_records = 0
    self.repatriation_claims = 0
    self.repatriation_completed = 0
    self.language_revitalization_active = 0
    self.last_audit_ns = 0
    return self
end

function IndigenousKnowledgeSystem:register_knowledge_record(record)
    if self.record_count >= MAX_KNOWLEDGE_RECORDS then return false, "RECORD_LIMIT" end
    if record:requires_tribal_consent() and not record.tribal_consent_obtained then
        return false, "TRIBAL_CONSENT_REQUIRED"
    end
    self.knowledge_records[self.record_count + 1] = record
    self.record_count = self.record_count + 1
    if record.tribal_consent_obtained then
        self.tribal_consent_records = self.tribal_consent_records + 1
    end
    return true, "OK"
end

function IndigenousKnowledgeSystem:register_knowledge_keeper(keeper)
    if self.keeper_count >= MAX_KNOWLEDGE_KEEPERS then return false, "KEEPER_LIMIT" end
    self.knowledge_keepers[self.keeper_count + 1] = keeper
    self.keeper_count = self.keeper_count + 1
    return true, "OK"
end

function IndigenousKnowledgeSystem:register_cultural_artifact(artifact)
    if self.artifact_count >= MAX_CULTURAL_ARTIFACTS then return false, "ARTIFACT_LIMIT" end
    self.cultural_artifacts[self.artifact_count + 1] = artifact
    self.artifact_count = self.artifact_count + 1
    if artifact:requires_repatriation() then
        self.repatriation_claims = self.repatriation_claims + 1
    end
    return true, "OK"
end

function IndigenousKnowledgeSystem:register_language_record(language)
    if self.language_count >= MAX_LANGUAGE_RECORDS then return false, "LANGUAGE_LIMIT" end
    language:compute_endangerment()
    self.language_records[self.language_count + 1] = language
    self.language_count = self.language_count + 1
    if language.revitalization_active then
        self.language_revitalization_active = self.language_revitalization_active + 1
    end
    return true, "OK"
end

function IndigenousKnowledgeSystem:register_traditional_practice(practice)
    if self.practice_count >= MAX_TRADITIONAL_PRACTICES then return false, "PRACTICE_LIMIT" end
    self.traditional_practices[self.practice_count + 1] = practice
    self.practice_count = self.practice_count + 1
    return true, "OK"
end

function IndigenousKnowledgeSystem:compute_knowledge_preservation_score()
    if self.record_count == 0 then return 0.0 end
    local consent_rate = self.tribal_consent_records / self.record_count
    local documentation_rate = 0.0
    local verified_count = 0
    for i = 1, self.record_count do
        if self.knowledge_records[i].date_verified_ns > 0 then
            verified_count = verified_count + 1
        end
    end
    documentation_rate = verified_count / self.record_count
    local access_score = self.total_records_accessible / self.record_count
    return (consent_rate * 0.40 + documentation_rate * 0.35 + access_score * 0.25)
end

function IndigenousKnowledgeSystem:compute_language_vitality_index()
    if self.language_count == 0 then return 0.0 end
    local total_vitality = 0.0
    for i = 1, self.language_count do
        local lang = self.language_records[i]
        local vitality = (6 - lang.endangerment_level) / 5.0
        if lang.revitalization_active then vitality = vitality + 0.2 end
        total_vitality = total_vitality + vitality
    end
    return total_vitality / self.language_count
end

function IndigenousKnowledgeSystem:compute_repatriation_progress()
    if self.repatriation_claims == 0 then return 1.0 end
    return self.repatriation_completed / self.repatriation_claims
end

function IndigenousKnowledgeSystem:identify_preservation_priorities()
    local priorities = {}
    local count = 0
    for i = 1, self.language_count do
        local lang = self.language_records[i]
        if lang.endangerment_level >= 4 then
            count = count + 1
            priorities[count] = {
                type = "LANGUAGE",
                name = lang.language_name,
                endangerment = lang.endangerment_level,
                speakers = lang.speaker_count,
                action = "URGENT_DOCUMENTATION_AND_REVITALIZATION",
                tribe = lang.tribe
            }
        end
    end
    for i = 1, self.keeper_count do
        local keeper = self.knowledge_keepers[i]
        if keeper.elder_status and keeper.apprentice_count < 2 then
            count = count + 1
            priorities[count] = {
                type = "KNOWLEDGE_TRANSMISSION",
                name = keeper.name,
                keeper_id = keeper.keeper_id,
                action = "APPRENTICESHIP_PROGRAM_REQUIRED",
                tribe = keeper.tribe
            }
        end
    end
    return priorities, count
end

function IndigenousKnowledgeSystem:get_system_status(now_ns)
    local active_keepers = 0
    for i = 1, self.keeper_count do
        if self.knowledge_keepers[i].active then
            active_keepers = active_keepers + 1
        end
    end
    local endangered_languages = 0
    for i = 1, self.language_count do
        if self.language_records[i].endangerment_level >= 3 then
            endangered_languages = endangered_languages + 1
        end
    end
    return {
        system_id = self.system_id,
        city_code = self.city_code,
        region = self.region,
        total_knowledge_records = self.record_count,
        tribal_consent_records = self.tribal_consent_records,
        total_knowledge_keepers = self.keeper_count,
        active_keepers = active_keepers,
        total_cultural_artifacts = self.artifact_count,
        repatriation_claims = self.repatriation_claims,
        repatriation_completed = self.repatriation_completed,
        total_language_records = self.language_count,
        endangered_languages = endangered_languages,
        language_revitalization_active = self.language_revitalization_active,
        total_traditional_practices = self.practice_count,
        preservation_score = self:compute_knowledge_preservation_score(),
        language_vitality_index = self:compute_language_vitality_index(),
        repatriation_progress = self:compute_repatriation_progress(),
        last_audit_ns = self.last_audit_ns,
        last_update_ns = now_ns
    }
end

function IndigenousKnowledgeSystem:compute_cultural_sovereignty_index()
    local consent_compliance = self.tribal_consent_records / self.record_count.max(1)
    local repatriation_rate = self:compute_repatriation_progress()
    local language_vitality = self:compute_language_vitality_index()
    local keeper_continuity = active_keepers / self.keeper_count.max(1)
    return (consent_compliance * 0.35 + repatriation_rate * 0.25 + 
            language_vitality * 0.25 + keeper_continuity * 0.15)
end

return {
    IndigenousKnowledgeSystem = IndigenousKnowledgeSystem,
    KnowledgeRecord = KnowledgeRecord,
    KnowledgeKeeper = KnowledgeKeeper,
    CulturalArtifact = CulturalArtifact,
    LanguageRecord = LanguageRecord,
    TraditionalPractice = TraditionalPractice,
    KnowledgeType = KnowledgeType,
    AccessLevel = AccessLevel,
    Tribe = Tribe,
    VERSION = KNOWLEDGE_PRESERVATION_VERSION,
}
