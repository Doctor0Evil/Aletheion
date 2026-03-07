// ============================================================================
// SOURCE: main.cpp
// PURPOSE: Edge node entry point for Aletheion Evidence Core
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#include "aletheion/evidence_core.hpp"
#include "aletheion/constants.hpp"
#include <iostream>
#include <csignal>
#include <atomic>

std::atomic<bool> g_running(true);

void signal_handler(int signum) {
    std::cout << "\n[Aletheion Edge Node] Received signal " << signum
              << ", shutting down..." << std::endl;
    g_running = false;
}

int main(int argc, char* argv[]) {
    // Setup signal handlers
    std::signal(SIGINT, signal_handler);
    std::signal(SIGTERM, signal_handler);

    std::cout << R"(
╔═══════════════════════════════════════════════════════════════╗
║           ALETHEION GOD-CITY EDGE NODE                        ║
║                                                               ║
║  Version: )" << aletheion::ALETHEION_VERSION << R"(
║  Owner DID: )" << aletheion::OWNER_DID << R"(
║  Safety Kernel: )" << aletheion::SAFETY_KERNEL_REF << R"(
║  Neurorights Policy: )" << aletheion::NEURORIGHTS_POLICY << R"(
║                                                               ║
║  Compliance: GDPR, HIPAA, EU AI Act 2024, Neurorights v1     ║
╚═══════════════════════════════════════════════════════════════╝
)" << std::endl;

    // Initialize Evidence Core
    aletheion::EvidenceCore core;
    auto result = core.initialize();

    if (result.has_value()) {
        std::cerr << "[Aletheion Edge Node] Initialization failed: "
                  << result->message << std::endl;
        return 1;
    }

    std::cout << "[Aletheion Edge Node] Running... (Press Ctrl+C to stop)" << std::endl;

    // Main loop
    while (g_running) {
        // In production, this would process incoming evidence records
        // and sync with the ROW ledger
        std::this_thread::sleep_for(std::chrono::seconds(1));

        // Periodic audit check
        static int audit_counter = 0;
        if (++audit_counter >= 60) {  // Every 60 seconds
            double completeness = core.getCompletenessScore();
            if (completeness < aletheion::MIN_EVIDENCE_COMPLETENESS) {
                std::cerr << "[Aletheion Edge Node] WARNING: Evidence completeness below threshold: "
                          << completeness << std::endl;
            }
            audit_counter = 0;
        }
    }

    std::cout << "[Aletheion Edge Node] Shutdown complete" << std::endl;
    return 0;
}
