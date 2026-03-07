# ADR-0001: Synthexis Habitat Continuity and LNP Planner v1

## Status

Accepted – Synthexis baseline implemented for Phoenix / Sonoran context as of 2026-03-07.

## Context

Aletheion must enforce cross-species rights and continuity for non-human agents (bats, birds, pollinators, desert plants, soil biota) as first-class constraints on urban operations in Phoenix.[file:1]  
Sonoran Desert pollinators and bats are strongly shaped by native vegetation, water availability, and nighttime light and noise conditions.[web:4][web:10]

Phoenix’s existing tree canopy, cool pavement experiments, depaving initiatives, and pollinator-plant programs define a realistic intervention space for corridors and habitat patches.[file:1][web:10]  
We therefore need a reusable modeling and optimization layer – Synthexis – that can:

- Represent species tolerances and priorities.  
- Compile BioticTreaties into machine-verifiable constraints.  
- Evaluate whether the built form, vegetation, hydrology, and traffic enable contiguous, low-barrier corridors.  
- Propose light, noise, and pesticide configurations that respect both human comfort and non-human limits.[file:1]

## Decision

We introduce three core components:

1. **ALN model file** `SYN-SPECIES-MODEL-001.aln`  
   - Encodes `SpeciesAgent`, `BioticTreaty`, and corridor primitives.  
   - Uses explicit, Phoenix-relevant fields like riparian dependency, native plant IDs, and open-water proximity thresholds.[file:1][web:10]

2. **Rust engine module** `SYN-HABITAT-CONTINUITY-001.rs`  
   - Defines built-form, vegetation, hydrology, and traffic graph types.  
   - Implements `HabitatContinuityEngine::evaluate` to score each block for each species and treaty using light, noise, particulate, vegetation, riparian, and traffic attributes.  
   - Emits `CorridorProposal` and per-segment scores for downstream routing and optimization layers.[file:1]

3. **Lua planner** `SYN-LNP-PLANNER-001.lua`  
   - Consumes BioticTreaties and human comfort targets per block.  
   - Produces block-level configuration deltas for lighting, noise windows, and pesticide blackout/buffer rules.  
   - Encodes “most restrictive wins” semantics so high-priority species corridors cap lux and schedule spraying while human comfort remains explicit in the inputs.[file:1]

These components respect the approved technology stack (ALN, Rust, Lua) and avoid blacklisted terms and technologies.[file:1]

## Consequences

Positive:

- We gain a reusable, cross-species interface layer that can be applied to bats following dark riparian corridors, native bees depending on specific plant guilds, and heat-stressed desert vegetation.[web:4][web:10]  
- Phoenix-specific environmental data (canopy fraction, cool pavement segments, depaving candidates, hydrology) can be integrated without changing the core Synthexis API.[file:1]  
- LexEthos and governance layers can reference BioticTreaties to enforce non-human rights in micro-treaties and operational policies.

Negative / open work:

- Current corridor proposals in Rust are “block-sets” with placeholder detour cost; we still need shortest-path routing with barrier-weighted edges.  
- LNP planner uses simple max/min semantics and does not yet optimize across multiple treaties in a fully multi-objective sense.  
- Calibration requires real Phoenix datasets (light measurements, noise maps, native plant distribution, pollinator observations) and will be iterative.[file:1][web:10]

## Next Steps

- Add species-specific calibration entries for:  
  - At least one nectar-feeding bat species,  
  - One native bee guild,  
  - One saguaro-associated bird species.[web:4][web:10]  
- Connect Synthexis outputs to city lighting and pesticide management systems via ALN smart contracts and governance workflows.  
- Integrate with depaving and tree-planting planning so corridor interventions propose specific combinations of planting, depaving, and lighting changes in Phoenix neighborhoods.[file:1]
