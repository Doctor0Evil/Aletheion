// Aletheion Compliance Core – FPIC & Neurorights Contracts Surface
// Destination: aletheion/compliance/lexethos
// Language: Rust
// Purpose: Define metadata traits and helper types so any module that
// touches Indigenous territories, biosignals, or recognition logic
// can be statically required to expose FPIC / neurorights contracts
// and be checked by centralized compliance preflight pipelines.

#![forbid(unsafe_code)]

use std::fmt;
use std::time::SystemTime;

/// High-level domain where a module may interact with protected data or rights.
/// These are intentionally broad so they remain stable across subsystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RightsDomain {
    /// Water, land, and corridor operations intersecting Indigenous territories
    /// and associated FPIC / treaty requirements.
    IndigenousWaterLand,
    /// Any deviceless or device-adjacent pattern recognition touching movement,
    /// somatic envelopes, or mental privacy surfaces.
    SomaticNeuro,
    /// Cross-species corridors governed by BioticTreaties and Synthexis envelopes.
    CrossSpeciesBiotic,
    /// General citizen data with consent and privacy primitives.
    CitizenData,
    /// Other domains that still must respect exclusion and rights grammars.
    Other,
}

/// Minimal status enum for FPIC requirements on a module or plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FpicRequirement {
    /// This module never touches Indigenous territories or resources.
    NotApplicable,
    /// This module can touch such territories/resources but only indirectly
    /// (e.g., read-only metrics) and is subject to monitoring rather than
    /// per-actuation FPIC gates.
    Indirect,
    /// This module can directly actuate or materially affect Indigenous
    /// territories/resources and must be gated on FPIC proofs for each plan.
    Direct,
}

/// Minimal representation of whether FPIC has been properly wired for a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FpicContractStatus {
    /// Module declares it is cleanly outside FPIC scope.
    CleanOutsideScope,
    /// Module declares scope and references FPIC hooks/contexts, but central
    /// validators have not yet certified it.
    DeclaredButUnverified,
    /// Module has been verified to use FPIC contexts and hooks appropriately.
    Verified,
}

/// Minimal status for neurorights metadata exposure on a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeurorightsContractStatus {
    /// Module does not touch somatic/neuro domains.
    NotApplicable,
    /// Module touches somatic/neuro domains but has not yet declared envelopes.
    MissingDeclaration,
    /// Module declares FEAR/PAIN/SANITY (CryptoSomatic) envelopes, but has not
    /// yet been verified by the guard workflow.
    DeclaredButUnverified,
    /// Module has been verified against CryptoSomatic / neurorights guardrails.
    Verified,
}

/// Simple code-level view of which FEAR/PAIN/SANITY envelopes a module claims
/// to respect. These are intentionally coarse and are refined by ALN grammars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeurorightsEnvelopes {
    pub fear_enforced: bool,
    pub pain_enforced: bool,
    pub sanity_enforced: bool,
}

/// FPIC-related metadata that a module must expose when it can influence
/// Indigenous territories, waters, corridors, or treaty-bound assets.
pub trait FpicMetadata {
    /// Whether this module can influence Indigenous territories or resources
    /// in any way (direct or indirect).
    fn touches_indigenous_territory(&self) -> bool;

    /// What level of FPIC requirement applies to this module.
    fn fpic_requirement(&self) -> FpicRequirement;

    /// Which rights domain(s) this module participates in.
    fn rights_domains(&self) -> &'static [RightsDomain];

    /// Whether this module embeds or calls out to explicit FPIC protocols,
    /// context references, or corridor micro-treaty bindings.
    fn has_fpic_protocol_reference(&self) -> bool;

    /// High-level status indicating whether FPIC wiring is complete and verified.
    fn fpic_contract_status(&self) -> FpicContractStatus;
}

/// Neurorights / CryptoSomatic-related metadata that a module must expose
/// when it touches FEAR/PAIN/SANITY envelopes, somatic budgets, or mental
/// privacy-adjacent inferences.
pub trait NeurorightsEnvelopeMetadata {
    /// Whether this module operates in somatic / neuro / mental-privacy domains.
    fn touches_somatic_or_neuro_domain(&self) -> bool;

    /// Which rights domains this module participates in, viewed from the
    /// neurorights perspective.
    fn neurorights_domains(&self) -> &'static [RightsDomain];

    /// Whether FEAR/PAIN/SANITY envelopes are declared for all relevant flows.
    fn declares_neurorights_envelopes(&self) -> bool;

    /// Coarse envelope flags that should match the CryptoSomatic / ALN grammar
    /// declarations for this module.
    fn declared_envelopes(&self) -> NeurorightsEnvelopes;

    /// High-level status indicating whether neurorights wiring is complete
    /// and has been verified by a guard workflow.
    fn neurorights_contract_status(&self) -> NeurorightsContractStatus;
}

/// A compact summary that can be emitted by compliance preflight checks and CI
/// workflows to assess whether a module is acceptable for merge/deployment.
#[derive(Debug, Clone)]
pub struct RightsContractSummary {
    pub module_name: String,
    pub fpic_requirement: FpicRequirement,
    pub fpic_status: FpicContractStatus,
    pub neurorights_status: NeurorightsContractStatus,
    pub touches_indigenous: bool,
    pub touches_somatic_neuro: bool,
    pub evaluated_at: SystemTime,
}

