// Role:
// Tiny CI helper that:
// 1. Parses ALE-ERM-WATER-CANAL-FUNNEL-REQUIREMENTS-001.aln
// 2. Scans the repo tree for Rust/ALN modules under ERM/INF water+canal paths
// 3. Emits concrete pass/fail signals suitable for CI:
//
//    - Exit code 0: all targeted modules satisfy funnel requirements
//    - Exit code 1: at least one violation found
//
// Assumptions:
// - The ALN funnel spec file is available at:
//   aletheionermworkflow-index/ALE-ERM-WATER-CANAL-FUNNEL-REQUIREMENTS-001.aln
// - The repo checkout is the current working directory when this binary runs
// - CI wiring (e.g., in .github/workflows/...) will just `cargo run -p <crate>`
//   or `cargo test --test canal_funnel_check` depending on integration style.
// - This module stays self-contained and relies only on std + serde for ALN
//   parsing and WalkDir-like traversal logic implemented manually to avoid
//   extra dependencies in the core ERM stack.

#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::Deserialize;

// -----------------------------
// Minimal ALN model (subset)
// -----------------------------

// We treat the funnel ALN file as a simple, line-oriented spec with sections
// describing required imports and symbols for MAR and canal modules. The exact
// grammar of ALN is richer, but for this CI helper we parse just enough to
// extract the following concepts:
//
// - For MAR modules:
//     * required_imports = ["ALE-ERM-ECOSAFETY-TYPES-001.rs",
//                          "ALE-ERM-ECOSAFETY-CONTRACTS-001.rs"]
//     * required_symbols = ["CyboquaticNodeEcosafety",
//                           "requirecorridors",
//                           "evalcorridor",
//                           "decidenodeaction"]
//
// - For canal modules:
//     * required_imports = ["ALE-ERM-ECOSAFETY-TYPES-001.rs"]
//     * required_symbols = ["CyboquaticNodeEcosafety",
//                           "evalcorridor",
//                           "decidenodeaction"]
//
// The actual ALE-ERM-WATER-CANAL-FUNNEL-REQUIREMENTS-001.aln produced earlier
// encodes this semantically via RULEs like:
//   ecosafety_corridors_required_for_mar
//   ecosafety_corridors_required_for_canals
//
// Here we mirror those requirements as a hard-coded baseline, and allow ALN
// to override/extend via a minimal key=value format if needed.

#[derive(Debug, Deserialize)]
struct FunnelOverrides {
    // Optional lists that can extend/override the built-in baseline.
    mar_required_imports: Option<Vec<String>>,
    mar_required_symbols: Option<Vec<String>>,
    canal_required_imports: Option<Vec<String>>,
    canal_required_symbols: Option<Vec<String>>,
}

#[derive(Debug)]
struct FunnelRequirements {
    mar_required_imports: HashSet<String>,
    mar_required_symbols: HashSet<String>,
    canal_required_imports: HashSet<String>,
    canal_required_symbols: HashSet<String>,
}

