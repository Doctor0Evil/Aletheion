#![forbid(unsafe_code)]

use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PesticideUnit {
    GramPerHectare,
    LiterPerHectare,
    PoundPerAcre,
    OuncePerAcre,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationMethod {
    GroundBoom,
    Aerial,
    DripIrrigation,
    PivotIrrigation,
    HandSpray,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhoenixSoilSeries {
    CasaGrande,
    Mohall,
    Avondale,
    Gilman,
    Mohave,
    Laveen,
    Glendale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoCellId {
    pub township: u16,
    pub range: u16,
    pub section: u8,
    pub quarter: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FpicConsentTag {
    pub tribal_code: [u8; 4],
    pub consent_epoch_utc: u64,
    pub consent_version: u16,
    pub on_device_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TribalDataScope {
    None,
    JointStewardship,
    IndependentStewardship,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicatorIdentity {
    pub epa_establishment_id: [u8; 12],
    pub operator_local_id: [u8; 16],
    pub tribal_scope: TribalDataScope,
}

#[derive(Clone, Debug)]
pub struct PesticideEvent {
    pub event_epoch_utc: u64,
    pub geo_cell: GeoCellId,
    pub phoenix_soil: PhoenixSoilSeries,
    pub active_ingredient: [u8; 32],
    pub product_epa_reg_no: [u8; 12],
    pub application_method: ApplicationMethod,
    pub rate_value: u32,
    pub rate_unit: PesticideUnit,
    pub total_area_sq_m: u32,
    pub applicator: ApplicatorIdentity,
    pub fpic_consent: Option<FpicConsentTag>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PesticideEventError {
    MissingRegNumber,
    InvalidRateUnit,
    ZeroArea,
    InvalidGeoCell,
    MissingApplicator,
    MissingFpicForTribalScope,
}

impl fmt::Display for PesticideEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PesticideEventError::MissingRegNumber => write!(f, "missing EPA registration number"),
            PesticideEventError::InvalidRateUnit => write!(f, "invalid rate unit for Phoenix soils"),
            PesticideEventError::ZeroArea => write!(f, "total treated area is zero"),
            PesticideEventError::InvalidGeoCell => write!(f, "invalid PLSS geo cell"),
            PesticideEventError::MissingApplicator => write!(f, "missing applicator identity"),
            PesticideEventError::MissingFpicForTribalScope => {
                write!(f, "FPIC consent required for tribal scope event")
            }
        }
    }
}

impl PesticideEvent {
    pub fn validate(&self) -> Result<(), PesticideEventError> {
        if self.product_epa_reg_no.iter().all(|b| *b == 0) {
            return Err(PesticideEventError::MissingRegNumber);
        }
        if self.total_area_sq_m == 0 {
            return Err(PesticideEventError::ZeroArea);
        }
        if self.geo_cell.section == 0 || self.geo_cell.section > 36 {
            return Err(PesticideEventError::InvalidGeoCell);
        }
        if self.applicator.epa_establishment_id.iter().all(|b| *b == 0) {
            return Err(PesticideEventError::MissingApplicator);
        }
        match self.applicator.tribal_scope {
            TribalDataScope::None => {}
            TribalDataScope::JointStewardship | TribalDataScope::IndependentStewardship => {
                if self.fpic_consent.is_none() {
                    return Err(PesticideEventError::MissingFpicForTribalScope);
                }
            }
        }
        match self.rate_unit {
            PesticideUnit::GramPerHectare
            | PesticideUnit::LiterPerHectare
            | PesticideUnit::PoundPerAcre
            | PesticideUnit::OuncePerAcre => {}
        }
        Ok(())
    }
}
