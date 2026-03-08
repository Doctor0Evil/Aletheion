# Aletheion Workflow Architecture Spine  
`aletheion/docs/workflows/WORKFLOW-ARCHITECTURE-0001.md`

## 0. Purpose and scope

This document defines the mandatory workflow architecture for Aletheion. It applies to:

- The initial 25 Phoenix workflows, and  
- Any future automation that touches land, water, air, materials, biosignals, devices, or citizens.

Every workflow MUST implement the seven‑stage spine, integrate ALN schemas, honor Birth‑Signs, and comply with energy‑aware placement and governance rules described below.[file:3][file:4]

---

## 1. Seven‑stage workflow spine (MUST)

Every workflow is a pipeline with exactly seven stages. A module MAY be simple internally, but its public architecture MUST conform.

### 1.1 Stage S1 – Edge ingestion

Role: Bring physical reality into Aletheion with minimal, safe preprocessing.

Requirements:

- Languages: Rust or C only for device‑adjacent code.[file:4]  
- Responsibilities:
  - Connect to SCADA, IoT, wearables, vehicles, building systems, and consumer electronics.
  - Validate basic schema and ranges, perform low‑latency anomaly detection.
  - Attach metadata: `asset_id`, `device_id`, `BirthSignId`, `time`, `location`, `DID` (if personal).[file:3][file:4]
- Output: Append‑only event streams or snapshots into Layer 2 state models.

File patterns:

- `...-INGESTION-*.rs` / `...-INGEST-*.rs` (required).  
- Optional device‑specific submodules (e.g., `awp_cave_creek.rs`, `canal_gate_siemens.rs`).[file:3]

### 1.2 Stage S2 – State model update

Role: Maintain the operational mirror of Phoenix (and future cities) as a set of typed state models.

Requirements:

- Languages: Rust primary; ALN and Lua for agent logic and views.[file:4]  
- Responsibilities:
  - Store and update resource, infrastructure, biosignal, and social states.
  - Preserve Birth‑Sign tags and all regulatory context.
  - Expose read APIs for optimization, governance, and citizen interfaces.
- Must avoid: any “digital twin” semantics; use approved terms like **state model** and **operational mirror**.[file:4]

File patterns:

- `...-STATE-*.rs` for core models.  
- `schemas/*.aln` for declarative structures (e.g., `AwpSnapshot.aln`).[file:3][file:4]

### 1.3 Stage S3 – Trust append (Googolswarm, ALN/KYC/DID)

Role: Produce immutable, jurisdiction‑aware records for critical decisions.

Requirements:

- Languages: ALN for contracts, Rust/Lua for glue.[file:4]  
- Responsibilities:
  - Append resource movements, policy choices, consent events, and treaty evaluations to Googolswarm ledgers.
  - Record which `BirthSignId`s applied, which treaties and rights grammars were consulted, and which proofs passed or failed.[file:3][file:4]
  - Enforce ALN/KYC/DID rules for identity and multi‑sig attestation.

File patterns:

- `...-POLICY-*.aln`, `...-LEDGER-*.aln`, or domain‑specific (e.g., `ALE-GOV-POLICY-REGISTRY-001.rs`).[file:3][file:4]

### 1.4 Stage S4 – Optimization engine

Role: Compute candidate actions subject to constraints.

Requirements:

- Languages: Rust and/or C for compute, ALN for constraints and objectives.[file:4]  
- Responsibilities:
  - Implement multi‑objective optimization (e.g., NSGA‑II, MOEA‑D) for water, heat, energy, mobility, waste, care, etc.[file:3][file:4]
  - Treat all Birth‑Sign, BioticTreaty, neurorights, and LexEthos constraints as hard or high‑penalty. No “best effort” ignoring.[file:3]
  - Emit both recommended actions and justification metadata for audit and citizen interfaces.

File patterns:

- `...-ALLOCATION-*.aln` for objective and constraint definitions.  
- `...-ALLOCATION-RUNNER-*.rs` or `...-ENGINE-*.rs` for runtime.[file:3]

### 1.5 Stage S5 – ALN enforcement and validation

Role: Convert ethics, law, and protocols into deterministic checks.

Requirements:

- Languages: ALN primary; Rust/Lua wrappers.[file:4]  
- Responsibilities:
  - Evaluate:
    - Birth‑Signs and territorial obligations.  
    - Indigenous and FPIC rules.  
    - BioticTreaties and habitat continuity.  
    - CryptoSomatic FEAR/PAIN/SANITY envelopes, SomaticEnvelopes.  
    - LexEthos rights grammars and micro‑treaties.[file:3][file:4]
  - Produce explicit allow/deny plus minimal change suggestions (e.g., dim X, delay Y hours, reroute via Z).[file:3]

