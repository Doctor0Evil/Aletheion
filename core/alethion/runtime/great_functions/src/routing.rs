use crate::id::GreatFnId;

#[derive(Clone, Debug)]
pub struct GreatFnRoute {
    pub call: GreatFnCall,
    pub priority: u8,
}

#[derive(Clone, Debug)]
pub struct GreatFnCall {
    pub target: GreatFnId,
    pub city_zone: u16,
    pub epoch: u64,
    pub payload: GreatFnPayload,
}

#[derive(Clone, Debug)]
pub enum GreatFnPayload {
    Scalar(u64),
    Vector([u32; 4]),
    Grid([u16; 8]),
    Empty,
}

impl GreatFnRoute {
    pub fn new(target: GreatFnId, city_zone: u16, epoch: u64, payload: GreatFnPayload, priority: u8) -> Self {
        let call = GreatFnCall { target, city_zone, epoch, payload };
        Self { call, priority }
    }
}
