#![no_std]
#![feature(never_type)] // Rust 2024 stability align
use aletheion_aln::schema::ConsentEnvelope;
use aletheion_crypto::pqc_verify; // Abstracted PQC interface (No blacklisted algos)
use aletheion_time::UnixTimestamp_Nano;

pub struct ConsentValidator {
    current_time: UnixTimestamp_Nano,
    treaty_db: &'static TreatyMappingMatrix,
}

impl ConsentValidator {
    pub const fn new(time: UnixTimestamp_Nano, db: &'static TreatyMappingMatrix) -> Self {
        Self { current_time: time, treaty_db: db }
    }

    pub fn validate(&self, envelope: &ConsentEnvelope) -> Result<(), ConsentError> {
        // 1. Temporal Validity Check
        if self.current_time < envelope.valid_from || self.current_time > envelope.expires_at {
            return Err(ConsentError::TemporalBoundsExceeded);
        }

        // 2. Revocation Status Check (Critical Path)
        if envelope.revocation_status != crate::enums::RevocationStatus::Active {
            return Err(ConsentError::ConsentRevoked);
        }

        // 3. Cryptographic Signature Verification (PQC)
        if !pqc_verify::verify(&envelope.signature_bytes, &envelope.grantor_did) {
            return Err(ConsentError::SignatureInvalid);
        }

        // 4. Treaty Compliance Check (Hard Constraint)
        for treaty_id in &envelope.treaty_bindings {
            if !self.treaty_db.is_compliant(treaty_id, &envelope.scope_vector) {
                return Err(ConsentError::TreatyViolation);
            }
        }

        // 5. Neurorights Guardrail (No-Inference)
        for scope in &envelope.scope_vector {
            if scope.resource_id.starts_with("BIOSIGNAL_NERAL_") && scope.inference_allowed {
                return Err(ConsentError::NeurorightsViolation);
            }
        }

        Ok(())
    }
}

pub enum ConsentError {
    TemporalBoundsExceeded,
    ConsentRevoked,
    SignatureInvalid,
    TreatyViolation,
    NeurorightsViolation,
}
