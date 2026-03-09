use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use aletheion_corridors::DowntownFreightSomaplexCorridors;
use aletheion_corridors::DowntownFreightSegmentBindings;
use aletheion_erm_state::{SevenCapitalState, SegmentId};
use aletheion_smartchains::SmartChainRegistry;
use aletheion_somaplex::SomaplexCapitalView;
use aletheion_thermaphora::HeatBudgetEnvelope;
use aletheion_validation::{
    CapitalViolation, PlanCheck, PlanCheckStatus, SmartChainViolation, StructuralViolation,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FreightBand {
    LightVan,
    MediumTruck,
    HeavyTruck,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TimeWindowClass {
    DayGeneral,
    NightQuietSensitive,
    PeakCommute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightLeg {
    pub segment_id: SegmentId,
    pub departure_time: DateTime<Local>,
    pub arrival_time: DateTime<Local>,
    pub freight_band: FreightBand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightPlan {
    pub vehicle_id: String,
    pub legs: Vec<FreightLeg>,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightScenarioContext {
    pub seven_capital_snapshots: HashMap<SegmentId, SevenCapitalState>,
    pub heat_budget: HashMap<SegmentId, HeatBudgetEnvelope>,
    pub somaplex_view: HashMap<SegmentId, SomaplexCapitalView>,
}

/// Compute a candidate freight plan without yet enforcing Somaplex or corridors.
/// In practice this would call ALE-TM-ROUTER-001/002; here we keep it as a stub.
pub fn draft_freight_plan(
    vehicle_id: String,
    origin: SegmentId,
    dest: SegmentId,
    band: FreightBand,
    now: DateTime<Local>,
) -> FreightPlan {
    FreightPlan {
        vehicle_id,
        legs: vec![FreightLeg {
            segment_id: origin, // placeholder: real router would build full path
            departure_time: now,
            arrival_time: now,
            freight_band: band,
        }],
        region: "REGION_DOWNTOWN_CORE".to_string(),
    }
}

/// Apply Downtown freight–Somaplex corridor rules, SevenCapitalState,
/// and SMART-chain invariants to a candidate plan.
/// On violation, returns a PlanCheck with status Unsafe.
pub fn validate_freight_plan(
    plan: &FreightPlan,
    ctx: &FreightScenarioContext,
    smartchains: &SmartChainRegistry,
) -> PlanCheck {
    let mut structural: Vec<StructuralViolation> = Vec::new();
    let mut capital: Vec<CapitalViolation> = Vec::new();
    let mut smartchain: Vec<SmartChainViolation> = Vec::new();

    // Structural: region, non-empty legs
    if plan.region != "REGION_DOWNTOWN_CORE" {
        structural.push(StructuralViolation::RegionMismatch {
            expected: "REGION_DOWNTOWN_CORE".into(),
            found: plan.region.clone(),
        });
    }
    if plan.legs.is_empty() {
        structural.push(StructuralViolation::EmptyPath);
    }

    // Capital and corridor checks
    for leg in &plan.legs {
        let seg_id = leg.segment_id.clone();

        let cap = match ctx.seven_capital_snapshots.get(&seg_id) {
            Some(c) => c,
            None => {
                structural.push(StructuralViolation::MissingSevenCapitalState { segment: seg_id });
                continue;
            }
        };

        // SomaticCapital: avoid segments where somatic headroom is exhausted
        if cap.somatic.rx > 1.0 || cap.somatic.headroom <= 0.0 {
            capital.push(CapitalViolation::SomaticOverBudget {
                segment: seg_id.clone(),
                rx: cap.somatic.rx,
                headroom: cap.somatic.headroom,
            });
        }

        // ThermalCapital: keep heavy freight off highest-heat segments
        if let FreightBand::HeavyTruck = leg.freight_band {
            if cap.thermal.rx > 0.8 {
                capital.push(CapitalViolation::ThermalOverBudget {
                    segment: seg_id.clone(),
                    rx: cap.thermal.rx,
                });
            }
        }

        // TreatyCapital: any negative treaty headroom blocks freight
        if cap.treaty.headroom < 0.0 {
            capital.push(CapitalViolation::TreatyHeadroomNegative {
                segment: seg_id.clone(),
                headroom: cap.treaty.headroom,
            });
        }

        // EquityCapital: reduce freight through highest-risk blocks
        if cap.equity.rx > 0.9 && matches!(leg.freight_band, FreightBand::HeavyTruck) {
            capital.push(CapitalViolation::EquityOverExposure {
                segment: seg_id.clone(),
                rx: cap.equity.rx,
            });
        }

        // Corridor bindings from ALN: static no-go for certain combinations
        if DowntownFreightSegmentBindings::is_heavy_truck_blocked_on_care(&seg_id, leg.freight_band)
        {
            capital.push(CapitalViolation::SomaticRestrictedCorridor {
                segment: seg_id.clone(),
                reason: "CARE_ROUTE_PRIORITY".into(),
            });
        }
        if DowntownFreightSegmentBindings::is_night_quiet_blocked(
            &seg_id,
            leg.freight_band,
            leg.departure_time,
        ) {
            capital.push(CapitalViolation::SomaticRestrictedCorridor {
                segment: seg_id.clone(),
                reason: "NIGHT_QUIET_SENSITIVE".into(),
            });
        }
    }

    // SMART-chain: require appropriate domains and PQ strength
    let domains = &["logistics", "somatic", "thermal", "treaty", "equity"];
    smartchain.extend(
        aletheion_corridors::validate_plan_smartchains(smartchains, domains, &plan.region),
    );

    let status = if structural.is_empty() && capital.is_empty() && smartchain.is_empty() {
        PlanCheckStatus::Safe
    } else {
        PlanCheckStatus::Unsafe
    };

    PlanCheck {
        status,
        structural,
        capital,
        smartchain,
    }
}

/// Top-level orchestration: draft, validate, and recycle freight plans
/// until a corridor- and Somaplex-safe plan is found or we fall back.
pub fn orchestrate_freight_plan(
    vehicle_id: String,
    origin: SegmentId,
    dest: SegmentId,
    band: FreightBand,
    now: DateTime<Local>,
    ctx: &FreightScenarioContext,
    smartchains: &SmartChainRegistry,
) -> (FreightPlan, PlanCheck) {
    let mut attempt = 0usize;
    let max_attempts = 3usize;
    let mut current_band = band;
    let mut current_origin = origin.clone();

    loop {
        attempt += 1;
        let draft = draft_freight_plan(
            vehicle_id.clone(),
            current_origin.clone(),
            dest.clone(),
            current_band,
            now,
        );
        let check = validate_freight_plan(&draft, ctx, smartchains);

        if check.status == PlanCheckStatus::Safe {
            return (draft, check);
        }

        if attempt >= max_attempts {
            // Fall back: return last plan plus violations, upstream orchestrator can redirect
            return (draft, check);
        }

        // Recycling strategy: downgrade band or reroute origin to a safer ring segment
        if matches!(current_band, FreightBand::HeavyTruck) {
            current_band = FreightBand::MediumTruck;
        } else {
            current_origin =
                aletheion_corridors::pick_safer_ring_segment(&current_origin, &ctx.seven_capital_snapshots);
        }
    }
}
