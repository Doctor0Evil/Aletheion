# Aletheion Core - Rust Implementation

**Version:** 1.0.0  
**License:** Apache-2.0 WITH Neurorights-Extension  
**Owner DID:** `did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7`

## Overview

This is the production-ready Rust core for Aletheion GOD-City, providing:

- **Evidence Core:** Living index mapping spec clauses to ledger entries
- **ROW Ledger:** Immutable, DID-anchored record-of-work system
- **Neurorights Guard:** Safety kernel enforcement for augmented citizens

## Building

```bash
# Build release version
cargo build --release

# Build with consciousness preservation feature
cargo build --release --features consciousness_preservation

# Run all tests
cargo test --all

# Run with tracing enabled
RUST_LOG=aletheion_core=info cargo run --release
```

## Compliance

This implementation enforces:

- GDPR (General Data Protection Regulation)
- HIPAA (Health Insurance Portability and Accountability Act)
- EU AI Act 2024
- FCC Part 15 (RF exposure limits)
- Neurorights Charter v1

## Key Features

### Evidence Core

```rust
use aletheion_core::EvidenceCore;

let mut core = EvidenceCore::new()?;
let wallet = core.get_or_create_wallet(owner_did, Some(bci_device_id))?;
```

### ROW Ledger

```rust
use aletheion_core::RowLedger;

let mut ledger = RowLedger::initialize()?;
let hash = ledger.append(entry)?;
```

### Neurorights Guard

```rust
use aletheion_core::NeurorightsGuard;

let guard = NeurorightsGuard::activate()?;
guard.verify_equal_protection(owner_did, has_bci)?;
```

## Safety Guarantees

- **No unsafe code:** `#![deny(unsafe_code)]`
- **No unwrap:** `#![forbid(clippy::unwrap_used)]`
- **Immutable ledger:** Hash-chained, DID-anchored entries
- **Equal protection:** No discrimination based on augmentation status

## Contact

- **Security:** security@aletheion.city
- **Neurorights Ombud:** ombud@aletheion.city
```
