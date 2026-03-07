// ============================================================================
// HEADER: evidence_core.hpp
// PURPOSE: Core evidence management and living index functionality
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_EVIDENCE_CORE_HPP
#define ALETHEION_EVIDENCE_CORE_HPP

#include "types.hpp"
#include "constants.hpp"
#include "row_ledger.hpp"
#include "neurorights_guard.hpp"
#include <string>
#include <vector>
#include <map>
#include <memory>
#include <mutex>

namespace aletheion {

/**
 * @brief Main Evidence Core class for managing evidence wallets and living index
 */
class EvidenceCore {
public:
    /**
     * @brief Construct a new Evidence Core object
     * @return Result indicating success or error
     */
    Result initialize();

    /**
     * @brief Get or create an evidence wallet for an owner
     * @param owner_did Owner's Decentralized Identifier
     * @param linked_bci_device_id Optional linked BCI device ID
     * @return Pointer to EvidenceWallet or nullptr on error
     */
    EvidenceWallet* getOrCreateWallet(
        const std::string& owner_did,
        const std::optional<std::string>& linked_bci_device_id = std::nullopt
    );

    /**
     * @brief Add an evidence record to a wallet
     * @param owner_did Owner's DID
     * @param record Evidence record to add
     * @return Result indicating success or error
     */
    Result addEvidenceRecord(
        const std::string& owner_did,
        const EvidenceRecord& record
    );

    /**
     * @brief Run audit for undocumented behaviors
     * @param control_paths List of control paths to audit
     * @return Result indicating success or error
     */
    Result runAudit(const std::vector<std::string>& control_paths);

    /**
     * @brief Get evidence completeness score
     * @return double Completeness score (0.0 - 1.0)
     */
    double getCompletenessScore() const;

    /**
     * @brief Get wallet summary for an owner
     * @param owner_did Owner's DID
     * @return Pointer to EvidenceWallet or nullptr
     */
    EvidenceWallet* getWallet(const std::string& owner_did);

    /**
     * @brief Verify consciousness preservation eligibility
     * @param owner_did Owner's DID
     * @return Result indicating eligibility
     */
    Result verifyConsciousnessPreservation(const std::string& owner_did);

    /**
     * @brief Get living index reference
     * @return const LivingIndex& Reference to living index
     */
    const LivingIndex& getLivingIndex() const { return living_index_; }

    /**
     * @brief Get all wallets (for admin/audit)
     * @return std::map<std::string, EvidenceWallet>& Reference to wallets map
     */
    std::map<std::string, EvidenceWallet>& getWallets() { return wallets_; }

private:
    /**
     * @brief Recalculate wallet completeness score
     * @param wallet Reference to wallet
     */
    void recalculateWalletCompleteness(EvidenceWallet& wallet);

    /**
     * @brief Verify equal protection (neurorights)
     * @param owner_did Owner's DID
     * @param has_bci Whether owner has BCI
     * @return Result indicating success or error
     */
    Result verifyEqualProtection(
        const std::string& owner_did,
        bool has_bci
    );

    // Member variables
    std::map<std::string, EvidenceWallet> wallets_;
    LivingIndex living_index_;
    RowLedger ledger_;
    NeurorightsGuard neurorights_guard_;
    mutable std::mutex mutex_;
    bool initialized_;
};

} // namespace aletheion

#endif // ALETHEION_EVIDENCE_CORE_HPP