File patterns:

- Shared libraries:
  - `aletheion/governance/lexethos/*`  
  - `aletheion/synthexis/model/*` and `engine/*`  
  - `aletheion/health/augmentation/*`  
  - `aletheion/compliance/ALE-COMP-CORE-*.rs`[file:3]
- Per‑workflow ALN files referencing shared schemas.

### 1.6 Stage S6 – Actuation and orchestration

Role: Turn approved decisions into actions on devices, services, and procedures.

Requirements:

- Languages: Lua for orchestration; Rust/C++ for real‑time control kernels.[file:3][file:4]  
- Responsibilities:
  - Sequence commands to pumps, valves, HVAC, microgrids, fleets, UIs, and notifications.
  - Respect energy and safety constraints on each host device.
  - Ensure idempotency and forward‑only changes (no silent rollbacks).[file:3]

File patterns:

- `...-WORKFLOW-*.lua` (schedulers & playbooks).[file:3]  
- `...-CONTROL-*.rs` / `...-KERNEL-*.rs` for real‑time loops.

### 1.7 Stage S7 – Citizen surface and grievance loop

Role: Provide visibility, control, and contestability for humans.

Requirements:

- Languages: Kotlin (Android), JavaScript/React for web, Rust for consent gateways.[file:4]  
- Responsibilities:
  - Present decisions, rationales, and applicable Birth‑Signs and treaties in plain language.
  - Expose consent management (CryptoSomatic Shield) and data minimization by default.
  - Provide structured feedback channels that can trigger new micro‑treaties or override unsafe automations via governance workflows.[file:3][file:4]

File patterns:

- `...-APP-*.kt` for mobile clients.  
- `...-WEB-*.js` for dashboards.  
- `...-CONSENT-API-*.rs` for DID/consent enforcement.[file:3][file:4]

---

## 2. ALN schemas and Birth‑Signs (MUST)

### 2.1 Birth‑Sign basics

Every spatial tile in Aletheion MUST have exactly one Birth‑Sign record, versioned over time.

A Birth‑Sign encodes:[file:3][file:4]

- Public law and regulation (city → national + treaties).  
- Recognized Indigenous territories and FPIC protocols.  
- Environmental and cross‑species protections (BioticTreaties, habitat, light/noise/pesticides).  
- Local LexEthos micro‑treaties and citizen norms (cooling, mobility, data use).[file:3]

### 2.2 How modules MUST use Birth‑Signs

- S1/S2: every ingested asset/event MUST include `BirthSignId`, looked up by location and time.[file:3][file:4]  
- S4: optimizers MUST request the active constraint bundle for all affected tiles and treat them as non‑optional.  
- S5: ALN validators MUST record which Birth‑Signs were consulted and whether they were satisfied.  
- S3: trust layer MUST log `BirthSignId` sets for each committed decision to enable jurisdiction‑aware audits.[file:3]

No workflow may bypass Birth‑Signs or substitute hard‑coded assumptions.

---

## 3. Energy‑aware placement and firmware channels (MUST)

### 3.1 Placement model

Workflows MUST be deployable on self‑hosted VMs scheduled over already‑active consumer and urban electronics (appliances, building systems, street devices, vehicles) to minimize additional power draw.[file:4]

Principles:

- Prefer existing powered devices over new dedicated servers.  
- Respect thermal, safety, and duty‑cycle limits of each host.  
- Keep security isolation between critical safety logic and city analytics.[file:4]

### 3.2 Node capability registry

All nodes participating in Aletheion MUST register capabilities:

- Hardware: CPU, RAM, storage, thermal headroom.  
- Connectivity: bandwidth, latency, intermittent behavior.  
- Jurisdiction: `BirthSignId`s covered, physical location.  
- Safety: classification (life‑critical, building, mobility, consumer).[file:4]

Schedulers and orchestrators MUST consult this registry when deploying S1–S4 tasks.

### 3.3 Firmware channels

For each device class, the infrastructure layer MUST define a **firmware channel**:

- A signed, sandboxed runtime (e.g., Lua or WASM) for lightweight orchestration logic.  
- Strict limits on:
  - Max CPU and energy budget.  
  - Active time windows (e.g., prefer solar peaks, avoid heat waves for heavy compute).[file:4]

All edge and micro‑workflows (especially S1 and S6) MUST target these channels instead of ad‑hoc code injection.

### 3.4 Energy‑aware workflow behavior

