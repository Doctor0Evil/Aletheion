// Security: PQC-Compliant, Immutable Audit Log, Offline-Capable

#![no_std]
#![feature(never_type)]
use aletheion_aln::schema::GrievanceRecord;
use aletheion_crypto::pqc_verify;
use aletheion_time::{UnixTimestamp_Nano, Duration_Nano};
use aletheion_ledger::ImmutableLog;

pub struct GrievanceRouter {
    current_time: UnixTimestamp_Nano,
    treaty_db: &'static TreatyMappingMatrix,
    authority_registry: &'static AuthorityRegistry,
    audit_log: &'static mut ImmutableLog,
}

impl GrievanceRouter {
    pub const fn new(
        time: UnixTimestamp_Nano,
        treaty_db: &'static TreatyMappingMatrix,
        authority_reg: &'static AuthorityRegistry,
        log: &'static mut ImmutableLog,
    ) -> Self {
        Self { current_time: time, treaty_db: treaty_db, authority_registry: authority_reg, audit_log: log }
    }

    pub fn validate_and_route(&mut self, grievance: &GrievanceRecord) -> Result<RoutingDecision, GrievanceError> {
        // 1. Cryptographic Signature Verification
        if !pqc_verify::verify(&grievance.signature_bytes, &grievance.complainant_did) {
            return Err(GrievanceError::SignatureInvalid);
        }

        // 2. Temporal Validity (Must be filed within 90 days of incident)
        const FILING_WINDOW_NANOS: u64 = 90 * 24 * 60 * 60 * 1_000_000_000;
        if self.current_time - grievance.incident_timestamp > FILING_WINDOW_NANOS {
            return Err(GrievanceError::FilingWindowExpired);
        }

        // 3. Treaty Violation Check (Hard Constraint)
        for treaty_id in &grievance.treaty_references {
            if !self.treaty_db.is_recognized(treaty_id) {
                return Err(GrievanceError::TreatyNotRecognized);
            }
        }

        // 4. Severity-Based Routing Logic
        let routing = match grievance.urgency_flag {
            UrgencyFlag::Existential => self.route_existential(grievance)?,
            UrgencyFlag::Emergency => self.route_emergency(grievance)?,
            UrgencyFlag::Expedited => self.route_expedited(grievance)?,
            UrgencyFlag::Routine => self.route_routine(grievance)?,
        };

        // 5. Immutable Audit Log Append
        self.audit_log.append("GRIEVANCE_FILED", &grievance.grievance_id)?;

        Ok(routing)
    }

    fn route_existential(&self, g: &GrievanceRecord) -> Result<RoutingDecision, GrievanceError> {
        // Existential threats trigger immediate system halt + Tribal/City emergency session
        Ok(RoutingDecision {
            primary_authority: self.authority_registry.get("Emergency_Council")?,
            secondary_authorities: vec!["Tribal_Council", "City_Council", "AI_Oversight_Board"],
            response_deadline: self.current_time + (12 * 60 * 60 * 1_000_000_000), // 12 hours
            auto_halt_triggered: true,
            public_disclosure_required: true,
        })
    }

    fn route_emergency(&self, g: &GrievanceRecord) -> Result<RoutingDecision, GrievanceError> {
        Ok(RoutingDecision {
            primary_authority: self.authority_registry.get_by_incident_type(&g.incident_type)?,
            secondary_authorities: vec!["Civil_Rights_Office"],
            response_deadline: self.current_time + (24 * 60 * 60 * 1_000_000_000), // 24 hours
            auto_halt_triggered: false,
            public_disclosure_required: true,
        })
    }

    fn route_expedited(&self, g: &GrievanceRecord) -> Result<RoutingDecision, GrievanceError> {
        Ok(RoutingDecision {
            primary_authority: self.authority_registry.get_by_incident_type(&g.incident_type)?,
            secondary_authorities: vec![],
            response_deadline: self.current_time + (72 * 60 * 60 * 1_000_000_000), // 72 hours
            auto_halt_triggered: false,
            public_disclosure_required: false,
        })
    }

    fn route_routine(&self, g: &GrievanceRecord) -> Result<RoutingDecision, GrievanceError> {
        Ok(RoutingDecision {
            primary_authority: self.authority_registry.get_by_incident_type(&g.incident_type)?,
            secondary_authorities: vec![],
            response_deadline: self.current_time + (7 * 24 * 60 * 60 * 1_000_000_000), // 7 days
            auto_halt_triggered: false,
            public_disclosure_required: false,
        })
    }
}

pub struct RoutingDecision {
    pub primary_authority: DID_URI,
    pub secondary_authorities: Vec<&'static str>,
    pub response_deadline: UnixTimestamp_Nano,
    pub auto_halt_triggered: bool,
    pub public_disclosure_required: bool,
}

pub enum GrievanceError {
    SignatureInvalid,
    FilingWindowExpired,
    TreatyNotRecognized,
    AuthorityNotFound,
    AuditLogFailure,
}
