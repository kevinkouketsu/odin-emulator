use crate::messages::string::FixedSizeString;
use deku::prelude::*;

#[derive(Debug, DekuWrite, DekuRead)]
pub struct NumericTokenRaw {
    pub token: FixedSizeString<16>,
    pub state: u32,
}
