// Somaplex ↔ highway-management corridor glue.
// Validates Somaplex route plans against SevenCapital corridors and SMART05
// before any actuation or exposure to downstream mobility systems.
//
// ERM layers: L2 State Modeling, L4 Rule-check, L5 Citizen Interface (via router APIs).
// Domains: somatic, thermal, mobility, treaty, equity.
//
// This module depends on:
//
//   aletheionhighways/src/lib.rs
//     - CapitalSnapshot
//     - Plan, PlanAction
//     - PlanCheck, PlanCheckStatus
//     - CorridorService, SmartChainQuery
//     - validate_plan_structural, validate_plan_capitals, validate_plan_smartchain
//
//   aletheionsomaplex/SomaticRouteEngine.rs
//     - SomaticCost, SomaticSegmentView
//
//   aletheionthermaphora/DOWNTOWNHEATBUDGETENGINE-001.rs
//     - HeatCost, HeatSegmentView
//
//   aletheionerm-state/regions/REGIONDOWNTOWNCORE-001.aln
//     - RegionId = REGION_DOWNTOWN_CORE
//
// SMART chains:
//
//   SMART05SOMAPLEXROUTING
//     Domains: somatic, thermal, mobility
//     Mode: PQSTRICT
//     Rights grammars: RightToSafeMovement, RightToShade
//
//   SMART01AWPTHERMALTHERMAPHORA (read-only thermal / HeatBudget envelopes)

use std::collections::HashMap;

use crate::somatic::{
    SomaticSegmentView,
    SomaticCost,
};

use crate::heat::{
    HeatSegmentView,
    HeatCost,
};

use aletheionhighways::{
    CapitalSnapshot,
    Plan,
    PlanAction,
    PlanCheck,
    PlanCheckStatus,
    CorridorService,
    SmartChainQuery,
    validate_plan_structural,
    validate_plan_capitals,
    validate_plan_smartchain,
};

/// Somaplex trip category, used to specialize corridor and SMART-chain checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SomaplexTripKind {
    CareRoute,
    CitizenMobility,
    FreightSupport,
}

/// Region identifier for routing; for now we anchor on DowntownCentral Phoenix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SomaplexRegionId {
    DowntownCore,
    // Future: add more regions as Somaplex expands.
}

/// A single leg in a Somaplex route plan.
#[derive(Debug, Clone)]
pub struct SomaplexRouteLeg {
    pub from_node: u64,
    pub to_node: u64,
    pub segment_id: String,
    pub somatic_cost: SomaticCost,
    pub heat_cost: HeatCost,
}

/// High-level route plan submitted by Somaplex for validation.
#[derive(Debug, Clone)]
pub struct SomaplexRoutePlan {
    pub trip_id: String,
    pub persona_id: Option<String>, // DID or anonymized cohort id
    pub kind: SomaplexTripKind,
    pub region: SomaplexRegionId,
    pub legs: Vec<SomaplexRouteLeg>,
}

/// Local view of SevenCapital for the corridor segments touched by this route.
#[derive(Debug, Clone)]
pub struct SomaplexCapitalView {
    pub water: f64,
    pub thermal: f64,
    pub waste: f64,
    pub biotic: f64,
    pub somatic: f64,
    pub neurobiome: f64,
    pub treaty: f64,
}

/// Combined context passed into validation helpers for capital and SMART-chain checks.
pub struct SomaplexValidationContext<'a> {
    pub capitals_before: CapitalSnapshot,
    pub capitals_after: CapitalSnapshot,
    pub smartchain_registry: &'a dyn SmartChainQuery,
    pub corridor_id: &'a str,
}

/// Adapter that exposes Somaplex routing as a highway Plan.
///
/// This is intentionally minimal: it does *not* implement propose_plan/actuate;
/// it only builds and validates plans for callers that then talk to mobility routers.
#[derive(Debug)]
pub struct SomaplexCorridorAdapter<'a> {
    pub region: SomaplexRegionId,
    pub smartchain_registry: &'a dyn SmartChainQuery,
    pub capital_before: CapitalSnapshot,
    pub capital_after: CapitalSnapshot,
}

impl<'a> CorridorService for SomaplexCorridorAdapter<'a> {
    fn id(&self) -> &str {
        match self.region {
            SomaplexRegionId::DowntownCore => "CORRIDOR_DOWNTOWN_SOMAPLEX",
        }
    }

    fn read_capitals(&self) -> CapitalSnapshot {
        self.capital_before.clone()
    }

    fn read_demands(&self) -> aletheionhighways::DemandSnapshot {
        // Somaplex-specific adapter does not expose water/waste demands;
        // mobility trips are encoded as PlanAction::RouteSegment actions.
        aletheionhighways::DemandSnapshot {
            water_mgd: 0.0,
            sewer_load_mgd: 0.0,
            msw_tpd: 0.0,
            recycling_tpd: 0.0,
            hazmat_trips: 0,
            cleaning_tasks_pending: 0,
            mobility_trips: 0, // Caller can fill a more detailed view if needed.
        }
    }

