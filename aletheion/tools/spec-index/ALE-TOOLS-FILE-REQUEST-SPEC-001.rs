//! ALE-TOOLS-FILE-REQUEST-SPEC-001
//! Canonical FileRequestSpec types and ValidationOutcome taxonomy for Aletheion.
//! This module is the shared contract between front-ends, generators, and
//! index-aware validators (including FileRequestSpec::validate_against_index).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Supported generation languages (must match WORKFLOW-INDEX-0001 and CI families).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GenerationLanguage {
    ALN,
    Lua,
    Rust,
    Javascript,
    Kotlin,
    Cpp,
    YAML,
}

/// Seven-capital model used across ERM, SMART-chains, and corridor indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Capital {
    Water,
    Thermal,
    Waste,
    Biotic,
    Somatic,
    Neurobiome,
    Treaty,
}

/// Minimal PQ mode taxonomy aligned with SmartChainRegistry and smartchainvalidator.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PQMode {
    ClassicalOnly,
    HybridPreferred,
    PQStrictRequired,
}

/// Required PQ strength for a chain-domain pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequiredPQ {
    HybridPreferred,
    PQStrictRequired,
}

/// High-level categories for SMART-chain domains (water, biotic, somatic, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SmartDomain {
    Water,
    Thermal,
    Waste,
    Biotic,
    Somatic,
    Neurobiome,
    Treaty,
    Mobility,
    Equity,
}

/// Compliance hooks that map directly into CI / GitHub workflows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceHooks {
    /// Require centralized compliance engine preflight (policy grammar, rights checks).
    pub requires_compliance_preflight: bool,
    /// Require ecosafety grammar preflight (corridors, rx / Vt invariants).
    pub requires_ecosafety_preflight: bool,
    /// Require SMART-chain impact / coverage report in CI.
    pub requires_smart_chain_report: bool,
    /// Optional explicit CI workflow names that must run (language- or domain-specific).
    pub required_ci_workflows: Vec<String>,
}

/// Corridor binding context for this file: where it lives and what it touches.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorridorBinding {
    /// Corridor or treaty tags such as DOWNTOWN_CORE, CANAL_METROCENTER_PARKWAY, etc.
    pub corridor_tags: Vec<String>,
    /// SMART chain IDs that are expected to govern this file.
    pub smart_chain_ids: Vec<String>,
    /// Optional explicit corridor IDs required by ecosafety grammar or highway crate.
    pub required_corridor_ids: Vec<String>,
}

/// Optional research blockers that must be resolved before safe implementation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchBlocker {
    /// Human-readable description of the blocker (e.g. “Arizona microgrid tariff data”).
    pub description: String,
    /// Optional external reference (URL, citation id, legal code, dataset identifier).
    pub reference: Option<String>,
    /// Optional reference to enumerated research actions from the planning docs.
    pub required_research_action_id: Option<String>,
}

/// Canonical specification for a single to-be-generated file in the Aletheion repo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRequestSpec {
    /// Globally unique ALE-* identifier, e.g. ALE-RM-WATER-INGESTION-002.
    pub ale_id: String,
    /// Repo-relative path under the Aletheion root, must be “new, deeper, non-overlapping”.
    pub path: String,
    /// Target implementation language, which also drives CI / workflow family.
    pub language: GenerationLanguage,
    /// Optional reference to one of the canonical workflows (WF-XX-...).
    pub workflow_ref: Option<String>,
    /// Declared capital focus for this file (Water, Thermal, Biotic, etc.).
    pub capital_focus: Vec<Capital>,
    /// Expected SMART chains that will govern this file.
    pub smart_chains: Vec<String>,
    /// Corridor and treaty binding context.
    pub corridor_binding: CorridorBinding,
    /// Optional syntax profile, e.g. “Rust module”, “ALN policy file”.
    pub syntax_profile: Option<String>,
    /// Optional test surface description, e.g. “cargo test”, “aln_compile”.
    pub test_surface: Option<String>,
    /// Optional identifier of a predecessor file in the same workflow family.
    pub predecessor_id: Option<String>,
    /// Optional name of the canonical workflow family (e.g. “AWP_INGESTION_FAMILY”).
    pub workflow_family: Option<String>,
    /// Optional research blockers that must be documented for research-first workflows.
    pub research_blockers: Option<Vec<ResearchBlocker>>,
    /// Compliance hooks declaring which preflights CI must run.
    pub compliance_hooks: ComplianceHooks,
    /// Arbitrary labels for higher-level grouping, analytics, or dashboards.
    pub tags: Vec<String>,
    /// Extension space for future non-breaking additions (keyed metadata).
    #[serde(default)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Normalized / enriched version of FileRequestSpec after validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedFileRequestSpec {
    /// The spec with normalized fields (e.g. inferred workflow_ref, language defaults).
    pub spec: FileRequestSpec,
    /// Any fields the validator inferred or requires to be filled downstream.
    pub inferred_missing_fields: Vec<String>,
}

