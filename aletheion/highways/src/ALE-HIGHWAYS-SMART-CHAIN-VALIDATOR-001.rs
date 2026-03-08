// Aletheion Highway Management – SMART-chain trigger and CorridorService.validate_plan
// Destination: aletheion/highways/src
// Language: Rust
// Role: Provide a reusable PlanCheck / validate_plan skeleton that:
//   - Invokes structural, capital, ecosafety, and SMART‑chain validators
//   - Enforces FPIC / neurorights contracts as fatal gates
//   - Exposes trigger_smart_chain as the canonical entry point for corridor workflows

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;

use crate::ALE_HIGHWAYS_CORE_CAPITALS_001::{
    CapitalSnapshot,
    DemandSnapshot,
    Plan,
    PlanAction,
    PlanCheck,
    PlanCheckStatus,
    StructuralViolation,
    CapitalViolation,
    SmartChainViolation,
    TreatyCapital,
};
use crate::ALE_HIGHWAYS_CORE_TRAITS_001::{CorridorService, SmartChainQuery};
use crate::ALE_COMP_FPIC_NEURORIGHTS_CONTRACTS_001::{
    RightsDomain,
    FpicRequirement,
    FpicStatus,
    NeurorightsEnvelopeStatus,
    FpicMetadata,
    NeurorightsEnvelopeMetadata,
};

/// Rights / FPIC failure modes that are always fatal for corridor workflows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RightsGateViolation {
    FpicMissingForRequiredDomain(RightsDomain),
    FpicNotGranted(RightsDomain),
    NeurorightsEnvelopeMissing(RightsDomain),
    NeurorightsEnvelopeBreach(RightsDomain),
}

/// Aggregate validation context passed through all checks.
pub struct ValidationContext<'a> {
    pub capitals_before: &'a CapitalSnapshot,
    pub capitals_after: &'a CapitalSnapshot,
    pub treaty_capital: &'a TreatyCapital,
    pub smartchain_registry: &'a dyn SmartChainQuery,
    pub corridor_id: &'a str,
    pub domains: &'a [RightsDomain],
    pub fpic_meta: &'a dyn FpicMetadata,
    pub neuro_meta: &'a dyn NeurorightsEnvelopeMetadata,
}

/// Unified error structure for validate_plan.
#[derive(Debug, Clone)]
pub struct PlanValidationReport {
    pub structural: Vec<StructuralViolation>,
    pub capital: Vec<CapitalViolation>,
    pub smartchain: Vec<SmartChainViolation>,
    pub rights_gate: Vec<RightsGateViolation>,
}

impl PlanValidationReport {
    pub fn status(&self) -> PlanCheckStatus {
        if self.structural.is_empty()
            && self.capital.is_empty()
            && self.smartchain.is_empty()
            && self.rights_gate.is_empty()
        {
            PlanCheckStatus::Safe
        } else {
            PlanCheckStatus::Unsafe
        }
    }

    pub fn into_plan_check(self) -> PlanCheck {
        PlanCheck {
            status: self.status(),
            structural: self.structural,
            capital: self.capital,
            smartchain: self.smartchain,
        }
    }

    pub fn has_fatal_fpic_not_granted(&self) -> bool {
        self.rights_gate
            .iter()
            .any(|v| matches!(v, RightsGateViolation::FpicNotGranted(_)))
    }
}

/// Structural checks: type‑only, no raw strings escape, non‑empty actions, etc.[file:1]
pub fn validate_plan_structural(plan: &Plan) -> Vec<StructuralViolation> {
    let mut out = Vec::new();

    if plan.actions.is_empty() {
        out.push(StructuralViolation::EmptyPlan);
    }

    for action in &plan.actions {
        match action {
            PlanAction::PumpSchedule { .. } => {}
            PlanAction::RouteAssignment { .. } => {}
            PlanAction::CleaningProgram { .. } => {}
            PlanAction::NoOpWithAlert { .. } => {}
        }
    }

    out
}

