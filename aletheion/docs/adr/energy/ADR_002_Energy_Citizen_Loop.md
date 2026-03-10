# ADR 002: Energy-Citizen Loop & Resilience Architecture

* **Status:** Accepted
* **Date:** 2026-01-15
* **Deciders:** Aletheion Core Architecture Team, Augmented Citizens
* **Context:** Phoenix, Arizona (Sonoran Desert)
* **License:** BioticTreaty_Compliant_AGPLv3

## Problem Statement
Phoenix faces extreme heat (120°F+) and monsoon-related power outages. Traditional grids fail during peak heat when AC demand spikes. Citizens need real-time feedback to reduce load, and energy distribution must prioritize critical infrastructure and Indigenous sovereignty without relying on centralized cloud control.

## Decision Drivers
1. **Resilience:** Microgrids must island themselves during main grid failure.
2. **Sovereignty:** Akimel O'odham communities must have priority power access.
3. **Heat Safety:** Battery degradation must be prevented via thermal throttling.
4. **Neurorights:** Citizen alerts must be opt-in, not forced attention mechanisms.

## Accepted Solution
Implement a closed-loop Energy-Citizen system:
1. **Energy Mesh (Rust):** Localized P2P energy trading with thermal safety checks.
2. **Citizen Interface (Kotlin):** Opt-in alerts for load reduction during peak heat.
3. **Contracts (ALN):** Enforce sovereignty priority and thermal limits.

## Eco-Impact Assessment
| Metric | Score | Justification |
| :--- | :--- | :--- |
| **Knowledge (K)** | 0.95 | Formalizes energy sovereignty into code. |
| **Eco-Impact (E)** | 0.90 | Reduces peak load, prevents battery waste. |
| **Risk (R)** | 0.10 | Thermal isolation prevents fires/failures. |

## Compliance & Rights
* **Indigenous Rights:** `is_indigenous_site` flag triggers priority charging in ALN contract.
* **Neurorights:** Kotlin `NeuroConsent` class ensures notifications are user-controlled.
* **BioticTreaties:** Energy usage correlated with water pumping efficiency (Nexus).

## Implementation Plan
1. Deploy Rust Grid Controller to all solar+battery nodes.
2. Install Kotlin App on citizen devices (Opt-in).
3. Integrate ALN contracts into energy metering hardware.

## References
* Aletheion Rule (R):Hypotheticals, fictionals... are not-allowed.
* Aletheion Rule (L): Supported-language set: ALN, Lua, Rust...
* Phoenix 2025 Heat Data (31 days >110°F).
* APS Outage Reports (Monsoon 2025).