impl Default for FunnelRequirements {
    fn default() -> Self {
        let mar_required_imports = [
            "ALE-ERM-ECOSAFETY-TYPES-001.rs",
            "ALE-ERM-ECOSAFETY-CONTRACTS-001.rs",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mar_required_symbols = [
            "CyboquaticNodeEcosafety",
            "requirecorridors",
            "evalcorridor",
            "decidenodeaction",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let canal_required_imports = ["ALE-ERM-ECOSAFETY-TYPES-001.rs"]
            .into_iter()
            .map(String::from)
            .collect();

        let canal_required_symbols = [
            "CyboquaticNodeEcosafety",
            "evalcorridor",
            "decidenodeaction",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        FunnelRequirements {
            mar_required_imports,
            mar_required_symbols,
            canal_required_imports,
            canal_required_symbols,
        }
    }
}

impl FunnelRequirements {
    fn with_overrides(self, overrides: FunnelOverrides) -> Self {
        let mut req = self;

        if let Some(list) = overrides.mar_required_imports {
            req.mar_required_imports = list.into_iter().collect();
        }
        if let Some(list) = overrides.mar_required_symbols {
            req.mar_required_symbols = list.into_iter().collect();
        }
        if let Some(list) = overrides.canal_required_imports {
            req.canal_required_imports = list.into_iter().collect();
        }
        if let Some(list) = overrides.canal_required_symbols {
            req.canal_required_symbols = list.into_iter().collect();
        }

        req
    }
}

// -----------------------------
// Repo traversal + module classification
// -----------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ModuleKind {
    Mar,
    Canal,
}

#[derive(Debug)]
struct ModuleRecord {
    path: PathBuf,
    kind: ModuleKind,
    content: String,
}

fn is_rust_source(path: &Path) -> bool {
    path.extension() == Some(OsStr::new("rs"))
}

fn classify_module(path: &Path) -> Option<ModuleKind> {
    let path_str = path.to_string_lossy();
    // Treat modules under these directories as MAR or canal related.
    // This mirrors the deeper ERM/INF structure described in the
    // workflow research and type maps.
    //
    // MAR: managed aquifer recharge engines, vaults, etc.
    if path_str.contains("ALE-RM-MAR")
        || path_str.contains("marvault")
        || path_str.contains("rmwater") && path_str.contains("MAR")
    {
        return Some(ModuleKind::Mar);
    }

    // Canal: canal state, canal machinery autopilot, cyboquatic pump control,
    // canal stormwater, etc., typically under aletheioninfra/canals or
    // aletheionhighways corridor canal files.
    if path_str.contains("infracanals")
        || path_str.contains("ALE-INF-CANAL")
        || path_str.contains("corridorcanal")
    {
        return Some(ModuleKind::Canal);
    }

    None
}

fn walk_repo_and_collect_modules(root: &Path) -> io::Result<Vec<ModuleRecord>> {
    let mut modules = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !is_rust_source(&path) {
                continue;
            }
            if let Some(kind) = classify_module(&path) {
                let mut file = fs::File::open(&path)?;
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                modules.push(ModuleRecord {
                    path,
                    kind,
                    content,
                });
            }
        }
    }

    Ok(modules)
}

// -----------------------------
// ALN funnel spec loader
// -----------------------------

fn load_funnel_requirements(repo_root: &Path) -> FunnelRequirements {
    let aln_path = repo_root
        .join("aletheionermworkflow-index")
        .join("ALE-ERM-WATER-CANAL-FUNNEL-REQUIREMENTS-001.aln");

    let baseline = FunnelRequirements::default();

    let Ok(mut file) = fs::File::open(&aln_path) else {
        // If the ALN file is absent, we fall back to baseline requirements.
        return baseline;
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return baseline;
    }

    // Look for a minimal overrides block encoded as JSON between ALN comments:
    //
    // ; FUNNEL_OVERRIDES_BEGIN
    // { "mar_required_imports": [ "..." ], ... }
    // ; FUNNEL_OVERRIDES_END
    //
    let begin = "; FUNNEL_OVERRIDES_BEGIN";
    let end = "; FUNNEL_OVERRIDES_END";
    let maybe_json = contents
        .lines()
        .skip_while(|l| !l.trim_start().starts_with(begin))
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with(end))
        .collect::<Vec<_>>()
        .join("\n");

    if maybe_json.trim().is_empty() {
        return baseline;
    }

    match serde_json::from_str::<FunnelOverrides>(&maybe_json) {
        Ok(overrides) => baseline.with_overrides(overrides),
        Err(_) => baseline,
    }
}

// -----------------------------
// Static analysis helpers
// -----------------------------

fn module_has_import(module: &ModuleRecord, required: &str) -> bool {
    // Simple string containment is enough here because the funnel spec
    // cares only that the module depends on the ecosafety types/contracts
    // by filename; this keeps the helper robust across different import
    // syntaxes (mod, use, include_str!, etc.).
    module.content.contains(required)
}

fn module_has_symbol(module: &ModuleRecord, symbol: &str) -> bool {
    // Again, we use substring search for now.
    // CI is allowed to be conservative; if the symbol name appears, we
    // treat it as present. Missing strings indicate real violations.
    module.content.contains(symbol)
}

#[derive(Debug)]
struct ModuleViolations {
    missing_imports: Vec<String>,
    missing_symbols: Vec<String>,
}