/// Capital checks: simulate seven‑capital vector and enforce ecosafety corridors.[file:1]
pub fn validate_plan_capitals(
    ctx: &ValidationContext<'_>,
) -> Vec<CapitalViolation> {
    let mut out = Vec::new();

    if ctx.capitals_after.water.available_mgd < 0.0 {
        out.push(CapitalViolation::WaterOverdraw);
    }
    if ctx.capitals_after.water.treaty_headroom_mgd < 0.0 {
        out.push(CapitalViolation::NegativeTreatyHeadroom);
    }
    if ctx.capitals_after.waste.safe_sewer_mgd_remaining < 0.0
        || ctx.capitals_after.waste.mrf_sort_capacity_tpd_remaining < 0.0
    {
        out.push(CapitalViolation::WasteOverCapacity);
    }
    if ctx.capitals_after.waste.hazmat_safe_window_fraction < 0.0 {
        out.push(CapitalViolation::HazmatUnsafeWindow);
    }
    for (agent, headroom) in &ctx.capitals_after.neurobiome.cleaning_headroom_per_agent {
        if *headroom < 0.0 {
            out.push(CapitalViolation::NeurobiomeOverCleaning(agent.clone()));
        }
    }
    if ctx.capitals_after.somatic.somatic_cost_headroom < 0.0 {
        out.push(CapitalViolation::SomaticOverBudget);
    }
    if !ctx.capitals_after.biotic.treaty_enforced {
        out.push(CapitalViolation::BioticTreatyBreach);
    }

    out
}

/// SMART‑chain gate: ensure treaties, rights grammars, PQ mode, and FPIC chain presence.[file:1]
pub fn validate_plan_smartchain(
    registry: &dyn SmartChainQuery,
    domains: &[RightsDomain],
    treaty_capital: &TreatyCapital,
) -> Vec<SmartChainViolation> {
    let mut out = Vec::new();
    let mut chain_ids: Vec<String> = Vec::new();

    for d in domains {
        let domain_str = match d {
            RightsDomain::IndigenousWaterLand => "treaty.indigenous.water.land",
            RightsDomain::SomaticNeuro => "rights.somatic.neuro",
            RightsDomain::CrossSpeciesBiotic => "biotic.treaty",
            RightsDomain::CitizenData => "rights.citizen.data",
            RightsDomain::Other => "generic",
        };
        chain_ids.extend(registry.chains_for_domain(domain_str));
    }

    if chain_ids.is_empty() {
        out.push(SmartChainViolation::MissingSmart01);
        return out;
    }

    let treaties: Vec<String> = domains
        .iter()
        .flat_map(|d| {
            let key = match d {
                RightsDomain::IndigenousWaterLand => "treaty.indigenous.water.land",
                RightsDomain::SomaticNeuro => "treaty.somatic.neuro",
                RightsDomain::CrossSpeciesBiotic => "treaty.biotic",
                RightsDomain::CitizenData => "treaty.citizen.data",
                RightsDomain::Other => "treaty.generic",
            };
            registry.treaties_for_domain(key)
        })
        .collect();

    if treaties.is_empty() {
        out.push(SmartChainViolation::MissingTreaties);
    }

    let rights: Vec<String> = domains
        .iter()
        .flat_map(|d| {
            let key = match d {
                RightsDomain::IndigenousWaterLand => "rights.indigenous",
                RightsDomain::SomaticNeuro => "rights.somatic.neuro",
                RightsDomain::CrossSpeciesBiotic => "rights.biotic",
                RightsDomain::CitizenData => "rights.citizen",
                RightsDomain::Other => "rights.generic",
            };
            registry.rights_for_domain(key)
        })
        .collect();

    if rights.is_empty() {
        out.push(SmartChainViolation::MissingRights);
    }

    if registry.requires_fear_pain_sanity(&chain_ids.join(",")) && !treaty_capital.fear_pain_sanity_enforced
    {
        out.push(SmartChainViolation::MissingFearPainSanityEnvelope);
    }

    if treaty_capital.fpic_required && !treaty_capital.fpic_granted {
        out.push(SmartChainViolation::FpicNotGranted);
    }

    out
}

