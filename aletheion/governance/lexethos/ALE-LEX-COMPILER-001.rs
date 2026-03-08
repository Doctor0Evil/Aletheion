// Aletheion LexEthos Rights Grammar Compiler
// File: ALE-LEX-COMPILER-001.rs
// Path: aletheion/governance/lexethos/ALE-LEX-COMPILER-001.rs
//
// Role:
//   - Load ALN-based rights grammars (e.g. ALE-LEX-RIGHTS-GRAMMAR-001.aln)
//   - Validate structural correctness of RightsAtom / MicroTreaty definitions
//   - Compile them into an internal representation ready for MicroTreatyEngine
//   - Emit compact, queryable bundles keyed by DistrictId / CorridorId
//
// Notes:
//   - No blockchain or verification logic here (that belongs to MicroTreatyEngine).
//   - No dispute cooling; this stays in the compilation + static validation layer.
//   - Designed for energy-efficient, per-line interpretable patterns.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::time::SystemTime;

// ---- Core mirrored types (aligned with ALE-LEX-RIGHTS-GRAMMAR-001.aln) ----

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RightType {
    NoiseQuietHours,
    NoisePeakCeiling,
    ShadeWaitingArea,
    ShadeTransitPeakHeat,
    ShadeCrossingRefuge,
    WaterAccessBasic,
    AugmentationBoundary,
    DataPrivacyLocal,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MetricKind {
    DbaAtFacade,
    DbaAtLotLine,
    LuxHorizontal,
    LuxVertical,
    ShadeFraction,
    ShadeLengthM,
    DurationMinutes,
    DistanceM,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Comparator {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnforcementMode {
    HardBlock,
    Derate,
    WarnOnly,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CapitalTag {
    Water,
    Thermal,
    Waste,
    Biotic,
    Neurobiome,
    Somatic,
    Treaty,
    Unknown(String),
}

pub type RightId = String;
pub type TreatyId = String;
pub type DistrictId = String;
pub type CorridorId = String;
pub type AssetType = String;
pub type Did = String;
pub type GoogolswarmTxHash = String;

#[derive(Clone, Debug)]
pub struct TimeScope {
    pub start_time: String,      // "HH:MM"
    pub end_time: String,        // "HH:MM"
    pub days_of_week: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SeasonalWindow {
    pub start_date: String,      // "YYYY-MM-DD"
    pub end_date: String,
}

#[derive(Clone, Debug)]
pub struct Jurisdiction {
    pub districts: Vec<DistrictId>,
    pub corridors: Vec<CorridorId>,
    pub asset_types: Vec<AssetType>,
}

#[derive(Clone, Debug)]
pub struct Threshold {
    pub metric: MetricKind,
    pub comparator: Comparator,
    pub value: f64,
}

#[derive(Clone, Debug)]
pub struct RightsAtom {
    pub right_id: RightId,
    pub right_type: RightType,
    pub subject_role: String,
    pub obligor_role: String,

    pub jurisdiction: Jurisdiction,
    pub time_scope: TimeScope,
    pub season: Option<SeasonalWindow>,

    pub threshold: Threshold,
    pub enforcement: EnforcementMode,
    pub capital_links: Vec<CapitalTag>,

    pub provenance_did_issuer: Did,
    pub provenance_tx_anchor: GoogolswarmTxHash,
    pub version: String,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub struct MicroTreaty {
    pub treaty_id: TreatyId,
    pub name: String,
    pub description: String,
    pub atoms: Vec<RightsAtom>,

    pub valid_from: String,
    pub valid_until: String,

    pub jurisdictions: Vec<Jurisdiction>,
    pub supersedes: Vec<TreatyId>,
    pub authors: Vec<Did>,
    pub reviewers: Vec<Did>,
    pub version: String,
}

// ---- Internal compiled forms ----

/// Narrowed, query-optimized view of a RightsAtom for runtime engines.
#[derive(Clone, Debug)]
pub struct CompiledAtom {
    pub right_id: RightId,
    pub right_type: RightType,
    pub metric: MetricKind,
    pub comparator: Comparator,
    pub value: f64,
    pub enforcement: EnforcementMode,

    pub districts: BTreeSet<DistrictId>,
    pub corridors: BTreeSet<CorridorId>,
    pub asset_types: BTreeSet<AssetType>,

    pub capital_links: BTreeSet<CapitalTag>,
}

/// Bundle of compiled atoms per treaty with precomputed indices.
#[derive(Clone, Debug)]
pub struct CompiledTreaty {
    pub treaty_id: TreatyId,
    pub version: String,
    pub atoms: Vec<CompiledAtom>,
}

/// Index keyed by district and corridor, used by MicroTreatyVerifier.
#[derive(Clone, Debug, Default)]
pub struct TreatyIndex {
    pub by_district: BTreeMap<DistrictId, Vec<CompiledAtom>>,
    pub by_corridor: BTreeMap<CorridorId, Vec<CompiledAtom>>,
}

/// Compilation diagnostics: used by CI and governance review.
#[derive(Clone, Debug, Default)]
pub struct CompileDiagnostics {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CompileDiagnostics {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

// ---- ALN input abstraction ----

/// Minimal abstraction over an ALN document loaded from disk or memory.
/// In the real repo, an ALN parser crate or FFI bridge populates this.
pub trait AlnDocument {
    fn source_id(&self) -> &str;
    fn list_right_atoms(&self) -> Vec<RightsAtom>;
    fn list_micro_treaties(&self) -> Vec<MicroTreaty>;
}

// ---- Compiler core ----

pub struct RightsGrammarCompiler;

impl RightsGrammarCompiler {
    /// Top-level compile: ALN document -> compiled treaties + index + diagnostics.
    pub fn compile<D: AlnDocument>(
        doc: &D,
    ) -> (Vec<CompiledTreaty>, TreatyIndex, CompileDiagnostics) {
        let mut diags = CompileDiagnostics::default();
        let treaties = doc.list_micro_treaties();
        let mut compiled_treaties = Vec::with_capacity(treaties.len());
        let mut index = TreatyIndex::default();

        for mt in treaties {
            match Self::compile_treaty(&mt, &mut diags) {
                Some(ct) => {
                    Self::index_treaty(&ct, &mut index);
                    compiled_treaties.push(ct);
                }
                None => {
                    diags
                        .warnings
                        .push(format!("Skipped treaty {} due to previous errors", mt.treaty_id));
                }
            }
        }

        (compiled_treaties, index, diags)
    }

    fn compile_treaty(mt: &MicroTreaty, diags: &mut CompileDiagnostics) -> Option<CompiledTreaty> {
        let mut compiled_atoms = Vec::with_capacity(mt.atoms.len());

        for atom in &mt.atoms {
            match Self::compile_atom(atom, diags) {
                Some(ca) => compiled_atoms.push(ca),
                None => diags
                    .warnings
                    .push(format!("Skipped atom {} in treaty {}", atom.right_id, mt.treaty_id)),
            }
        }

        if compiled_atoms.is_empty() {
            diags
                .errors
                .push(format!("Treaty {} has no valid atoms", mt.treaty_id));
            return None;
        }

        Some(CompiledTreaty {
            treaty_id: mt.treaty_id.clone(),
            version: mt.version.clone(),
            atoms: compiled_atoms,
        })
    }

    fn compile_atom(atom: &RightsAtom, diags: &mut CompileDiagnostics) -> Option<CompiledAtom> {
        // Basic structural checks.
        if atom.right_id.trim().is_empty() {
            diags.errors.push("RightsAtom missing right_id".to_string());
            return None;
        }
        if atom.jurisdiction.districts.is_empty()
            && atom.jurisdiction.corridors.is_empty()
            && atom.jurisdiction.asset_types.is_empty()
        {
            diags.errors.push(format!(
                "RightsAtom {} has empty jurisdiction",
                atom.right_id
            ));
            return None;
        }

        if !Self::is_valid_timescope(&atom.time_scope) {
            diags.errors.push(format!(
                "RightsAtom {} has invalid time_scope",
                atom.right_id
            ));
            return None;
        }

        let compiled = CompiledAtom {
            right_id: atom.right_id.clone(),
            right_type: atom.right_type.clone(),
            metric: atom.threshold.metric.clone(),
            comparator: atom.threshold.comparator.clone(),
            value: atom.threshold.value,
            enforcement: atom.enforcement.clone(),
            districts: atom
                .jurisdiction
                .districts
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            corridors: atom
                .jurisdiction
                .corridors
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            asset_types: atom
                .jurisdiction
                .asset_types
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            capital_links: atom
                .capital_links
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
        };

        Some(compiled)
    }

    fn index_treaty(ct: &CompiledTreaty, index: &mut TreatyIndex) {
        for atom in &ct.atoms {
            for d in &atom.districts {
                index
                    .by_district
                    .entry(d.clone())
                    .or_default()
                    .push(atom.clone());
            }
            for c in &atom.corridors {
                index
                    .by_corridor
                    .entry(c.clone())
                    .or_default()
                    .push(atom.clone());
            }
        }
    }

    fn is_valid_timescope(ts: &TimeScope) -> bool {
        // Minimal sanity checks; deeper chrono parsing can be added later.
        if ts.start_time.len() != 5 || ts.end_time.len() != 5 {
            return false;
        }
        if !ts.start_time.chars().nth(2).map(|c| c == ':').unwrap_or(false) {
            return false;
        }
        if !ts.end_time.chars().nth(2).map(|c| c == ':').unwrap_or(false) {
            return false;
        }
        true
    }
}

// ---- Convenience loaders and stubs ----

/// A minimal in-memory ALN document implementation for early wiring.
/// In practice, this will be replaced by a proper ALN parser + loader.
pub struct InMemoryAlnDoc {
    pub source: String,
    pub atoms: Vec<RightsAtom>,
    pub treaties: Vec<MicroTreaty>,
}

impl AlnDocument for InMemoryAlnDoc {
    fn source_id(&self) -> &str {
        &self.source
    }

    fn list_right_atoms(&self) -> Vec<RightsAtom> {
        self.atoms.clone()
    }

    fn list_micro_treaties(&self) -> Vec<MicroTreaty> {
        self.treaties.clone()
    }
}

// Simple metadata helper for CI and audit logging.
pub fn compiler_build_fingerprint() -> String {
    let now = SystemTime::now();
    format!("ALE-LEX-COMPILER-001::{:?}", now)
}

// ---- Display helpers ----

impl fmt::Display for CompiledAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CompiledAtom {{ right_id: {}, metric: {:?}, comparator: {:?}, value: {}, enforcement: {:?} }}",
            self.right_id, self.metric, self.comparator, self.value, self.enforcement
        )
    }
}

impl fmt::Display for CompileDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CompileDiagnostics:")?;
        for e in &self.errors {
            writeln!(f, "  ERROR: {}", e)?;
        }
        for w in &self.warnings {
            writeln!(f, "  WARN:  {}", w)?;
        }
        Ok(())
    }
}
