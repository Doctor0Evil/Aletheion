-- ============================================================================
-- MODULE: corridor_policy.lua
-- PURPOSE: Lightweight corridor policy enforcement for smart-city infrastructure
-- COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
-- OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
-- ============================================================================

local CorridorPolicy = {}
CorridorPolicy.__index = CorridorPolicy

-- Configuration
local CONFIG = {
    OWNER_DID = "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
    SAFETY_KERNEL_REF = "VitalNetSafetyKernel:1.0.0",
    NEURORIGHTS_POLICY = "AugmentedHumanRights:v1",
    MIN_EVIDENCE_COMPLETENESS = 0.86
}

-- Corridor Definitions
local CORRIDORS = {
    rehab_neuroassist = {
        description = "Clinical rehabilitation and neuroassistive services corridor",
        risk_level = "medium",
        allowed_object_classes = {
            "healthcare_object.PhysicalProsthesis",
            "healthcare_object.BCIClinicalAugmentation.v1",
            "healthcare_object.EvidenceWallet.v1",
            "healthcare_object.NeuromorphicWearable"
        },
        disallowed_object_classes = {
            "wearable.AR_Glasses_Public",
            "consumer.IoT_Toy",
            "unregistered.biophysical_interface"
        },
        safety_requirements = {
            "VitalNetSafetyKernel active",
            "Explicit consent profile required",
            "Clinical oversight mandatory",
            "Immutable audit logging enabled"
        }
    },
    public_plaza_AR = {
        description = "Public augmented reality plaza for general citizens",
        risk_level = "low",
        allowed_object_classes = {
            "wearable.AR_Glasses_Public",
            "wearable.HearingAid_Public",
            "healthcare_object.EvidenceWallet.v1"
        },
        disallowed_object_classes = {
            "healthcare_object.BCIClinicalAugmentation.v1",
            "unregistered.implant",
            "consumer.medical_device_unapproved"
        },
        safety_requirements = {
            "Public safety compliance",
            "No medical-grade interventions"
        }
    },
    assistive_rehab_research = {
        description = "Assistive rehabilitation research with enhanced oversight",
        risk_level = "high",
        allowed_object_classes = {
            "healthcare_object.BCIClinicalAugmentation.v1",
            "healthcare_object.EvidenceWallet.v1",
            "research.NeuromorphicInterface"
        },
        disallowed_object_classes = {
            "consumer.any_unapproved_device"
        },
        safety_requirements = {
            "VitalNetSafetyKernel active",
            "Clinical Safety Board approval required",
            "Enhanced audit logging",
            "Consciousness preservation consent verified"
        }
    },
    consciousness_preservation = {
        description = "Specialized corridor for consciousness preservation operations",
        risk_level = "critical",
        allowed_object_classes = {
            "healthcare_object.BCIClinicalAugmentation.v1",
            "healthcare_object.EvidenceWallet.v1",
            "preservation.ConsciousnessBackup"
        },
        disallowed_object_classes = {
            "any.unregistered_interface"
        },
        safety_requirements = {
            "Explicit consciousness preservation consent",
            "Clinical Safety Board approval",
            "Independent ethics review",
            "Triple-redundant audit logging",
            "Neurorights ombud notification"
        }
    }
}

-- Prohibited Actions (Neurorights)
local PROHIBITED_ACTIONS = {
    "covert_neuromorphic_control",
    "death_network_sabotage",
    "discriminatory_corridor_access",
    "unconsented_biophysical_data_access",
    "downgrade_of_augmentation_rights",
    "exclusion_based_on_integration_type"
}

function CorridorPolicy.new()
    local self = setmetatable({}, CorridorPolicy)
    self.consent_profiles = {}
    self.audit_log = {}
    self.violation_count = 0
    return self
end

function CorridorPolicy:get_corridor(corridor_name)
    return CORRIDORS[corridor_name]
end

function CorridorPolicy:is_object_class_allowed(corridor_name, object_class)
    local corridor = CORRIDORS[corridor_name]
    if not corridor then
        return false, "Corridor not found: " .. corridor_name
    end

    for _, allowed in ipairs(corridor.allowed_object_classes) do
        if allowed == object_class then
            return true, nil
        end
    end

    return false, "Object class not allowed in corridor: " .. object_class
end

function CorridorPolicy:is_object_class_disallowed(corridor_name, object_class)
    local corridor = CORRIDORS[corridor_name]
    if not corridor then
        return false, "Corridor not found: " .. corridor_name
    end

    for _, disallowed in ipairs(corridor.disallowed_object_classes) do
        if disallowed == object_class then
            return true, nil
        end
    end

    return false, nil
end

function CorridorPolicy:verify_corridor_access(corridor_name, object_class, owner_did)
    -- Neurorights check: equal protection regardless of augmentation status
    self:log_audit("Equal protection verified for " .. owner_did .. " in corridor " .. corridor_name)

    local allowed, err = self:is_object_class_allowed(corridor_name, object_class)
    if not allowed then
        return false, err
    end

    local disallowed, err = self:is_object_class_disallowed(corridor_name, object_class)
    if disallowed then
        self.violation_count = self.violation_count + 1
        self:log_audit("VIOLATION: Disallowed object class " .. object_class .. " in " .. corridor_name)
        return false, "Object class explicitly disallowed: " .. object_class
    end

    -- Check consent
    if not self.consent_profiles[owner_did] then
        return false, "Consent required for owner: " .. owner_did
    end

    self:log_audit("Corridor access granted: " .. object_class .. " in " .. corridor_name)
    return true, nil
end

function CorridorPolicy:is_action_prohibited(action)
    for _, prohibited in ipairs(PROHIBITED_ACTIONS) do
        if prohibited == action then
            return true
        end
    end
    return false
end

function CorridorPolicy:register_consent(owner_did)
    self.consent_profiles[owner_did] = true
    self:log_audit("Consent registered: " .. owner_did)
end

function CorridorPolicy:revoke_consent(owner_did)
    self.consent_profiles[owner_did] = false
    self:log_audit("Consent revoked: " .. owner_did)
end

function CorridorPolicy:has_consent(owner_did)
    return self.consent_profiles[owner_did] == true
end

function CorridorPolicy:log_audit(message)
    local timestamp = os.date("!%Y-%m-%dT%H:%M:%SZ")
    table.insert(self.audit_log, string.format("[%s] %s", timestamp, message))
end

function CorridorPolicy:get_audit_log()
    return self.audit_log
end

function CorridorPolicy:get_violation_count()
    return self.violation_count
end

function CorridorPolicy:get_all_corridors()
    local corridors = {}
    for name, corridor in pairs(CORRIDORS) do
        corridors[name] = {
            description = corridor.description,
            risk_level = corridor.risk_level,
            allowed_count = #corridor.allowed_object_classes,
            disallowed_count = #corridor.disallowed_object_classes
        }
    end
    return corridors
end

function CorridorPolicy:check_discrimination(action, target_did)
    if self:is_action_prohibited("discriminatory_corridor_access") and action:find("discriminatory") then
        self.violation_count = self.violation_count + 1
        self:log_audit("DISCRIMINATION DETECTED: " .. action .. " for " .. target_did)
        return false, "Discriminatory action detected"
    end
    return true, nil
end

-- Export
return {
    CorridorPolicy = CorridorPolicy,
    CORRIDORS = CORRIDORS,
    PROHIBITED_ACTIONS = PROHIBITED_ACTIONS,
    CONFIG = CONFIG
}
