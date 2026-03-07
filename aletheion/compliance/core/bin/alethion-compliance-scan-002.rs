// CLI wrapper for ALE-COMP-CORE-ENGINE-002.
// Scans source files for blacklist / Digital Twin Exclusion violations
// and exits non-zero on any Error-level finding.

use std::env;
use std::path::PathBuf;

use alethion_compliance_core_engine_002::{
    ComplianceEngine,
    ComplianceFindingSeverity,
};

fn main() {
    // Simple arg parsing: `alethion-compliance-scan-002 --root <path>`
    let args: Vec<String> = env::args().collect();
    let mut root: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--root" if i + 1 < args.len() => {
                root = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                std::process::exit(2);
            }
        }
    }

    let root = root.unwrap_or_else(|| PathBuf::from("."));

    let engine = ComplianceEngine::phoenix_default();
    let files = collect_source_files(&root);

    let report = engine.scan_paths(&files.iter().map(|p| p.as_path()).collect::<Vec<_>>());

    for finding in &report.findings {
        let sev = match finding.severity {
            ComplianceFindingSeverity::Info => "INFO",
            ComplianceFindingSeverity::Warning => "WARN",
            ComplianceFindingSeverity::Error => "ERROR",
        };
        let file = finding.file.as_deref().unwrap_or("-");
        eprintln!("[{}] {} — {}", sev, file, finding.message);
    }

    if report.passed {
        println!("Compliance preflight passed for {} files", files.len());
        std::process::exit(0);
    } else {
        eprintln!("Compliance preflight FAILED with {} findings", report.findings.len());
        std::process::exit(1);
    }
}

fn collect_source_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files
}

fn walk(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip typical build/vendor dirs.
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                walk(&path, out);
            } else if is_source_file(&path) {
                out.push(path);
            }
        }
    }
}

fn is_source_file(path: &PathBuf) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") | Some("aln") | Some("lua") | Some("js") | Some("kt") | Some("kts") | Some("md") => true,
        _ => false,
    }
}

fn print_help() {
    eprintln!(
        "Usage: alethion-compliance-scan-002 --root <path>\n\
         Scans Rust/ALN/Lua/JS/Kotlin/MD sources under <path> for compliance."
    );
}

// Re-export engine crate so this bin can compile once you declare it in Cargo.toml
mod alethion_compliance_core_engine_002 {
    pub use crate_engine::*;
}

// The actual engine crate would be exposed via Cargo with a proper name; here we
// assume ALE-COMP-CORE-ENGINE-002 is compiled as `crate_engine`.
extern crate alethion_comp_core_engine_002 as crate_engine;
