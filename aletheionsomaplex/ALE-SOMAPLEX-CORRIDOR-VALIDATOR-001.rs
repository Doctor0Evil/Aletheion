// Shared Somaplex corridor validator for Care Routes, citizen mobility, and freight
// enforcing SevenCapitalState + ecosafety + SMART-chain invariants.

use std::collections::HashMap;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use aletheion_erm_state::{SevenCapitalState, SegmentId};
use aletheion_somaplex::{SomaplexCapitalView, PersonaClass};
use aletheion_thermaphora::HeatCostView;
use aletheion_smartchains::SmartChainRegistry;
use aletheion_highways::{CapitalSnapshot, Plan, PlanAction, PlanCheck, PlanCheckStatus,
                         StructuralViolation, CapitalViolation, SmartChainViolation,
                         ValidationContext, validate_plan_structural, validate_plan_capitals,
                         validate_plan_smartchain, summarize_plancheck};

/// High-level trip type to differentiate routing constraints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SomaplexTripKind {
    CareRoute,
    CitizenMobility,
    FreightSupport, // freight-support legs that must still obey somatic corridors
}

/// Route leg as seen by Somaplex validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaplexRouteLeg {
    pub segment_id: SegmentId,
    pub departure_time: DateTime<Local>,
    pub arrival_time: DateTime<Local>,
}

/// Complete route plan from router (Somaplex + Thermaphora).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaplexRoutePlan {
    pub trip_id: String,
    pub persona: PersonaClass,
    pub kind: SomaplexTripKind,
    pub legs: Vec<SomaplexRouteLeg>,
    pub region: String,
}

/// Context for validating a Somaplex route against SevenCapital and heat budgets.
#[derive(Debug, Clone)]
pub struct SomaplexValidationContext {
    pub seven_capital_snapshots: HashMap<SegmentId, SevenCapitalState>,
    pub somaplex_view: HashMap<SegmentId, SomaplexCapitalView>,
    pub heat_cost: HashMap<SegmentId, HeatCostView>,
}

/// Convert SevenCapitalState + Somaplex/heat slices into the generic CapitalSnapshot
/// used by the corridor/highway validator.
fn capital_snapshot_from_seven(
    seven: &SevenCapitalState,
    somaview: Option<&SomaplexCapitalView>,
    heat: Option<&HeatCostView>,
) -> CapitalSnapshot {
    let mut snap = CapitalSnapshot::from_seven_capitals(seven);

    if let Some(s) = somaview {
        snap.somatic.somatic_cost_headroom = s.somatic_headroom;
    }
    if let Some(h) = heat {
        snap.thermal.heat_budget_minutes = h.remaining_minutes_budget;
    }

    snap
}

/// Domains used for Somaplex-related SMART-chain checks.
fn somaplex_domains(kind: SomaplexTripKind) -> &'static [&'static str] {
    match kind {
        SomaplexTripKind::CareRoute => &["somatic", "thermal", "water", "equity", "treaty"],
        SomaplexTripKind::CitizenMobility => &["somatic", "thermal", "equity"],
        SomaplexTripKind::FreightSupport => &["somatic", "thermal", "logistics", "equity"],
    }
}

