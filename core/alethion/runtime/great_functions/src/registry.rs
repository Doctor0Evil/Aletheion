use core::collections::BTreeMap;

use crate::id::GreatFnId;
use crate::{GreatFunction, GreatFnDescriptor};

#[derive(Clone, Debug)]
pub struct GreatFnHandle {
    pub id: GreatFnId,
    pub index: u32,
}

pub struct GreatFnRegistry {
    next_index: u32,
    funcs: BTreeMap<GreatFnId, Box<dyn GreatFunction>>,
    descriptors: BTreeMap<GreatFnId, GreatFnDescriptor>,
}

impl GreatFnRegistry {
    pub fn new() -> Self {
        Self {
            next_index: 1,
            funcs: BTreeMap::new(),
            descriptors: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, func: Box<dyn GreatFunction>) -> GreatFnHandle {
        let id = func.descriptor().id.clone();
        let descriptor = func.descriptor().clone();
        let index = self.next_index;
        self.next_index = self.next_index.saturating_add(1);
        self.funcs.insert(id.clone(), func);
        self.descriptors.insert(id.clone(), descriptor);
        GreatFnHandle { id, index }
    }

    pub fn get_mut(&mut self, id: &GreatFnId) -> Option<&mut Box<dyn GreatFunction>> {
        self.funcs.get_mut(id)
    }

    pub fn descriptor(&self, id: &GreatFnId) -> Option<&GreatFnDescriptor> {
        self.descriptors.get(id)
    }
}
