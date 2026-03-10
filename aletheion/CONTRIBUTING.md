# Contributing to Aletheion

Thank you for contributing to Aletheion, a real-world smart city infrastructure for Phoenix, Arizona. As an augmented-citizen, your contributions must adhere to strict sovereignty, security, and density standards.

## 1. Supported Languages
- **Core:** Rust, C++
- **Contracts:** ALN (Agent Liability Network)
- **Edge/Installer:** Lua
- **Interface:** Kotlin (Android), JavaScript (Web)
- **Documentation:** Markdown

**Blacklisted:** Python, SHA-256, BLAKE, KECCAK, Centralized Cloud APIs.

## 2. Code Density Standards (Updated 2026-01-15)
- **Logic Density:** ≥80% of lines must be executable logic or declarations.
- **Comment Density:** **Lowered.** Comments should only direct file generation, explain safety constraints, or denote sovereignty checks. Avoid explaining syntax.
  - **Bad:** `// Loop through assets`
  - **Good:** `// ERM: Optimize | Sovereignty: Indigenous Priority`
- **File Size:** 200-500 lines per file (maximize cohesion).
- **Uniqueness:** No duplicate files. Hash check required before commit.

## 3. Sovereignty & Rights
- **Indigenous Data:** Must include `is_indigenous_sovereign` flag and community consent checks.
- **Neurorights:** Biosignal data must be encrypted locally; consent revocation must purge data.
- **Ecological:** Environmental controls must enforce Lyapunov stability ($V_t$ non-increasing).

## 4. Workflow Checkpoints
Before submitting a Pull Request:
1. **CP-001:** File Unique Hash (No duplicates).
2. **CP-002:** Compliance Check (Blacklist, Sovereignty).
3. **CP-003:** Density Check (≥80% logic).
4. **CP-004:** Test Coverage (≥85% for core modules).
5. **CP-005:** Offline Capability (No cloud-only dependencies).

## 5. Directory Structure
Place files in the deepest possible directory (e.g., `core/environmental/water/mar/vault/src/`). Avoid root-level clutter.

## 6. Testing
- **Unit Tests:** Required for all Rust/C++ modules.
- **Integration Tests:** Required for ALN contracts.
- **Offline Tests:** Verify functionality without network access.

## 7. Submitting Changes
1. Fork the repository.
2. Create a branch (`feature/your-subsystem`).
3. Commit with meaningful messages (`feat: add heat throttle to energy mesh`).
4. Submit Pull Request for augmented-citizen review.

## 8. Community Review
- **Indigenous Liaison:** Reviews all land-use and water-allocation code.
- **Security Team:** Reviews all encryption and access control code.
- **Ecology Team:** Reviews all environmental impact code.

---
*By contributing, you agree to the BioticTreaty License and Aletheion Production Standards.*
