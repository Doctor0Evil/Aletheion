// Aletheion Path Ontology Validator v0.1
// Validates that all tracked files conform to patterns defined in aletheion/path-ontology.aln.
//
// Usage (from repo root):
//   cargo run -p path_ontology_validator
//
// Integration:
//   - Pre-commit: call this binary and fail if exit code != 0.
//   - CI: same command as a step in your pipeline.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let repo_root = find_repo_root().unwrap_or_else(|| {
        eprintln!("Error: could not locate .git directory.");
        std::process::exit(1);
    });

    let ontology_path = repo_root.join("aletheion/path-ontology.aln");
    if !ontology_path.exists() {
        eprintln!(
            "Error: ontology file not found at {}",
            ontology_path.display()
        );
        std::process::exit(1);
    }

    let ontology = match std::fs::read_to_string(&ontology_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error: failed to read ontology file {}: {}",
                ontology_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let glob_map = parse_globs(&ontology);
    if glob_map.is_empty() {
        eprintln!("Warning: no path globs found in ontology; nothing to validate.");
        std::process::exit(0);
    }

    let changed_files = match git_tracked_or_staged_files(&repo_root) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error: failed to get tracked/staged files: {}", e);
            std::process::exit(1);
        }
    };

    let mut violations = Vec::new();

    for rel in changed_files {
        // Only enforce under aletheion/
        if !rel.starts_with("aletheion/") {
            continue;
        }
        if !is_allowed_by_ontology(&rel, &glob_map) {
            violations.push(rel);
        }
    }

    if !violations.is_empty() {
        eprintln!("Path ontology violations detected:");
        for v in &violations {
            eprintln!("  - {}", v);
        }
        eprintln!("\nEach file under aletheion/ must match at least one glob in aletheion/path-ontology.aln.");
        std::process::exit(1);
    }
}

/// Find the repo root by walking up until a .git directory is found.
fn find_repo_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        if dir.join(".git").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Very simple parser: scan `ontology` text for lines containing `glob = "..."`
/// and collect the glob strings into a vector.
fn parse_globs(ontology: &str) -> Vec<String> {
    let mut globs = Vec::new();
    for line in ontology.lines() {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find("glob") {
            let rest = &trimmed[idx..];
            if let Some(eq_idx) = rest.find('=') {
                let after_eq = rest[eq_idx + 1..].trim();
                if after_eq.starts_with('"') {
                    if let Some(end_idx) = after_eq[1..].find('"') {
                        let glob_str = &after_eq[1..1 + end_idx];
                        globs.push(glob_str.to_string());
                    }
                }
            }
        }
    }
    globs
}

/// Get tracked or staged files using `git diff --cached --name-only` plus new files.
fn git_tracked_or_staged_files(repo_root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--name-only")
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to run git diff: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git diff --cached failed with status {}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files: HashMap<String, ()> = HashMap::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            files.insert(trimmed.to_string(), ());
        }
    }

    // Also include untracked files (git ls-files --others --exclude-standard)
    let output_untracked = Command::new("git")
        .arg("ls-files")
        .arg("--others")
        .arg("--exclude-standard")
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to run git ls-files: {}", e))?;

    if !output_untracked.status.success() {
        return Err(format!(
            "git ls-files --others failed with status {}",
            output_untracked.status
        ));
    }

    let stdout_untracked = String::from_utf8_lossy(&output_untracked.stdout);
    for line in stdout_untracked.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            files.insert(trimmed.to_string(), ());
        }
    }

    Ok(files.into_keys().collect())
}

/// Check if a given path matches any of the configured globs.
/// This uses a very small subset of glob semantics: `**`, `*`, and `?`.
fn is_allowed_by_ontology(path: &str, globs: &[String]) -> bool {
    globs.iter().any(|g| glob_match(g, path))
}

/// Simple glob matcher supporting `*`, `?`, and `**` across `/` boundaries.
/// Not as powerful as globset, but avoids external dependencies.
fn glob_match(pattern: &str, text: &str) -> bool {
    fn inner(p: &[u8], t: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        if p[0] == b'*' {
            // Handle ** specially: match any (including /)
            if p.len() >= 2 && p[1] == b'*' {
                let rest = &p[2..];
                if rest.is_empty() {
                    return true;
                }
                for i in 0..=t.len() {
                    if inner(rest, &t[i..]) {
                        return true;
                    }
                }
                return false;
            } else {
                // Single *: match any sequence except path separator
                let rest = &p[1..];
                let mut i = 0;
                while i <= t.len() {
                    if i < t.len() && t[i] == b'/' {
                        break;
                    }
                    if inner(rest, &t[i..]) {
                        return true;
                    }
                    i += 1;
                }
                return false;
            }
        } else if p[0] == b'?' {
            if t.is_empty() || t[0] == b'/' {
                return false;
            }
            return inner(&p[1..], &t[1..]);
        } else {
            if t.is_empty() || p[0] != t[0] {
                return false;
            }
            return inner(&p[1..], &t[1..]);
        }
    }
    inner(pattern.as_bytes(), text.as_bytes())
}
