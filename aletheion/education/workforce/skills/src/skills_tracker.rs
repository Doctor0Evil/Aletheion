#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

pub const SKILLS_TRACKER_VERSION: u32 = 20260310;
pub const MAX_SKILL_CATEGORIES: usize = 512;
pub const MAX_INDIVIDUAL_SKILLS: usize = 65536;
pub const MAX_CITIZEN_PROFILES: usize = 524288;
pub const MAX_JOB_MATCHES: usize = 1048576;
pub const PHOENIX_LIVING_WAGE_USD_HOUR: f64 = 22.50;
pub const SKILL_VALIDITY_YEARS: u32 = 5;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SkillCategory {
    Technical = 0, SoftSkills = 1, Trade = 2, Healthcare = 3,
    IT = 4, Education = 5, Construction = 6, Manufacturing = 7,
    Hospitality = 8, Transportation = 9, Agriculture = 10, Energy = 11,
    IndigenousKnowledge = 12, Sustainability = 13, Leadership = 14,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProficiencyLevel {
    Novice = 0, Beginner = 1, Intermediate = 2, Advanced = 3,
    Expert = 4, Master = 5,
}

#[derive(Clone, Copy, Debug)]
pub struct SkillDefinition {
    pub skill_id: u64,
    pub skill_name: [u8; 128],
    pub category: SkillCategory,
    pub description: [u8; 512],
    pub proficiency_levels: [ProficiencyLevel; 6],
    pub assessment_criteria: [u8; 256],
    pub validity_years: u32,
    pub industry_recognized: bool,
    pub indigenous_knowledge: bool,
    pub accessibility_accommodations: bool,
    pub demand_score: f64,
    pub average_wage_usd_hour: f64,
    pub created_at_ns: u64,
    pub updated_at_ns: u64,
}

impl SkillDefinition {
    pub fn is_in_demand(&self) -> bool {
        self.demand_score >= 0.7
    }
    pub fn pays_living_wage(&self) -> bool {
        self.average_wage_usd_hour >= PHOENIX_LIVING_WAGE_USD_HOUR
    }
    pub fn requires_renewal(&self, acquired_at_ns: u64, now_ns: u64) -> bool {
        let elapsed_years = (now_ns - acquired_at_ns) / 31536000000000000;
        elapsed_years >= self.validity_years as u64
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CitizenSkillProfile {
    pub citizen_id: u64,
    pub citizen_did: [u8; 32],
    pub acquired_skills: [u64; 64],
    pub skill_count: u8,
    pub proficiency_scores: [f64; 64],
    pub acquisition_dates_ns: [u64; 64],
    pub certifications: [u64; 32],
    pub certification_count: u8,
    pub work_experience_years: u16,
    pub education_level: u8,
    pub indigenous_community: bool,
    pub disability_status: bool,
    pub low_income_status: bool,
    pub employment_status: u8,
    pub target_wage_usd_hour: f64,
    pub career_goals: [u8; 256],
    pub skill_gaps: [u64; 32],
    pub skill_gap_count: u8,
    pub last_updated_ns: u64,
}

impl CitizenSkillProfile {
    pub fn compute_skill_match(&self, required_skills: &[u64]) -> f64 {
        let mut matched = 0u8;
        for &req_skill in required_skills.iter() {
            for i in 0..self.skill_count {
                if self.acquired_skills[i as usize] == req_skill {
                    matched += 1;
                    break;
                }
            }
        }
        if required_skills.is_empty() { return 1.0; }
        matched as f64 / required_skills.len() as f64
    }
    pub fn compute_wage_potential(&self, skills: &[SkillDefinition; 1024]) -> f64 {
        let mut total_wage = 0.0;
        let mut count = 0u8;
        for i in 0..self.skill_count {
            let skill_id = self.acquired_skills[i as usize];
            for skill in skills.iter() {
                if skill.skill_id == skill_id {
                    total_wage += skill.average_wage_usd_hour * self.proficiency_scores[i as usize];
                    count += 1;
                    break;
                }
            }
        }
        if count == 0 { return 0.0; }
        total_wage / count as f64
    }
    pub fn requires_upskilling(&self, now_ns: u64, skills: &[SkillDefinition; 1024]) -> bool {
        for i in 0..self.skill_count {
            let skill_id = self.acquired_skills[i as usize];
            for skill in skills.iter() {
                if skill.skill_id == skill_id {
                    if skill.requires_renewal(self.acquisition_dates_ns[i as usize], now_ns) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JobMatching {
    pub match_id: u64,
    pub citizen_id: u64,
    pub job_id: u64,
    pub match_score: f64,
    pub skill_match_pct: f64,
    pub wage_match_pct: f64,
    pub location_match_score: f64,
    pub accessibility_match: bool,
    pub indigenous_preference_applied: bool,
    pub recommended_at_ns: u64,
    pub applied_at_ns: u64,
    pub hired_at_ns: u64,
    pub status: u8,
}

pub struct SkillsCompetencyTracker {
    pub tracker_id: u64,
    pub city_code: [u8; 8],
    pub skill_definitions: [Option<SkillDefinition>; 1024],
    pub skill_count: usize,
    pub citizen_profiles: [Option<CitizenSkillProfile>; MAX_CITIZEN_PROFILES],
    pub profile_count: usize,
    pub job_matchings: [Option<JobMatching>; MAX_JOB_MATCHES],
    pub matching_count: usize,
    pub total_skills_tracked: u64,
    pub total_certifications: u64,
    pub total_job_matches: u64,
    pub total_hires: u64,
    pub average_match_score: f64,
    pub living_wage_placement_rate: f64,
    pub indigenous_employment_rate: f64,
    pub disability_employment_rate: f64,
    pub last_matching_run_ns: u64,
    pub audit_checksum: u64,
}

impl SkillsCompetencyTracker {
    pub fn new(tracker_id: u64, city_code: [u8; 8], init_ns: u64) -> Self {
        Self {
            tracker_id,
            city_code,
            skill_definitions: Default::default(),
            skill_count: 0,
            citizen_profiles: Default::default(),
            profile_count: 0,
            job_matchings: Default::default(),
            matching_count: 0,
            total_skills_tracked: 0,
            total_certifications: 0,
            total_job_matches: 0,
            total_hires: 0,
            average_match_score: 0.0,
            living_wage_placement_rate: 0.0,
            indigenous_employment_rate: 0.0,
            disability_employment_rate: 0.0,
            last_matching_run_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn register_skill(&mut self, skill: SkillDefinition) -> Result<u64, &'static str> {
        if self.skill_count >= 1024 { return Err("SKILL_LIMIT_EXCEEDED"); }
        if !skill.accessibility_accommodations { return Err("ACCESSIBILITY_COMPLIANCE_REQUIRED"); }
        self.skill_definitions[self.skill_count] = Some(skill);
        self.skill_count += 1;
        self.total_skills_tracked += 1;
        self.update_audit_checksum();
        Ok(skill.skill_id)
    }
    pub fn create_citizen_profile(&mut self, profile: CitizenSkillProfile) -> Result<u64, &'static str> {
        if self.profile_count >= MAX_CITIZEN_PROFILES { return Err("PROFILE_LIMIT_EXCEEDED"); }
        self.citizen_profiles[self.profile_count] = Some(profile);
        self.profile_count += 1;
        self.total_certifications += profile.certification_count as u64;
        self.update_audit_checksum();
        Ok(profile.citizen_id)
    }
    pub fn match_citizen_to_job(&mut self, citizen_id: u64, job_id: u64, 
                                 required_skills: &[u64], target_wage: f64,
                                 now_ns: u64) -> Result<u64, &'static str> {
        if self.matching_count >= MAX_JOB_MATCHES { return Err("MATCHING_LIMIT_EXCEEDED"); }
        let citizen = self.citizen_profiles.iter()
            .filter_map(|p| p.as_ref())
            .find(|p| p.citizen_id == citizen_id)
            .ok_or("CITIZEN_NOT_FOUND")?;
        let skill_match = citizen.compute_skill_match(required_skills);
        let wage_match = if target_wage > 0.0 {
            (citizen.compute_wage_potential(unsafe { 
                &*(&self.skill_definitions as *const _ as *const [SkillDefinition; 1024]) 
            }) / target_wage).min(1.0)
        } else { 1.0 };
        let match_score = (skill_match * 0.6 + wage_match * 0.4).min(1.0);
        let matching = JobMatching {
            match_id: self.matching_count as u64,
            citizen_id,
            job_id,
            match_score,
            skill_match_pct: skill_match * 100.0,
            wage_match_pct: wage_match * 100.0,
            location_match_score: 0.8,
            accessibility_match: true,
            indigenous_preference_applied: citizen.indigenous_community,
            recommended_at_ns: now_ns,
            applied_at_ns: 0,
            hired_at_ns: 0,
            status: 1,
        };
        self.job_matchings[self.matching_count] = Some(matching);
        self.matching_count += 1;
        self.total_job_matches += 1;
        self.update_average_match_score();
        self.update_audit_checksum();
        Ok(matching.match_id)
    }
    pub fn record_hire(&mut self, match_id: u64, hired_at_ns: u64, wage_usd_hour: f64) -> Result<(), &'static str> {
        for i in 0..self.matching_count {
            if let Some(ref mut matching) = self.job_matchings[i] {
                if matching.match_id == match_id {
                    matching.hired_at_ns = hired_at_ns;
                    matching.status = 3;
                    self.total_hires += 1;
                    if wage_usd_hour >= PHOENIX_LIVING_WAGE_USD_HOUR {
                        self.living_wage_placement_rate += 0.01;
                    }
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("MATCHING_NOT_FOUND")
    }
    fn update_average_match_score(&mut self) {
        let mut total_score = 0.0;
        let mut count = 0u64;
        for i in 0..self.matching_count {
            if let Some(ref matching) = self.job_matchings[i] {
                total_score += matching.match_score;
                count += 1;
            }
        }
        if count > 0 {
            self.average_match_score = total_score / count as f64;
        }
    }
    pub fn identify_skill_gaps(&self, citizen_id: u64) -> Vec<u64> {
        let mut gaps = Vec::new();
        for i in 0..self.profile_count {
            if let Some(ref profile) = self.citizen_profiles[i] {
                if profile.citizen_id == citizen_id {
                    for j in 0..profile.skill_gap_count {
                        gaps.push(profile.skill_gaps[j as usize]);
                    }
                    break;
                }
            }
        }
        gaps
    }
    pub fn recommend_upskilling(&self, citizen_id: u64, now_ns: u64) -> Vec<u64> {
        let mut recommendations = Vec::new();
        for i in 0..self.profile_count {
            if let Some(ref profile) = self.citizen_profiles[i] {
                if profile.citizen_id == citizen_id {
                    if profile.requires_upskilling(now_ns, unsafe {
                        &*(&self.skill_definitions as *const _ as *const [SkillDefinition; 1024])
                    }) {
                        for j in 0..profile.skill_count {
                            let skill_id = profile.acquired_skills[j as usize];
                            for skill in self.skill_definitions.iter().flatten() {
                                if skill.skill_id == skill_id && skill.is_in_demand() {
                                    recommendations.push(skill_id);
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        recommendations
    }
    pub fn get_tracker_status(&self, now_ns: u64) -> SkillsTrackerStatus {
        let employed_citizens = self.citizen_profiles.iter()
            .filter(|p| p.as_ref().map(|profile| profile.employment_status == 1).unwrap_or(false))
            .count();
        let indigenous_citizens = self.citizen_profiles.iter()
            .filter(|p| p.as_ref().map(|profile| profile.indigenous_community).unwrap_or(false))
            .count();
        let indigenous_employed = self.citizen_profiles.iter()
            .filter(|p| p.as_ref().map(|profile| 
                profile.indigenous_community && profile.employment_status == 1).unwrap_or(false))
            .count();
        let disabled_citizens = self.citizen_profiles.iter()
            .filter(|p| p.as_ref().map(|profile| profile.disability_status).unwrap_or(false))
            .count();
        let disabled_employed = self.citizen_profiles.iter()
            .filter(|p| p.as_ref().map(|profile| 
                profile.disability_status && profile.employment_status == 1).unwrap_or(false))
            .count();
        self.indigenous_employment_rate = if indigenous_citizens > 0 {
            indigenous_employed as f64 / indigenous_citizens as f64
        } else { 0.0 };
        self.disability_employment_rate = if disabled_citizens > 0 {
            disabled_employed as f64 / disabled_citizens as f64
        } else { 0.0 };
        SkillsTrackerStatus {
            tracker_id: self.tracker_id,
            total_skills: self.skill_count,
            in_demand_skills: self.skill_definitions.iter()
                .filter(|s| s.as_ref().map(|skill| skill.is_in_demand()).unwrap_or(false))
                .count(),
            total_citizen_profiles: self.profile_count,
            employed_citizens,
            indigenous_citizens,
            indigenous_employed,
            disabled_citizens,
            disabled_employed,
            total_job_matches: self.total_job_matches,
            total_hires: self.total_hires,
            average_match_score: self.average_match_score,
            living_wage_placement_rate: self.living_wage_placement_rate.min(1.0),
            indigenous_employment_rate: self.indigenous_employment_rate,
            disability_employment_rate: self.disability_employment_rate,
            last_matching_run_ns: self.last_matching_run_ns,
            last_update_ns: now_ns,
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        sum ^= (self.skill_count as u64).wrapping_mul(self.profile_count as u64);
        sum ^= self.total_job_matches;
        sum ^= self.total_hires;
        for i in 0..self.profile_count {
            if let Some(ref profile) = self.citizen_profiles[i] {
                sum ^= profile.citizen_id.wrapping_mul(profile.skill_count as u64);
            }
        }
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        sum ^= (self.skill_count as u64).wrapping_mul(self.profile_count as u64);
        sum ^= self.total_job_matches;
        sum ^= self.total_hires;
        for i in 0..self.profile_count {
            if let Some(ref profile) = self.citizen_profiles[i] {
                sum ^= profile.citizen_id.wrapping_mul(profile.skill_count as u64);
            }
        }
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct SkillsTrackerStatus {
    pub tracker_id: u64,
    pub total_skills: usize,
    pub in_demand_skills: usize,
    pub total_citizen_profiles: usize,
    pub employed_citizens: usize,
    pub indigenous_citizens: usize,
    pub indigenous_employed: usize,
    pub disabled_citizens: usize,
    pub disabled_employed: usize,
    pub total_job_matches: u64,
    pub total_hires: u64,
    pub average_match_score: f64,
    pub living_wage_placement_rate: f64,
    pub indigenous_employment_rate: f64,
    pub disability_employment_rate: f64,
    pub last_matching_run_ns: u64,
    pub last_update_ns: u64,
}

impl SkillsTrackerStatus {
    pub fn workforce_equity_index(&self) -> f64 {
        let overall_employment = self.employed_citizens as f64 / self.total_citizen_profiles.max(1) as f64;
        let indigenous_equity = self.indigenous_employment_rate;
        let disability_equity = self.disability_employment_rate;
        let living_wage_rate = self.living_wage_placement_rate;
        (overall_employment * 0.30 + indigenous_equity * 0.25 + 
         disability_equity * 0.25 + living_wage_rate * 0.20).min(1.0)
    }
}
