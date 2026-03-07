// Aletheion ERM – Water Allocation and Resilience Monitoring
// Phase I: Advanced Purified Water (AWP) & Groundwater Recharge Ingestion
// Language: Rust (no blacklisted tech, exclusion-compliant semantics)

#![forbid(unsafe_code)]

mod compliance_prelude {
    /// Thin facade to the centralized compliance utilities.
    /// Actual implementation lives in the shared compliance crate/microservice.
    pub fn pre_flight_check() {
        // In real deployment, this would call:
        //  - scan_blacklist(code_metadata)
        //  - check_neurorights_compliance(module_manifest)
        // Here we only keep the call site to satisfy the hook pattern.
        crate::compliance_utils::pre_flight_check();
    }
}

pub mod compliance_utils {
    // Stubbed interface – to be provided by centralized compliance layer.
    // Kept minimal on purpose; this module should be linked, not re‑implemented.
    pub fn pre_flight_check() {
        // No-op placeholder. Real logic lives in shared compliance utilities
        // and may halt execution if violations are detected.
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifier types keep strong domain meaning.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FacilityId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RechargeSiteId(pub String);

/// High‑level status for plants and recharge sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalStatus {
    Online,
    ReducedCapacity,
    OfflinePlanned,
    OfflineUnplanned,
}

/// Basic enumeration of water source categories for portfolio accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterSourceCategory {
    Reclaimed,
    Groundwater,
    SurfaceImported,
    Desalination,
    Other,
}

/// Representation of an Advanced Purified Water facility in Phoenix.
/// Calibrated to Cave Creek, 91st Avenue, and North Gateway programs,
/// with capacities on the order of several to tens of MGD in realistic deployment
/// (e.g., North Gateway ~7–8 MGD, Cave Creek in similar range, 91st Ave embedded
/// in a >200 MGD regional wastewater context) [web:139][web:136][web:208][web:207][web:148][web:56].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwfFacility {
    pub id: FacilityId,
    pub name: String,
    /// Design capacity in million gallons per day.
    pub design_capacity_mgd: f64,
    /// Short‑term operational ceiling in MGD (may differ from design).
    pub operational_max_mgd: f64,
    /// City portfolio classification of the underlying source mix
    /// (e.g., proportion of reclaimed vs. imported surface water) [web:207][web:56].
    pub primary_source_category: WaterSourceCategory,
    /// Whether this facility contributes to the 100‑year water supply strategy
    /// (i.e., part of the long‑range portfolio planning) [web:207][web:137][web:56].
    pub participates_in_long_range_portfolio: bool,
}

/// Snapshot of plant operations at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwfOperationalSnapshot {
    pub facility_id: FacilityId,
    pub timestamp: DateTime<Utc>,
    /// Actual treated flow delivered toward potable or non‑potable reuse (MGD).
    pub delivered_flow_mgd: f64,
    /// Fraction of design capacity currently in use (0.0–1.0).
    pub utilization_ratio: f64,
    pub status: OperationalStatus,
    /// True if the facility is experiencing an unplanned outage event.
    pub unplanned_outage_flag: bool,
    /// Simple quality indicator derived from regulatory/process metrics
    /// (e.g., all permit parameters within limits) [web:208][web:56].
    pub quality_compliant: bool,
}

/// Representation of a managed aquifer recharge or spreading site.
/// Dryland cities use such sites to store water and to spread occasional
/// flood discharges from major rivers to increase long‑term storage [web:207][web:208].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundwaterRechargeSite {
    pub id: RechargeSiteId,
    pub name: String,
    /// Source river or project name (e.g., Colorado River related imports).
    pub source_descriptor: String,
    /// Typical recharge rate in acre‑feet per year.
    pub recharge_rate_afy: f64,
    /// Estimated current stored volume in acre‑feet.
    pub stored_volume_af: f64,
    pub status: OperationalStatus,
}

/// Snapshot of a recharge site at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundwaterRechargeSnapshot {
    pub site_id: RechargeSiteId,
    pub timestamp: DateTime<Utc>,
    pub instantaneous_recharge_cfs: f64,
    pub cumulative_recharge_af: f64,
    pub status: OperationalStatus,
}

/// Ingestion error types for robust upstream logging and metrics.
#[derive(Debug)]
pub enum IngestionError {
    Connectivity { source: String, detail: String },
    Parse { source: String, detail: String },
    Validation { entity: String, detail: String },
}

impl fmt::Display for IngestionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestionError::Connectivity { source, detail } => {
                write!(f, "Connectivity error from {}: {}", source, detail)
            }
            IngestionError::Parse { source, detail } => {
                write!(f, "Parse error from {}: {}", source, detail)
            }
            IngestionError::Validation { entity, detail } => {
                write!(f, "Validation error on {}: {}", entity, detail)
            }
        }
    }
}

impl std::error::Error for IngestionError {}

/// Normalized payload that this module publishes to the Layer_2 state model.
/// This is the boundary object connecting Phase I ingestion to the overall
/// water state representation for the city.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterIngestionBatch {
    pub generated_at: DateTime<Utc>,
    pub awf_snapshots: Vec<AwfOperationalSnapshot>,
    pub recharge_snapshots: Vec<GroundwaterRechargeSnapshot>,
}

/// Trait describing the interface expected by Layer_2 for ingesting
/// new water data into the state model.
/// Implemented by whatever adapter bridges this module to the core state model.
pub trait WaterStateModelSink {
    fn apply_ingestion_batch(&mut self, batch: WaterIngestionBatch) -> Result<(), IngestionError>;
}

