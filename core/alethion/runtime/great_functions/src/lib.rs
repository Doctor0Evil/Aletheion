#![forbid(unsafe_code)]

pub mod metrics;
pub mod registry;
pub mod routing;
pub mod id;
pub mod state;

use metrics::{GreatFnMetricKind, GreatFnMetrics};
use registry::{GreatFnHandle, GreatFnRegistry};
use routing::{GreatFnCall, GreatFnRoute};
use state::{GreatFnContext, GreatFnState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GreatFnClass {
    Signal,
    Flow,
    Topology,
    Policy,
    Ledger,
    Device,
    Civic,
    Custom(&'static str),
}

#[derive(Clone, Debug)]
pub struct GreatFnDescriptor {
    pub id: id::GreatFnId,
    pub name: &'static str,
    pub class: GreatFnClass,
    pub version_major: u16,
    pub version_minor: u16,
    pub ops_per_tick_hint: u16,
    pub city_scope: &'static str,
}

#[derive(Clone, Debug)]
pub struct GreatFnOutcome {
    pub ok: bool,
    pub ops_used: u16,
    pub warnings: u8,
}

pub trait GreatFunction: Send + Sync + 'static {
    fn descriptor(&self) -> &GreatFnDescriptor;
    fn warmup(&mut self, ctx: &mut GreatFnContext) -> GreatFnOutcome;
    fn tick(&mut self, ctx: &mut GreatFnContext, call: GreatFnCall) -> GreatFnOutcome;
    fn cooldown(&mut self, ctx: &mut GreatFnContext) -> GreatFnOutcome;
}

pub struct GreatFnEngine {
    registry: GreatFnRegistry,
    state: GreatFnState,
    metrics: GreatFnMetrics,
}

impl GreatFnEngine {
    pub fn new() -> Self {
        let registry = GreatFnRegistry::new();
        let state = GreatFnState::new();
        let metrics = GreatFnMetrics::new();
        Self { registry, state, metrics }
    }

    pub fn register(&mut self, func: Box<dyn GreatFunction>) -> GreatFnHandle {
        let id = func.descriptor().id.clone();
        let class = func.descriptor().class.clone();
        let handle = self.registry.register(func);
        self.metrics.bootstrap_for(&id, &class);
        handle
    }

    pub fn route(&mut self, route: GreatFnRoute) {
        self.state.enqueue(route);
    }

    pub fn tick(&mut self) {
        let mut ops_budget: u32 = 0;
        let mut processed: u16 = 0;
        while let Some(call) = self.state.dequeue() {
            if processed > 63 {
                break;
            }
            if let Some(func) = self.registry.get_mut(&call.target) {
                let mut ctx = GreatFnContext::new(call.city_zone, call.epoch);
                let outcome = func.tick(&mut ctx, call.clone());
                ops_budget += outcome.ops_used as u32;
                if !outcome.ok {
                    self.metrics.bump(&call.target, GreatFnMetricKind::Error);
                } else if outcome.warnings > 0 {
                    self.metrics.bump(&call.target, GreatFnMetricKind::Warn);
                } else {
                    self.metrics.bump(&call.target, GreatFnMetricKind::Ok);
                }
                processed += 1;
                if ops_budget > 512 {
                    break;
                }
            } else {
                self.metrics.bump_orphan(&call.target);
            }
        }
    }

    pub fn snapshot_metrics(&self) -> GreatFnMetrics {
        self.metrics.clone()
    }
}