    fn propose_plan(&self, _horizon_min: u32) -> Plan {
        // Somaplex does not auto-propose plans from this adapter;
        // plans are constructed upstream and passed into validate_plan / actuate.
        Plan { actions: Vec::new(), metadata: HashMap::new() }
    }

    fn validate_plan(&self, plan: Plan) -> PlanCheck {
        let structural = validate_plan_structural(&plan);

        let ctx = SomaplexValidationContext {
            capitals_before: self.capital_before.clone(),
            capitals_after: self.capital_after.clone(),
            smartchain_registry: self.smartchain_registry,
            corridor_id: self.id(),
        };

        let capital = validate_plan_capitals(&ctx);
        let smartchain = validate_plan_smartchain(&ctx, &plan, &["SMART05SOMAPLEXROUTING", "SMART01AWPTHERMALTHERMAPHORA"]);

        aletheionhighways::summarize_plancheck(structural, capital, smartchain)
    }

    fn actuate(&self, plan: Plan) -> aletheionhighways::ActuationResult {
        let check = self.validate_plan(plan.clone());
        if check.status != PlanCheckStatus::Safe {
            return aletheionhighways::ActuationResult::Rejected(check);
        }
        // Somaplex adapter does not directly actuate hardware.
        // It simply returns Executed with a synthetic count so the caller
        // can forward the safe route to mobility routers / citizen apps.
        aletheionhighways::ActuationResult::Executed { actions_count: plan.actions.len() }
    }
}

/// Build a generic highway Plan from a SomaplexRoutePlan.
///
/// Each route leg becomes a PlanAction::RouteSegment with bounded, typed parameters.
/// No free-form strings beyond IDs already validated by Somaplex and the registry.
pub fn build_highway_plan_from_somaplex(route: &SomaplexRoutePlan) -> Plan {
    let mut actions = Vec::with_capacity(route.legs.len());

    for leg in &route.legs {
        actions.push(PlanAction::RouteSegment {
            segment_id: leg.segment_id.clone(),
            from_node: leg.from_node,
            to_node: leg.to_node,
            // Somatic/heat costs are numeric and bounded upstream; we simply carry them through.
            somatic_cost: leg.somatic_cost.value(),
            heat_cost: leg.heat_cost.value(),
        });
    }

    let mut metadata = HashMap::new();
    metadata.insert("trip_id".to_string(), route.trip_id.clone());
    metadata.insert(
        "kind".to_string(),
        match route.kind {
            SomaplexTripKind::CareRoute => "CareRoute".to_string(),
            SomaplexTripKind::CitizenMobility => "CitizenMobility".to_string(),
            SomaplexTripKind::FreightSupport => "FreightSupport".to_string(),
        },
    );
    metadata.insert(
        "region".to_string(),
        match route.region {
            SomaplexRegionId::DowntownCore => "REGION_DOWNTOWN_CORE".to_string(),
        },
    );

    Plan { actions, metadata }
}

/// Convenience: validate a Somaplex route against corridor capital + SMART chains.
///
/// This is the main entry point Somaplex should call before exposing any route
/// to mobility routers or citizen interfaces.
pub fn validate_somaplex_route<'a>(
    route: &SomaplexRoutePlan,
    capital_view_before: &SomaplexCapitalView,
    capital_view_after: &SomaplexCapitalView,
    smartchain_registry: &'a dyn SmartChainQuery,
) -> (Plan, PlanCheck) {
    let capitals_before = to_capital_snapshot(capital_view_before);
    let capitals_after = to_capital_snapshot(capital_view_after);

    let adapter = SomaplexCorridorAdapter {
        region: route.region,
        smartchain_registry,
        capital_before: capitals_before.clone(),
        capital_after: capitals_after.clone(),
    };

    let plan = build_highway_plan_from_somaplex(route);
    let check = adapter.validate_plan(plan.clone());
    (plan, check)
}

/// Map SomaplexCapitalView into the shared CapitalSnapshot used by highway validators.
fn to_capital_snapshot(view: &SomaplexCapitalView) -> CapitalSnapshot {
    CapitalSnapshot {
        water: view.water,
        thermal: view.thermal,
        waste: view.waste,
        biotic: view.biotic,
        somatic: view.somatic,
        neurobiome: view.neurobiome,
        treaty: view.treaty,
    }
}

/// Minimal SomaticCost accessor required by this module.
/// The concrete SomaticCost type lives in Somaplex core; here we just assume a value() API.
mod somatic {
    #[derive(Debug, Clone, Copy)]
    pub struct SomaticCost(pub f64);

    impl SomaticCost {
        pub fn value(self) -> f64 {
            self.0
        }
    }

    #[derive(Debug, Clone)]
    pub struct SomaticSegmentView {
        pub segment_id: String,
        pub cost: SomaticCost,
    }
}

/// Minimal HeatCost accessor required by this module.
/// The concrete HeatCost type lives in Thermaphora; here we just assume a value() API.
mod heat {
    #[derive(Debug, Clone, Copy)]
    pub struct HeatCost(pub f64);

    impl HeatCost {
        pub fn value(self) -> f64 {
            self.0
        }
    }

    #[derive(Debug, Clone)]
    pub struct HeatSegmentView {
        pub segment_id: String,
        pub cost: HeatCost,
    }
}
