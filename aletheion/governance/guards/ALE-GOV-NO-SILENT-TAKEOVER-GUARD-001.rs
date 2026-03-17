// aletheion/governance/guards/ALE-GOV-NO-SILENT-TAKEOVER-GUARD-001.rs
// Guard library to prevent silent takeovers or silent reinvention of governance and trust roles.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoPath(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoleId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId(pub String);

// Minimal manifest view used by the guard.
#[derive(Debug, Clone)]
pub struct WorkflowManifest {
    pub module_id: ModuleId,
    pub repo_path: RepoPath,
    pub declared_roles: Vec<RoleBinding>,
    pub capabilities: Vec<CapabilityBinding>,
    pub imports: Vec<String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RoleBinding {
    pub role_id: RoleId,
    pub principal: String, // DID or logical actor
}

#[derive(Debug, Clone)]
pub struct CapabilityBinding {
    pub principal: String, // DID or logical actor
    pub capability: CapabilityId,
    pub scope: String,     // e.g., "Water", "Governance", "Actuation"
}

// Canonical identifiers
pub const GOOGOLSWARM_LOCAL_ATTESTOR_ID: &str = "aletheion:attestor:googolswarm:local-only:v1";

pub const CAPABILITY_ATTEST: &str = "Attest";
pub const CAPABILITY_GOVERN: &str = "Govern";
pub const CAPABILITY_OPTIMIZE: &str = "Optimize";
pub const CAPABILITY_ACTUATE: &str = "Actuate";

// Required imports indicating that governance envelopes are respected.
const REQUIRED_GOV_IMPORTS: &[&str] = &[
    "ALE-GOV-RIGHTS-TREATY-GRAMMAR-001",
    "ALE-GOV-BIRTH-SIGN-MODEL-001",
    "ALE-TRUST-GOVERNED-DECISION-TX-001",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardViolationKind {
    GoogolswarmRoleEscalation,
    GoogolswarmCapabilityEscalation,
    HiddenGovernanceModule,
    MissingGovernanceEnvelopeImports,
    BirthSignBypass,
}

#[derive(Debug, Clone)]
pub struct GuardViolation {
    pub kind: GuardViolationKind,
    pub message: String,
    pub module_id: ModuleId,
    pub repo_path: RepoPath,
}

#[derive(Debug, Clone)]
pub struct GuardReport {
    pub violations: Vec<GuardViolation>,
}

impl GuardReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

// Entry point: scan a batch of manifests before merge or deployment.
pub fn check_no_silent_takeover(manifests: &[WorkflowManifest]) -> GuardReport {
    let mut violations = Vec::new();

    for m in manifests {
        violations.extend(check_manifest(m));
    }

    GuardReport { violations }
}

fn check_manifest(m: &WorkflowManifest) -> Vec<GuardViolation> {
    let mut v = Vec::new();
    v.extend(check_googolswarm_role_escalation(m));
    v.extend(check_googolswarm_capabilities(m));
    v.extend(check_hidden_governance(m));
    v.extend(check_governance_imports(m));
    v.extend(check_birthsign_bypass(m));
    v
}

fn violation(m: &WorkflowManifest, kind: GuardViolationKind, msg: &str) -> GuardViolation {
    GuardViolation {
        kind,
        message: msg.to_string(),
        module_id: m.module_id.clone(),
        repo_path: m.repo_path.clone(),
    }
}

// 1. Googolswarm must never hold governance/controller roles.
fn check_googolswarm_role_escalation(m: &WorkflowManifest) -> Vec<GuardViolation> {
    let mut v = Vec::new();
    for rb in &m.declared_roles {
        if rb.principal == GOOGOLSWARM_LOCAL_ATTESTOR_ID {
            let lowered = rb.role_id.0.to_ascii_lowercase();
            if lowered.contains("governor") || lowered.contains("controller") || lowered.contains("owner") {
                v.push(violation(
                    m,
                    GuardViolationKind::GoogolswarmRoleEscalation,
                    "Googolswarm attestor must not be bound to governance/controller/owner roles.",
                ));
            }
        }
    }
    v
}

// 2. Googolswarm may only ever claim Attest + AuditView-style capabilities.
fn check_googolswarm_capabilities(m: &WorkflowManifest) -> Vec<GuardViolation> {
    let mut v = Vec::new();
    for cb in &m.capabilities {
        if cb.principal == GOOGOLSWARM_LOCAL_ATTESTOR_ID {
            let cap = cb.capability.0.as_str();
            if cap == CAPABILITY_GOVERN || cap == CAPABILITY_OPTIMIZE || cap == CAPABILITY_ACTUATE {
                v.push(violation(
                    m,
                    GuardViolationKind::GoogolswarmCapabilityEscalation,
                    "Googolswarm attestor cannot be granted Govern/Optimize/Actuate capabilities.",
                ));
            }
        }
    }
    v
}

// 3. Hidden governance modules: governance must not live in non-governance paths.
fn check_hidden_governance(m: &WorkflowManifest) -> Vec<GuardViolation> {
    let mut v = Vec::new();
    let path = m.repo_path.0.to_ascii_lowercase();
    let looks_governancey = |s: &str| {
        let l = s.to_ascii_lowercase();
        l.contains("policy") || l.contains("treaty") || l.contains("rights") || l.contains("govern")
    };

    let mut has_governance_annotation = false;
    for (k, val) in &m.annotations {
        if looks_governancey(k) || looks_governancey(val) {
            has_governance_annotation = true;
            break;
        }
    }

    let is_governance_tree = path.contains("/governance/") || path.contains("/trust/") || path.contains("/compliance/");

    if has_governance_annotation && !is_governance_tree {
        v.push(violation(
            m,
            GuardViolationKind::HiddenGovernanceModule,
            "Governance/treaty/rights logic must live under governance/trust/compliance trees, not hidden in other dirs.",
        ));
    }

    v
}

// 4. Require canonical governance envelopes to be imported when emitting decisions.
fn check_governance_imports(m: &WorkflowManifest) -> Vec<GuardViolation> {
    let mut v = Vec::new();
    let path = m.repo_path.0.to_ascii_lowercase();

    // Only apply to modules that touch trust/governance.
    if !(path.contains("/trust/") || path.contains("/governance/") || path.contains("decision") || path.contains("tx")) {
        return v;
    }

    let import_set: HashSet<String> = m.imports.iter().cloned().collect();
    let mut missing = Vec::new();
    for req in REQUIRED_GOV_IMPORTS {
        if !import_set.iter().any(|i| i.contains(req)) {
            missing.push(*req);
        }
    }
    if !missing.is_empty() {
        v.push(violation(
            m,
            GuardViolationKind::MissingGovernanceEnvelopeImports,
            &format!(
                "Module emits governed decisions but is missing required governance imports: {:?}",
                missing
            ),
        ));
    }
    v
}

// 5. BirthSign bypass detection – any module that touches land/water/air/biosignals must declare BirthSign usage.
fn check_birthsign_bypass(m: &WorkflowManifest) -> Vec<GuardViolation> {
    let mut v = Vec::new();

    let mut high_risk_scope = false;
    for cb in &m.capabilities {
        let s = cb.scope.to_ascii_lowercase();
        if s.contains("land")
            || s.contains("water")
            || s.contains("air")
            || s.contains("biosignal")
            || s.contains("augmentation")
            || s.contains("mobility")
        {
            high_risk_scope = true;
            break;
        }
    }
    if !high_risk_scope {
        return v;
    }

    let mut has_birthsign_annotation = false;
    for (k, val) in &m.annotations {
        let k_l = k.to_ascii_lowercase();
        let v_l = val.to_ascii_lowercase();
        if k_l.contains("birthsign") || v_l.contains("birthsign") {
            has_birthsign_annotation = true;
            break;
        }
    }

    if !has_birthsign_annotation {
        v.push(violation(
            m,
            GuardViolationKind::BirthSignBypass,
            "High-risk module (land/water/air/biosignals/augmentation/mobility) missing explicit BirthSign bindings.",
        ));
    }

    v
}

// Convenience: panic on any violation; intended for CI binaries.
pub fn assert_no_silent_takeover(manifests: &[WorkflowManifest]) {
    let report = check_no_silent_takeover(manifests);
    if !report.is_clean() {
        eprintln!("Silent-takeover guard violations detected:");
        for v in &report.violations {
            eprintln!(
                "- [{:?}] module={} path={} msg={}",
                v.kind,
                (v.module_id).0,
                (v.repo_path).0,
                v.message
            );
        }
        panic!("ALE-GOV-NO-SILENT-TAKEOVER-GUARD-001.rs: refusing to continue.");
    }
}
