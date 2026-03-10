# STD_002: Code Production Protocol Update (Comment Density)

* **Status:** Accepted
* **Date:** 2026-01-15
* **Deciders:** Aletheion Core Architecture Team, Augmented Citizens
* **Context:** Phoenix, Arizona (Sonoran Desert)
* **License:** BioticTreaty_Compliant_AGPLv3
* **Supersedes:** STD_001 Section 2.2 (Comment Density)

## 1. Rationale
Initial standards (STD_001) required 20% comment density (1 comment per 5 logic lines). Production experience indicates this reduces logic density and clutters high-performance code. Comments should only exist to direct file generation, explain safety constraints, or denote sovereignty checks.

## 2. Updated Comment Density Standard
- **Previous:** ≥20% comment lines.
- **New:** **Sparse.** Comments only for:
  - **ERM/SMART Chain:** Denoting workflow stage (e.g., `// ERM: Optimize`).
  - **Sovereignty:** Denoting Indigenous rights checks (e.g., `// RIGHTS: Akimel_O'odham_Veto`).
  - **Safety:** Denoting hard constraints (e.g., `// CONSTRAINT: Max_Temp_50C`).
  - **Generation:** Directing automated file creation (e.g., `// GENERATE: MarVaultController`).
- **Prohibited:** Explaining syntax, redundant variable descriptions, decorative headers.

## 3. Example Comparison
```rust
// OLD (20% Density):
// This function calculates risk
// Input: val (f32)
// Output: risk (f32)
fn calc_risk(val: f32) -> f32 {
    // Multiply by 0.5
    let risk = val * 0.5;
    // Return risk
    return risk;
}

// NEW (Sparse):
// ERM: Model → Optimize | K=0.94 | R=0.12
fn calc_risk(val: f32) -> f32 { val * 0.5 } // Normalized risk coordinate
```

## 4. Logic Density
- **Requirement:** ≥80% of lines must be executable logic or declarations.
- **Enforcement:** CI/CD pipeline rejects files with excessive whitespace or comments.

## 5. Implementation
- **Effective:** Immediately for all new files (Batch 8+).
- **Legacy:** Existing files (Batch 1-7) remain unchanged (No Overwrites Rule).
- **Audit:** Quarterly review of comment-to-logic ratio in new submissions.

## 6. Compliance
- **Automated:** Linter checks for comment density ≤10%.
- **Manual:** Augmented Citizens review for safety/sovereignty comment adequacy.

## References
* Aletheion Rule (R): New-Functions, New-Syntax, New-Programming.
* Aletheion Rule (L): High-Density Codes, Syntax_Ladders.
* STD_001_Code_Production_Protocol.md (Original Standard).
