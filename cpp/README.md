# Aletheion C++ Edge Computing Core

**Version:** 1.0.0  
**License:** Apache-2.0 WITH Neurorights-Extension  
**Owner DID:** `did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7`

## Overview

This C++ edge computing core provides high-performance evidence processing for Aletheion GOD-City edge nodes. It is optimized for low-latency, resource-constrained environments while maintaining full neurorights compliance and evidence integrity.

## Features

- **High-Performance Evidence Processing:** Optimized for edge deployment
- **Immutable ROW Ledger:** Hash-chained, DID-anchored entries
- **Neurorights Guard:** Safety kernel enforcement for augmented citizens
- **Biofield Monitor:** Real-time biofield load tracking and enforcement
- **Consciousness Preservation:** Secure preservation request management
- **Thread-Safe:** All operations protected by mutexes

## Building

```bash
# Create build directory
mkdir build && cd build

# Configure with CMake
cmake .. -DCMAKE_BUILD_TYPE=Release

# Build
make -j$(nproc)

# Run tests
ctest --output-on-failure

# Install
sudo make install
```

## Dependencies

- CMake 3.20+
- C++17 compiler (GCC 9+, Clang 10+)
- OpenSSL 1.1.1+
- nlohmann_json 3.11.0+

## Usage

```cpp
#include <aletheion/evidence_core.hpp>

using namespace aletheion;

int main() {
    EvidenceCore core;
    core.initialize();

    EvidenceRecord record;
    record.evidence_type = EVIDENCE_TYPE_HEALTH;
    record.metric = "respiratory_improvement";
    record.delta = 15.5;
    record.owner_did = OWNER_DID;
    record.corridor = CORRIDOR_REHAB_NEUROASSIST;

    core.addEvidenceRecord(OWNER_DID, record);

    double completeness = core.getCompletenessScore();
    std::cout << "Evidence completeness: " << completeness << std::endl;

    return 0;
}
```

## Compliance

This implementation enforces:

- **GDPR:** Data minimization and explicit consent
- **HIPAA:** Encrypted health data storage
- **EU AI Act 2024:** High-risk AI system transparency
- **Neurorights Charter v1:** No covert control, equal protection

## Neurorights Statement

> All residents, regardless of race, disability, or augmentation status, are represented in the same DID and consent systems. Organic BCI and neuroprosthetic interfaces are modeled as `BCIClinicalAugmentation` to ensure medical-grade safeguards and neurorights protection for all augmented users. Exclusion from general city-gadget catalogs reflects risk-based classification (implant vs. wearable) and regulatory requirements, not any judgement about the user's identity or worth.

## Security

- **No unsafe code:** Strict compiler warnings enabled (`-Werror`)
- **Thread-safe:** All shared state protected by mutexes
- **Immutable ledger:** Hash-chained entries with cryptographic signatures
- **Equal protection:** No discrimination based on augmentation status

## Contact

- **Security:** security@aletheion.city
- **Neurorights Ombud:** ombud@aletheion.city
- **Clinical Safety Board:** safety-board@aletheion.city
