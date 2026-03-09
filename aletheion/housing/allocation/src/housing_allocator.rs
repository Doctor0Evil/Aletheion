#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const HOUSING_ALLOCATOR_VERSION: u32 = 20260310;
pub const MAX_HOUSING_UNITS: usize = 65536;
pub const MAX_APPLICANTS: usize = 131072;
pub const MAX_WAITLIST_ENTRIES: usize = 262144;
pub const PHOENIX_AMI_TARGET_PCT: f64 = 60.0;
pub const AFFORDABILITY_RATIO_TARGET: f64 = 0.30;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HousingType {
    Studio = 0, OneBedroom = 1, TwoBedroom = 2, ThreeBedroom = 3,
    FourBedroom = 4, SRO = 5, Transitional = 6, PermanentSupportive = 7,
    SeniorLiving = 8, DisabilityAccessible = 9, FamilyUnit = 10, Veteran = 11,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationStatus {
    Pending = 0, Verified = 1, Approved = 2, Waitlisted = 3,
    Offered = 4, Accepted = 5, Occupied = 6, Vacant = 7,
    Denied = 8, Withdrawn = 9, Expired = 10,
}

#[derive(Clone, Copy, Debug)]
pub struct HousingUnit {
    pub unit_id: u64,
    pub building_id: u32,
    pub housing_type: HousingType,
    pub bedrooms: u8,
    pub bathrooms: u8,
    pub area_sqft: u16,
    pub monthly_rent_usd: u32,
    pub ami_percentage: u8,
    pub accessibility_features: u16,
    pub energy_efficiency_rating: u8,
    pub solar_equipped: bool,
    pub water_efficient: bool,
    pub occupancy_status: ApplicationStatus,
    pub current_resident_id: u64,
    pub lease_start_ns: u64,
    pub lease_end_ns: u64,
    pub last_inspection_ns: u64,
    pub inspection_score: u8,
    pub maintenance_required: bool,
}

impl HousingUnit {
    pub fn is_available(&self, now_ns: u64) -> bool {
        self.occupancy_status == ApplicationStatus::Vacant &&
        !self.maintenance_required &&
        self.inspection_score >= 80 &&
        now_ns > self.lease_end_ns
    }
    pub fn meets_affordability(&self, household_income_usd: f64) -> bool {
        let annual_rent = self.monthly_rent_usd as f64 * 12.0;
        let ratio = annual_rent / household_income_usd.max(1.0);
        ratio <= AFFORDABILITY_RATIO_TARGET
    }
    pub fn accessibility_compliant(&self, features_required: u16) -> bool {
        (self.accessibility_features & features_required) == features_required
    }
}

#[derive(Clone, Debug)]
pub struct HousingApplicant {
    pub applicant_id: u64,
    pub household_size: u8,
    pub annual_income_usd: f64,
    pub ami_percentage: f64,
    pub application_date_ns: u64,
    pub verification_status: ApplicationStatus,
    pub priority_score: f64,
    pub homelessness_history: bool,
    pub disability_status: bool,
    pub veteran_status: bool,
    pub senior_status: bool,
    pub family_with_children: bool,
    pub preferred_locations: [u32; 5],
    pub required_bedrooms: u8,
    pub accessibility_needs: u16,
    pub offer_count: u8,
    pub last_offer_ns: u64,
}

impl HousingApplicant {
    pub fn compute_priority_score(&mut self) -> f64 {
        let mut score = 0.0;
        score += (100.0 - self.ami_percentage).min(50.0);
        if self.homelessness_history { score += 30.0; }
        if self.disability_status { score += 15.0; }
        if self.veteran_status { score += 20.0; }
        if self.senior_status { score += 10.0; }
        if self.family_with_children { score += 15.0; }
        let wait_time_days = (SystemTime::now_ns() - self.application_date_ns) / 86400000000000;
        score += (wait_time_days as f64 * 0.1).min(20.0);
        self.priority_score = score.min(100.0);
        self.priority_score
    }
    pub fn matches_unit(&self, unit: &HousingUnit) -> bool {
        unit.required_bedrooms >= self.required_bedrooms &&
        unit.accessibility_compliant(self.accessibility_needs) &&
        unit.meets_affordability(self.annual_income_usd) &&
        self.preferred_locations.contains(&unit.building_id)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WaitlistEntry {
    pub entry_id: u64,
    pub applicant_id: u64,
    pub priority_score: f64,
    pub application_date_ns: u64,
    pub housing_type_preference: HousingType,
    pub location_preference: u32,
    pub status: ApplicationStatus,
    pub offers_made: u8,
    pub offers_accepted: u8,
    pub last_updated_ns: u64,
}

pub struct AffordableHousingAllocator {
    pub allocator_id: u64,
    pub city_code: [u8; 8],
    pub housing_units: [Option<HousingUnit>; MAX_HOUSING_UNITS],
    pub unit_count: usize,
    pub applicants: [Option<HousingApplicant>; MAX_APPLICANTS],
    pub applicant_count: usize,
    pub waitlist: [Option<WaitlistEntry>; MAX_WAITLIST_ENTRIES],
    pub waitlist_count: usize,
    pub total_units_affordable: u64,
    pub total_units_occupied: u64,
    pub total_units_vacant: u64,
    pub average_ami_served: f64,
    pub allocation_fairness_index: f64,
    pub discrimination_complaints: u64,
    pub last_audit_ns: u64,
    pub audit_checksum: u64,
}

impl AffordableHousingAllocator {
    pub fn new(allocator_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            allocator_id,
            city_code,
            housing_units: Default::default(),
            unit_count: 0,
            applicants: Default::default(),
            applicant_count: 0,
            waitlist: Default::default(),
            waitlist_count: 0,
            total_units_affordable: 0,
            total_units_occupied: 0,
            total_units_vacant: 0,
            average_ami_served: 0.0,
            allocation_fairness_index: 1.0,
            discrimination_complaints: 0,
            last_audit_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_housing_unit(&mut self, unit: HousingUnit) -> Result<u64, &'static str> {
        if self.unit_count >= MAX_HOUSING_UNITS {
            return Err("UNIT_LIMIT_EXCEEDED");
        }
        self.housing_units[self.unit_count] = Some(unit);
        self.unit_count += 1;
        if unit.ami_percentage <= 60 {
            self.total_units_affordable += 1;
        }
        self.update_audit_checksum();
        Ok(unit.unit_id)
    }
    pub fn register_applicant(&mut self, mut applicant: HousingApplicant) -> Result<u64, &'static str> {
        if self.applicant_count >= MAX_APPLICANTS {
            return Err("APPLICANT_LIMIT_EXCEEDED");
        }
        applicant.compute_priority_score();
        self.applicants[self.applicant_count] = Some(applicant);
        self.applicant_count += 1;
        self.update_audit_checksum();
        Ok(applicant.applicant_id)
    }
    pub fn add_to_waitlist(&mut self, applicant_id: u64, now_ns: u64) -> Result<u64, &'static str> {
        if self.waitlist_count >= MAX_WAITLIST_ENTRIES {
            return Err("WAITLIST_LIMIT_EXCEEDED");
        }
        let applicant = self.applicants.iter()
            .filter_map(|a| a.as_ref())
            .find(|a| a.applicant_id == applicant_id)
            .ok_or("APPLICANT_NOT_FOUND")?;
        let entry = WaitlistEntry {
            entry_id: self.waitlist_count as u64,
            applicant_id,
            priority_score: applicant.priority_score,
            application_date_ns: applicant.application_date_ns,
            housing_type_preference: HousingType::TwoBedroom,
            location_preference: 0,
            status: ApplicationStatus::Waitlisted,
            offers_made: 0,
            offers_accepted: 0,
            last_updated_ns: now_ns,
        };
        self.waitlist[self.waitlist_count] = Some(entry);
        self.waitlist_count += 1;
        self.update_audit_checksum();
        Ok(entry.entry_id)
    }
    pub fn match_applicant_to_unit(&mut self, applicant_id: u64, now_ns: u64) -> Result<u64, &'static str> {
        let applicant = self.applicants.iter()
            .filter_map(|a| a.as_ref())
            .find(|a| a.applicant_id == applicant_id)
            .ok_or("APPLICANT_NOT_FOUND")?;
        for i in 0..self.unit_count {
            if let Some(ref mut unit) = self.housing_units[i] {
                if unit.is_available(now_ns) && applicant.matches_unit(unit) {
                    unit.occupancy_status = ApplicationStatus::Offered;
                    unit.current_resident_id = applicant_id;
                    self.update_audit_checksum();
                    return Ok(unit.unit_id);
                }
            }
        }
        Err("NO_MATCHING_UNIT")
    }
    pub fn accept_housing_offer(&mut self, applicant_id: u64, unit_id: u64, 
                                 lease_start_ns: u64, lease_end_ns: u64) -> Result<(), &'static str> {
        let applicant = self.applicants.iter_mut()
            .filter_map(|a| a.as_mut())
            .find(|a| a.applicant_id == applicant_id)
            .ok_or("APPLICANT_NOT_FOUND")?;
        let unit = self.housing_units.iter_mut()
            .filter_map(|u| u.as_mut())
            .find(|u| u.unit_id == unit_id)
            .ok_or("UNIT_NOT_FOUND")?;
        applicant.verification_status = ApplicationStatus::Accepted;
        unit.occupancy_status = ApplicationStatus::Occupied;
        unit.lease_start_ns = lease_start_ns;
        unit.lease_end_ns = lease_end_ns;
        self.total_units_occupied += 1;
        self.update_audit_checksum();
        Ok(())
    }
    pub fn compute_allocation_fairness(&mut self) -> f64 {
        let mut ami_distribution = [0u64; 10];
        for i in 0..self.unit_count {
            if let Some(ref unit) = self.housing_units[i] {
                if unit.occupancy_status == ApplicationStatus::Occupied {
                    let ami_bucket = (unit.ami_percentage / 10) as usize;
                    ami_distribution[ami_bucket.min(9)] += 1;
                }
            }
        }
        let total = ami_distribution.iter().sum::<u64>();
        if total == 0 { return 1.0; }
        let expected = total / 10;
        let mut variance = 0.0;
        for &count in &ami_distribution {
            let diff = count as f64 - expected as f64;
            variance += diff * diff;
        }
        self.allocation_fairness_index = 1.0 - (variance / (total as f64)).min(1.0);
        self.allocation_fairness_index
    }
    pub fn get_allocator_status(&self, now_ns: u64) -> AllocatorStatus {
        let available_units = self.housing_units.iter()
            .filter(|u| u.as_ref().map(|unit| unit.is_available(now_ns)).unwrap_or(false))
            .count();
        let waitlist_active = self.waitlist.iter()
            .filter(|w| w.as_ref().map(|e| e.status == ApplicationStatus::Waitlisted).unwrap_or(false))
            .count();
        AllocatorStatus {
            allocator_id: self.allocator_id,
            total_units: self.unit_count,
            affordable_units: self.total_units_affordable,
            occupied_units: self.total_units_occupied,
            vacant_units: available_units as u64,
            total_applicants: self.applicant_count,
            active_waitlist: waitlist_active,
            average_ami_served: self.average_ami_served,
            allocation_fairness_index: self.allocation_fairness_index,
            discrimination_complaints: self.discrimination_complaints,
            last_audit_ns: self.last_audit_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.unit_count as u64).wrapping_mul(self.total_units_occupied);
        sum ^= (self.applicant_count as u64);
        sum ^= (self.waitlist_count as u64);
        sum ^= self.discrimination_complaints;
        for i in 0..self.unit_count {
            if let Some(ref unit) = self.housing_units[i] {
                sum ^= unit.unit_id.wrapping_mul(unit.monthly_rent_usd as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.unit_count as u64).wrapping_mul(self.total_units_occupied);
        sum ^= (self.applicant_count as u64);
        sum ^= (self.waitlist_count as u64);
        sum ^= self.discrimination_complaints;
        for i in 0..self.unit_count {
            if let Some(ref unit) = self.housing_units[i] {
                sum ^= unit.unit_id.wrapping_mul(unit.monthly_rent_usd as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct AllocatorStatus {
    pub allocator_id: u64,
    pub total_units: usize,
    pub affordable_units: u64,
    pub occupied_units: u64,
    pub vacant_units: u64,
    pub total_applicants: usize,
    pub active_waitlist: usize,
    pub average_ami_served: f64,
    pub allocation_fairness_index: f64,
    pub discrimination_complaints: u64,
    pub last_audit_ns: u64,
    pub last_update_ns: u64,
}

impl AllocatorStatus {
    pub fn housing_security_score(&self) -> f64 {
        let occupancy_rate = self.occupied_units as f64 / self.total_units.max(1) as f64;
        let affordability_rate = self.affordable_units as f64 / self.total_units.max(1) as f64;
        let fairness_score = self.allocation_fairness_index;
        let complaint_penalty = (self.discrimination_complaints as f64 * 0.02).min(0.3);
        (occupancy_rate * 0.3 + affordability_rate * 0.3 + fairness_score * 0.25 + 
         (1.0 - complaint_penalty) * 0.15).min(1.0)
    }
}
