# Aletheion Syntax Corpus Index v1  
Path: `aletheion/docs/index/SYNTAX-CORPUS-INDEX-0001.md`  

This index is the canonical map from `DOMAIN.LAYER.LANGUAGE` keys to concrete research artifacts, YAML mini-specs, and GitHub file anchors across Aletheion’s multi-language stack.[file:5][file:2]  
Every new automation or code-generation request must pass through this index and its referenced YAML specs before new files are created.[file:1][file:2]

---

## 0. Conventions

- **Domains** (subset of existing ERM + seven-capital vocabulary):  
  - `WATER`, `THERMAL`, `WASTE`, `BIOTIC`, `NEUROBIOME`, `MOBILITY`, `ENERGY`, `MATERIALS`, `GOVERNANCE`, `SYNTHEXIS`, `ECOSAFETY`, `METRICS`, `TRUST`.[file:5][file:1][file:2]
- **Layers** (ERM layers):  
  - `L1_EDGE`, `L2_STATE`, `L3_TRUST`, `L4_OPT`, `L5_CITIZEN`.[file:5][file:2]
- **Languages** (supported for syntactic mapping):  
  - `rust`, `aln`, `kotlin`, `javascript`, `lua`, `yaml`, `powershell`.[file:5][file:2]
- **Index key**: `DOMAIN.LAYER.LANGUAGE` (e.g., `WATER.L4_OPT.rust`).[file:5]
- **Mini-specs**: YAML or Markdown frontmatter files that define types, rights, corridors, workflows, file placement, and CI hooks; they are the single source of truth for generation and must never be embedded as comments inside code.[file:1][file:2]

Each table row below is a corpus entry binding a key to:  
- a **pattern ID** (ALE-style family identifier),  
- **mini-spec path**,  
- **research artifacts** (Space file IDs or design docs),  
- **example code anchors**,  
- **extraction queries**,  
- **CI profiles**,  
- and **versioning metadata** for Googolswarm provenance.[file:5][file:1][file:2]

---

## 1. WATER layer entries (initial focus)

### 1.1 WATER · L1_EDGE · rust  
Key: `WATER.L1_EDGE.rust`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-RM-WATER-INGESTION-001`.[file:5] |
| description       | AWP plant + recharge telemetry ingestion into the ERM water bus (Phoenix: Cave Creek, North Gateway, 91st Ave).[file:5][file:8] |
| domain            | `WATER` |
| layer             | `L1_EDGE`.[file:5] |
| language          | `rust` |
| mini_spec_path    | `aletheion/specs/water/ingestion/WATER-INGESTION-RUST-0001.yml` (to be authored alongside WF 6).[file:5] |
| research_artifacts| `file:5` (GOD-City 25 Workflow Blueprint, WF 6), `file:8` (ERM Architecture).[file:5][file:8] |
| example_anchors   | `aletheion/rm/water/ALE-RM-WATER-INGESTION-001.rs` (AWP ingestion), `aletheion/rm/water/ingest/awp.rs` (per-plant connectors).[file:5] |
| canonical_types   | `AwpFacilityId`, `AwpFacilitySnapshot`, `WaterStateModel` (L2 consumer), all aligned with ERM water models.[file:5][file:8] |
| extract_query_ast | “struct or enum in path matching `ALE-RM-WATER-INGESTION-001.rs` where name ∈ {`AwpFacilityId`,`AwpFacilitySnapshot` }”.[file:5] |
| extract_query_regex | `^\\s*pub\\s+struct\\s+AwpFacilitySnapshot\\b`.[file:5] |
| ci_profiles       | `erm-sync-regression`, `aletheion-compliance`, `ecosafety-grammar-preflight`, `workflow-pattern-preflight`.[file:5][file:1][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:00Z` |

---

### 1.2 WATER · L2_STATE · rust  
Key: `WATER.L2_STATE.rust`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-RM-WATER-MODEL-CORE-002`.[file:5] |
| description       | Core water state model (sources, demand zones, allocations) as the operational mirror feeding allocation and co-optimization engines.[file:5][file:8] |
| domain            | `WATER` |
| layer             | `L2_STATE`.[file:5][file:2] |
| language          | `rust` |
| mini_spec_path    | `aletheion/specs/water/state/WATER-STATE-RUST-0001.yml`.[file:5] |
| research_artifacts| `file:5` (WF 6–10 water/thermal), `file:8` (ERM water state design).[file:5][file:8] |
| example_anchors   | `aletheion/rm/water/model/ALE-RM-WATER-MODEL-CORE-002.rs` (state model), `aletheion/rm/water/ALE-RM-WATER-ALLOCATION-RUNNER-001.rs` (consumer).[file:5] |
| canonical_types   | `WaterSourceId`, `WaterDemandZoneId`, `WaterVolume`, `WaterStateModel`, `PortfolioIndicators`, `AllocationRecord`.[file:5] |
| extract_query_ast | “struct in `ALE-RM-WATER-MODEL-CORE-002.rs` where name ∈ {`WaterStateModel`, `PortfolioIndicators`}”.[file:5] |
| extract_query_regex | `^\\s*pub\\s+struct\\s+WaterStateModel\\b`.[file:5] |
| ci_profiles       | `erm-sync-regression`, `water-allocation-nightly`, `aletheion-compliance`, `workflow-pattern-preflight`.[file:5][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:05Z` |

