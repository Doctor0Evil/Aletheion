//! EcoNet Stakeholder Terminal Participation Batch types (Phoenix 2026 Q1).
//!
//! ERM layers: State Modeling (L2), Blockchain Trust (L3), Optimization/Scoring (L4).
//! Domain: qpudatashards / EcoNet stake terminal batches.
//!
//! This module mirrors the EcoNetStakeTerminalBatch2026Q1 ALN spec and
//! is designed for ingestion by highway / ecosafety / SMART-chain validators.

use std::time::SystemTime;

/// Host class for an EcoNet stake terminal.
/// Matches ALN `host_class` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcoStakeHostClass {
    EdgeNode,
    Desktop,
    LabNode,
    Server,
    Unknown,
}

impl EcoStakeHostClass {
    pub fn from_str(s: &str) -> Self {
        match s {
            "EDGE_NODE" => EcoStakeHostClass::EdgeNode,
            "DESKTOP" => EcoStakeHostClass::Desktop,
            "LAB_NODE" => EcoStakeHostClass::LabNode,
            "SERVER" => EcoStakeHostClass::Server,
            _ => EcoStakeHostClass::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EcoStakeHostClass::EdgeNode => "EDGE_NODE",
            EcoStakeHostClass::Desktop => "DESKTOP",
            EcoStakeHostClass::LabNode => "LAB_NODE",
            EcoStakeHostClass::Server => "SERVER",
            EcoStakeHostClass::Unknown => "UNKNOWN",
        }
    }
}

/// Operational lane for a stake terminal.
/// Mirrors ALN `lane` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcoStakeLane {
    Prod,
    Exp,
    Sandbox,
    Unknown,
}

impl EcoStakeLane {
    pub fn from_str(s: &str) -> Self {
        match s {
            "PROD" => EcoStakeLane::Prod,
            "EXP" => EcoStakeLane::Exp,
            "SANDBOX" => EcoStakeLane::Sandbox,
            _ => EcoStakeLane::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EcoStakeLane::Prod => "PROD",
            EcoStakeLane::Exp => "EXP",
            EcoStakeLane::Sandbox => "SANDBOX",
            EcoStakeLane::Unknown => "UNKNOWN",
        }
    }
}

/// Core per-row payload for EcoNet stake terminal participation.
/// Mirrors EcoNetStakeTerminalBatch2026Q1 ALN `field` list.
#[derive(Debug, Clone)]
pub struct EcoStakeTerminalRow {
    /// DID/ALN/Bostrom id, e.g., "BOSTROM-PHX-01".
    pub identity_id: String,
    /// Bostrom address (primary or alternate).
    pub bostrom_addr: String,
    /// Hardware class.
    pub host_class: EcoStakeHostClass,
    /// Total logical CPU cores.
    pub cpu_cores: u16,
    /// Max fraction of CPU available to EcoNet tasks (0.0..1.0).
    pub cpu_frac_max: f32,
    /// Max RAM (MiB) allowed for EcoNet tasks.
    pub ram_mb_max: u32,
    /// Soft power cap (W) attributable to EcoNet tasks.
    pub power_w_cap: f32,

    /// Corridor-compliance flags (ecosafety grammar).
    pub corridor_cpu_ok: bool,
    pub corridor_ram_ok: bool,
    pub corridor_power_ok: bool,

    /// Validated pollutant / CO2 mass in kilograms.
    pub ceim_mass_kg: f64,
    /// Hex proof tying to upstream CEIM run.
    pub ceim_mass_hex: String,

    /// Normalized EcoNet ecoimpact score (0.0..1.0).
    pub ecoimpact_score: f32,
    /// Knowledge factor (0.0..1.0).
    pub ker_k: f32,
    /// Eco-impact factor (0.0..1.0).
    pub ker_e: f32,
    /// Risk-of-harm factor (0.0..1.0).
    pub ker_r: f32,

    /// Telemetry trust (0.0..1.0).
    pub dt_sensor_trust: f32,
    /// Uncertainty coordinate (0.0..1.0), aligned with rsigma.
    pub rsigma_uncert: f32,

    /// Operational lane.
    pub lane: EcoStakeLane,

    /// EcoNet Karma minted for this batch.
    pub karma_units: f64,
    /// Hex of Karma Update proof.
    pub karma_hex: String,
    /// Aggregated hex of underlying qpudatashards.
    pub evidence_hex: String,

    /// Optional local timestamp (not in ALN but useful for ingestion).
    pub ingested_at: Option<SystemTime>,
}

/// Batch-level metadata (maps the [Meta] header in the ALN file).
#[derive(Debug, Clone)]
pub struct EcoStakeBatchMeta {
    pub version: String,
    pub region: String,
    pub timespan_start: String,
    pub timespan_end: String,
    pub spec_hex: String,
}

/// Entire batch (metadata plus rows) as ingested from qpudatashards.
#[derive(Debug, Clone)]
pub struct EcoStakeTerminalBatch {
    pub meta: EcoStakeBatchMeta,
    pub rows: Vec<EcoStakeTerminalRow>,
}

/// Simple ecosafety band used by lane_prod invariant.
/// Keep defaults aligned with 2026 ecosafety grammar spine (research band K>=0.90, E>=0.90, R<=0.13).
#[derive(Debug, Clone, Copy)]
pub struct EcoStakeKerBand {
    pub k_min: f32,
    pub e_min: f32,
    pub r_max: f32,
}

impl EcoStakeKerBand {
    pub fn phoenix_2026_default() -> Self {
        Self {
            k_min: 0.90,
            e_min: 0.90,
            r_max: 0.13,
        }
    }
}

