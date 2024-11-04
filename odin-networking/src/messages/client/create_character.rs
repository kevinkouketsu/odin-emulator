use crate::messages::string::FixedSizeString;
use deku::prelude::*;

#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct CreateCharacterRaw {
    pub slot: u32,
    pub name: FixedSizeString<16>,
    pub class: i32,
}
