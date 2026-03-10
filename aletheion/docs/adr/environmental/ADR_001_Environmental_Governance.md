# ADR 001: Environmental Governance & Risk Engine Architecture

* **Status:** Accepted
* **Date:** 2026-01-15
* **Deciders:** Aletheion Core Architecture Team, Augmented Citizens
* **Context:** Phoenix, Arizona (Sonoran Desert)
* **License:** BioticTreaty_Compliant_AGPLv3

## Problem Statement
Building a smart city in Phoenix requires managing extreme heat, water scarcity, and dust storms while respecting Indigenous land rights and ecological stability. Traditional IoT systems lack formal safety guarantees and often violate sovereignty or environmental corridors.

## Decision Drivers
1. **Safety:** Need for hard constraints (no-corridor-no-build).
2. **Sovereignty:** Akimel O'odham and Piipaash water rights must be computationally enforced.
3. **Resilience:** Systems must operate offline during network outages (common in storms).
4. **Ecology:** Lyapunov stability ensures actions do not degrade the environment over time.

## Accepted Solution
Implement a layered governance architecture:
1. **Risk Engine (Rust):** Calculates normalized risk coordinates ($r_x$) and Lyapunov residuals ($V_t$).
2. **Contracts (ALN):** Enforces "derate-or-stop" logic based on risk thresholds.
3. **Edge Control (Lua):** Localized mitigation for dust and heat without cloud dependency.

## Eco-Impact Assessment
| Metric | Score | Justification |
| :--- | :--- | :--- |
| **Knowledge (K)** | 0.95 | Formalizes environmental safety into code. |
| **Eco-Impact (E)** | 0.90 | Prevents aquifer depletion and habitat damage. |
| **Risk (R)** | 0.10 | Hard stops prevent catastrophic failure. |

## Compliance & Rights
* **Indigenous Rights:** Contracts include `indigenous_water_claim` boolean hard-stop.
* **Neurorights:** Citizen alerts are opt-in via Kotlin interface (not forced).
* **BioticTreaties:** Soil moisture and native flora health are primary risk coordinates.

## Implementation Plan
1. Deploy Risk Engine to all MAR vault controllers.
2. Install Lua edge nodes on public buildings for dust monitoring.
3. Integrate ALN contracts into city-wide deployment pipeline.

## References
* Aletheion Rule (R):Hypotheticals, fictionals... are not-allowed.
* Aletheion Rule (L): Supported-language set: ALN, Lua, Rust...
* Phoenix 2025 Monsoon Data (2.71" rainfall avg).
* Pure Water Phoenix Model (97-99% reclamation).
