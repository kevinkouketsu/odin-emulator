use crate::messages::string::FixedSizeString;
use deku::prelude::*;

#[derive(Debug, DekuWrite, DekuRead)]
pub struct EnterWorldRaw {
    pub slot: u32,
    pub force: u32,
    pub secret_code: FixedSizeString<16>,
}
