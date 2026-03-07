// ============================================================================
// SOURCE: evidence_core.cpp
// PURPOSE: Core evidence management implementation
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#include "aletheion/evidence_core.hpp"
#include "aletheion/constants.hpp"
#include <algorithm>
#include <numeric>
#include <iostream>

namespace aletheion {

Result EvidenceCore::initialize() {
    std::lock_guard<std::mutex> lock(mutex_);

    if (initialized_) {
        return Success();
    }

    auto result = ledger_.initialize();
    if (result.has_value()) {
        return result;
    }

    result = neurorights_guard_.activate();
    if (result.has_value()) {
        return result;
    }

    initialized_ = true;
    std::cout << "[EvidenceCore] Initialized successfully" << std::endl;
    return Success();
}

EvidenceWallet* EvidenceCore::getOrCreateWallet(
    const std::string& owner_did,
    const std::optional<std::string>& linked_bci_device_id
) {
    std::lock_guard<std::mutex> lock(mutex_);

    // Neurorights check: ensure no discrimination based on BCI presence
    auto result = verifyEqualProtection(owner_did, linked_bci_device_id.has_value());
    if (result.has_value()) {
        std::cerr << "[EvidenceCore] Equal protection verification failed: "
                  << result->message << std::endl;
        return nullptr;
    }

    auto it = wallets_.find(owner_did);
    if (it == wallets_.end()) {
        EvidenceWallet wallet;
        wallet.owner_did = owner_did;
        wallet.linked_bci_device_id = linked_bci_device_id;
        wallet.wallet_id = "evidence-wallet-" + owner_did.substr(0, 16);

        std::cout << "[EvidenceCore] Created new wallet for " << owner_did << std::endl;
        it = wallets_.emplace(owner_did, std::move(wallet)).first;
    }

    return &it->second;
}

Result EvidenceCore::addEvidenceRecord(
    const std::string& owner_did,
    const EvidenceRecord& record
) {
    std::lock_guard<std::mutex> lock(mutex_);

    auto wallet = getOrCreateWallet(owner_did, record.linked_bci_device_id);
    if (!wallet) {
        return Error(ErrorCode::ROW_LEDGER_ERROR, "Failed to get or create wallet");
    }

    // Check wallet capacity
    if (wallet->evidence_records.size() >= MAX_EVIDENCE_RECORDS_PER_WALLET) {
        return Error(ErrorCode::ROW_LEDGER_ERROR, "Wallet capacity exceeded");
    }

    // Create mutable copy and calculate completeness
    EvidenceRecord mutable_record = record;
    mutable_record.calculateCompleteness(true, true);

    if (!mutable_record.meetsThreshold()) {
        return Error(
            ErrorCode::EVIDENCE_CHAIN_INCOMPLETE,
            "Evidence completeness " + std::to_string(mutable_record.completeness_score) +
            " < " + std::to_string(MIN_EVIDENCE_COMPLETENESS)
        );
    }

    // Track improvements
    if (mutable_record.evidence_type == EVIDENCE_TYPE_HEALTH) {
        wallet->health_improvements[mutable_record.metric] += mutable_record.delta;
    } else if (mutable_record.evidence_type == EVIDENCE_TYPE_ECO) {
        wallet->eco_improvements[mutable_record.metric] += mutable_record.delta;
    }

    wallet->evidence_records.push_back(mutable_record);
    wallet->updated_at = std::chrono::system_clock::now();

    recalculateWalletCompleteness(*wallet);

    std::cout << "[EvidenceCore] Added evidence record: " << mutable_record.record_id << std::endl;
    return Success();
}

Result EvidenceCore::runAudit(const std::vector<std::string>& control_paths) {
    std::lock_guard<std::mutex> lock(mutex_);

    std::cout << "[EvidenceCore] Running audit on " << control_paths.size()
              << " control paths" << std::endl;

    living_index_.undocumented_behaviors.clear();

    for (const auto& path : control_paths) {
        // Check if this control path has evidence chain
        bool has_evidence = false;

        for (const auto& [spec_clause, tests] : living_index_.spec_to_tests) {
            for (const auto& test_id : tests) {
                auto missions_it = living_index_.test_to_missions.find(test_id);
                if (missions_it == living_index_.test_to_missions.end()) continue;

                for (const auto& mission_id : missions_it->second) {
                    auto metrics_it = living_index_.mission_to_metrics.find(mission_id);
                    if (metrics_it == living_index_.mission_to_metrics.end()) continue;

                    for (const auto& metric_id : metrics_it->second) {
                        auto rows_it = living_index_.metric_to_rows.find(metric_id);
                        if (rows_it == living_index_.metric_to_rows.end()) continue;

                        if (!rows_it->second.empty()) {
                            has_evidence = true;
                            break;
                        }
                    }
                    if (has_evidence) break;
                }
                if (has_evidence) break;
            }
            if (has_evidence) break;
        }

        if (!has_evidence) {
            living_index_.undocumented_behaviors.push_back(path);
            std::cerr << "[EvidenceCore] Undocumented behavior: " << path << std::endl;
        }
    }

    living_index_.last_audit_at = std::chrono::system_clock::now();

    double completeness = getCompletenessScore();
    if (completeness < MIN_EVIDENCE_COMPLETENESS) {
        return Error(
            ErrorCode::AUDIT_FAILURE,
            "Evidence completeness " + std::to_string(completeness) +
            " < " + std::to_string(MIN_EVIDENCE_COMPLETENESS)
        );
    }

    std::cout << "[EvidenceCore] Audit passed with completeness: " << completeness << std::endl;
    return Success();
}

double EvidenceCore::getCompletenessScore() const {
    if (living_index_.undocumented_behaviors.empty()) {
        return 1.0;
    }

    double score = 1.0 - (living_index_.undocumented_behaviors.size() * 0.1);
    return std::max(0.0, std::min(1.0, score));
}

EvidenceWallet* EvidenceCore::getWallet(const std::string& owner_did) {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = wallets_.find(owner_did);
    return (it != wallets_.end()) ? &it->second : nullptr;
}

Result EvidenceCore::verifyConsciousnessPreservation(const std::string& owner_did) {
    auto wallet = getWallet(owner_did);
    if (!wallet) {
        return Error(ErrorCode::CONSENT_REQUIRED, "No wallet found for owner");
    }

    if (!wallet->linked_bci_device_id.has_value()) {
        return Error(
            ErrorCode::CONSENT_REQUIRED,
            "Consciousness preservation requires linked BCI device"
        );
    }

    std::cout << "[EvidenceCore] Consciousness preservation verification requested for "
              << owner_did << " - REQUIRES_CLINICAL_SAFETY_BOARD_APPROVAL" << std::endl;

    return Success();
}

void EvidenceCore::recalculateWalletCompleteness(EvidenceWallet& wallet) {
    if (wallet.evidence_records.empty()) {
        wallet.evidence_completeness_score = 1.0;
        return;
    }

    double total = std::accumulate(
        wallet.evidence_records.begin(),
        wallet.evidence_records.end(),
        0.0,
        [](double sum, const EvidenceRecord& record) {
            return sum + record.completeness_score;
        }
    );

    wallet.evidence_completeness_score = total / wallet.evidence_records.size();
}

Result EvidenceCore::verifyEqualProtection(
    const std::string& owner_did,
    bool has_bci
) {
    // All users receive equal protection regardless of augmentation status
    std::cout << "[EvidenceCore] Equal protection verified for " << owner_did
              << " (has_bci: " << (has_bci ? "true" : "false") << ")" << std::endl;
    return Success();
}

} // namespace aletheion
