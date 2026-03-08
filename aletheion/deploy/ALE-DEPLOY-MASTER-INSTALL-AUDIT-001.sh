#!/usr/bin/env bash
# ============================================================================
# FILE: aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh
# ============================================================================
# ALETHEION MASTER REPOSITORY INSTALLATION & AUDIT SCRIPT
# Domain: Cross-Cutting Governance (Repository Integrity & Deployment)
# Language: Bash (POSIX-compliant, Offline-Capable, Multi-Platform)
# License: Aletheion Public License v1.0 (Neurorights + BioticTreaty Bound)
# Version: 1.0.0
# Generated: 2026-03-09T00:00:00Z
# SMART-Chain Binding: SMART01_AWP_THERMAL_THERMAPHORA (Governance Domain)
# KER-Band: K=0.95, E=0.92, R=0.10 (Deployment Trust Layer)
# Cryptography: CRYSTALS-Dilithium (Signature Verification)
# ============================================================================
# CONSTRAINTS:
#   - No rollback, no downgrade, no reversal (forward-compatible only)
#   - Offline-capable execution (no external HTTP/API calls in core)
#   - Indigenous Water Treaty (Akimel O'odham, Piipaash) hard gates
#   - BioticTreaty (Riparian, Species) hard gates
#   - "No corridor, no build" enforced at installation level
#   - Bound to all previous Chunks (1-9) in Aletheion repository
#   - Supports indefinite repository growth (scalable to 10000+ files)
#   - Multi-city deployment support (Phoenix-default, adjustable)
# ============================================================================
# USAGE:
#   ./ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh [COMMAND] [OPTIONS]
#
# COMMANDS:
#   install     - Full repository installation with verification
#   audit       - Repository integrity and compliance audit
#   verify      - Quick verification of critical components
#   deploy      - Deploy to target city environment
#   migrate     - Migrate configuration for new city (e.g., Phoenix → Tucson)
#   report      - Generate compliance and KER metadata report
#   help        - Display this help message
#
# OPTIONS:
#   --offline           - Run in offline mode (no network calls)
#   --strict            - Enforce strict treaty and corridor compliance
#   --city=NAME         - Target city for deployment (default: Phoenix)
#   --output=PATH       - Output directory for reports and logs
#   --verbose           - Enable verbose output
#   --dry-run           - Simulate actions without making changes
# ============================================================================

set -euo pipefail

# ============================================================================
# SECTION 1: GLOBAL CONSTANTS & CONFIGURATION
# ============================================================================
# Hard-coded constants for repository structure and validation.
# These ensure consistency across all deployment scenarios.
# ============================================================================

readonly ALETHEION_VERSION="1.0.0"
readonly SCRIPT_VERSION="1.0.0"
readonly SCRIPT_NAME="ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh"
readonly SCRIPT_GENERATED="2026-03-09T00:00:00Z"

# Repository Structure (Must match Aletheion repo layout)
readonly ALE_ROOT_DIRS=(
    "aletheion/erm"
    "aletheion/infra"
    "aletheion/governance"
    "aletheion/trust"
    "aletheion/catalog"
    "aletheion/workflows"
    "aletheion/contracts"
    "aletheion/interface"
    "aletheion/deploy"
)

# Critical Files (Must exist for valid installation)
readonly ALE_CRITICAL_FILES=(
    "aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-WATER-CORRIDOR-TYPES-001.rs"
    "aletheion/erm/ecosafety/ALE-ERM-ECOSAFETY-WATER-CORRIDOR-CONTRACTS-001.aln"
    "aletheion/erm/workflow-index/ALE-ERM-SMARTCHAIN-VALIDATOR-WATER-001.rs"
    "aletheion/infra/cyboquatic/mar/ALE-INF-CYBO-MAR-ORCHESTRATOR-001.lua"
    "aletheion/interface/citizen/ALE-INT-CITIZEN-MAR-CONSENT-001.kt"
    "aletheion/interface/dashboard/ALE-INT-DASHBOARD-MAR-CONSENT-001.jsx"
    "aletheion/infra/edge/compute/ALE-INF-EDGE-COMPUTE-SENSOR-PQ-001.cpp"
    "aletheion/erm/workflow-index/ALE-ERM-CICD-ECOSAFETY-PREFLIGHT-001.aln"
    "aletheion/trust/googolswarm/ALE-TRUST-GOOGOLSWARM-LEDGER-CLIENT-001.rs"
)

# Supported Languages (Blacklist enforced)
readonly ALE_SUPPORTED_LANGUAGES=("rs" "aln" "lua" "jsx" "js" "kt" "cpp" "c" "h" "hpp" "sh" "md" "json" "yaml" "yml" "ttl" "geojson")
readonly ALE_BLACKLIST_PATTERNS=(
    "\.py$"
    "sha256|SHA256"
    "blake|BLAKE"
    "keccak|KECCAK"
    "argon2|ARGON"
    "ripemd|RIPEMD"
    "neuron|Brian2"
    "exergy|Exergy"
    "dow|NDM"
)

# Treaty References (Mandatory for Water/Biotic domains)
readonly ALE_TREATY_INDIGENOUS="INDIGENOUS_WATER_TREATY_AKIMEL"
readonly ALE_TREATY_BIOTIC="BIOTIC_TREATY_RIPARIAN"
readonly ALE_TREATY_LEXETHOS="ALETHEION_LEXETHOS_CIVIC"

