# Aletheion XR‑Grid & Seven‑Stage Spine — Rust Module Layout

This document defines a canonical Rust module layout for Aletheion’s XR‑grid fabric and the seven‑stage workflow spine (Sense → Normalize → Intent → Optimize/Simulate → Treaty → Actuate → Audit). It is designed so each crate can be compiled, tested, and extended independently while running on edge nodes and central orchestration tiers.[file:21][web:15]

---

## 1. Crate Map (Top Level)

All crates live under `infra/rust/` and are designed for `no_std`-capable builds at the edge where possible, with WASM targets for embedded XR‑nodes and native targets for city‑scale services.[file:18][web:31][web:32]

- `infra/rust/edge_xr_node/`
- `infra/rust/signal_intent/`
- `infra/rust/optimizer_sim/`
- `infra/rust/treaty_engine/`
- `infra/rust/actuation_bus/`
- `infra/rust/googolswarm_client/`
- `infra/rust/citizen_interface_adapter/`
- `infra/rust/spine_orchestrator/`

Each crate exposes a minimal, stable API so new workflows can be added without changing the core contracts.

---

## 2. Edge XR‑Node Runtime (`edge_xr_node`)

**Path:** `infra/rust/edge_xr_node/`

Purpose: Provide a Rust runtime and I/O abstraction for per‑block XR‑nodes with sensors, WASM micro‑actuators, local treaty caches, and mesh networking.[file:19][web:15][web:38]

Key modules:

- `src/node_runtime.rs`  
  - Async runtime bootstrapping (Tokio or `embassy` on embedded targets), task scheduling, and lifecycle management.[file:18][web:15]
- `src/sensors.rs`  
  - Traits for sensor drivers (air, water, soil, heat, waste, mobility, biosignals) and normalized reading types.[file:21]
- `src/actuators.rs`  
  - Traits for actuator drivers (valves, pumps, diverters, signals, drones, misters, bioprocessing units).
- `src/mesh.rs`  
  - Mesh networking abstraction (UDP/CoAP/MQTT over Wi‑Fi or sub‑GHz) for node‑to‑node coordination.[file:19][web:28]
- `src/treaty_cache.rs`  
  - Local cache of treaty bundles keyed by `birth_sign_id`, with signed manifest verification and hot‑reload.[file:18]
- `src/wasm_host.rs`  
  - WASM host runtime wrapper (e.g., WasmEdge/Wasmer) for sandboxed rules, using `wasm32-wasip1` builds for edge functions.[web:29][web:32]

Core types:

- `EdgeNodeId`, `GeoTileId`
- `SensorReading`, `SensorKind`
- `ActuatorCommand`, `ActuatorKind`
- `BirthSignRef` (opaque ID; types live in governance crates)
- `MeshEnvelope` (messages between nodes)

---

## 3. Signal & Intent Layer (`signal_intent`)

**Path:** `infra/rust/signal_intent/`

Purpose: Turn raw sensor readings into normalized signals and infrastructure intents (“divert water”, “route waste”, “cool zone”, “restore soil”).[file:21]

Modules:

- `src/normalize.rs`  
  - Deterministic, pure functions mapping raw readings into canonical vectors with quality flags.[file:18]
- `src/intent_types.rs`  
  - Enum and structs for intents, including domain tags and severity:
  - `InfrastructureIntent` (variants: `DivertWater`, `IncreaseCooling`, `RouteWasteToBioprocessor`, `SlowMobilityCorridor`, `TriggerEcologicalRestoration`, etc.).
- `src/intent_classifier.rs`  
  - Stateless classifiers mapping sequences of normalized signals to one or more `InfrastructureIntent` values with confidence scores.

Inputs/outputs:

- Input: `SensorReading` + `BirthSignRef`
- Output: `(NormalizedSignal, InfrastructureIntent)` for the optimizer.

---

## 4. Optimization & Simulation Engine (`optimizer_sim`)

**Path:** `infra/rust/optimizer_sim/`

Purpose: Run multi‑objective optimization and simulation for water, heat, waste, mobility, energy, soil, and microclimate, both locally (on XR‑nodes) and centrally.[file:20][file:21]

Modules:

- `src/state_model.rs`  
  - Shared resource graph models (canals, pipes, sewers, roads, trees, cool pavements, XR‑grid nodes).
- `src/objectives.rs`  
  - Objective functions for ecological impact, equity indices, energy cost, and treaty‑derived penalties.
- `src/solvers.rs`  
  - NSGA‑II / MOEA‑style front search and deterministic solvers, with traits so domain‑specific workflows (e.g., heat‑water‑tree) plug in.[file:20]
- `src/simulation.rs`  
  - Forward simulation of candidate scenarios (flows, temperatures, emissions, corridor connectivity) over short horizons.

Inputs/outputs:

- Input: `InfrastructureIntent`, `ActionContext` (without decision), state snapshots.
- Output: `ProposedActionPlan` objects capturing routes, schedules, and expected impacts.

---

## 5. Treaty Engine (`treaty_engine`)

**Path:** `infra/rust/treaty_engine/`

Purpose: Bind Rust to the ALN treaty grammar and Birth‑Signs, evaluating proposed actions against Indigenous rights, water compacts, EJ zones, BioticTreaties, neurorights, and micro‑treaties.[file:18][file:21][file:22]

Modules:

