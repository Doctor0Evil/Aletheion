#![no_std]
#![deny(warnings)]
#![forbid(unsafe_code)]

use core::fmt::Debug;
use core::ops::{Add, Sub, Mul};

pub const CORRIDOR_VERSION: u32 = 20260310;
pub const MAX_RISK_COORDINATES: usize = 64;
pub const LYAPUNOV_EPSILON: f64 = 1e-9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoordinate {
    pub id: u16,
    pub rx: f64,
    pub threshold: f64,
    pub timestamp_ns: u64,
}

impl RiskCoordinate {
    pub const fn new(id: u16, rx: f64, threshold: f64, ts: u64) -> Self {
        Self { id, rx, threshold, timestamp_ns: ts }
    }
    pub fn violated(&self) -> bool { self.rx > self.threshold }
    pub fn margin(&self) -> f64 { self.threshold - self.rx }
    pub fn normalized(&self) -> f64 { self.rx / self.threshold }
}

#[derive(Clone, Debug)]
pub struct LyapunovResidual {
    pub vt: f64,
    pub vt_dot: f64,
    pub stable: bool,
    pub epoch: u64,
}

impl LyapunovResidual {
    pub fn new(vt: f64, vt_dot: f64, epoch: u64) -> Self {
        let stable = vt_dot <= -LYAPUNOV_EPSILON * vt.abs();
        Self { vt, vt_dot, stable, epoch }
    }
    pub fn converging(&self) -> bool { self.vt_dot < 0.0 && self.stable }
    pub fn diverging(&self) -> bool { self.vt_dot > LYAPUNOV_EPSILON }
}

#[derive(Clone)]
pub struct CorridorTable {
    pub coordinates: [Option<RiskCoordinate>; MAX_RISK_COORDINATES],
    pub count: usize,
    pub checksum: u64,
}

impl CorridorTable {
    pub const fn new() -> Self {
        const NONE: Option<RiskCoordinate> = None;
        Self { coordinates: [NONE; MAX_RISK_COORDINATES], count: 0, checksum: 0 }
    }
    pub fn insert(&mut self, rc: RiskCoordinate) -> Result<(), &'static str> {
        if self.count >= MAX_RISK_COORDINATES { return Err("CORRIDOR_FULL"); }
        for i in 0..self.count {
            if self.coordinates[i].unwrap().id == rc.id {
                self.coordinates[i] = Some(rc);
                return Ok(());
            }
        }
        self.coordinates[self.count] = Some(rc);
        self.count += 1;
        self.update_checksum();
        Ok(())
    }
    pub fn all_valid(&self) -> bool {
        for i in 0..self.count {
            if self.coordinates[i].unwrap().violated() { return false; }
        }
        true
    }
    pub fn worst_margin(&self) -> f64 {
        let mut min_margin = f64::MAX;
        for i in 0..self.count {
            let m = self.coordinates[i].unwrap().margin();
            if m < min_margin { min_margin = m; }
        }
        min_margin
    }
    fn update_checksum(&mut self) {
        let mut sum: u64 = 0;
        for i in 0..self.count {
            let rc = self.coordinates[i].unwrap();
            sum ^= (rc.id as u64).wrapping_mul((rc.rx * 1e6) as u64);
            sum = sum.rotate_left(7);
        }
        self.checksum = sum;
    }
    pub fn verify(&self) -> bool {
        let mut sum: u64 = 0;
        for i in 0..self.count {
            let rc = self.coordinates[i].unwrap();
            sum ^= (rc.id as u64).wrapping_mul((rc.rx * 1e6) as u64);
            sum = sum.rotate_left(7);
        }
        sum == self.checksum
    }
}

#[derive(Clone, Debug)]
pub struct EcosafetyState {
    pub corridors: CorridorTable,
    pub lyapunov: LyapunovResidual,
    pub derate_factor: f64,
    pub halt_required: bool,
    pub last_check_ns: u64,
}

impl EcosafetyState {
    pub fn new(epoch: u64) -> Self {
        Self {
            corridors: CorridorTable::new(),
            lyapunov: LyapunovResidual::new(1.0, -0.01, epoch),
            derate_factor: 1.0,
            halt_required: false,
            last_check_ns: epoch,
        }
    }
    pub fn check(&mut self, now_ns: u64) -> bool {
        self.last_check_ns = now_ns;
        if !self.corridors.all_valid() {
            self.halt_required = true;
            self.derate_factor = 0.0;
            return false;
        }
        if self.lyapunov.diverging() {
            self.derate_factor = (self.derate_factor * 0.9).max(0.1);
            if self.derate_factor < 0.2 { self.halt_required = true; }
            return false;
        }
        if self.lyapunov.converging() && self.derate_factor < 1.0 {
            self.derate_factor = (self.derate_factor + 0.05).min(1.0);
        }
        self.halt_required = false;
        true
    }
    pub fn allowed_power(&self, max_power: f64) -> f64 {
        if self.halt_required { return 0.0; }
        max_power * self.derate_factor
    }
}

pub trait CorridorEnforced {
    fn ecosafety_state(&self) -> &EcosafetyState;
    fn ecosafety_state_mut(&mut self) -> &mut EcosafetyState;
    fn pre_actuation_check(&mut self, now_ns: u64) -> Result<(), &'static str> {
        if self.ecosafety_state_mut().check(now_ns) { Ok(()) }
        else { Err("CORRIDOR_VIOLATION") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_corridor_insert_and_verify() {
        let mut ct = CorridorTable::new();
        let rc = RiskCoordinate::new(1, 0.5, 0.8, 1000);
        assert!(ct.insert(rc).is_ok());
        assert!(ct.verify());
        assert!(ct.all_valid());
    }
    #[test]
    fn test_violation_detection() {
        let mut ct = CorridorTable::new();
        let rc = RiskCoordinate::new(1, 0.9, 0.8, 1000);
        assert!(ct.insert(rc).is_ok());
        assert!(!ct.all_valid());
    }
    #[test]
    fn test_lyapunov_stability() {
        let lr = LyapunovResidual::new(1.0, -0.02, 1000);
        assert!(lr.converging());
        assert!(!lr.diverging());
    }
}
