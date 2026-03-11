use core::fmt;
use core::hash::{Hash, Hasher};

#[derive(Clone, Debug, Eq)]
pub struct GreatFnId {
    city: u16,
    district: u16,
    block: u16,
    lane: u16,
}

impl GreatFnId {
    pub fn new(city: u16, district: u16, block: u16, lane: u16) -> Self {
        Self { city, district, block, lane }
    }

    pub fn city(&self) -> u16 {
        self.city
    }

    pub fn district(&self) -> u16 {
        self.district
    }

    pub fn block(&self) -> u16 {
        self.block
    }

    pub fn lane(&self) -> u16 {
        self.lane
    }
}

impl PartialEq for GreatFnId {
    fn eq(&self, other: &Self) -> bool {
        self.city == other.city
            && self.district == other.district
            && self.block == other.block
            && self.lane == other.lane
    }
}

impl Hash for GreatFnId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.city);
        state.write_u16(self.district);
        state.write_u16(self.block);
        state.write_u16(self.lane);
    }
}

impl fmt::Display for GreatFnId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}-{}-{}", self.city, self.district, self.block, self.lane)
    }
}