/// Configuration for connecting to Phoenix’s AWP facilities and related sources.
/// URLs, credentials, polling intervals and schema versions live here so that
/// the core logic can remain deterministic and testable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    pub cave_creek_endpoint: String,
    pub north_gateway_endpoint: String,
    pub ninety_first_ave_endpoint: String,
    pub recharge_data_endpoint: String,
    /// Minimum polling interval in seconds.
    pub polling_interval_seconds: u64,
    /// Optional schema/version hints for parsing external feeds.
    pub schema_version: String,
}

/// Main ingestion engine for Phase I water data.
/// This is designed to be driven by a scheduler (Lua or otherwise) that calls
/// `run_once` on a fixed interval.
pub struct WaterIngestionEngine<S: WaterStateModelSink> {
    config: IngestionConfig,
    sink: S,
}

impl<S: WaterStateModelSink> WaterIngestionEngine<S> {
    /// Create a new engine instance; performs compliance pre‑flight before use.
    pub fn new(config: IngestionConfig, sink: S) -> Result<Self, IngestionError> {
        // Centralized compliance hook, enforced at module entry.
        compliance_prelude::pre_flight_check();
        Ok(Self { config, sink })
    }

    /// One‑shot ingestion cycle: fetch snapshots from all configured sources,
    /// normalize them, and publish a batch into the state model.
    pub fn run_once(&mut self) -> Result<(), IngestionError> {
        let now = Utc::now();

        let awf_snapshots = self.collect_awf_snapshots(now)?;
        let recharge_snapshots = self.collect_recharge_snapshots(now)?;

        let batch = WaterIngestionBatch {
            generated_at: now,
            awf_snapshots,
            recharge_snapshots,
        };

        self.sink.apply_ingestion_batch(batch)
    }

    fn collect_awf_snapshots(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<AwfOperationalSnapshot>, IngestionError> {
        let mut snapshots = Vec::new();

        // In real deployment, these would be HTTP/fieldbus/SCADA calls
        // to plant data services, returning JSON/CSV payloads reflecting
        // flows, quality, and status.
        //
        // The calibration anchors for max capacities come from Phoenix’s
        // Advanced Purified Water plans: Cave Creek, 91st Avenue, and
        // North Gateway facilities ramping toward tens of MGD total
        // reclaimed supply [web:139][web:136][web:208][web:207][web:148][web:56].

        // Cave Creek placeholder
        snapshots.push(AwfOperationalSnapshot {
            facility_id: FacilityId("cave_creek_awp".to_string()),
            timestamp: now,
            delivered_flow_mgd: 0.0,
            utilization_ratio: 0.0,
            status: OperationalStatus::OfflinePlanned,
            unplanned_outage_flag: false,
            quality_compliant: true,
        });

        // North Gateway placeholder
        snapshots.push(AwfOperationalSnapshot {
            facility_id: FacilityId("north_gateway_awp".to_string()),
            timestamp: now,
            delivered_flow_mgd: 0.0,
            utilization_ratio: 0.0,
            status: OperationalStatus::OfflinePlanned,
            unplanned_outage_flag: false,
            quality_compliant: true,
        });

        // 91st Avenue placeholder
        snapshots.push(AwfOperationalSnapshot {
            facility_id: FacilityId("ninety_first_ave_awp".to_string()),
            timestamp: now,
            delivered_flow_mgd: 0.0,
            utilization_ratio: 0.0,
            status: OperationalStatus::OfflinePlanned,
            unplanned_outage_flag: false,
            quality_compliant: true,
        });

        Ok(snapshots)
    }

    fn collect_recharge_snapshots(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<GroundwaterRechargeSnapshot>, IngestionError> {
        let mut snapshots = Vec::new();

        // Real implementation would ingest managed aquifer recharge metrics
        // from city/utility or regional partners, including Colorado River
        // related recharge projects that increase storage by spreading flood
        // discharges in dryland settings [web:207][web:208].

        snapshots.push(GroundwaterRechargeSnapshot {
            site_id: RechargeSiteId("maricopa_recharge_site_1".to_string()),
            timestamp: now,
            instantaneous_recharge_cfs: 0.0,
            cumulative_recharge_af: 0.0,
            status: OperationalStatus::Online,
        });

        Ok(snapshots)
    }
}

/// Simple in‑memory sink implementation useful for tests, prototyping, or
/// local simulations of the state model interface.
pub struct InMemoryWaterStateModel {
    pub last_batch: Option<WaterIngestionBatch>,
}

impl InMemoryWaterStateModel {
    pub fn new() -> Self {
        Self { last_batch: None }
    }
}

impl WaterStateModelSink for InMemoryWaterStateModel {
    fn apply_ingestion_batch(&mut self, batch: WaterIngestionBatch) -> Result<(), IngestionError> {
        self.last_batch = Some(batch);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_runs_once_and_populates_sink() {
        let config = IngestionConfig {
            cave_creek_endpoint: "https://example/cave_creek".into(),
            north_gateway_endpoint: "https://example/north_gateway".into(),
            ninety_first_ave_endpoint: "https://example/91st_ave".into(),
            recharge_data_endpoint: "https://example/recharge".into(),
            polling_interval_seconds: 300,
            schema_version: "v1".into(),
        };

        let mut sink = InMemoryWaterStateModel::new();
        let mut engine = WaterIngestionEngine::new(config, &mut sink).expect("engine init failed");

        engine.run_once().expect("ingestion run failed");

        let batch = sink.last_batch.as_ref().expect("no batch stored");
        assert_eq!(batch.awf_snapshots.len(), 3);
        assert!(!batch.recharge_snapshots.is_empty());
    }

    #[test]
    fn ingestion_error_display_is_human_readable() {
        let err = IngestionError::Connectivity {
            source: "cave_creek_endpoint".into(),
            detail: "timeout".into(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Connectivity error"));
        assert!(msg.contains("cave_creek_endpoint"));
    }
}