impl fmt.Display for RightsContractSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RightsContractSummary[module={:?}, fpic_req={:?}, fpic_status={:?}, neuro_status={:?}, touches_indigenous={}, touches_somatic_neuro={}]",
            self.module_name,
            self.fpic_requirement,
            self.fpic_status,
            self.neurorights_status,
            self.touches_indigenous,
            self.touches_somatic_neuro
        )
    }
}

/// Compliance preflight decision result for FPIC / neurorights checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RightsPreflightDecision {
    /// All required declarations exist and are verified (or intentionally
    /// out-of-scope). Merge / deployment allowed.
    Pass,
    /// Declarations exist but remain unverified; allowed only in explicitly
    /// non-production contexts (e.g., experimental branches).
    WarnOnly,
    /// Required declarations are missing or FPICNotGranted conditions would
    /// be violated; merge / deployment must be blocked.
    Fail,
}

/// Evaluate FPIC and neurorights contracts for a given module metadata view.
/// This function is expected to be called from a centralized compliance
/// preflight runner (e.g., ALE-COMP-CORE-001.rs) and wired into CI.
///
/// Policy (v1, conservative):
/// - If a module touches Indigenous territories and does not expose FPIC
///   references or has status != Verified, mark Fail.
/// - If a module touches somatic/neuro domains and has MissingDeclaration,
///   mark Fail; if DeclaredButUnverified, mark WarnOnly.
pub fn evaluate_rights_contracts<M>(
    module_name: &str,
    m: &M,
) -> (RightsPreflightDecision, RightsContractSummary)
where
    M: FpicMetadata + NeurorightsEnvelopeMetadata,
{
    let now = SystemTime::now();

    let touches_indigenous = m.touches_indigenous_territory();
    let fpic_req = m.fpic_requirement();
    let fpic_status = m.fpic_contract_status();

    let touches_somatic_neuro = m.touches_somatic_or_neuro_domain();
    let neuro_status = m.neurorights_contract_status();

    let mut decision = RightsPreflightDecision::Pass;

    // FPIC policy gate
    if touches_indigenous {
        match (fpic_req, fpic_status, m.has_fpic_protocol_reference()) {
            // Direct FPIC requirement but missing protocol or verification: hard fail.
            (FpicRequirement::Direct, FpicContractStatus::CleanOutsideScope, _)
            | (FpicRequirement::Direct, FpicContractStatus::DeclaredButUnverified, false)
            | (FpicRequirement::Direct, FpicContractStatus::CleanOutsideScope, false) => {
                decision = RightsPreflightDecision::Fail;
            }
            // Direct FPIC requirement, protocol present, but not yet Verified:
            // treat as WarnOnly in non-production; production policy can override.
            (FpicRequirement::Direct, FpicContractStatus::DeclaredButUnverified, true) => {
                if decision != RightsPreflightDecision::Fail {
                    decision = RightsPreflightDecision::WarnOnly;
                }
            }
            // Verified direct FPIC contract is acceptable.
            (FpicRequirement::Direct, FpicContractStatus::Verified, true) => { /* ok */ }

            // Indirect FPIC modules must at least declare and reference FPIC;
            // otherwise we treat as WarnOnly to prompt remediation.
            (FpicRequirement::Indirect, FpicContractStatus::CleanOutsideScope, _)
            | (FpicRequirement::Indirect, FpicContractStatus::DeclaredButUnverified, false) => {
                if decision != RightsPreflightDecision::Fail {
                    decision = RightsPreflightDecision::WarnOnly;
                }
            }
            _ => { /* other combinations are acceptable */ }
        }
    }

    // Neurorights policy gate
    if touches_somatic_neuro {
        match neuro_status {
            NeurorightsContractStatus::NotApplicable | NeurorightsContractStatus::Verified => { /* ok */ }
            NeurorightsContractStatus::MissingDeclaration => {
                // Operating in somatic/neuro space without envelopes is not allowed.
                decision = RightsPreflightDecision::Fail;
            }
            NeurorightsContractStatus::DeclaredButUnverified => {
                if decision != RightsPreflightDecision::Fail {
                    decision = RightsPreflightDecision::WarnOnly;
                }
            }
        }
    }

    let summary = RightsContractSummary {
        module_name: module_name.to_string(),
        fpic_requirement: fpic_req,
        fpic_status,
        neurorights_status: neuro_status,
        touches_indigenous,
        touches_somatic_neuro,
        evaluated_at: now,
    };

    (decision, summary)
}

// ----- Example stub implementations for wiring and tests -----

/// Example module metadata struct that a concrete service (e.g., corridor
/// planner, Somaplex route engine, or recognition module) can use to expose
/// its FPIC and neurorights declarations without coupling to internals.
#[derive(Debug, Clone)]
pub struct ModuleRightsDescriptor {
    pub module_name: String,
    pub rights_domains: &'static [RightsDomain],
    pub fpic_requirement: FpicRequirement,
    pub fpic_status: FpicContractStatus,
    pub has_fpic_reference: bool,
    pub neurorights_status: NeurorightsContractStatus,
    pub neurorights_envelopes: NeurorightsEnvelopes,
}