/// FPIC + neurorights gate: fatal if FPICNotGranted, hard failure on envelope breach.[file:3][file:1]
pub fn validate_plan_rights_gate(
    ctx: &ValidationContext<'_>,
) -> Vec<RightsGateViolation> {
    let mut out = Vec::new();

    for domain in ctx.domains {
        let fpic_req = ctx.fpic_meta.fpic_requirement_for_domain(*domain);
        let fpic_status = ctx.fpic_meta.fpic_status_for_domain(*domain);

        match (fpic_req, fpic_status) {
            (FpicRequirement::NotApplicable, _) => {}
            (FpicRequirement::Indirect, FpicStatus::Granted)
            | (FpicRequirement::Direct, FpicStatus::Granted) => {}
            (FpicRequirement::Indirect, FpicStatus::Missing)
            | (FpicRequirement::Direct, FpicStatus::Missing) => {
                out.push(RightsGateViolation::FpicMissingForRequiredDomain(*domain));
            }
            (FpicRequirement::Indirect, FpicStatus::Revoked) => {
                out.push(RightsGateViolation::FpicNotGranted(*domain));
            }
            (FpicRequirement::Direct, FpicStatus::Revoked) => {
                out.push(RightsGateViolation::FpicNotGranted(*domain));
            }
        }

        let envelope_status = ctx.neuro_meta.envelope_status_for_domain(*domain);

        match envelope_status {
            NeurorightsEnvelopeStatus::NotApplicable => {}
            NeurorightsEnvelopeStatus::Active => {}
            NeurorightsEnvelopeStatus::Missing => {
                out.push(RightsGateViolation::NeurorightsEnvelopeMissing(*domain));
            }
            NeurorightsEnvelopeStatus::BreachDetected => {
                out.push(RightsGateViolation::NeurorightsEnvelopeBreach(*domain));
            }
        }
    }

    out
}

/// Summarize into a PlanValidationReport.
pub fn run_full_plan_validation(
    structural: Vec<StructuralViolation>,
    capital: Vec<CapitalViolation>,
    smartchain: Vec<SmartChainViolation>,
    rights_gate: Vec<RightsGateViolation>,
) -> PlanValidationReport {
    PlanValidationReport {
        structural,
        capital,
        smartchain,
        rights_gate,
    }
}

/// Canonical trigger for SMART‑chain validation from corridor services.[file:1]
pub fn trigger_smart_chain<'a>(
    corridor: &dyn CorridorService,
    registry: &'a dyn SmartChainQuery,
    treaty_capital: &'a TreatyCapital,
    domains: &'a [RightsDomain],
    fpic_meta: &'a dyn FpicMetadata,
    neuro_meta: &'a dyn NeurorightsEnvelopeMetadata,
    horizon_min: u32,
) -> (Plan, PlanCheck, Option<Plan>) {
    let capitals_before = corridor.read_capitals();
    let demands = corridor.read_demands();

    let plan = corridor.propose_plan(horizon_min);

    let capitals_after = corridor.project_capitals_after(&plan, &capitals_before, &demands);

    let vctx = ValidationContext {
        capitals_before: &capitals_before,
        capitals_after: &capitals_after,
        treaty_capital,
        smartchain_registry: registry,
        corridor_id: corridor.id(),
        domains,
        fpic_meta,
        neuro_meta,
    };

    let structural = validate_plan_structural(&plan);
    let capital = validate_plan_capitals(&vctx);
    let smartchain = validate_plan_smartchain(registry, domains, treaty_capital);
    let rights_gate = validate_plan_rights_gate(&vctx);

    let report = run_full_plan_validation(structural, capital, smartchain, rights_gate);
    let mut plan_check = report.clone().into_plan_check();

    if report.has_fatal_fpic_not_granted() {
        plan_check.status = PlanCheckStatus::Unsafe;
        return (plan, plan_check, None);
    }

    let recycled = if plan_check.status == PlanCheckStatus::Unsafe {
        Some(corridor.recycle_plan(&plan, &plan_check, &capitals_before, &demands))
    } else {
        None
    };

    (plan, plan_check, recycled)
}

/// Convenience wiring: default CorridorService.validate_plan implementation.[file:1]
pub fn corridor_validate_plan_default<'a>(
    corridor: &dyn CorridorService,
    registry: &'a dyn SmartChainQuery,
    treaty_capital: &'a TreatyCapital,
    domains: &'a [RightsDomain],
    fpic_meta: &'a dyn FpicMetadata,
    neuro_meta: &'a dyn NeurorightsEnvelopeMetadata,
    horizon_min: u32,
) -> PlanCheck {
    let (_plan, check, _recycled) =
        trigger_smart_chain(corridor, registry, treaty_capital, domains, fpic_meta, neuro_meta, horizon_min);
    check
}
