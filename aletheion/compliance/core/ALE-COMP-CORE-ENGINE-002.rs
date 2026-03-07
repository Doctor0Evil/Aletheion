// Central compliance engine for Aletheion Phoenix.
// Provides runtime checks used by CI workflows and, where required,
// by long-lived services.
//
// Responsibilities:
// - Enforce Digital Twin Exclusion Protocol (terminology scan)
// - Enforce basic blacklist of forbidden primitives
// - Provide FEAR/PAIN/SANITY neurorights envelope metadata interface (no thresholds yet)
// - Provide FPIC hook interface for modules touching Indigenous territories or biosignals.

use std::path::Path;

/// Basic result type for compliance checks.
#[derive(Debug)]
pub struct ComplianceReport {
    pub passed: bool,
    pub findings: Vec<ComplianceFinding>,
}

#[derive(Debug)]
pub enum ComplianceFindingSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub enum ComplianceFindingCode {
    DigitalTwinTerm,
    ForbiddenPrimitive,
    NeurorightsMissingEnvelopeMetadata,
    FpicHookMissing,
}

#[derive(Debug)]
pub struct ComplianceFinding {
    pub severity: ComplianceFindingSeverity,
    pub code: ComplianceFindingCode,
    pub message: String,
    pub file: Option<String>,
}

pub struct ComplianceEngine {
    forbidden_terms: Vec<&'static str>,
    forbidden_primitives: Vec<&'static str>,
}

impl ComplianceEngine {
    pub fn phoenix_default() -> Self {
        Self {
            forbidden_terms: vec![
                "digital twin",
                "digital-twin",
                "virtual replica",
                "virtual twin",
            ],
            forbidden_primitives: vec![
                // crypto/hash primitives and languages you blacklisted globally
                "SHA-256",
                "SHA3-256",
                "RIPEMD-160",
                "BLAKE2b-256",
                "BLAKE2S256_ALT",
                "XXH3_128",
                "Python",
                "Exergy",
            ],
        }
    }

    /// Run textual compliance scan on a single UTF-8 file.
    pub fn scan_file(&self, path: &Path) -> ComplianceReport {
        let mut findings = Vec::new();
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                findings.push(ComplianceFinding {
                    severity: ComplianceFindingSeverity::Error,
                    code: ComplianceFindingCode::ForbiddenPrimitive, // reuse for IO here
                    message: format!("Failed to read file: {}", e),
                    file: Some(path.display().to_string()),
                });
                return ComplianceReport {
                    passed: false,
                    findings,
                };
            }
        };

        let lower = content.to_lowercase();

        for term in &self.forbidden_terms {
            if lower.contains(&term.to_lowercase()) {
                findings.push(ComplianceFinding {
                    severity: ComplianceFindingSeverity::Error,
                    code: ComplianceFindingCode::DigitalTwinTerm,
                    message: format!(
                        "Forbidden terminology '{}' detected (use 'state model' / 'operational mirror')",
                        term
                    ),
                    file: Some(path.display().to_string()),
                });
            }
        }

        for primitive in &self.forbidden_primitives {
            if content.contains(primitive) {
                findings.push(ComplianceFinding {
                    severity: ComplianceFindingSeverity::Error,
                    code: ComplianceFindingCode::ForbiddenPrimitive,
                    message: format!("Forbidden primitive '{}' detected", primitive),
                    file: Some(path.display().to_string()),
                });
            }
        }

        ComplianceReport {
            passed: findings.is_empty(),
            findings,
        }
    }

    /// Aggregate scan over many files.
    pub fn scan_paths(&self, paths: &[&Path]) -> ComplianceReport {
        let mut all_findings = Vec::new();
        let mut passed_all = true;

        for p in paths {
            let report = self.scan_file(p);
            if !report.passed {
                passed_all = false;
            }
            all_findings.extend(report.findings);
        }

        ComplianceReport {
            passed: passed_all,
            findings: all_findings,
        }
    }
}

// -------- Neurorights & FPIC metadata stubs --------

/// Modules that handle biosignals, BCIs, or augmentations should expose
/// metadata via this trait to prove they respect FEAR/PAIN/SANITY envelopes.
/// In v1, this is a declarative contract checked by CI.
pub trait NeurorightsEnvelopeMetadata {
    fn declares_fear_envelope(&self) -> bool;
    fn declares_pain_envelope(&self) -> bool;
    fn declares_sanity_envelope(&self) -> bool;
}

/// Modules operating on Indigenous territories or resources should expose
/// FPIC metadata. CI can assert that any such module implements this trait.
pub trait FpicMetadata {
    fn touches_indigenous_territory(&self) -> bool;
    fn has_fpic_protocol_reference(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn detects_forbidden_digital_twin_term() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "This mentions a Digital Twin by mistake.").unwrap();

        let engine = ComplianceEngine::phoenix_default();
        let report = engine.scan_file(tmp.path());
        assert!(!report.passed);
        assert!(report
            .findings
            .iter()
            .any(|f| matches!(f.code, ComplianceFindingCode::DigitalTwinTerm)));
    }

    #[test]
    fn passes_clean_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "This file discusses a state model and operational mirror.").unwrap();

        let engine = ComplianceEngine::phoenix_default();
        let report = engine.scan_file(tmp.path());
        assert!(report.passed);
    }
}