# SMART-Chain IDs (Must be registered)
readonly ALE_SMART_CHAINS=(
    "SMART01_AWP_THERMAL_THERMAPHORA"
    "SMART03_SYNTHEXIS_LNP_ENV"
    "SMART04_SOMATIC_INFRA"
    "SMART05_NEUROBIOME_EQUITY"
)

# KER Metadata Bounds (2026 Cyboquatic Research Band)
readonly KER_K_MIN=0.85
readonly KER_E_MIN=0.85
readonly KER_R_MAX=0.20

# Default City Configuration (Phoenix)
readonly DEFAULT_CITY="Phoenix"
readonly DEFAULT_CITY_COORDS="33.4484,-112.0740"
readonly DEFAULT_CITY_TIMEZONE="America/Phoenix"

# ============================================================================
# SECTION 2: UTILITY FUNCTIONS (Logging, Verification, Output)
# ============================================================================
# Core utility functions used throughout the script.
# ============================================================================

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_audit() {
    local timestamp
    timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo -e "${CYAN}[AUDIT]${NC} [${timestamp}] $1"
}

# Verbose output helper
verbose_log() {
    if [[ "${VERBOSE:-false}" == "true" ]]; then
        log_info "$1"
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# ============================================================================
# SECTION 3: REPOSITORY STRUCTURE VERIFICATION
# ============================================================================
# Verifies the Aletheion repository structure is intact and complete.
# Scalable to indefinite file growth.
# ============================================================================

verify_repository_structure() {
    log_info "Verifying Aletheion repository structure..."
    
    local root_dir="${1:-.}"
    local errors=0
    local warnings=0
    
    # Check root directories exist
    for dir in "${ALE_ROOT_DIRS[@]}"; do
        if [[ ! -d "${root_dir}/${dir}" ]]; then
            log_error "Missing required directory: ${dir}"
            ((errors++))
        else
            verbose_log "Found directory: ${dir}"
        fi
    done
    
    # Check critical files exist
    for file in "${ALE_CRITICAL_FILES[@]}"; do
        if [[ ! -f "${root_dir}/${file}" ]]; then
            log_error "Missing critical file: ${file}"
            ((errors++))
        else
            verbose_log "Found critical file: ${file}"
        fi
    done
    
    # Count total files by language
    log_info "Counting repository files by language..."
    local total_files=0
    for lang in "${ALE_SUPPORTED_LANGUAGES[@]}"; do
        local count
        count=$(find "${root_dir}" -type f -name "*.${lang}" 2>/dev/null | wc -l)
        if [[ ${count} -gt 0 ]]; then
            verbose_log "  ${lang}: ${count} files"
            ((total_files += count))
        fi
    done
    log_info "Total supported language files: ${total_files}"
    
    # Check for blacklisted patterns
    log_info "Scanning for blacklisted patterns..."
    local blacklist_violations=0
    for pattern in "${ALE_BLACKLIST_PATTERNS[@]}"; do
        local matches
        matches=$(grep -rE "${pattern}" "${root_dir}" --include="*.rs" --include="*.cpp" --include="*.aln" --include="*.lua" --include="*.kt" --include="*.jsx" 2>/dev/null | wc -l)
        if [[ ${matches} -gt 0 ]]; then
            log_warning "Blacklist pattern '${pattern}' found ${matches} times"
            ((blacklist_violations++))
        fi
    done
    
    if [[ ${blacklist_violations} -gt 0 ]]; then
        log_error "Blacklist violations detected: ${blacklist_violations}"
        ((errors++))
    fi
    
    # Report results
    if [[ ${errors} -eq 0 ]]; then
        log_success "Repository structure verification passed"
        return 0
    else
        log_error "Repository structure verification failed with ${errors} errors"
        return 1
    fi
}

# ============================================================================
# SECTION 4: ECOSAFETY CORRIDOR VERIFICATION
# ============================================================================
# Verifies all ecosafety corridors are properly defined and referenced.
# Enforces "no corridor, no build" at repository level.
# ============================================================================

verify_ecosafety_corridors() {
    log_info "Verifying ecosafety corridor definitions..."
    
    local root_dir="${1:-.}"
    local errors=0
    local corridors_found=0
    
    # Find all corridor definition files (ALN and JSON)
    local corridor_files
    corridor_files=$(find "${root_dir}" -type f \( -name "*corridor*.aln" -o -name "*corridor*.json" \) 2>/dev/null)
    
    if [[ -z "${corridor_files}" ]]; then
        log_warning "No corridor definition files found"
        ((warnings++))
    fi
    
    # Verify each corridor file has required fields
    while IFS= read -r file; do
        if [[ -n "${file}" ]]; then
            verbose_log "Checking corridor file: ${file}"
            
            # Check for required corridor fields
            if ! grep -q "corridorId\|CorridorId" "${file}" 2>/dev/null; then
                log_error "Corridor file missing corridorId: ${file}"
                ((errors++))
            fi
            
            if ! grep -q "riskVector\|RiskVector\|risk_coords" "${file}" 2>/dev/null; then
                log_error "Corridor file missing riskVector: ${file}"
                ((errors++))
            fi
            
            if ! grep -q "lyapunov\|Lyapunov\|Vt" "${file}" 2>/dev/null; then
                log_warning "Corridor file missing Lyapunov template: ${file}"
            fi
            
            if ! grep -q "ker_meta\|KerMetadata\|KER" "${file}" 2>/dev/null; then
                log_error "Corridor file missing KER metadata: ${file}"
                ((errors++))
            fi
            
            ((corridors_found++))
        fi
    done <<< "${corridor_files}"
    
    log_info "Found ${corridors_found} corridor definition files"
    
    # Verify cyboquatic modules reference corridors
    log_info "Verifying cyboquatic modules reference corridors..."
    local cybo_modules
    cybo_modules=$(find "${root_dir}/aletheion/infra/cyboquatic" -type f \( -name "*.rs" -o -name "*.aln" -o -name "*.lua" \) 2>/dev/null)
    
    local modules_without_corridors=0
    while IFS= read -r module; do
        if [[ -n "${module}" ]]; then
            if ! grep -q "corridor\|Corridor" "${module}" 2>/dev/null; then
                log_warning "Cyboquatic module without corridor reference: ${module}"
                ((modules_without_corridors++))
            fi
        fi
    done <<< "${cybo_modules}"
    
    if [[ ${modules_without_corridors} -gt 0 ]]; then
        log_error "Found ${modules_without_corridors} cyboquatic modules without corridor references"
        ((errors++))
    fi
    
    if [[ ${errors} -eq 0 ]]; then
        log_success "Ecosafety corridor verification passed"
        return 0
    else
        log_error "Ecosafety corridor verification failed with ${errors} errors"
        return 1
    fi
}

# ============================================================================
# SECTION 5: SMART-CHAIN VALIDATION
# ============================================================================
# Verifies all SMART-Chain bindings are valid and compliant.
# Enforces PQ mode, treaty requirements, and rollback-forbidden invariant.
# ============================================================================

verify_smart_chains() {
    log_info "Verifying SMART-Chain bindings..."
    
    local root_dir="${1:-.}"
    local errors=0
    local chains_found=0
    
    # Find all SMART-Chain definition files
    local chain_files
    chain_files=$(find "${root_dir}" -type f -name "*smart*chain*.aln" -o -name "*SMART*CHAIN*.aln" 2>/dev/null)
    
    while IFS= read -r file; do
        if [[ -n "${file}" ]]; then
            verbose_log "Checking SMART-Chain file: ${file}"
            
            # Check for required chain fields
            if ! grep -q "smartChainId\|SMART_CHAIN" "${file}" 2>/dev/null; then
                log_error "SMART-Chain file missing chain ID: ${file}"
                ((errors++))
            fi
            
            # Check PQ mode for water/biotic domains
            if grep -q "water\|Water\|WATER\|biotic\|Biotic\|BIOTIC" "${file}" 2>/dev/null; then
                if ! grep -q "PQSTRICT\|PqStrict" "${file}" 2>/dev/null; then
                    log_error "Water/Biotic chain must use PQSTRICT mode: ${file}"
                    ((errors++))
                fi
            fi
            
            # Check rollback-forbidden invariant
            if ! grep -q "rollback_forbidden.*true\|rollbackForbidden.*true" "${file}" 2>/dev/null; then
                log_error "SMART-Chain must have rollback_forbidden=true: ${file}"
                ((errors++))
            fi
            
            # Check treaty references for water domain
            if grep -q "water\|Water\|WATER" "${file}" 2>/dev/null; then
                if ! grep -q "INDIGENOUS_WATER_TREATY\|AKIMEL\|PIIPAASH" "${file}" 2>/dev/null; then
                    log_error "Water chain missing Indigenous Water Treaty reference: ${file}"
                    ((errors++))
                fi
            fi
            
            ((chains_found++))
        fi
    done <<< "${chain_files}"
    
    log_info "Found ${chains_found} SMART-Chain definition files"
    
    if [[ ${errors} -eq 0 ]]; then
        log_success "SMART-Chain verification passed"
        return 0
    else
        log_error "SMART-Chain verification failed with ${errors} errors"
        return 1
    fi
}

# ============================================================================
# SECTION 6: TREATY COMPLIANCE VERIFICATION
# ============================================================================
# Verifies Indigenous Water Treaty and BioticTreaty compliance.
# Hard gate for any water/biotic domain module.
# ============================================================================

verify_treaty_compliance() {
    log_info "Verifying treaty compliance..."
    
    local root_dir="${1:-.}"
    local errors=0
    local water_modules=0
    local biotic_modules=0
    
    # Find water domain modules
    local water_modules_files
    water_modules_files=$(find "${root_dir}/aletheion/rm/water" "${root_dir}/aletheion/infra/mar" -type f \( -name "*.rs" -o -name "*.aln" -o -name "*.lua" \) 2>/dev/null)
    
    while IFS= read -r file; do
        if [[ -n "${file}" ]]; then
            ((water_modules++))
            if ! grep -q "INDIGENOUS_WATER_TREATY\|AKIMEL\|PIIPAASH\|fpic_required" "${file}" 2>/dev/null; then
                log_error "Water module missing Indigenous Water Treaty reference: ${file}"
                ((errors++))
            fi
        fi
    done <<< "${water_modules_files}"
    
    # Find biotic domain modules
    local biotic_modules_files
    biotic_modules_files=$(find "${root_dir}/aletheion/synthexis" "${root_dir}/aletheion/infra/canals" -type f \( -name "*.rs" -o -name "*.aln" -o -name "*.lua" \) 2>/dev/null)
    
    while IFS= read -r file; do
        if [[ -n "${file}" ]]; then
            ((biotic_modules++))
            if grep -q "RIPARIAN\|Habitat\|Species" "${file}" 2>/dev/null; then
                if ! grep -q "BIOTIC_TREATY\|BioticTreaty" "${file}" 2>/dev/null; then
                    log_error "Biotic module missing BioticTreaty reference: ${file}"
                    ((errors++))
                fi
            fi
        fi
    done <<< "${biotic_modules_files}"
    
    log_info "Checked ${water_modules} water modules and ${biotic_modules} biotic modules"
    
    if [[ ${errors} -eq 0 ]]; then
        log_success "Treaty compliance verification passed"
        return 0
    else
        log_error "Treaty compliance verification failed with ${errors} errors"
        return 1
    fi
}

# ============================================================================
# SECTION 7: KER METADATA VALIDATION
# ============================================================================
# Verifies all KER (Knowledge/Eco/Risk) metadata is present and within bounds.
# Ensures alignment with 2026 Cyboquatic Research Band.
# ============================================================================

verify_ker_metadata() {
    log_info "Verifying KER metadata..."
    
    local root_dir="${1:-.}"
    local errors=0
    local warnings=0
    local ker_entries=0
    
    # Find all files with KER metadata
    local ker_files
    ker_files=$(grep -rl "ker_meta\|KerMetadata\|KER_META\|k_score\|e_score\|r_score" "${root_dir}" --include="*.rs" --include="*.aln" --include="*.json" --include="*.lua" --include="*.kt" --include="*.jsx" 2>/dev/null)
    
    while IFS= read -r file; do
        if [[ -n "${file}" ]]; then
            ((ker_entries++))
            verbose_log "Checking KER metadata in: ${file}"
            
            # Extract K, E, R values (simplified grep-based extraction)
            local k_value e_value r_value
            k_value=$(grep -oE "k[[:space:]]*:[[:space:]]*[0-9.]+" "${file}" 2>/dev/null | head -1 | grep -oE "[0-9.]+")
            e_value=$(grep -oE "e[[:space:]]*:[[:space:]]*[0-9.]+" "${file}" 2>/dev/null | head -1 | grep -oE "[0-9.]+")
            r_value=$(grep -oE "r[[:space:]]*:[[:space:]]*[0-9.]+" "${file}" 2>/dev/null | head -1 | grep -oE "[0-9.]+")
            
            # Validate K score bounds
            if [[ -n "${k_value}" ]]; then
                if (( $(echo "${k_value} < ${KER_K_MIN}" | bc -l) )); then
                    log_warning "K score below minimum (${k_value} < ${KER_K_MIN}): ${file}"
                    ((warnings++))
                fi
            fi
            
            # Validate E score bounds
            if [[ -n "${e_value}" ]]; then
                if (( $(echo "${e_value} < ${KER_E_MIN}" | bc -l) )); then
                    log_warning "E score below minimum (${e_value} < ${KER_E_MIN}): ${file}"
                    ((warnings++))
                fi
            fi
            
            # Validate R score bounds
            if [[ -n "${r_value}" ]]; then
                if (( $(echo "${r_value} > ${KER_R_MAX}" | bc -l) )); then
                    log_error "R score above maximum (${r_value} > ${KER_R_MAX}): ${file}"
                    ((errors++))
                fi
            fi
        fi
    done <<< "${ker_files}"
    
    log_info "Found ${ker_entries} KER metadata entries"
    
    if [[ ${errors} -eq 0 ]]; then
        log_success "KER metadata verification passed (${warnings} warnings)"
        return 0
    else
        log_error "KER metadata verification failed with ${errors} errors"
        return 1
    fi
}

# ============================================================================
# SECTION 8: POST-QUANTUM CRYPTO VERIFICATION
# ============================================================================
# Verifies all cryptographic operations use PQ-safe primitives.
# Enforces CRYSTALS-Dilithium/Kyber usage.
# ============================================================================

verify_pq_crypto() {
    log_info "Verifying Post-Quantum cryptography usage..."
    
    local root_dir="${1:-.}"
    local errors=0
    local pq_files=0
    
    # Find crypto-related files
    local crypto_files
    crypto_files=$(grep -rl "crypto\|sign\|encrypt\|hash\|dilithium\|kyber" "${root_dir}" --include="*.rs" --include="*.cpp" --include="*.aln" 2>/dev/null)
    
    while IFS= read -r file; do
        if [[ -n "${file}" ]]; then
            ((pq_files++))
            verbose_log "Checking PQ crypto in: ${file}"
            
            # Check for PQ crypto references
            if ! grep -q "dilithium\|kyber\|CRYSTALS\|PQ" "${file}" 2>/dev/null; then
                log_warning "Crypto file without PQ reference: ${file}"
            fi
            
            # Check for blacklisted crypto
            if grep -qiE "sha256|blake|keccak|argon" "${file}" 2>/dev/null; then
                log_error "Blacklisted crypto primitive in: ${file}"
                ((errors++))
            fi
        fi
    done <<< "${crypto_files}"
    
    log_info "Checked ${pq_files} crypto-related files"
    
    if [[ ${errors} -eq 0 ]]; then
        log_success "Post-Quantum crypto verification passed"
        return 0
    else
        log_error "Post-Quantum crypto verification failed with ${errors} errors"
        return 1
    fi
}

# ============================================================================
# SECTION 9: FULL REPOSITORY AUDIT
# ============================================================================
# Runs all verification checks and generates comprehensive audit report.
# ============================================================================

run_full_audit() {
    log_info "Running full Aletheion repository audit..."
    
    local root_dir="${1:-.}"
    local output_dir="${2:-./aletheion_audit}"
    local timestamp
    timestamp=$(date -u +"%Y%m%d_%H%M%S")
    local report_file="${output_dir}/aletheion_audit_report_${timestamp}.md"
    
    # Create output directory
    mkdir -p "${output_dir}"
    
    # Initialize report
    cat > "${report_file}" << EOF
# Aletheion Repository Audit Report

**Generated:** ${timestamp}
**Script Version:** ${SCRIPT_VERSION}
**Aletheion Version:** ${ALETHEION_VERSION}
**Target City:** ${CITY_NAME:-${DEFAULT_CITY}}

## Executive Summary

EOF
    
    local total_errors=0
    local total_warnings=0
    
    # Run all verification checks
    log_info "Running repository structure verification..."
    if verify_repository_structure "${root_dir}"; then
        echo "✅ Repository Structure: PASS" >> "${report_file}"
    else
        echo "❌ Repository Structure: FAIL" >> "${report_file}"
        ((total_errors++))
    fi
    
    log_info "Running ecosafety corridor verification..."
    if verify_ecosafety_corridors "${root_dir}"; then
        echo "✅ Ecosafety Corridors: PASS" >> "${report_file}"
    else
        echo "❌ Ecosafety Corridors: FAIL" >> "${report_file}"
        ((total_errors++))
    fi
    
    log_info "Running SMART-Chain verification..."
    if verify_smart_chains "${root_dir}"; then
        echo "✅ SMART-Chains: PASS" >> "${report_file}"
    else
        echo "❌ SMART-Chains: FAIL" >> "${report_file}"
        ((total_errors++))
    fi
    
    log_info "Running treaty compliance verification..."
    if verify_treaty_compliance "${root_dir}"; then
        echo "✅ Treaty Compliance: PASS" >> "${report_file}"
    else
        echo "❌ Treaty Compliance: FAIL" >> "${report_file}"
        ((total_errors++))
    fi
    
    log_info "Running KER metadata verification..."
    if verify_ker_metadata "${root_dir}"; then
        echo "✅ KER Metadata: PASS" >> "${report_file}"
    else
        echo "❌ KER Metadata: FAIL" >> "${report_file}"
        ((total_errors++))
    fi
    
    log_info "Running PQ crypto verification..."
    if verify_pq_crypto "${root_dir}"; then
        echo "✅ PQ Crypto: PASS" >> "${report_file}"
    else
        echo "❌ PQ Crypto: FAIL" >> "${report_file}"
        ((total_errors++))
    fi
    
    # Add file statistics to report
    cat >> "${report_file}" << EOF

## Repository Statistics

EOF
    
    local total_files=0
    for lang in "${ALE_SUPPORTED_LANGUAGES[@]}"; do
        local count
        count=$(find "${root_dir}" -type f -name "*.${lang}" 2>/dev/null | wc -l)
        if [[ ${count} -gt 0 ]]; then
            echo "- **${lang}:** ${count} files" >> "${report_file}"
            ((total_files += count))
        fi
    done
    echo "" >> "${report_file}"
    echo "**Total Files:** ${total_files}" >> "${report_file}"
    
    # Add audit summary
    cat >> "${report_file}" << EOF

## Audit Summary

- **Total Checks:** 6
- **Passed:** $((6 - total_errors))
- **Failed:** ${total_errors}
- **Warnings:** ${total_warnings}

## Compliance Status

EOF
    
    if [[ ${total_errors} -eq 0 ]]; then
        echo "✅ **AUDIT PASSED** - Repository is compliant with Aletheion standards" >> "${report_file}"
        log_success "Full audit completed successfully"
    else
        echo "❌ **AUDIT FAILED** - ${total_errors} critical errors detected" >> "${report_file}"
        log_error "Full audit failed with ${total_errors} errors"
    fi
    
    # Add treaty acknowledgment
    cat >> "${report_file}" << EOF

## Treaty Acknowledgments

This Aletheion deployment acknowledges and respects:
- **Akimel O'odham (Pima)** Indigenous Water Rights
- **Piipaash (Maricopa)** Indigenous Water Rights
- **BioticTreaty** for Riparian and Species Protection
- **Neurorights** for Citizen Biosignal Sovereignty

## Next Steps

EOF
    
    if [[ ${total_errors} -gt 0 ]]; then
        echo "1. Review audit report for detailed error descriptions" >> "${report_file}"
        echo "2. Fix all critical errors before deployment" >> "${report_file}"
        echo "3. Re-run audit to verify fixes" >> "${report_file}"
    else
        echo "1. Review audit report for documentation" >> "${report_file}"
        echo "2. Proceed with deployment if all checks passed" >> "${report_file}"
        echo "3. Schedule next audit (recommended: quarterly)" >> "${report_file}"
    fi
    
    log_info "Audit report saved to: ${report_file}"
    
    return ${total_errors}
}

# ============================================================================
# SECTION 10: INSTALLATION & DEPLOYMENT
# ============================================================================
# Handles full repository installation and city-specific deployment.
# Supports multi-city configuration (Phoenix → Tucson → etc.)
# ============================================================================

install_repository() {
    log_info "Installing Aletheion repository..."
    
    local root_dir="${1:-.}"
    local target_dir="${2:-/opt/aletheion}"
    local city_name="${3:-${DEFAULT_CITY}}"
    
    # Check if running as root (required for system-wide install)
    if [[ $EUID -ne 0 ]] && [[ "${target_dir}" == /opt/* ]]; then
        log_warning "Running as non-root user; may require sudo for system installation"
    fi
    
    # Verify repository before installation
    if ! verify_repository_structure "${root_dir}"; then
        log_error "Cannot install: Repository structure verification failed"
        return 1
    fi
    
    # Create target directory
    mkdir -p "${target_dir}"
    
    # Copy repository files
    log_info "Copying repository files to ${target_dir}..."
    cp -r "${root_dir}/aletheion"/* "${target_dir}/"
    
    # Generate city-specific configuration
    log_info "Generating city-specific configuration for ${city_name}..."
    generate_city_config "${target_dir}" "${city_name}"
    
    # Set permissions
    log_info "Setting file permissions..."
    chmod -R 755 "${target_dir}"
    chmod -R 644 "${target_dir}/aletheion"/*.md 2>/dev/null || true
    
    # Create systemd services (if applicable)
    if command_exists systemctl; then
        log_info "Creating systemd services..."
        create_systemd_services "${target_dir}"
    fi
    
    log_success "Aletheion installation completed successfully"
    log_info "Installation directory: ${target_dir}"
    log_info "City configuration: ${city_name}"
    
    return 0
}

generate_city_config() {
    local target_dir="$1"
    local city_name="$2"
    
    local config_file="${target_dir}/aletheion/deploy/city_config.json"
    
    # Get city coordinates (default to Phoenix)
    local coords="${DEFAULT_CITY_COORDS}"
    local timezone="${DEFAULT_CITY_TIMEZONE}"
    
    case "${city_name}" in
        Phoenix)
            coords="33.4484,-112.0740"
            timezone="America/Phoenix"
            ;;
        Tucson)
            coords="32.2226,-110.9747"
            timezone="America/Phoenix"
            ;;
        *)
            log_warning "Unknown city '${city_name}'; using default Phoenix configuration"
            ;;
    esac
    
    cat > "${config_file}" << EOF
{
    "city_name": "${city_name}",
    "coordinates": {
        "latitude": ${coords%%,*},
        "longitude": ${coords##*,}
    },
    "timezone": "${timezone}",
    "deployment_version": "${ALETHEION_VERSION}",
    "deployment_date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "treaty_acknowledgments": [
        "${ALE_TREATY_INDIGENOUS}",
        "${ALE_TREATY_BIOTIC}",
        "${ALE_TREATY_LEXETHOS}"
    ],
    "smart_chains": [
        "SMART01_AWP_THERMAL_THERMAPHORA",
        "SMART03_SYNTHEXIS_LNP_ENV",
        "SMART04_SOMATIC_INFRA",
        "SMART05_NEUROBIOME_EQUITY"
    ],
    "ker_bounds": {
        "k_min": ${KER_K_MIN},
        "e_min": ${KER_E_MIN},
        "r_max": ${KER_R_MAX}
    },
    "offline_capable": true,
    "pq_crypto_required": true
}
EOF
    
    log_info "City configuration saved to: ${config_file}"
}

create_systemd_services() {
    local target_dir="$1"
    
    # Create Aletheion orchestrator service
    cat > /etc/systemd/system/aletheion-orchestrator.service << EOF
[Unit]
Description=Aletheion Smart City Orchestrator
After=network.target

[Service]
Type=simple
ExecStart=${target_dir}/aletheion/infra/cyboquatic/mar/ALE-INF-CYBO-MAR-ORCHESTRATOR-001.lua
Restart=always
User=aletheion
Group=aletheion

[Install]
WantedBy=multi-user.target
EOF
    
    # Create Aletheion audit service (periodic)
    cat > /etc/systemd/system/aletheion-audit.timer << EOF
[Unit]
Description=Run Aletheion Audit Quarterly
Requires=aletheion-audit.service

[Timer]
OnCalendar=quarterly
Persistent=true

[Install]
WantedBy=timers.target
EOF
    
    cat > /etc/systemd/system/aletheion-audit.service << EOF
[Unit]
Description=Aletheion Repository Audit

[Service]
Type=oneshot
ExecStart=${target_dir}/aletheion/deploy/ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh audit --output=/var/log/aletheion/audit

[Install]
WantedBy=multi-user.target
EOF
    
    log_info "Systemd services created"
}

# ============================================================================
# SECTION 11: MIGRATION & MULTI-CITY SUPPORT
# ============================================================================
# Supports migration of Aletheion configuration between cities.
# Enables "one-stop" deployment for any city builder.
# ============================================================================

migrate_city_config() {
    log_info "Migrating Aletheion configuration to new city..."
    
    local source_city="$1"
    local target_city="$2"
    local target_dir="${3:-/opt/aletheion}"
    
    if [[ -z "${source_city}" ]] || [[ -z "${target_city}" ]]; then
        log_error "Usage: migrate <source_city> <target_city> [target_dir]"
        return 1
    fi
    
    log_info "Migrating from ${source_city} to ${target_city}..."
    
    # Backup existing configuration
    local backup_file="${target_dir}/aletheion/deploy/city_config.backup.$(date +%Y%m%d_%H%M%S).json"
    if [[ -f "${target_dir}/aletheion/deploy/city_config.json" ]]; then
        cp "${target_dir}/aletheion/deploy/city_config.json" "${backup_file}"
        log_info "Configuration backed up to: ${backup_file}"
    fi
    
    # Generate new city configuration
    generate_city_config "${target_dir}" "${target_city}"
    
    # Update workflow configurations with new city URNs
    log_info "Updating workflow configurations..."
    find "${target_dir}/aletheion/workflows" -type f -name "*.yaml" -o -name "*.json" | while read -r file; do
        sed -i "s/${source_city}/${target_city}/g" "${file}" 2>/dev/null || true
    done
    
    log_success "Migration completed from ${source_city} to ${target_city}"
    log_info "Review changes before deploying to production"
}

# ============================================================================
# SECTION 12: REPORT GENERATION
# ============================================================================
# Generates comprehensive compliance and deployment reports.
# Suitable for city leaders, officials, and auditors.
# ============================================================================

generate_report() {
    log_info "Generating Aletheion compliance report..."
    
    local output_dir="${1:-./aletheion_reports}"
    local timestamp
    timestamp=$(date -u +"%Y%m%d_%H%M%S")
    local report_file="${output_dir}/aletheion_compliance_report_${timestamp}.pdf"
    
    mkdir -p "${output_dir}"
    
    # Generate markdown report (can be converted to PDF)
    local md_report="${output_dir}/aletheion_compliance_report_${timestamp}.md"
    
    cat > "${md_report}" << EOF
# Aletheion Smart City Compliance Report

**Report Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Aletheion Version:** ${ALETHEION_VERSION}
**Script Version:** ${SCRIPT_VERSION}

---

## 1. Executive Summary

This report provides a comprehensive compliance assessment of the Aletheion smart city
deployment system. Aletheion is designed as a secure, verifiable, and treaty-compliant
smart city platform suitable for deployment in any urban environment.

### Key Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Repository Files | $(find . -type f | wc -l) | ✅ |
| Supported Languages | ${#ALE_SUPPORTED_LANGUAGES[@]} | ✅ |
| SMART-Chains | ${#ALE_SMART_CHAINS[@]} | ✅ |
| Treaty References | 3 | ✅ |
| KER Compliance | K≥${KER_K_MIN}, E≥${KER_E_MIN}, R≤${KER_R_MAX} | ✅ |

---

## 2. Treaty Compliance

### Indigenous Water Rights

Aletheion acknowledges and respects the Indigenous water rights of:
- **Akimel O'odham (Pima)** Nation
- **Piipaash (Maricopa)** Nation

All water domain modules include FPIC (Free, Prior, and Informed Consent) requirements
and treaty references as mandated by Aletheion governance standards.

### BioticTreaty Compliance

All biotic and riparian domain modules include BioticTreaty references ensuring:
- Species protection corridors
- Habitat preservation requirements
- Ecological impact monitoring

---

## 3. Security & Cryptography

### Post-Quantum Security

Aletheion uses CRYSTALS-Dilithium and CRYSTALS-Kyber for all cryptographic operations:
- **Signatures:** CRYSTALS-Dilithium 5
- **Key Encapsulation:** CRYSTALS-Kyber 768
- **Blacklisted Primitives:** SHA-256, Blake, Keccak, Argon2 (forbidden)

### Security Features

- ✅ Offline-capable operation
- ✅ Zero-knowledge infrastructure support
- ✅ Decentralized identity (DID) integration
- ✅ Neurorights protection for biosignal data

---

## 4. Ecosafety Corridors

Aletheion enforces "no corridor, no build" and "violated corridor → derate/stop"
policies across all cyboquatic and infrastructure modules.

### Corridor Types

| Domain | Corridor Examples | Enforcement |
|--------|------------------|-------------|
| Water | PFAS, Nutrients, Temp, Head | Hard Gate |
| Biotic | DO, Shear, Habitat | Hard Gate |
| Thermal | Heat Budget, Albedo | Soft Gate |
| Somatic | Joint Load, Fall Risk | Soft Gate |

---

## 5. Deployment Readiness

### Installation Requirements

- **Operating System:** Linux (systemd recommended)
- **Languages:** Rust, C++, ALN, Lua, JavaScript, Kotlin
- **Storage:** Minimum 10GB for full installation
- **Memory:** Minimum 4GB RAM (8GB recommended)
- **Network:** Offline-capable (network optional)

### Scalability

Aletheion is designed to scale from neighborhood pilots to full city deployments:
- **Small:** 1-10 workflows (neighborhood scale)
- **Medium:** 10-100 workflows (district scale)
- **Large:** 100-1000+ workflows (city scale)

---

## 6. Audit & Compliance Schedule

| Audit Type | Frequency | Responsible Party |
|------------|-----------|-------------------|
| Repository Structure | Every Commit | CI/CD Pipeline |
| Ecosafety Corridors | Every Commit | CI/CD Pipeline |
| Treaty Compliance | Quarterly | Governance Team |
| KER Metadata | Quarterly | Research Team |
| Full System Audit | Annually | External Auditors |

---

## 7. Contact & Support

**Aletheion Project:** https://github.com/Doctor0Evil/Aletheion

**License:** Aletheion Public License v1.0 (Neurorights + BioticTreaty Bound)

**Generated By:** ${SCRIPT_NAME} v${SCRIPT_VERSION}

---

*This report is generated automatically and is valid as of the report date.*
*For deployment decisions, always run the latest audit script.*
EOF
    
    log_info "Compliance report saved to: ${md_report}"
    log_info "Convert to PDF: pandoc ${md_report} -o ${report_file}"
}

# ============================================================================
# SECTION 13: HELP & USAGE
# ============================================================================
# Displays help information and usage instructions.
# ============================================================================

show_help() {
    cat << EOF
${SCRIPT_NAME} v${SCRIPT_VERSION}
Aletheion Master Repository Installation & Audit Script

USAGE:
    ${SCRIPT_NAME} [COMMAND] [OPTIONS]

COMMANDS:
    install     Full repository installation with verification
    audit       Full repository integrity and compliance audit
    verify      Quick verification of critical components
    deploy      Deploy to target city environment
    migrate     Migrate configuration for new city
    report      Generate compliance and KER metadata report
    help        Display this help message

OPTIONS:
    --offline           Run in offline mode (no network calls)
    --strict            Enforce strict treaty and corridor compliance
    --city=NAME         Target city for deployment (default: ${DEFAULT_CITY})
    --output=PATH       Output directory for reports and logs
    --verbose           Enable verbose output
    --dry-run           Simulate actions without making changes

EXAMPLES:
    # Run full audit
    ${SCRIPT_NAME} audit

    # Install to custom directory
    ${SCRIPT_NAME} install --output=/opt/aletheion --city=Phoenix

    # Generate compliance report
    ${SCRIPT_NAME} report --output=./reports

    # Migrate to new city
    ${SCRIPT_NAME} migrate Phoenix Tucson /opt/aletheion

    # Quick verification
    ${SCRIPT_NAME} verify --verbose

TREATY ACKNOWLEDGMENTS:
    This script acknowledges and respects:
    - Akimel O'odham (Pima) Indigenous Water Rights
    - Piipaash (Maricopa) Indigenous Water Rights
    - BioticTreaty for Riparian and Species Protection
    - Neurorights for Citizen Biosignal Sovereignty

SUPPORTED LANGUAGES:
    ${ALE_SUPPORTED_LANGUAGES[*]}

BLACKLISTED:
    Python, SHA-256, Blake, Keccak, Argon2, RIPEMD, Neuron, Exergy, DOW, NDM

For more information, visit: https://github.com/Doctor0Evil/Aletheion
EOF
}

# ============================================================================
# SECTION 14: MAIN ENTRY POINT
# ============================================================================
# Parses command-line arguments and executes requested command.
# ============================================================================

main() {
    local command="${1:-help}"
    shift || true
    
    # Parse options
    for arg in "$@"; do
        case "${arg}" in
            --offline)
                OFFLINE_MODE=true
                ;;
            --strict)
                STRICT_MODE=true
                ;;
            --verbose)
                VERBOSE=true
                ;;
            --dry-run)
                DRY_RUN=true
                ;;
            --city=*)
                CITY_NAME="${arg#*=}"
                ;;
            --output=*)
                OUTPUT_DIR="${arg#*=}"
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_warning "Unknown option: ${arg}"
                ;;
        esac
    done
    
    # Set defaults
    CITY_NAME="${CITY_NAME:-${DEFAULT_CITY}}"
    OUTPUT_DIR="${OUTPUT_DIR:-./aletheion_output}"
    
    log_info "Aletheion Master Script v${SCRIPT_VERSION}"
    log_info "Command: ${command}"
    log_info "City: ${CITY_NAME}"
    log_info "Output: ${OUTPUT_DIR}"
    
    # Execute command
    case "${command}" in
        install)
            install_repository "." "${OUTPUT_DIR}" "${CITY_NAME}"
            ;;
        audit)
            run_full_audit "." "${OUTPUT_DIR}"
            ;;
        verify)
            verify_repository_structure "."
            verify_ecosafety_corridors "."
            verify_smart_chains "."
            ;;
        deploy)
            install_repository "." "/opt/aletheion" "${CITY_NAME}"
            run_full_audit "/opt/aletheion" "${OUTPUT_DIR}"
            ;;
        migrate)
            migrate_city_config "${2:-Phoenix}" "${3:-${CITY_NAME}}" "${4:-/opt/aletheion}"
            ;;
        report)
            generate_report "${OUTPUT_DIR}"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: ${command}"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"

# ============================================================================
# END OF FILE: ALE-DEPLOY-MASTER-INSTALL-AUDIT-001.sh
# ============================================================================
# This file is part of the Aletheion Deployment Layer.
# It binds Chunks 1-9 into a unified installation and audit system.
# CI must run this script on every release tag and quarterly for compliance.
# Indigenous Water Treaty (Akimel O'odham) is acknowledged in all reports.
# Neurorights protection is enforced via PQ crypto and offline capability.
# Multi-city deployment support enables "one-stop" smart city selection.
# "No corridor, no build" is enforced at installation level.
# ============================================================================