- `src/context.rs`  
  - Rust struct mirroring `ActionContext` from `specs/treaty_dsl/aletheion_treaty_os.aln`, plus helpers to construct from optimizer output.[file:18]
- `src/decision.rs`  
  - Rust equivalents of `TreatyModuleResult` and `TreatyDecision`.
- `src/modules/indigenous.rs`  
  - Evaluation of Indigenous water/land protocols and FPIC requirements.
- `src/modules/water_compact.rs`
- `src/modules/biotic.rs`
- `src/modules/ej.rs`
- `src/modules/neurorights.rs`
- `src/registry.rs`  
  - Registry of active treaty modules and composition logic for an overall decision (min‑decision rule, override handling).

Contract:

- Each module exposes `fn evaluate(ctx: ActionContext) -> TreatyModuleResult`, and the crate exposes `fn evaluate_all(ctx: ActionContext) -> TreatyDecision`, aligned with the ALN TreatyEngineContract schema.[file:18]

---

## 6. Actuation Bus (`actuation_bus`)

**Path:** `infra/rust/actuation_bus/`

Purpose: Provide a secure, rate‑limited, signed actuation layer between approved actions and physical devices (valves, pumps, lights, drones, traffic signals, waste diverters, etc.).[file:21][file:19]

Modules:

- `src/commands.rs`  
  - Typed actuation commands and batches, including metadata (`action_id`, `workflow_id`, `birth_sign_id`, safety envelopes).
- `src/drivers/`  
  - Pluggable drivers per device class and protocol (Modbus/TCP, OPC‑UA, MQTT, CAN bus).
- `src/rate_limit.rs`  
  - Per‑device, per‑zone, and per‑domain rate‑limits.
- `src/signing.rs`  
  - Command signing and verification hooks (integrated with Googolswarm identities).
- `src/reversibility.rs`  
  - Mechanisms to compute and issue compensating actions where physically possible.

Inputs/outputs:

- Input: `TreatyDecision` + `ProposedActionPlan`
- Output: `ActuatorCommand` streams, plus status events for the audit client.

---

## 7. Googolswarm Client (`googolswarm_client`)

**Path:** `infra/rust/googolswarm_client/`

Purpose: Append governed decision records (including Birth‑Signs and ALN norms) to the Googolswarm ledger with multi‑sig attestation and energy‑aware batching.[file:18][file:21]

Modules:

- `src/schema.rs`  
  - Rust structs matching the governed decision transaction schema (ALN provenance grammar) with fields for `birthSignId`, `alnNorms`, DIDs, and outcome.[file:21]
- `src/client.rs`  
  - Asynchronous append client with retry and backoff; supports batching to minimize energy/network use.
- `src/attestation.rs`  
  - Multi‑sig attestation workflows, including verification of module signatures and XR‑node signatures.

Inputs/outputs:

- Input: `TreatyDecision`, `ProposedActionPlan`, actuation results.
- Output: Ledger transaction IDs and receipts.

---

## 8. Citizen Interface Adapter (`citizen_interface_adapter`)

**Path:** `infra/rust/citizen_interface_adapter/`

Purpose: Transform internal decisions and logs into citizen‑readable explanations, consent states, and grievance handles.[file:18][file:22]

Modules:

- `src/explanation.rs`  
  - Templates turning `TreatyDecision` + simulation summaries into short explanations (“what happened, why, which treaties were applied, which alternatives were rejected”).
- `src/consent_view.rs`  
  - Read‑only adapter for consent states (biosignal use, opt‑outs), backed by DID and treaty metadata.
- `src/grievance_ref.rs`  
  - Generate stable references to specific ledger entries for appeal and oversight.

Outputs:

- JSON/Protobuf payloads for mobile apps, kiosks, and web dashboards.

---

## 9. Spine Orchestrator (`spine_orchestrator`)

**Path:** `infra/rust/spine_orchestrator/`

Purpose: Implement the seven‑stage workflow spine as a composable orchestrator that glues all crates together for each workflow (water, heat, waste, mobility, etc.).[file:21]

Modules:

- `src/pipeline.rs`  
  - Generic pipeline abstraction: `Sense → Normalize → Intent → Optimize → Treaty → Actuate → Audit → CitizenSurface`.
- `src/workflow_manifest.rs`  
  - Manifest format describing which domain modules to plug in at each stage for a given workflow (e.g., water allocation vs. waste routing).
- `src/runtime.rs`  
  - Engine that instantiates pipelines per XR‑node, district, or city‑scale workflow; schedules runs and handles failures.

Key traits:

- `trait SenseStage`
- `trait NormalizeStage`
- `trait IntentStage`
- `trait OptimizeStage`
- `trait TreatyStage`
- `trait ActuateStage`
- `trait AuditStage`

These traits are implemented using the crates described above to ensure all workflows share the same canonical skeleton enforced by CI.[file:21]

---

## 10. Build Targets and Tooling

- Edge XR‑nodes:
  - Target: `wasm32-wasip1` or embedded targets with Rust for IoT, following current embedded Rust patterns for mission‑critical systems.[web:31][web:38]
- City‑scale services:
  - Target: `x86_64-unknown-linux-gnu` or `aarch64-unknown-linux-gnu` with async runtimes and containerization.

CI should enforce:

- All workflow crates depend only on these spine and XR‑grid crates for governance‑critical paths.
- No ad‑hoc treaty logic; all decisions pass through `treaty_engine`.
- All actuation passes through `actuation_bus` and `googolswarm_client`.