---

### 1.3 WATER · L4_OPT · aln  
Key: `WATER.L4_OPT.aln`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-RM-WATER-ALLOCATION-001`.[file:5] |
| description       | ALN objective and constraint grammar for AWP-centric water allocation (WF 8), including reuse fraction, Colorado exposure, ecological reserves, and heat-vulnerability weights.[file:5] |
| domain            | `WATER` |
| layer             | `L4_OPT`.[file:5][file:2] |
| language          | `aln` |
| mini_spec_path    | `aletheion/specs/water/optimization/WATER-OPTIMIZATION-ALN-0001.yml`.[file:5] |
| research_artifacts| `file:5` (WF 7–9 state + objectives; ready-to-code verdict), `file:8` (optimization layer).[file:5][file:8] |
| example_anchors   | `aletheion/rm/water/ALE-RM-WATER-ALLOCATION-001.aln`, `aletheion/rm/water/scenarios/`.[file:5] |
| canonical_types   | `WaterSourceState`, `WaterDemandZoneState`, `AllocationDecision`, `PortfolioIndicators`, `ConstraintOutcome`, `ObjectiveScore` (as ALN atoms mirroring Rust types).[file:5] |
| extract_query_ast | “atom or rule in `ALE-RM-WATER-ALLOCATION-001.aln` tagged with `constraint` or `objective` for sources/zones”.[file:5] |
| extract_query_regex | `^\\s*rule\\s+constraint` and `^\\s*fn\\s+objectivescore`-style signatures.[file:5] |
| ci_profiles       | `water-allocation-nightly`, `aletheion-compliance`, `workflow-pattern-preflight`, `governance-schema-preflight`.[file:5][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:10Z` |

---

## 2. ECOSAFETY layer entries

### 2.1 ECOSAFETY · L2_STATE · rust  
Key: `ECOSAFETY.L2_STATE.rust`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-ERM-ECOSAFETY-TYPES-001`.[file:1] |
| description       | Shared Rust types for ecosafety grammar spine: risk coordinates, risk vectors, Lyapunov residuals, corridors, and cyboquatic node ecosafety state.[file:1] |
| domain            | `ECOSAFETY` |
| layer             | `L2_STATE`.[file:1][file:3] |
| language          | `rust` |
| mini_spec_path    | `aletheion/specs/ecosafety/grammar/ECOSAFETY-TYPES-RUST-0001.yml`.[file:1] |
| research_artifacts| `file:1` (Ecosafety Framework), `file:3` (Corridor framework), `file:5` (ecosafety tie-ins in workflows).[file:1][file:3][file:5] |
| example_anchors   | `aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-TYPES-001.rs`.[file:1] |
| canonical_types   | `RiskCoord`, `RiskVector`, `LyapunovResidual`, `Corridor`, `CorridorEvalResult`, `CyboquaticNodeEcosafety`, `NodeAction`.[file:1] |
| extract_query_ast | “pub struct with name ∈ {`RiskCoord`,`RiskVector`,`LyapunovResidual`} in ecosafety types crate”.[file:1] |
| extract_query_regex | `^\\s*pub\\s+struct\\s+RiskCoord\\b` etc.[file:1] |
| ci_profiles       | `ecosafety-grammar-preflight`, `aletheion-compliance`, `workflow-pattern-preflight`.[file:1][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:15Z` |

---

### 2.2 ECOSAFETY · L4_OPT · rust  
Key: `ECOSAFETY.L4_OPT.rust`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-ERM-ECOSAFETY-CONTRACTS-001`.[file:1] |
| description       | Runtime ecosafety contracts, `safestep` gate, and FOG routing primitives for cyboquatic and biodegradable nodes, returning `Allow/Derate/Stop` plus qpudata violation shards.[file:1] |
| domain            | `ECOSAFETY` |
| layer             | `L4_OPT`.[file:1] |
| language          | `rust` |
| mini_spec_path    | `aletheion/specs/ecosafety/contracts/ECOSAFETY-CONTRACTS-RUST-0001.yml`.[file:1] |
| research_artifacts| `file:1` (Operationalizing Safety via FOG + safestep), `file:3` (Three-tier corridors), `file:5` (workflow integration).[file:1][file:3][file:5] |
| example_anchors   | `aletheion/erm/ecosafety/rust/ALE-ERM-ECOSAFETY-CONTRACTS-001.rs`.[file:1] |
| canonical_types   | `SafeStepResult`, `NodeShardSummary`, `FogRoutingPolicy`, `FogRouteDecision`, `QpuDataShard`.[file:1] |
| extract_query_ast | “fn named `safestep` and `fogroutenodes` in ecosafety contracts crate”.[file:1] |
| extract_query_regex | `^\\s*pub\\s+fn\\s+safestep\\b`.[file:1] |
| ci_profiles       | `ecosafety-grammar-preflight`, `aletheion-compliance`, `workflow-pattern-preflight`.[file:1][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:20Z` |

