use core::collections::VecDeque;

use crate::routing::GreatFnRoute;

#[derive(Clone, Debug)]
pub struct GreatFnContext {
    pub city_zone: u16,
    pub epoch: u64,
    pub ops_budget: u16,
}

impl GreatFnContext {
    pub fn new(city_zone: u16, epoch: u64) -> Self {
        Self { city_zone, epoch, ops_budget: 64 }
    }

    pub fn consume(&mut self, ops: u16) -> bool {
        if ops > self.ops_budget {
            return false;
        }
        self.ops_budget -= ops;
        true
    }
}

#[derive(Clone, Debug)]
pub struct GreatFnState {
    queue: VecDeque<GreatFnRoute>,
}

impl GreatFnState {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn enqueue(&mut self, route: GreatFnRoute) {
        self.queue.push_back(route);
    }

    pub fn dequeue(&mut self) -> Option<GreatFnRoute> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
