# ADR-0001: Thermaphora Heat Budget and Microclimate Craft v1

## Status

Accepted – Thermaphora v1 core model and simulators established for Phoenix desert heat.

## Context

Phoenix already deploys cool pavement, shade plans, and experimental atmospheric water harvesting to manage extreme heat, but current practice treats these as infrastructure projects rather than part of a continuous, per-body heat budget.[file:1][web:31][web:36][web:37]  
Aletheion needs a subsystem that can represent how each citizen experiences heat over time and how block-level microclimate interventions change actual risk.

## Decision

We introduce Thermaphora with three initial components:

1. `THERM-CORE-MODEL-001.aln`  
   - Defines `HeatBudgetProfile`, `MicroclimateField`, `SegmentHeatLoad`, `HeatBudgetResult`, and `BlockHeatRisk`.  
   - Captures individual factors (activity, clothing, medications) and block factors (shade, cool pavement, evap capacity) grounded in Phoenix conditions.[file:1][web:31][web:36]

2. `THERM-HEAT-BUDGET-SIMULATOR-001.rs`  
   - Provides `HeatBudgetSimulator::simulate_day` that maps a day’s schedule and microclimate fields to heat strain indices and time above safe thresholds.  
   - Uses a deliberately simple, transparent heat-load formula that can be upgraded to a full heat balance model later.[file:1]

3. `THERM-MICROCLIMATE-FIELD-DESIGNER-001.lua`  
   - Suggests block-level combinations of trees, shade structures, cool pavement, and misting/evap based on baseline fields and water constraints.  
   - Aligns with Phoenix’s Shade Plan, cool pavement trials, and AWG/evaporative testbeds as real levers, not abstract knobs.[file:1][web:31][web:36][web:37]

## Consequences

Positive:

- Thermaphora becomes a reusable layer that other subsystems can call: Chronaurea (time-of-day scheduling), Somaplex (route selection with shade), PraxisWeave (heat-aware care routing).[file:1]  
- Heat mitigation decisions can be justified by predicted reductions in HeatLoadIndex and minutes above threshold, not just air temperature averages.[file:1][web:31]

Negative / open work:

- The current simulator is approximate and must be calibrated with local health and meteorological data, including recent Phoenix heat waves.  
- Microclimate designer ignores ERM water portfolio dynamics; integration with AWP and groundwater constraints is planned for Thermaphora/FieldDesign v2.[file:1][web:31][web:37]

## Future Branches

- **Thermaphora/Sim/Events** – replay historic heat events and stress-test proposed interventions.  
- **Thermaphora/Radar** – citywide HeatVulnerabilityRadar keyed to forecast and citizen profiles.  
- **Thermaphora/Interface** – citizen apps and operator dashboards exposing heat risk and suggested actions in real time.