/// Invariant violations for a single row.
/// These can be surfaced to ecosafety grammar and SMARTChain validators.
#[derive(Debug, Clone)]
pub enum EcoStakeInvariantViolation {
    /// PROD lane row does not satisfy K/E/R bounds.
    ProdKerBandViolation {
        identity_id: String,
        ker_k: f32,
        ker_e: f32,
        ker_r: f32,
    },
    /// Any corridor flag is false while lane is PROD.
    ProdCorridorViolation {
        identity_id: String,
        corridor_cpu_ok: bool,
        corridor_ram_ok: bool,
        corridor_power_ok: bool,
    },
}

/// Result of applying invariants over a full batch.
#[derive(Debug, Clone)]
pub struct EcoStakeBatchCheck {
    pub ok: bool,
    pub violations: Vec<EcoStakeInvariantViolation>,
}

impl EcoStakeTerminalBatch {
    /// Apply KER and corridor invariants to all rows in the batch.
    ///
    /// Mirrors ALN:
    ///   invariant K_min          0.90
    ///   invariant E_min          0.90
    ///   invariant R_max          0.13
    ///   invariant lane_prod      (lane == "PROD") ->
    ///     ((ker_K >= K_min) & (ker_E >= E_min) & (ker_R <= R_max))
    ///
    /// plus an additional requirement that all three corridor_*_ok flags
    /// must be TRUE for PROD rows.
    pub fn check_invariants(&self, band: EcoStakeKerBand) -> EcoStakeBatchCheck {
        let mut violations = Vec::new();

        for row in &self.rows {
            if row.lane == EcoStakeLane::Prod {
                if !(row.ker_k >= band.k_min
                    && row.ker_e >= band.e_min
                    && row.ker_r <= band.r_max)
                {
                    violations.push(EcoStakeInvariantViolation::ProdKerBandViolation {
                        identity_id: row.identity_id.clone(),
                        ker_k: row.ker_k,
                        ker_e: row.ker_e,
                        ker_r: row.ker_r,
                    });
                }

                if !(row.corridor_cpu_ok && row.corridor_ram_ok && row.corridor_power_ok) {
                    violations.push(EcoStakeInvariantViolation::ProdCorridorViolation {
                        identity_id: row.identity_id.clone(),
                        corridor_cpu_ok: row.corridor_cpu_ok,
                        corridor_ram_ok: row.corridor_ram_ok,
                        corridor_power_ok: row.corridor_power_ok,
                    });
                }
            }
        }

        EcoStakeBatchCheck {
            ok: violations.is_empty(),
            violations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_meta() -> EcoStakeBatchMeta {
        EcoStakeBatchMeta {
            version: "2026.1".to_string(),
            region: "Phoenix-AZ-US".to_string(),
            timespan_start: "2026-01-01T00:00:00Z".to_string(),
            timespan_end: "2026-03-31T23:59:59Z".to_string(),
            spec_hex: "0xa1b2c3d4e5f67890f1e2d3c4b5a6978899aa77cc55ee3311".to_string(),
        }
    }

    fn sample_row_prod_ok() -> EcoStakeTerminalRow {
        EcoStakeTerminalRow {
            identity_id: "BOSTROM-PHX-01".into(),
            bostrom_addr: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
            host_class: EcoStakeHostClass::EdgeNode,
            cpu_cores: 4,
            cpu_frac_max: 0.15,
            ram_mb_max: 1024,
            power_w_cap: 12.0,
            corridor_cpu_ok: true,
            corridor_ram_ok: true,
            corridor_power_ok: true,
            ceim_mass_kg: 1850.0,
            ceim_mass_hex: "0x1122334455667788".into(),
            ecoimpact_score: 0.88,
            ker_k: 0.94,
            ker_e: 0.90,
            ker_r: 0.13,
            dt_sensor_trust: 0.96,
            rsigma_uncert: 0.18,
            lane: EcoStakeLane::Prod,
            karma_units: 1239.5,
            karma_hex: "0x99aabbccddeeff00".into(),
            evidence_hex: "0xa1b2c3d4e5f67890".into(),
            ingested_at: None,
        }
    }

    #[test]
    fn prod_row_passes_default_band_and_corridors() {
        let batch = EcoStakeTerminalBatch {
            meta: sample_meta(),
            rows: vec![sample_row_prod_ok()],
        };
        let band = EcoStakeKerBand::phoenix_2026_default();
        let check = batch.check_invariants(band);
        assert!(check.ok);
        assert!(check.violations.is_empty());
    }

    #[test]
    fn prod_row_with_bad_ker_fails() {
        let mut row = sample_row_prod_ok();
        row.ker_e = 0.85; // below 0.90
        let batch = EcoStakeTerminalBatch {
            meta: sample_meta(),
            rows: vec![row],
        };
        let band = EcoStakeKerBand::phoenix_2026_default();
        let check = batch.check_invariants(band);
        assert!(!check.ok);
        assert!(!check.violations.is_empty());
    }

    #[test]
    fn prod_row_with_bad_corridor_flag_fails() {
        let mut row = sample_row_prod_ok();
        row.corridor_power_ok = false;
        let batch = EcoStakeTerminalBatch {
            meta: sample_meta(),
            rows: vec![row],
        };
        let band = EcoStakeKerBand::phoenix_2026_default();
        let check = batch.check_invariants(band);
        assert!(!check.ok);
        assert!(!check.violations.is_empty());
    }
}