impl FpicMetadata for ModuleRightsDescriptor {
    fn touches_indigenous_territory(&self) -> bool {
        self.rights_domains
            .iter()
            .any(|d| *d == RightsDomain::IndigenousWaterLand)
    }

    fn fpic_requirement(&self) -> FpicRequirement {
        self.fpic_requirement
    }

    fn rights_domains(&self) -> &'static [RightsDomain] {
        self.rights_domains
    }

    fn has_fpic_protocol_reference(&self) -> bool {
        self.has_fpic_reference
    }

    fn fpic_contract_status(&self) -> FpicContractStatus {
        self.fpic_status
    }
}

impl NeurorightsEnvelopeMetadata for ModuleRightsDescriptor {
    fn touches_somatic_or_neuro_domain(&self) -> bool {
        self.rights_domains
            .iter()
            .any(|d| *d == RightsDomain::SomaticNeuro)
    }

    fn neurorights_domains(&self) -> &'static [RightsDomain] {
        self.rights_domains
    }

    fn declares_neurorights_envelopes(&self) -> bool {
        matches!(
            self.neurorights_status,
            NeurorightsContractStatus::DeclaredButUnverified
                | NeurorightsContractStatus::Verified
        )
    }

    fn declared_envelopes(&self) -> NeurorightsEnvelopes {
        self.neurorights_envelopes
    }

    fn neurorights_contract_status(&self) -> NeurorightsContractStatus {
        self.neurorights_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDIGENOUS_ONLY: &[RightsDomain] = &[RightsDomain::IndigenousWaterLand];
    const SOMATIC_ONLY: &[RightsDomain] = &[RightsDomain::SomaticNeuro];

    #[test]
    fn fail_when_direct_fpic_missing_protocol() {
        let desc = ModuleRightsDescriptor {
            module_name: "corridor_water_actuator".to_string(),
            rights_domains: INDIGENOUS_ONLY,
            fpic_requirement: FpicRequirement::Direct,
            fpic_status: FpicContractStatus::CleanOutsideScope,
            has_fpic_reference: false,
            neurorights_status: NeurorightsContractStatus::NotApplicable,
            neurorights_envelopes: NeurorightsEnvelopes {
                fear_enforced: false,
                pain_enforced: false,
                sanity_enforced: false,
            },
        };

        let (decision, _summary) = evaluate_rights_contracts(&desc.module_name, &desc);
        assert_eq!(decision, RightsPreflightDecision::Fail);
    }

    #[test]
    fn warn_when_direct_fpic_declared_not_verified() {
        let desc = ModuleRightsDescriptor {
            module_name: "somaplex_corridor_planner".to_string(),
            rights_domains: INDIGENOUS_ONLY,
            fpic_requirement: FpicRequirement::Direct,
            fpic_status: FpicContractStatus::DeclaredButUnverified,
            has_fpic_reference: true,
            neurorights_status: NeurorightsContractStatus::NotApplicable,
            neurorights_envelopes: NeurorightsEnvelopes {
                fear_enforced: false,
                pain_enforced: false,
                sanity_enforced: false,
            },
        };

        let (decision, _summary) = evaluate_rights_contracts(&desc.module_name, &desc);
        assert_eq!(decision, RightsPreflightDecision::WarnOnly);
    }

    #[test]
    fn fail_when_neuro_missing_declaration() {
        let desc = ModuleRightsDescriptor {
            module_name: "deviceless_pattern_recognizer".to_string(),
            rights_domains: SOMATIC_ONLY,
            fpic_requirement: FpicRequirement::NotApplicable,
            fpic_status: FpicContractStatus::CleanOutsideScope,
            has_fpic_reference: false,
            neurorights_status: NeurorightsContractStatus::MissingDeclaration,
            neurorights_envelopes: NeurorightsEnvelopes {
                fear_enforced: false,
                pain_enforced: false,
                sanity_enforced: false,
            },
        };

        let (decision, _summary) = evaluate_rights_contracts(&desc.module_name, &desc);
        assert_eq!(decision, RightsPreflightDecision::Fail);
    }

    #[test]
    fn pass_when_contracts_verified() {
        let desc = ModuleRightsDescriptor {
            module_name: "smart_chain_corridor_validator".to_string(),
            rights_domains: &[RightsDomain::IndigenousWaterLand, RightsDomain::SomaticNeuro],
            fpic_requirement: FpicRequirement::Direct,
            fpic_status: FpicContractStatus::Verified,
            has_fpic_reference: true,
            neurorights_status: NeurorightsContractStatus::Verified,
            neurorights_envelopes: NeurorightsEnvelopes {
                fear_enforced: true,
                pain_enforced: true,
                sanity_enforced: true,
            },
        };

        let (decision, _summary) = evaluate_rights_contracts(&desc.module_name, &desc);
        assert_eq!(decision, RightsPreflightDecision::Pass);
    }
}