---

### 2.3 ECOSAFETY · L3_TRUST · aln  
Key: `ECOSAFETY.L3_TRUST.aln`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-ERM-ECOSAFETY-GRAMMAR-001`.[file:1] |
| description       | Canonical ALN grammar for ecosafety risk coordinates, Lyapunov residuals, corridors, node ecosafety declarations, and SMART-chain restrictions (`no corridor, no build`, `derate/stop`, `PQSTRICT` checks).[file:1] |
| domain            | `ECOSAFETY` |
| layer             | `L3_TRUST`.[file:1][file:2] |
| language          | `aln` |
| mini_spec_path    | `aletheion/specs/ecosafety/grammar/ECOSAFETY-GRAMMAR-ALN-0001.yml`.[file:1] |
| research_artifacts| `file:1` (full grammar already emitted), `file:3` (corridor examples), `file:5` (workflow hooks, ecosafety CI job).[file:1][file:3][file:5] |
| example_anchors   | `aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-GRAMMAR-001.aln`.[file:1] |
| canonical_types   | `RiskName`, `DomainName`, `TestProtocolId`, `CorridorAtom`, `NodeEcosafetyAtom`, `EcosafetyPolicyAtom`, `EvaluationStatusAtom`, `NodeActionAtom`.[file:1] |
| extract_query_ast | “atoms/enums in ecosafety grammar namespace `ALE.ERM.ECOSAFETY.GRAMMAR.001`”.[file:1] |
| extract_query_regex | `^\\s*atom\\s+RiskCoordAtom\\b` etc.[file:1] |
| ci_profiles       | `ecosafety-grammar-preflight`, `aletheion-compliance`, `governance-schema-preflight`.[file:1][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:25Z` |

---

## 3. GOVERNANCE & TRUST entries (Birth-Signs, decisions, codegen)

### 3.1 GOVERNANCE · L2_STATE · rust  
Key: `GOVERNANCE.L2_STATE.rust`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-GOV-BIRTH-SIGN-MODEL-001`.[file:2] |
| description       | Birth-Sign core types: tile-scoped governance signatures binding laws, Indigenous governance, ecological treaties, and local LexEthos overlays to ERM state and workflows.[file:2] |
| domain            | `GOVERNANCE` |
| layer             | `L2_STATE`.[file:2][file:3] |
| language          | `rust` |
| mini_spec_path    | `aletheion/specs/governance/birthsigns/BIRTH-SIGN-RUST-0001.yml`.[file:2] |
| research_artifacts| `file:2` (Birth-Signs architecture), `file:3` (Phoenix corridor + governance ties), `file:5` (workflow hooks).[file:2][file:3][file:5] |
| example_anchors   | `aletheion/governance/birthsigns/ALE-GOV-BIRTH-SIGN-MODEL-001.rs`.[file:2] |
| canonical_types   | `BirthSignId`, `BirthSign`, `GovernanceDomain`, `LawRef`, `IndigenousGovernance`, `EcologicalProtections`, `LocalOverlay`, `FpicRequirement`, `FpicStatus`.[file:2] |
| extract_query_ast | “structs and enums in `ALE-GOV-BIRTH-SIGN-MODEL-001.rs` named `BirthSign`, `GovernanceDomain`, `FpicStatus`.”[file:2] |
| extract_query_regex | `^\\s*pub\\s+struct\\s+BirthSign\\b`.[file:2] |
| ci_profiles       | `governance-schema-preflight`, `aletheion-compliance`, `workflow-pattern-preflight`.[file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:30Z` |

---

