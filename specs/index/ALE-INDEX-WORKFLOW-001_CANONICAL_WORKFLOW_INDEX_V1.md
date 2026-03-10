# ALE-INDEX-WORKFLOW-001: Canonical Workflow Index v1.0
**Repository:** `github.com/Doctor0Evil/Aletheion`  
**Path:** `/specs/index/ALE-INDEX-WORKFLOW-001_CANONICAL_WORKFLOW_INDEX_V1.md`  
**Status:** `ACTIVE` | **Version:** `1.0.0` | **ERM-Layers:** `L1-L5`  
**Dependencies:** `ALE-SPEC-GOV-001`, `ALE-SPEC-GOV-002`, `ALE-SPEC-ENV-001`  
**Purpose:** Single Source of Truth for Aletheion City Factory Code Generation  
**Security:** `PQC-Compliant`, `Immutable-Reference`, `Machine-Readable`  

## 1.0 Index Schema Definition (ALN)
This schema defines the structure of every workflow entry. All code generators must validate against this schema.

```aln
// Destination: /src/aln/schemas/workflow_index_entry_v1.aln

namespace Aletheion.Index.Workflow;

struct WorkflowEntry {
    // Identity
    id: String,                      // e.g., "ALE-WF-001"
    title: String,                   // Human-readable title
    version: SemVer,                 // e.g., "1.0.0"
    
    // Architecture
    erm_layers: Vec<Enum { L1, L2, L3, L4, L5 }>,
    languages: Vec<Enum { Rust, Cpp, ALN, Lua, JS, Kotlin }>,
    repo_paths: Vec<String>,         // Canonical destination paths
    
    // Compliance & Treaties
    treaties_touched: Vec<String>,   // e.g., "FPIC", "Neurorights", "BioticTreaty"
    rights_catalog_refs: Vec<ALN_ID>, // Links to ALE-SPEC-GOV-001
    
    // Operations
    ci_jobs: Vec<CIJobRef>,          // Composite CI job definitions
    triggers: Vec<TriggerType>,      // e.g., "Sensor_Data", "Grievance_Filed", "Schedule"
    offline_capable: bool,
    pqc_security_level: u8,          // NIST PQC Security Level (1-5)
    
    // Phoenix Context
    assets_touched: Vec<String>,     // e.g., "AWP_Plant_04", "Central_Ave_Corridor"
    climate_zones: Vec<Enum { Urban_Core, Desert_Fringe, Tribal_Land, Industrial }>,
};

struct CIJobRef {
    job_name: String,                // e.g., "ci-compliance-preflight.yml"
    job_type: Enum { Primary, Secondary, Shared, Audit },
    parallel_allowed: bool,
};
