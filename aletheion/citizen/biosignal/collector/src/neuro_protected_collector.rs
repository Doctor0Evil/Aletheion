#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

use core::fmt::Debug;
use core::ops::Add;

pub const NEURORIGHTS_VERSION: u32 = 20260310;
pub const MAX_BIOSIGNAL_CHANNELS: usize = 64;
pub const MAX_CONSENT_RECORDS: usize = 128;
pub const SAMPLE_RATE_HZ: u32 = 1000;
pub const ENCRYPTION_BLOCK_SIZE: usize = 32;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BiosignalType {
    EEG = 0, ECG = 1, EMG = 2, EOG = 3, GSR = 4, PPG = 5, TEMP = 6, RESP = 7,
    BCI_COMMAND = 8, NEURAL_PATTERN = 9, COGNITIVE_STATE = 10, EMOTIONAL_INDEX = 11,
}

#[derive(Clone, Copy, Debug)]
pub struct BiosignalSample {
    pub channel_id: u8,
    pub signal_type: BiosignalType,
    pub value_microvolts: i32,
    pub timestamp_ns: u64,
    pub quality_0_1: f64,
    pub encrypted: bool,
}

#[derive(Clone, Debug)]
pub struct ConsentRecord {
    pub consent_id: u64,
    pub citizen_did: [u8; 32],
    pub data_purpose: [u8; 64],
    pub granted_at_ns: u64,
    pub expires_at_ns: u64,
    pub revoked: bool,
    pub data_categories: u64,
    pub sharing_scope: u8,
    pub audit_hash: [u8; 32],
}

