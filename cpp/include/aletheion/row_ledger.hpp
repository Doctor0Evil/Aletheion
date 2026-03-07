// ============================================================================
// HEADER: row_ledger.hpp
// PURPOSE: Immutable ROW (Record-of-Work) ledger with DID anchoring
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_ROW_LEDGER_HPP
#define ALETHEION_ROW_LEDGER_HPP

#include "types.hpp"
#include "constants.hpp"
#include <string>
#include <vector>
#include <optional>
#include <mutex>
#include <fstream>

namespace aletheion {

/**
 * @brief Immutable ROW Ledger for evidence records
 */
class RowLedger {
public:
    /**
     * @brief Initialize the ledger
     * @param file_path Path to ledger file
     * @return Result indicating success or error
     */
    Result initialize(const std::string& file_path = "aletheion_ledger.row");

    /**
     * @brief Append a new entry to the ledger
     * @param entry Row entry to append
     * @return std::optional<std::string> Entry hash on success
     */
    std::optional<std::string> append(RowEntry& entry);

    /**
     * @brief Verify chain integrity
     * @return true if chain is valid
     */
    bool verifyChain() const;

    /**
     * @brief Get entry by hash
     * @param hash Entry hash
     * @return std::optional<const RowEntry&> Entry if found
     */
    std::optional<const RowEntry*> getEntry(const std::string& hash) const;

    /**
     * @brief Get all entries for a specific owner
     * @param owner_did Owner's DID
     * @return std::vector<const RowEntry*> List of entries
     */
    std::vector<const RowEntry*> getEntriesByOwner(const std::string& owner_did) const;

    /**
     * @brief Get all entries for a specific corridor
     * @param corridor Corridor name
     * @return std::vector<const RowEntry*> List of entries
     */
    std::vector<const RowEntry*> getEntriesByCorridor(const std::string& corridor) const;

    /**
     * @brief Get last hash in chain
     * @return std::string Last hash
     */
    const std::string& getLastHash() const { return last_hash_; }

    /**
     * @brief Get entry count
     * @return size_t Number of entries
     */
    size_t getEntryCount() const { return entries_.size(); }

private:
    /**
     * @brief Calculate SHA256 hash of data
     * @param data Data to hash
     * @return std::string Hex-encoded hash
     */
    std::string calculateHash(const std::string& data) const;

    /**
     * @brief Persist ledger to file
     * @return Result indicating success or error
     */
    Result persist() const;

    /**
     * @brief Load ledger from file
     * @return Result indicating success or error
     */
    Result load();

    // Member variables
    std::string file_path_;
    std::vector<RowEntry> entries_;
    std::string last_hash_;
    mutable std::mutex mutex_;
};

} // namespace aletheion

#endif // ALETHEION_ROW_LEDGER_HPP