fn check_module_against_funnel(
    module: &ModuleRecord,
    funnel: &FunnelRequirements,
) -> ModuleViolations {
    let mut missing_imports = Vec::new();
    let mut missing_symbols = Vec::new();

    match module.kind {
        ModuleKind::Mar => {
            for imp in &funnel.mar_required_imports {
                if !module_has_import(module, imp) {
                    missing_imports.push(imp.clone());
                }
            }
            for sym in &funnel.mar_required_symbols {
                if !module_has_symbol(module, sym) {
                    missing_symbols.push(sym.clone());
                }
            }
        }
        ModuleKind::Canal => {
            for imp in &funnel.canal_required_imports {
                if !module_has_import(module, imp) {
                    missing_imports.push(imp.clone());
                }
            }
            for sym in &funnel.canal_required_symbols {
                if !module_has_symbol(module, sym) {
                    missing_symbols.push(sym.clone());
                }
            }
        }
    }

    ModuleViolations {
        missing_imports,
        missing_symbols,
    }
}

// -----------------------------
// CI entrypoint
// -----------------------------

fn main() {
    // Determine repo root: either from ALETHEION_REPO_ROOT or current dir.
    let repo_root = env::var_os("ALETHEION_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let funnel = load_funnel_requirements(&repo_root);

    let modules = match walk_repo_and_collect_modules(&repo_root) {
        Ok(m) => m,
        Err(err) => {
            eprintln!(
                "[ALE-ERM-WATER-CANAL-FUNNEL-CHECK] ERROR: failed to traverse repo: {}",
                err
            );
            std::process::exit(1);
        }
    };

    if modules.is_empty() {
        println!(
            "[ALE-ERM-WATER-CANAL-FUNNEL-CHECK] INFO: no MAR/canal modules detected; treating as pass."
        );
        std::process::exit(0);
    }

    let mut any_violation = false;

    for module in &modules {
        let violations = check_module_against_funnel(module, &funnel);
        if violations.missing_imports.is_empty() && violations.missing_symbols.is_empty() {
            println!(
                "[ALE-ERM-WATER-CANAL-FUNNEL-CHECK] OK   {:?} ({:?})",
                module.path, module.kind
            );
            continue;
        }

        any_violation = true;

        println!(
            "[ALE-ERM-WATER-CANAL-FUNNEL-CHECK] FAIL {:?} ({:?})",
            module.path, module.kind
        );

        if !violations.missing_imports.is_empty() {
            println!("  missing imports:");
            for imp in violations.missing_imports {
                println!("    - {}", imp);
            }
        }

        if !violations.missing_symbols.is_empty() {
            println!("  missing symbols:");
            for sym in violations.missing_symbols {
                println!("    - {}", sym);
            }
        }
    }

    if any_violation {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

// -----------------------------
// Minimal tests (optional)
// -----------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_funnel_defaults() {
        let f = FunnelRequirements::default();
        assert!(f.mar_required_imports.iter().any(|s| s.contains("ECOSAFETY-TYPES")));
        assert!(f.mar_required_symbols.contains("CyboquaticNodeEcosafety"));
        assert!(f.canal_required_symbols.contains("evalcorridor"));
    }

    #[test]
    fn module_check_detects_missing() {
        let m = ModuleRecord {
            path: PathBuf::from("aletheioninfracanals/ALE-INF-CANAL-SEGMENT-STATE-001.rs"),
            kind: ModuleKind::Canal,
            content: "// no ecosafety here yet".to_string(),
        };
        let funnel = FunnelRequirements::default();
        let v = check_module_against_funnel(&m, &funnel);
        assert!(!v.missing_imports.is_empty());
        assert!(!v.missing_symbols.is_empty());
    }

    #[test]
    fn module_check_passes_when_all_present() {
        let content = r#"
            use crate::ecosafety::CyboquaticNodeEcosafety;
            mod ecosafety_types; // ALE-ERM-ECOSAFETY-TYPES-001.rs
            mod ecosafety_contracts; // ALE-ERM-ECOSAFETY-CONTRACTS-001.rs

            fn requirecorridors() {}
            fn evalcorridor() {}
            fn decidenodeaction() {}
        "#;
        let m = ModuleRecord {
            path: PathBuf::from("aletheionermwater/ALE-RM-MAR-SHARD-001.rs"),
            kind: ModuleKind::Mar,
            content: content.to_string(),
        };
        let funnel = FunnelRequirements::default();
        let v = check_module_against_funnel(&m, &funnel);
        assert!(v.missing_imports.is_empty());
        assert!(v.missing_symbols.is_empty());
    }
}