impl ConsentRecord {
    pub fn is_valid(&self, now_ns: u64) -> bool {
        !self.revoked && now_ns >= self.granted_at_ns && now_ns < self.expires_at_ns
    }
    pub fn allows_category(&self, category_bit: u8) -> bool {
        (self.data_categories & (1u64 << category_bit)) != 0
    }
    pub fn scope_level(&self) -> u8 {
        match self.sharing_scope {
            0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NeurorightsPolicy {
    pub mental_privacy: bool,
    pub personal_identity: bool,
    pub free_will: bool,
    pub fair_access: bool,
    pub protection_from_bias: bool,
    pub min_encryption_level: u8,
    pub max_retention_days: u32,
    pub require_explicit_consent: bool,
    pub allow_commercial_use: bool,
    pub allow_research_use: bool,
}

impl NeurorightsPolicy {
    pub const fn default_phoenix() -> Self {
        Self {
            mental_privacy: true,
            personal_identity: true,
            free_will: true,
            fair_access: true,
            protection_from_bias: true,
            min_encryption_level: 4,
            max_retention_days: 90,
            require_explicit_consent: true,
            allow_commercial_use: false,
            allow_research_use: true,
        }
    }
    pub fn all_protections_active(&self) -> bool {
        self.mental_privacy && self.personal_identity && self.free_will &&
        self.fair_access && self.protection_from_bias
    }
}

#[derive(Clone)]
pub struct BiosignalBuffer {
    pub samples: [Option<BiosignalSample>; 4096],
    pub write_index: usize,
    pub read_index: usize,
    pub count: usize,
    pub overflow_count: u64,
}

impl BiosignalBuffer {
    pub const fn new() -> Self {
        const NONE: Option<BiosignalSample> = None;
        Self {
            samples: [NONE; 4096],
            write_index: 0,
            read_index: 0,
            count: 0,
            overflow_count: 0,
        }
    }
    pub fn push(&mut self, sample: BiosignalSample) -> Result<(), &'static str> {
        if self.count >= 4096 {
            self.overflow_count += 1;
            return Err("BUFFER_FULL");
        }
        self.samples[self.write_index] = Some(sample);
        self.write_index = (self.write_index + 1) % 4096;
        self.count += 1;
        Ok(())
    }
    pub fn pop(&mut self) -> Option<BiosignalSample> {
        if self.count == 0 { return None; }
        let sample = self.samples[self.read_index];
        self.samples[self.read_index] = None;
        self.read_index = (self.read_index + 1) % 4096;
        self.count -= 1;
        sample
    }
    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
}

pub struct NeurorightsProtectedCollector {
    pub collector_id: u64,
    pub citizen_did: [u8; 32],
    pub active_channels: u8,
    pub sample_rate_hz: u32,
    pub policy: NeurorightsPolicy,
    pub consent_records: [Option<ConsentRecord>; MAX_CONSENT_RECORDS],
    pub consent_count: usize,
    pub buffer: BiosignalBuffer,
    pub encryption_key_id: [u8; 16],
    pub total_samples_collected: u64,
    pub consent_violations: u64,
    pub last_consent_check_ns: u64,
    pub audit_checksum: u64,
}

impl NeurorightsProtectedCollector {
    pub fn new(collector_id: u64, citizen_did: [u8; 32], init_ns: u64) -> Self {
        Self {
            collector_id,
            citizen_did,
            active_channels: 0,
            sample_rate_hz: SAMPLE_RATE_HZ,
            policy: NeurorightsPolicy::default_phoenix(),
            consent_records: Default::default(),
            consent_count: 0,
            buffer: BiosignalBuffer::new(),
            encryption_key_id: [0u8; 16],
            total_samples_collected: 0,
            consent_violations: 0,
            last_consent_check_ns: init_ns,
            audit_checksum: 0,
        }
    }
    pub fn add_consent_record(&mut self, consent: ConsentRecord) -> Result<(), &'static str> {
        if self.consent_count >= MAX_CONSENT_RECORDS {
            return Err("CONSENT_LIMIT_EXCEEDED");
        }
        if !self.policy.require_explicit_consent && consent.sharing_scope < 3 {
            return Err("CONSENT_SCOPE_INSUFFICIENT");
        }
        self.consent_records[self.consent_count] = Some(consent);
        self.consent_count += 1;
        self.update_audit_checksum();
        Ok(())
    }
    pub fn find_valid_consent(&self, purpose_hash: &[u8; 32], now_ns: u64) -> Option<&ConsentRecord> {
        for i in 0..self.consent_count {
            if let Some(ref consent) = self.consent_records[i] {
                if consent.is_valid(now_ns) && &consent.audit_hash == purpose_hash {
                    return Some(consent);
                }
            }
        }
        None
    }
    pub fn collect_sample(&mut self, sample: BiosignalSample, now_ns: u64) -> Result<(), &'static str> {
        if !self.policy.all_protections_active() {
            self.consent_violations += 1;
            return Err("NEURORIGHTS_POLICY_VIOLATION");
        }
        let mut encrypted_sample = sample;
        encrypted_sample.encrypted = true;
        self.buffer.push(encrypted_sample)?;
        self.total_samples_collected += 1;
        if now_ns - self.last_consent_check_ns > 3600000000000 {
            self.verify_all_consents(now_ns);
            self.last_consent_check_ns = now_ns;
        }
        self.update_audit_checksum();
        Ok(())
    }
    fn verify_all_consents(&mut self, now_ns: u64) {
        for i in 0..self.consent_count {
            if let Some(ref mut consent) = self.consent_records[i] {
                if !consent.is_valid(now_ns) && !consent.revoked {
                    consent.revoked = true;
                }
            }
        }
    }
    pub fn revoke_consent(&mut self, consent_id: u64, now_ns: u64) -> Result<(), &'static str> {
        for i in 0..self.consent_count {
            if let Some(ref mut consent) = self.consent_records[i] {
                if consent.consent_id == consent_id {
                    consent.revoked = true;
                    self.update_audit_checksum();
                    return Ok(());
                }
            }
        }
        Err("CONSENT_NOT_FOUND")
    }
    pub fn export_data_for_purpose(
        &mut self,
        purpose_hash: &[u8; 32],
        max_samples: usize,
        now_ns: u64
    ) -> Result<Vec<BiosignalSample>, &'static str> {
        let consent = self.find_valid_consent(purpose_hash, now_ns)
            .ok_or("NO_VALID_CONSENT")?;
        if !consent.allows_category(0) {
            return Err("DATA_CATEGORY_NOT_AUTHORIZED");
        }
        let mut exported = Vec::new();
        let mut count = 0;
        while count < max_samples {
            if let Some(sample) = self.buffer.pop() {
                if sample.quality_0_1 > 0.5 {
                    exported.push(sample);
                    count += 1;
                }
            } else {
                break;
            }
        }
        Ok(exported)
    }
    pub fn compute_retention_compliance(&self, now_ns: u64) -> f64 {
        let max_retention_ns = (self.policy.max_retention_days as u64) * 86400000000000;
        let mut compliant = 0u64;
        let mut total = 0u64;
        for i in 0..self.buffer.len() {
            if let Some(ref sample) = self.buffer.samples[i] {
                total += 1;
                if now_ns - sample.timestamp_ns < max_retention_ns {
                    compliant += 1;
                }
            }
        }
        if total == 0 { return 1.0; }
        compliant as f64 / total as f64
    }
    pub fn get_collector_status(&self, now_ns: u64) -> CollectorStatus {
        let valid_consents = self.consent_records.iter()
            .filter(|c| c.as_ref().map(|r| r.is_valid(now_ns)).unwrap_or(false))
            .count();
        let retention_compliance = self.compute_retention_compliance(now_ns);
        CollectorStatus {
            collector_id: self.collector_id,
            active_channels: self.active_channels,
            buffer_fill_pct: self.buffer.len() as f64 / 4096.0 * 100.0,
            valid_consents: valid_consents,
            total_consents: self.consent_count,
            consent_violations: self.consent_violations,
            total_samples: self.total_samples_collected,
            retention_compliance,
            neurorights_active: self.policy.all_protections_active(),
        }
    }
    fn update_audit_checksum(&mut self) {
        let mut sum: u64 = 0;
        for i in 0..self.consent_count {
            if let Some(ref consent) = self.consent_records[i] {
                sum ^= consent.consent_id.wrapping_mul(if consent.revoked { 2 } else { 1 });
                sum = sum.rotate_left(5);
            }
        }
        sum ^= self.consent_violations;
        sum ^= self.total_samples_collected;
        self.audit_checksum = sum;
    }
    pub fn verify_audit_integrity(&self) -> bool {
        let mut sum: u64 = 0;
        for i in 0..self.consent_count {
            if let Some(ref consent) = self.consent_records[i] {
                sum ^= consent.consent_id.wrapping_mul(if consent.revoked { 2 } else { 1 });
                sum = sum.rotate_left(5);
            }
        }
        sum ^= self.consent_violations;
        sum ^= self.total_samples_collected;
        sum == self.audit_checksum
    }
}

#[derive(Clone, Debug)]
pub struct CollectorStatus {
    pub collector_id: u64,
    pub active_channels: u8,
    pub buffer_fill_pct: f64,
    pub valid_consents: usize,
    pub total_consents: usize,
    pub consent_violations: u64,
    pub total_samples: u64,
    pub retention_compliance: f64,
    pub neurorights_active: bool,
}

impl CollectorStatus {
    pub fn health_score(&self) -> f64 {
        let mut score = 1.0;
        if !self.neurorights_active { score -= 0.5; }
        if self.consent_violations > 0 { score -= (self.consent_violations as f64 * 0.01).min(0.3); }
        if self.retention_compliance < 0.95 { score -= 0.2; }
        if self.valid_consents == 0 && self.total_consents > 0 { score -= 0.3; }
        score.max(0.0)
    }
}