- Batch heavy optimization: S4 intensive runs SHOULD be scheduled when renewable supply is high and grid stress is low; only deltas are pushed to edge nodes.[file:3][file:4]  
- Co‑locate with sensors: where a device is the only sensor, S1/S2 SHOULD run minimal transforms locally, but MUST offload heavy ML or multi‑scenario modeling to better‑provisioned nodes.  
- Honor thermaphora: any heat‑intensive workload in hot seasons MUST integrate with Thermaphora heat budgets to avoid raising risk for nearby humans and species.[file:3]

---

## 4. Governance, rights, and compliance hooks (MUST)

### 4.1 ALN rights grammars and micro‑treaties

Workflows MUST integrate with:

- LexEthos RightsGrammarCompiler and MicroTreatyEngine to interpret local norms and obligations as enforceable ALN constraints.[file:3]  
- DisputeCoolingProtocol for automated, minimal interventions in case of conflict (retiming, rerouting, micro‑recognition).[file:3]

### 4.2 Neurorights and CryptoSomatic Shield

Any workflow that touches biosignals, posture, augmentation, or attention MUST:

- Use CryptoSomatic Shield schemas (FEAR/PAIN/SANITY, SomaticEnvelopes, ConsentStateMachine).[file:3][file:4]  
- Route enforcement through AugmentationSafetyKernel and SomaticAnomalyDetector where applicable.[file:3]  
- Expose consent and revocation via S7 citizen surfaces.

### 4.3 Compliance core and CI

The central compliance workflow is mandatory for all repos:

- Library: `aletheion/compliance/ALE-COMP-CORE-001.rs` (and successors) MUST be included in builds.[file:3]  
- CI: `.github/workflows/aletheion-compliance.yml` MUST run on every PR and release to:
  - Enforce language and cryptography blacklists.  
  - Verify absence of excluded semantics (e.g., “digital twin”).  
  - Ensure Birth‑Sign, ALN, and neurorights hooks are wired for all workflows touched.[file:3][file:4]

No code may be merged or deployed if compliance fails.

### 4.4 Governance and forward‑only change

Policy and workflow changes MUST:

- Flow through governance modules (liquid democracy, policy registry) and be committed as Googolswarm contracts.[file:3]  
- Respect “forward‑only” semantics: changes may deprecate but not silently revert prior obligations; new ADRs MUST document evolution.  
- Be visible to PM metrics workflows, which track coverage, test status, and deployment readiness for the entire city factory.[file:3]

---

## 5. Repository conventions and extension rules (MUST/SHOULD)

### 5.1 Naming and structure

All new workflows MUST declare:

- Home repo (e.g., `aletheion/rm/water`, `aletheion/infra/sewer`, `aletheion/synthexis`, `aletheion/governance/lexethos`, `aletheion/citizeninterface`).[file:3]  
- File set covering S1–S7, using the patterns in section 1.  
- CI entries for:
  - Unit and integration tests (≥85% target).  
  - Compliance.  
  - PM metrics inclusion.[file:3]

### 5.2 Extending beyond the first 25 workflows

New workflows MUST:

- Reuse the seven‑stage spine unchanged.  
- Reuse existing ALN schemas where possible (Birth‑Signs, BioticTreaties, LexEthos, CryptoSomatic Shield); if new schemas are needed, they MUST be added to shared libraries, not inlined.[file:3][file:4]  
- Define energy placement strategies (firmware channels, VM profiles, scheduling windows) as part of their spec.  
- Declare which 25‑workflow patterns they extend (e.g., “variant of AWP allocation skeleton” or “extended Somaplex mobility engine”).[file:3]

---

## 6. Minimal checklist for any new workflow

Before a workflow is accepted into Aletheion, the author MUST be able to answer “yes” to all of the following:

1. Spine  
   - Does it implement S1–S7 with appropriate languages and file patterns?  
2. Governance context  
   - Are all affected tiles annotated with Birth‑Signs?  
   - Does the optimization stage call ALN validators with those Birth‑Signs?  
3. Rights and treaties  
   - Have relevant Indigenous, BioticTreaty, neurorights, and LexEthos constraints been wired as ALN schemas?  
4. Energy and placement  
   - Is there an explicit placement plan on existing devices and/or nodes, with duty‑cycle and thermal limits?  
5. Compliance and CI  
   - Does the workflow build under `ALE-COMP-CORE` and pass `.github/workflows/aletheion-compliance.yml` and PM CI?  
6. Citizen interface  
   - Are decisions explainable through S7, with visible governance context and a grievance path?

Only workflows that satisfy this architecture are allowed to operate inside the Aletheion city factory.