### 3.2 TRUST · L3_TRUST · aln  
Key: `TRUST.L3_TRUST.aln`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALE-TRUST-GOVERNED-DECISION-TX-001`.[file:2] |
| description       | Canonical Googolswarm transaction schema for governed decisions, including BirthSign IDs, ALN norms invoked, DIDs, hashes, and outcomes.[file:2] |
| domain            | `TRUST` |
| layer             | `L3_TRUST`.[file:2] |
| language          | `aln` |
| mini_spec_path    | `aletheion/specs/trust/governed-decision/TRUST-GOVERNED-DECISION-ALN-0001.yml`.[file:2] |
| research_artifacts| `file:2` (provenance schema proposal), `file:5` (trust-layer append workflows).[file:2][file:5] |
| example_anchors   | `aletheion/trust/schemas/ALE-TRUST-GOVERNED-DECISION-TX-001.aln`.[file:2] |
| canonical_types   | ALN equivalents of `GovernedDecisionEnvelope`, `DecisionOutcome`, `GovernanceEvaluation`.[file:2] |
| extract_query_ast | “ALN transaction atoms whose fields include `birthSignId`, `alnNorms`, `didSubject`, `decisionOutcome`.”[file:2] |
| extract_query_regex | `^\\s*atom\\s+GovernedDecisionTx\\b` (or configured label).[file:2] |
| ci_profiles       | `governance-schema-preflight`, `aletheion-compliance`, `trust-contract-ci`.[file:2][file:5] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:35Z` |

---

### 3.3 GOVERNANCE · L4_OPT · powershell  
Key: `GOVERNANCE.L4_OPT.powershell`

| field             | value |
|-------------------|-------|
| pattern_id        | `ALETHEION-CODEGEN-PIPELINE-0001`.[file:5] |
| description       | Repo-sovereign PowerShell codegen pipeline that reads this syntax index and YAML mini-specs to emit new code skeletons with correct IDs, paths, and CI wiring.[file:5] |
| domain            | `GOVERNANCE` |
| layer             | `L4_OPT` (meta-optimization: repo structure and workflow expansion). |
| language          | `powershell` |
| mini_spec_path    | `aletheion/specs/tooling/codegen/CODEGEN-PIPELINE-PS-0001.yml`.[file:5] |
| research_artifacts| `file:5` (PowerShell mass-generation suggestion, ALE file naming patterns), plus existing workflow & ecosafety spines.[file:5][file:1][file:2] |
| example_anchors   | `aletheion/tools/powershell/ALETHEION-CODEGEN-PIPELINE-0001.ps1`.[file:5] |
| canonical_types   | Internal PS object model for `CorpusEntry`, `MiniSpecDescriptor`, `NewFilePlan`, `CiProfileBinding`.[file:5] |
| extract_query_ast | N/A (PowerShell); use regex to find functions like `New-AletheionCodegenPlan` and `Invoke-AletheionCodegen`.[file:5] |
| extract_query_regex | `^function\\s+Invoke-AletheionCodegen\\b`.[file:5] |
| ci_profiles       | `aletheion-compliance`, `workflow-pattern-preflight`, `governance-schema-preflight` (ensuring codegen itself obeys spines).[file:5][file:2] |
| version           | `1` |
| timestamp         | `2026-03-08T21:47:40Z` |

---

## 4. AI and codegen usage

1. **Human workflows**  
   - When you author a new workflow, you must first choose or add a `DOMAIN.LAYER.LANGUAGE` entry here and create the referenced YAML mini-spec.[file:5][file:1][file:2]  
   - File paths and ALE IDs in the mini-spec must conform to existing directory spines (ERM, infra, governance, trust).[file:5][file:3]

2. **AI assistants**  
   - When asked to generate code, an AI must:  
     - Resolve the request to a key (e.g., `WATER.L4_OPT.rust`).  
     - Read the corresponding row, then the mini-spec indicated by `mini_spec_path`.  
     - Import the canonical types and CI profiles specified here; never introduce new governance or ecosafety primitives outside the shared grammars.[file:1][file:2][file:5]  
     - Output the YAML mini-spec (or its frontmatter) before any code in its response, as this index + mini-spec pair is the single source of truth.[file:1][file:2]

3. **Blockchain provenance**  
   - For every governed decision or generated file, the responsible workflow must record the `pattern_id`, `mini_spec_path`, and index `key` into the `ALE-TRUST-GOVERNED-DECISION-TX-001` envelope, along with `version` and `timestamp` from this index.[file:2][file:1]  
   - Googolswarm transactions thus carry a reproducible link back to the research corpus and syntax mapping used at generation time.[file:2]

---

## 5. Extension rules

- New entries must:
  - Use an unused `pattern_id` and, where appropriate, a deeper ALE-ID suffix (`-002`, `-003`, etc.).[file:5]
  - Point to an actual or planned YAML mini-spec path under `aletheion/specs/...`.[file:5]
  - Reuse existing spines: ecosafety grammar, Birth-Signs, LexEthos, Synthexis, CryptoSomatic, SMART-chain patterns.[file:1][file:2][file:5]
  - Declare at least one CI profile; CI preflights are allowed to fail PRs that introduce entries without matching mini-specs or anchors.[file:2][file:1]

This document is versioned like any other artifact; changes must be reflected in associated YAML mini-specs and, where applicable, on-chain provenance schemas.[file:2][file:1][file:5]
