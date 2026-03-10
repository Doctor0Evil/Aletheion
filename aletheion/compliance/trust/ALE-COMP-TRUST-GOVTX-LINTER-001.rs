// Trust-schema governance preflight linter.
// Ensures that every ALN transaction schema under aletheion/trust/**
// contains or references the canonical GovernedDecisionTxEnvelope
// defined in ALE-TRUST-GOVERNED-DECISION-TX-001.aln.
//
// Usage (CI or local):
//   cargo run --bin alethion-trust-govtx-linter -- \
//       --root aletheion/trust \
//       --envelope-id "GovernedDecisionTxEnvelope" \
//       --schema-ref "ALE-TRUST-GOVERNED-DECISION-TX-001"
//
// Exit code 0 = OK, 1 = violations found.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct LintConfig {
    root: PathBuf,
    envelope_type_name: String,
    schema_ref_id: String,
}

#[derive(Debug)]
struct LintFinding {
    file: PathBuf,
    message: String,
}

#[derive(Debug)]
struct LintReport {
    passed: bool,
    findings: Vec<LintFinding>,
}

fn parse_args() -> LintConfig {
    let mut root = None;
    let mut envelope = None;
    let mut schema_ref = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => {
                if let Some(v) = args.next() {
                    root = Some(PathBuf::from(v));
                }
            }
            "--envelope-id" => {
                if let Some(v) = args.next() {
                    envelope = Some(v);
                }
            }
            "--schema-ref" => {
                if let Some(v) = args.next() {
                    schema_ref = Some(v);
                }
            }
            _ => {}
        }
    }

    LintConfig {
        root: root.unwrap_or_else(|| PathBuf::from("aletheion/trust")),
        envelope_type_name: envelope
            .unwrap_or_else(|| "GovernedDecisionTxEnvelope".to_string()),
        schema_ref_id: schema_ref
            .unwrap_or_else(|| "ALE-TRUST-GOVERNED-DECISION-TX-001".to_string()),
    }
}

fn is_aln_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("aln"))
        .unwrap_or(false)
}

fn walk_aln_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if is_aln_file(&path) {
                    out.push(path);
                }
            }
        }
    }
    out
}

fn lint_file(path: &Path, cfg: &LintConfig) -> Option<LintFinding> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return Some(LintFinding {
                file: path.to_path_buf(),
                message: format!("Failed to read file: {e}"),
            })
        }
    };

    let lower = content.to_lowercase();
    let envelope_lower = cfg.envelope_type_name.to_lowercase();
    let schema_ref_lower = cfg.schema_ref_id.to_lowercase();

    let mentions_envelope = lower.contains(&envelope_lower);
    let mentions_schema_ref = lower.contains(&schema_ref_lower);

    if mentions_envelope || mentions_schema_ref {
        None
    } else {
        Some(LintFinding {
            file: path.to_path_buf(),
            message: format!(
                "Trust Tx schema does not reference '{}' or '{}' (required for governed decisions).",
                cfg.envelope_type_name, cfg.schema_ref_id
            ),
        })
    }
}

fn run_lint(cfg: &LintConfig) -> LintReport {
    let mut findings = Vec::new();
    let files = walk_aln_files(&cfg.root);

    for f in files {
        if let Some(finding) = lint_file(&f, cfg) {
            findings.push(finding);
        }
    }

    LintReport {
        passed: findings.is_empty(),
        findings,
    }
}

fn main() {
    let cfg = parse_args();
    let report = run_lint(&cfg);

    if report.passed {
        println!(
            "[OK] All trust schemas under '{}' reference GovernedDecisionTxEnvelope or {}.",
            cfg.root.display(),
            cfg.schema_ref_id
        );
        std::process::exit(0);
    } else {
        eprintln!("[FAIL] Some trust schemas are missing GovernedDecisionTxEnvelope references:");
        for f in &report.findings {
            eprintln!("- {} :: {}", f.file.display(), f.message);
        }
        std::process::exit(1);
    }
}
