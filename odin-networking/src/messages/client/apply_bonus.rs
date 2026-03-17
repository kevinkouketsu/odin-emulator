use deku::prelude::*;

#[derive(Debug, DekuRead, DekuWrite)]
pub struct ApplyBonusRaw {
    pub bonus_type: i16,
    pub detail: i16,
    pub target_id: u16,
}