/// Rule violations represent hard failures; the generator MUST NOT proceed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleViolation {
    /// ALE-ID does not match ^ALE-[A-Z0-9-]+$.
    AleIdInvalidShape { ale_id: String },
    /// ALE-ID is already in use or collides with an existing workflow or file.
    AleIdNotUnique { ale_id: String, existing_path: String },
    /// Path is outside the Aletheion repo root (must start with “aletheion/”).
    PathOutsideRoot { path: String },
    /// Path is too shallow (must be at least 3 segments deep).
    PathTooShallow { path: String, min_segments: usize },
    /// Path collides with an existing ALE-ID or file.
    PathCollision { path: String, existing_ale_id: String },
    /// Referenced workflow does not exist in WORKFLOW-INDEX-0001.
    UnknownWorkflowRef { workflow_ref: String },
    /// Path family does not match the repo anchors for the workflow.
    WorkflowFamilyMismatch {
        workflow_ref: String,
        expected_prefixes: Vec<String>,
        found_path: String,
    },
    /// Capital declared with no SMART-chain coverage.
    MissingSmartChainForCapital { capital: Capital },
    /// SMART-chain PQ mode is too weak for its domains.
    WeakPQMode {
        chain_id: String,
        domain: SmartDomain,
        required: RequiredPQ,
        found: PQMode,
    },
    /// Required treaties are missing for a capital or corridor.
    MissingTreatyForCapital { capital: Capital },
    /// Required rights grammars are missing for somatic / thermal / treaty contexts.
    MissingRightsForCapital { capital: Capital },
    /// Required somatic envelope is missing for somatic / mobility contexts.
    MissingSomaticEnvelope { capital: Capital },
    /// FPIC context is missing for Indigenous / treaty corridors.
    MissingFPICContext { corridor_tag: String },
    /// Required ecosafety corridor / grammar coverage is missing.
    MissingEcosafetyCoverage { path: String, corridor_tag: Option<String> },
    /// Required research blockers not declared for research-first workflows.
    MissingPrerequisiteResearch { workflow_ref: Option<String> },
    /// Workflow reference points outside the known index family (e.g. not in first 25 and no extension index).
    WorkflowNotInCanonicalIndex { workflow_ref: String },
    /// Required index (workflow, SMART-chain, repo, corridor) is unavailable.
    MissingRequiredIndex { index_name: String },
}

/// Rule hints represent soft guidance; they never block generation on their own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleHint {
    /// Suggested workflow(s) based on path, capitals, or SMART-chains.
    SuggestWorkflowRef { likely_workflows: Vec<String> },
    /// Suggested SMART-chains to satisfy a capital.
    SuggestSmartChainForCapital {
        capital: Capital,
        candidates: Vec<String>,
    },
    /// Suggested path prefix family for a workflow.
    SuggestPathFamily {
        workflow_ref: String,
        suggested_prefixes: Vec<String>,
    },
    /// Syntax profile is missing.
    MissingSyntaxProfile,
    /// Test surface is missing.
    MissingTestSurface,
    /// Research blockers should be declared for this workflow.
    MissingResearchBlockers { workflow_ref: Option<String> },
    /// Suggest additional corridor tags based on workflow and capitals.
    SuggestCorridorTags { candidates: Vec<String> },
    /// Suggest additional compliance hooks for critical domains.
    SuggestComplianceHooks { recommended_hooks: Vec<String> },
}

