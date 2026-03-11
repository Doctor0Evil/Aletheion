use core::collections::BTreeMap;

use crate::id::GreatFnId;
use crate::GreatFnClass;

#[derive(Clone, Debug)]
pub enum GreatFnMetricKind {
    Ok,
    Warn,
    Error,
}

#[derive(Clone, Debug, Default)]
pub struct GreatFnMetricRow {
    pub oks: u64,
    pub warns: u64,
    pub errors: u64,
    pub orphans: u64,
}

#[derive(Clone, Debug, Default)]
pub struct GreatFnMetrics {
    rows: BTreeMap<GreatFnId, GreatFnMetricRow>,
    classes: BTreeMap<GreatFnId, GreatFnClass>,
}

impl GreatFnMetrics {
    pub fn new() -> Self {
        Self {
            rows: BTreeMap::new(),
            classes: BTreeMap::new(),
        }
    }

    pub fn bootstrap_for(&mut self, id: &GreatFnId, class: &GreatFnClass) {
        self.rows.entry(id.clone()).or_insert_with(GreatFnMetricRow::default);
        self.classes.entry(id.clone()).or_insert_with(|| class.clone());
    }

    pub fn bump(&mut self, id: &GreatFnId, kind: GreatFnMetricKind) {
        let row = self.rows.entry(id.clone()).or_insert_with(GreatFnMetricRow::default);
        match kind {
            GreatFnMetricKind::Ok => row.oks = row.oks.saturating_add(1),
            GreatFnMetricKind::Warn => row.warns = row.warns.saturating_add(1),
            GreatFnMetricKind::Error => row.errors = row.errors.saturating_add(1),
        }
    }

    pub fn bump_orphan(&mut self, id: &GreatFnId) {
        let row = self.rows.entry(id.clone()).or_insert_with(GreatFnMetricRow::default);
        row.orphans = row.orphans.saturating_add(1);
    }

    pub fn row(&self, id: &GreatFnId) -> Option<&GreatFnMetricRow> {
        self.rows.get(id)
    }
}
