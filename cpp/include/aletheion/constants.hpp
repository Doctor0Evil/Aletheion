// ============================================================================
// HEADER: constants.hpp
// PURPOSE: Global constants for Aletheion Edge Core
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_CONSTANTS_HPP
#define ALETHEION_CONSTANTS_HPP

#include <string>
#include <cstdint>

namespace aletheion {

// ============================================================================
// IDENTITY & COMPLIANCE
// ============================================================================

constexpr const char* OWNER_DID = "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
constexpr const char* SAFETY_KERNEL_REF = "VitalNetSafetyKernel:1.0.0";
constexpr const char* NEURORIGHTS_POLICY = "AugmentedHumanRights:v1";
constexpr const char* ALETHEION_VERSION = "1.0.0";

// ============================================================================
// EVIDENCE THRESHOLDS
// ============================================================================

constexpr double MIN_EVIDENCE_COMPLETENESS = 0.86;
constexpr double MIN_KER_SCORE = 0.86;
constexpr uint32_t MAX_EVIDENCE_RECORDS_PER_WALLET = 10000;
constexpr uint32_t MAX_AUDIT_LOG_ENTRIES = 100000;

// ============================================================================
// BIOFIELD LIMITS (per FCC/ICNIRP)
// ============================================================================

constexpr double BIOFIELD_LOAD_CEILING_DEFAULT = 0.5;  // W/kg
constexpr double BIOFIELD_LOAD_CEILING_MAX = 4.0;      // W/kg (absolute max)
constexpr const char* NEUROCLASS_HUMAN_CORTEX = "human_cortex_v1";
constexpr const char* NEUROCLASS_HUMAN_PNS = "human_PNS";

// ============================================================================
// CORRIDOR TYPES
// ============================================================================

constexpr const char* CORRIDOR_REHAB_NEUROASSIST = "rehab_neuroassist";
constexpr const char* CORRIDOR_PUBLIC_PLAZA_AR = "public_plaza_AR";
constexpr const char* CORRIDOR_ASSISTIVE_RESEARCH = "assistive_rehab_research";
constexpr const char* CORRIDOR_CONSCIOUSNESS_PRESERVATION = "consciousness_preservation";

// ============================================================================
// EVIDENCE TYPES
// ============================================================================

constexpr const char* EVIDENCE_TYPE_HEALTH = "health";
constexpr const char* EVIDENCE_TYPE_ECO = "eco";
constexpr const char* EVIDENCE_TYPE_POLICY = "policy";
constexpr const char* EVIDENCE_TYPE_MISSION = "mission";
constexpr const char* EVIDENCE_TYPE_AUDIT = "audit";

// ============================================================================
// WALLET STATUSES
// ============================================================================

constexpr const char* WALLET_STATUS_ACTIVE = "active";
constexpr const char* WALLET_STATUS_SUSPENDED = "suspended";
constexpr const char* WALLET_STATUS_ARCHIVED = "archived";
constexpr const char* WALLET_STATUS_PRESERVED = "preserved";

// ============================================================================
// AUDIT STATUSES
// ============================================================================

constexpr const char* AUDIT_STATUS_PENDING = "pending";
constexpr const char* AUDIT_STATUS_COMPLIANT = "compliant";
constexpr const char* AUDIT_STATUS_NON_COMPLIANT = "non_compliant";
constexpr const char* AUDIT_STATUS_UNDER_REVIEW = "under_review";

// ============================================================================
// PROHIBITED ACTIONS (Neurorights)
// ============================================================================

constexpr const char* PROHIBITED_COVERT_CONTROL = "covert_neuromorphic_control";
constexpr const char* PROHIBITED_DEATH_NETWORK = "death_network_sabotage";
constexpr const char* PROHIBITED_DISCRIMINATORY = "discriminatory_corridor_access";
constexpr const char* PROHIBITED_UNCONSENTED = "unconsented_biophysical_data_access";
constexpr const char* PROHIBITED_DOWNGRADE = "downgrade_of_augmentation_rights";
constexpr const char* PROHIBITED_EXCLUSION = "exclusion_based_on_integration_type";

// ============================================================================
// TIME CONSTANTS
// ============================================================================

constexpr uint64_t MS_PER_SECOND = 1000;
constexpr uint64_t MS_PER_MINUTE = 60000;
constexpr uint64_t MS_PER_HOUR = 3600000;
constexpr uint64_t MS_PER_DAY = 86400000;

// ============================================================================
// CRYPTO CONSTANTS
// ============================================================================

constexpr size_t SHA256_DIGEST_LENGTH = 32;
constexpr size_t ED25519_SIGNATURE_LENGTH = 64;
constexpr size_t ED25519_PUBLIC_KEY_LENGTH = 32;
constexpr size_t ED25519_PRIVATE_KEY_LENGTH = 64;

// ============================================================================
// LOGGING
// ============================================================================

constexpr const char* LOG_TARGET_EVIDENCE = "aletheion.evidence";
constexpr const char* LOG_TARGET_NEURORIGHTS = "aletheion.neurorights";
constexpr const char* LOG_TARGET_SAFETY = "aletheion.safety";
constexpr const char* LOG_TARGET_CONSCIOUSNESS = "aletheion.consciousness";

} // namespace aletheion

#endif // ALETHEION_CONSTANTS_HPP
