// ============================================================================
// HEADER: types.hpp
// PURPOSE: Type definitions for Aletheion Edge Core
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_TYPES_HPP
#define ALETHEION_TYPES_HPP

#include <string>
#include <vector>
#include <map>
#include <optional>
#include <chrono>
#include <cstdint>
#include "constants.hpp"

namespace aletheion {

// ============================================================================
// TIME TYPE
// ============================================================================

using Timestamp = std::chrono::system_clock::time_point;

inline std::string timestamp_to_iso8601(Timestamp ts) {
    auto time = std::chrono::system_clock::to_time_t(ts);
    std::string result = std::ctime(&time);
    if (!result.empty() && result.back() == '\n') {
        result.pop_back();
    }
    return result;
}

inline Timestamp timestamp_from_iso8601(const std::string& iso8601) {
    std::tm tm = {};
    std::istringstream ss(iso8601);
    ss >> std::get_time(&tm, "%Y-%m-%dT%H:%M:%SZ");
    return std::chrono::system_clock::from_time_t(std::mktime(&tm));
}

// ============================================================================
// EVIDENCE RECORD
// ============================================================================

struct EvidenceRecord {
    std::string record_id;
    std::string row_ref;
    std::string evidence_type;
    std::string metric;
    double delta;
    std::string unit;
    Timestamp timestamp;
    std::string owner_did;
    std::string corridor;
    double completeness_score;
    std::optional<std::string> linked_bci_device_id;
    bool consciousness_preservation_relevant;

    EvidenceRecord()
        : delta(0.0)
        , completeness_score(0.0)
        , consciousness_preservation_relevant(false)
        , timestamp(std::chrono::system_clock::now())
    {}
};

// ============================================================================
// EVIDENCE WALLET
// ============================================================================

struct EvidenceWallet {
    std::string wallet_id;
    std::string owner_did;
    std::optional<std::string> linked_bci_device_id;
    std::vector<EvidenceRecord> evidence_records;
    std::map<std::string, double> health_improvements;
    std::map<std::string, double> eco_improvements;
    std::vector<std::string> care_access_providers;
    std::optional<std::vector<uint8_t>> consciousness_preservation_data;
    std::string wallet_status;
    Timestamp created_at;
    Timestamp updated_at;
    double evidence_completeness_score;

    EvidenceWallet()
        : evidence_completeness_score(1.0)
        , wallet_status(WALLET_STATUS_ACTIVE)
        , created_at(std::chrono::system_clock::now())
        , updated_at(std::chrono::system_clock::now())
    {}
};

// ============================================================================
// BCI CLINICAL AUGMENTATION
// ============================================================================

struct BCIClinicalAugmentation {
    std::string device_id;
    std::string device_model;
    std::string species_neuroclass;
    double biofield_load_ceiling;
    std::string consent_profile;
    std::string clinical_context;
    std::string owner_did;
    std::string safety_kernel_ref;
    std::string neurorights_policy;
    bool consciousness_preservation_enabled;
    std::optional<std::string> evidence_wallet_ref;
    Timestamp created_at;
    Timestamp last_audit_at;
    std::string audit_status;

    BCIClinicalAugmentation()
        : biofield_load_ceiling(BIOFIELD_LOAD_CEILING_DEFAULT)
        , consciousness_preservation_enabled(false)
        , created_at(std::chrono::system_clock::now())
        , last_audit_at(std::chrono::system_clock::now())
        , audit_status(AUDIT_STATUS_PENDING)
    {}
};

// ============================================================================
// ROW ENTRY
// ============================================================================

struct RowSignature {
    std::vector<uint8_t> signature_bytes;
    std::vector<uint8_t> public_key;
    std::string signer_did;
    Timestamp signed_at;
    std::string algorithm;

    RowSignature() : algorithm("Ed25519") {}
};

struct RowEntry {
    std::string entry_id;
    std::string previous_hash;
    std::string entry_hash;
    std::string entry_type;
    std::string data;
    std::string owner_did;
    std::string corridor;
    double ker_score;
    Timestamp timestamp;
    RowSignature signature;
    std::optional<uint32_t> multisig_threshold;
    std::vector<RowSignature> multisig_signatures;

    RowEntry() : ker_score(0.0), timestamp(std::chrono::system_clock::now()) {}
};

// ============================================================================
// LIVING INDEX
// ============================================================================

struct LivingIndex {
    std::string index_id;
    std::map<std::string, std::vector<std::string>> spec_to_tests;
    std::map<std::string, std::vector<std::string>> test_to_missions;
    std::map<std::string, std::vector<std::string>> mission_to_metrics;
    std::map<std::string, std::vector<std::string>> metric_to_rows;
    Timestamp created_at;
    Timestamp last_audit_at;
    std::vector<std::string> undocumented_behaviors;

    LivingIndex()
        : created_at(std::chrono::system_clock::now())
        , last_audit_at(std::chrono::system_clock::now())
    {}
};

// ============================================================================
// NEURORIGHTS POLICY
// ============================================================================

struct NeurorightsPolicy {
    std::string version;
    std::vector<std::string> principles;
    std::vector<std::string> prohibited_actions;
    std::vector<std::string> required_safeguards;

    NeurorightsPolicy() : version(NEURORIGHTS_POLICY) {}
};

// ============================================================================
// ERROR TYPES
// ============================================================================

enum class ErrorCode {
    SUCCESS = 0,
    NEURORIGHTS_VIOLATION,
    EVIDENCE_CHAIN_INCOMPLETE,
    ROW_LEDGER_ERROR,
    SAFETY_KERNEL_VIOLATION,
    CONSENT_REQUIRED,
    BIOFIELD_LOAD_EXCEEDED,
    DISCRIMINATORY_ACTION,
    AUDIT_FAILURE,
    CRYPTO_ERROR,
    CONFIG_ERROR
};

struct AletheionError {
    ErrorCode code;
    std::string message;

    AletheionError(ErrorCode c, const std::string& msg) : code(c), message(msg) {}
};

using Result = std::optional<AletheionError>;

inline Result Success() { return std::nullopt; }
inline Result Error(ErrorCode code, const std::string& msg) {
    return AletheionError(code, msg);
}

} // namespace aletheion

#endif // ALETHEION_TYPES_HPP
