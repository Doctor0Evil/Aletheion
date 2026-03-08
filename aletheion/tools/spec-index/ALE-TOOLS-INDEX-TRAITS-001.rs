//! ALE-TOOLS-INDEX-TRAITS-001
//! Trait surfaces for WorkflowIndex, SmartChainRegistry, and RepoIndex.
//!
//! These traits are the minimal contracts that FileRequestSpec::validate_against_index
//! depends on. They are intentionally narrow, read-only, and index-focused, so they
//! can be implemented by in-memory stubs, ALN-backed registries, or filesystem
//! scanners without changing the calling code.
//!
//! This module does not encode Phoenix-specific logic; it only exposes the
//! invariants and lookups needed to decide whether a requested file is coherent
//! with the existing workflow, SMART-chain, and repository structure.

use std::collections::{BTreeMap, BTreeSet};

/// Canonical identifier for a high-level workflow (from WORKFLOW-INDEX-0001).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowId(pub String);

/// Canonical identifier for a SMART chain (from SMART-HIERARCHY-CHAINS-001).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SmartChainId(pub String);

/// Canonical identifier for a repository path anchor (directory or file).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoPath(pub String);

/// ERM layer shorthand used in the workflow index (L1–L5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErmLayer {
    L1,
    L2,
    L3,
    L4,
    L5,
}

/// Domain tags mirrored from the workflow index and SMART-chain registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DomainTag {
    Water,
    Thermal,
    Waste,
    Biotic,
    Somatic,
    Neurobiome,
    Treaty,
    Mobility,
    Equity,
    Governance,
    Security,
    Materials,
}

/// Minimal view of a workflow’s metadata needed for validation.
#[derive(Debug, Clone)]
pub struct WorkflowMeta {
    /// Human-readable name; not used for logic, only for diagnostics.
    pub name: String,
    /// ERM layers this workflow touches.
    pub erm_layers: BTreeSet<ErmLayer>,
    /// Primary domains (water, thermal, etc.).
    pub domains: BTreeSet<DomainTag>,
    /// Existing repo anchors this workflow already owns.
    pub repo_anchors: BTreeSet<RepoPath>,
    /// SMART chains that this workflow is declared to participate in.
    pub smart_chain_ids: BTreeSet<SmartChainId>,
}

/// Minimal view of a SMART chain used during file validation.
#[derive(Debug, Clone)]
pub struct SmartChainMeta {
    /// Declarative scope of the chain.
    pub domains: BTreeSet<DomainTag>,
    pub layers: BTreeSet<ErmLayer>,
    /// Whether rollback is globally forbidden on this chain.
    pub rollback_forbidden: bool,
    /// Whether FEAR/PAIN/SANITY envelopes must be enforced.
    pub requires_fear_pain_sanity: bool,
    /// Whether Biotic/Indigenous treaties must be bound for this chain.
    pub requires_treaties: bool,
    /// Whether LexEthos rights grammars must be present.
    pub requires_rights_grammars: bool,
}

/// Minimal view of a repo index for path and duplication checks.
#[derive(Debug, Clone)]
pub struct RepoEntryMeta {
    /// True if the path already exists on disk.
    pub exists: bool,
    /// True if the path is already claimed in a workflow index anchor.
    pub reserved_by_index: bool,
    /// Optional owning workflow, if known.
    pub owner_workflow: Option<WorkflowId>,
}

/// Trait: read-only access to the workflow index.
pub trait WorkflowIndex {
    /// Fetch workflow metadata by id.
    fn get_workflow(&self, id: &WorkflowId) -> Option<WorkflowMeta>;

    /// List all workflows that touch a given domain.
    fn workflows_for_domain(&self, domain: DomainTag) -> Vec<WorkflowId>;

    /// List all workflows that attach to a given SMART chain.
    fn workflows_for_chain(&self, chain_id: &SmartChainId) -> Vec<WorkflowId>;

    /// Return all repo anchors (directories and files) already claimed in the index.
    fn all_repo_anchors(&self) -> BTreeSet<RepoPath>;
}

/// Trait: thin facade over the SMART-chain registry used at tooling level.
///
/// This is intentionally a subset of the full smartchainvalidator machinery:
/// we only need to know whether a chain exists and what invariants it carries.
pub trait SmartChainRegistryView {
    /// Return metadata for a specific SMART chain by id.
    fn get_chain(&self, id: &SmartChainId) -> Option<SmartChainMeta>;

    /// List all chains that touch a given domain.
    fn chains_for_domain(&self, domain: DomainTag) -> Vec<SmartChainId>;

    /// Return all SMART chain ids, for diagnostics and completeness checks.
    fn all_chain_ids(&self) -> BTreeSet<SmartChainId>;
}

/// Trait: repository index for path-level validation.
///
/// Implementations can be backed by a live filesystem scan, a cached manifest,
/// or a Git index snapshot. FileRequestSpec does not assume any particular
/// storage model.
pub trait RepoIndex {
    /// Inspect a path and return existence and ownership metadata.
    fn inspect_path(&self, path: &RepoPath) -> RepoEntryMeta;

    /// List all known child entries under a directory prefix, if any.
    fn list_under(&self, prefix: &RepoPath) -> BTreeMap<RepoPath, RepoEntryMeta>;

    /// Check whether any existing path already uses the same ALE- prefix.
    ///
    /// This is used to enforce "no duplicate ALE- IDs" and to push new files
    /// into deeper directories when siblings exist.
    fn ale_id_in_use(&self, ale_prefix: &str) -> bool;
}