/// Validate a Somaplex route using the shared corridor/highway spine.
pub fn validate_somaplex_route(
    plan: &SomaplexRoutePlan,
    ctx: &SomaplexValidationContext,
    smartchains: &SmartChainRegistry,
) -> PlanCheck {
    let mut structural: Vec<StructuralViolation> = Vec::new();
    let mut capital: Vec<CapitalViolation> = Vec::new();
    let mut smartchain: Vec<SmartChainViolation> = Vec::new();

    // Structural: region and non-empty legs.
    if plan.region != "REGION_DOWNTOWN_CORE" {
        structural.push(StructuralViolation::RegionMismatch {
            expected: "REGION_DOWNTOWN_CORE".into(),
            found: plan.region.clone(),
        });
    }
    if plan.legs.is_empty() {
        structural.push(StructuralViolation::EmptyPath);
    }

    if !structural.is_empty() {
        return PlanCheck {
            status: PlanCheckStatus::Unsafe,
            structural,
            capital,
            smartchain,
        };
    }

    // Aggregate capitals before/after traversing the route.
    // v0: use per-segment caps; future: simulate cumulative load over time.
    let mut before = None;
    let mut after = None;

    for leg in &plan.legs {
        let seg_id = leg.segment_id.clone();

        let seven = match ctx.seven_capital_snapshots.get(&seg_id) {
            Some(s) => s,
            None => {
                structural.push(StructuralViolation::MissingSevenCapitalState { segment: seg_id });
                continue;
            }
        };
        let somaview = ctx.somaplex_view.get(&seg_id);
        let heat = ctx.heat_cost.get(&seg_id);

        let snap = capital_snapshot_from_seven(seven, somaview, heat);

        before = before.or(Some(snap.clone()));
        after = Some(match after {
            None => snap,
            Some(prev) => prev.max_with(&snap),
        });

        // Per-segment somatic corridor: block high somatic cost or exhausted headroom.
        if snap.somatic.somatic_cost_headroom <= 0.0 {
            capital.push(CapitalViolation::SomaticOverBudget {
                segment: seg_id.clone(),
                rx: seven.somatic.rx,
                headroom: snap.somatic.somatic_cost_headroom,
            });
        }

        // Per-segment thermal corridor: block segments where remaining heat budget is gone.
        if snap.thermal.heat_budget_minutes <= 0.0 {
            capital.push(CapitalViolation::ThermalOverBudget {
                segment: seg_id.clone(),
                rx: seven.thermal.rx,
            });
        }

        // TreatyCapital: any negative headroom blocks movement.
        if seven.treaty.headroom < 0.0 {
            capital.push(CapitalViolation::TreatyHeadroomNegative {
                segment: seg_id.clone(),
                headroom: seven.treaty.headroom,
            });
        }

        // EquityCapital: Care Routes get stricter thresholds in high-risk blocks.
        if matches!(plan.kind, SomaplexTripKind::CareRoute) && seven.equity.rx > 0.9 {
            capital.push(CapitalViolation::EquityOverExposure {
                segment: seg_id.clone(),
                rx: seven.equity.rx,
            });
        }
    }

    // If we could not compute a before/after snapshot, fail structurally.
    let (before_caps, after_caps) = match (before, after) {
        (Some(b), Some(a)) => (b, a),
        _ => {
            if structural.is_empty() {
                structural.push(StructuralViolation::MissingCapitalSnapshot);
            }
            let status = PlanCheckStatus::Unsafe;
            return PlanCheck {
                status,
                structural,
                capital,
                smartchain,
            };
        }
    };

    // Build a generic Plan shell for SMART-chain & ecosafety checks.
    let corridor_plan = Plan {
        corridor_id: format!("SOMAPLEX:{}", plan.region),
        horizon_min: 0,
        actions: plan
            .legs
            .iter()
            .map(|leg| PlanAction::Mobility {
                policy_id: format!("SEGMENT:{}", leg.segment_id),
            })
            .collect(),
    };

    // Structural checks on the abstract Plan.
    structural.extend(validate_plan_structural(&corridor_plan));

    // Capital projection: use before/after capitals and ecosafety rules.
    let vctx = ValidationContext {
        capitals_before: &before_caps,
        capitals_after: &after_caps,
        smartchain_registry: smartchains,
        corridor_id: &corridor_plan.corridor_id,
    };
    capital.extend(validate_plan_capitals(&vctx));

    // SMART-chain checks: Somaplex + Thermaphora chains.
    smartchain.extend(validate_plan_smartchain(
        smartchains,
        somaplex_domains(plan.kind),
        &SevenCapitalState::treaty_from_pair(&before_caps, &after_caps),
    ));

    summarize_plancheck(structural, capital, smartchain)
}
