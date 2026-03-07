// ============================================================================
// HEADER: consciousness_preservation.hpp
// PURPOSE: Consciousness preservation management and safety checks
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_CONSCIOUSNESS_PRESERVATION_HPP
#define ALETHEION_CONSCIOUSNESS_PRESERVATION_HPP

#include "types.hpp"
#include "constants.hpp"
#include <string>
#include <map>
#include <mutex>
#include <vector>

namespace aletheion {

/**
 * @brief Consciousness Preservation Manager for handling preservation requests
 */
class ConsciousnessPreservationManager {
public:
    /**
     * @brief Initialize the preservation manager
     * @return Result indicating success or error
     */
    Result initialize();

    /**
     * @brief Request consciousness preservation
     * @param owner_did Owner's DID
     * @param bci_device_id BCI device ID
     * @return Result indicating success or error
     */
    Result requestPreservation(
        const std::string& owner_did,
        const std::string& bci_device_id
    );

    /**
     * @brief Check preservation request status
     * @param owner_did Owner's DID
     * @return std::string Status ("pending", "approved", "rejected", "completed")
     */
    std::string getRequestStatus(const std::string& owner_did) const;

    /**
     * @brief Verify preservation eligibility
     * @param owner_did Owner's DID
     * @return Result indicating eligibility
     */
    Result verifyEligibility(const std::string& owner_did);

    /**
     * @brief Store preservation data (encrypted)
     * @param owner_did Owner's DID
     * @param data Encrypted preservation data
     * @return Result indicating success or error
     */
    Result storePreservationData(
        const std::string& owner_did,
        const std::vector<uint8_t>& data
    );

    /**
     * @brief Retrieve preservation data
     * @param owner_did Owner's DID
     * @return std::optional<std::vector<uint8_t>> Preservation data if available
     */
    std::optional<std::vector<uint8_t>> getPreservationData(
        const std::string& owner_did
    ) const;

    /**
     * @brief Get all pending requests
     * @return std::vector<std::string> List of owner DIDs with pending requests
     */
    std::vector<std::string> getPendingRequests() const;

    /**
     * @brief Approve preservation request (requires Clinical Safety Board)
     * @param owner_did Owner's DID
     * @param approver_did Approver's DID (Clinical Safety Board)
     * @return Result indicating success or error
     */
    Result approveRequest(
        const std::string& owner_did,
        const std::string& approver_did
    );

    /**
     * @brief Reject preservation request
     * @param owner_did Owner's DID
     * @param reason Rejection reason
     * @return Result indicating success or error
     */
    Result rejectRequest(const std::string& owner_did, const std::string& reason);

private:
    struct PreservationRequest {
        std::string owner_did;
        std::string bci_device_id;
        std::string status;  // "pending", "approved", "rejected", "completed"
        Timestamp request_timestamp;
        std::optional<Timestamp> approval_timestamp;
        std::optional<std::string> approver_did;
        std::optional<std::string> rejection_reason;
        std::optional<std::vector<uint8_t>> preservation_data;
    };

    std::map<std::string, PreservationRequest> requests_;
    mutable std::mutex mutex_;
};

} // namespace aletheion

#endif // ALETHEION_CONSCIOUSNESS_PRESERVATION_HPP
