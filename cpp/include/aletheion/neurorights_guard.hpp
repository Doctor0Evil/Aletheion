// ============================================================================
// HEADER: neurorights_guard.hpp
// PURPOSE: Neurorights protection and safety kernel enforcement
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_NEURORIGHTS_GUARD_HPP
#define ALETHEION_NEURORIGHTS_GUARD_HPP

#include "types.hpp"
#include "constants.hpp"
#include <string>
#include <map>
#include <vector>
#include <mutex>

namespace aletheion {

/**
 * @brief Neurorights Guard for enforcing neurorights and safety policies
 */
class NeurorightsGuard {
public:
    /**
     * @brief Activate the neurorights guard
     * @return Result indicating success or error
     */
    Result activate();

    /**
     * @brief Verify equal protection for an owner
     * @param owner_did Owner's DID
     * @param has_bci Whether owner has BCI
     * @return Result indicating success or error
     */
    Result verifyEqualProtection(const std::string& owner_did, bool has_bci);

    /**
     * @brief Check for discriminatory actions
     * @param action Action description
     * @param target_did Target DID
     * @return Result indicating success or error
     */
    Result checkDiscrimination(const std::string& action, const std::string& target_did);

    /**
     * @brief Verify an action is not prohibited
     * @param action Action to verify
     * @return true if action is allowed
     */
    bool isActionAllowed(const std::string& action) const;

    /**
     * @brief Register consent for an owner
     * @param owner_did Owner's DID
     * @return Result indicating success or error
     */
    Result registerConsent(const std::string& owner_did);

    /**
     * @brief Revoke consent for an owner
     * @param owner_did Owner's DID
     * @return Result indicating success or error
     */
    Result revokeConsent(const std::string& owner_did);

    /**
     * @brief Check if owner has consent
     * @param owner_did Owner's DID
     * @return true if consent is registered
     */
    bool hasConsent(const std::string& owner_did) const;

    /**
     * @brief Verify biofield load ceiling
     * @param neuroclass Neuroclass type
     * @param load Current load
     * @return true if load is within limits
     */
    bool verifyBiofieldLoad(const std::string& neuroclass, double load) const;

    /**
     * @brief Get violation count
     * @return uint32_t Number of violations
     */
    uint32_t getViolationCount() const { return violation_count_; }

    /**
     * @brief Report a neurorights violation
     * @param violation_type Type of violation
     * @param details Violation details
     */
    void reportViolation(const std::string& violation_type, const std::string& details);

    /**
     * @brief Get policy reference
     * @return const NeurorightsPolicy& Reference to policy
     */
    const NeurorightsPolicy& getPolicy() const { return policy_; }

private:
    /**
     * @brief Log an audit entry
     * @param message Audit message
     */
    void logAudit(const std::string& message);

    // Member variables
    NeurorightsPolicy policy_;
    std::map<std::string, bool> consent_profiles_;
    std::vector<std::string> audit_log_;
    uint32_t violation_count_;
    mutable std::mutex mutex_;
};

} // namespace aletheion

#endif // ALETHEION_NEURORIGHTS_GUARD_HPP