/// Successful validation outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationPass {
    pub normalized_spec: NormalizedFileRequestSpec,
    pub warnings: Vec<RuleHint>,
}

/// Failed validation outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationFailure {
    pub errors: Vec<RuleViolation>,
    pub warnings: Vec<RuleHint>,
}

/// Unified outcome enum (Ok / Err) for index-aware validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationOutcome {
    Ok(ValidationPass),
    Err(ValidationFailure),
}

impl ValidationOutcome {
    /// Convenience helper to check if validation succeeded.
    pub fn is_ok(&self) -> bool {
        matches!(self, ValidationOutcome::Ok(_))
    }

    /// Convenience helper to check if validation failed.
    pub fn is_err(&self) -> bool {
        matches!(self, ValidationOutcome::Err(_))
    }
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuleViolation::*;
        match self {
            AleIdInvalidShape { ale_id } => {
                write!(f, "ALE-ID '{}' does not match ^ALE-[A-Z0-9-]+$", ale_id)
            }
            AleIdNotUnique { ale_id, existing_path } => {
                write!(
                    f,
                    "ALE-ID '{}' is already used by path '{}'",
                    ale_id, existing_path
                )
            }
            PathOutsideRoot { path } => {
                write!(f, "path '{}' must start with 'aletheion/'", path)
            }
            PathTooShallow { path, min_segments } => {
                write!(
                    f,
                    "path '{}' is too shallow; expected at least {} segments",
                    path, min_segments
                )
            }
            PathCollision { path, existing_ale_id } => {
                write!(
                    f,
                    "path '{}' is already claimed by ALE-ID '{}'",
                    path, existing_ale_id
                )
            }
            UnknownWorkflowRef { workflow_ref } => {
                write!(f, "unknown workflow_ref '{}'", workflow_ref)
            }
            WorkflowFamilyMismatch {
                workflow_ref,
                expected_prefixes,
                found_path,
            } => {
                write!(
                    f,
                    "path '{}' does not match expected prefixes {:?} for workflow '{}'",
                    found_path, expected_prefixes, workflow_ref
                )
            }
            MissingSmartChainForCapital { capital } => {
                write!(
                    f,
                    "no SMART-chain covers declared capital '{:?}'",
                    capital
                )
            }
            WeakPQMode {
                chain_id,
                domain,
                required,
                found,
            } => {
                write!(
                    f,
                    "SMART-chain '{}' PQ mode {:?} is weaker than required {:?} for domain {:?}",
                    chain_id, found, required, domain
                )
            }
            MissingTreatyForCapital { capital } => {
                write!(
                    f,
                    "missing treaty coverage for capital '{:?}'",
                    capital
                )
            }
            MissingRightsForCapital { capital } => {
                write!(
                    f,
                    "missing rights grammars for capital '{:?}'",
                    capital
                )
            }
            MissingSomaticEnvelope { capital } => {
                write!(
                    f,
                    "missing somatic envelope for capital '{:?}' or implied mobility context",
                    capital
                )
            }
            MissingFPICContext { corridor_tag } => {
                write!(
                    f,
                    "missing FPIC context for Indigenous / treaty corridor '{}'",
                    corridor_tag
                )
            }
            MissingEcosafetyCoverage { path, corridor_tag } => {
                if let Some(tag) = corridor_tag {
                    write!(
                        f,
                        "missing ecosafety grammar coverage for path '{}' in corridor '{}'",
                        path, tag
                    )
                } else {
                    write!(
                        f,
                        "missing ecosafety grammar coverage for path '{}'",
                        path
                    )
                }
            }
            MissingPrerequisiteResearch { workflow_ref } => {
                if let Some(wf) = workflow_ref {
                    write!(
                        f,
                        "missing prerequisite research_blockers for workflow '{}'",
                        wf
                    )
                } else {
                    write!(f, "missing prerequisite research_blockers for this spec")
                }
            }
            WorkflowNotInCanonicalIndex { workflow_ref } => {
                write!(
                    f,
                    "workflow_ref '{}' is not in canonical WORKFLOW-INDEX-0001 and no extension index is present",
                    workflow_ref
                )
            }
            MissingRequiredIndex { index_name } => {
                write!(f, "required index '{}' is unavailable", index_name)
            }
        }
    }
}
