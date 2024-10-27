use crate::messages::string::FixedSizeString;
use deku::prelude::*;

#[derive(Debug, DekuWrite, DekuRead)]
pub struct LoginMessageRaw {
    pub password: FixedSizeString<16>,
    pub username: FixedSizeString<16>,
    pub tid: [u8; 52],
    pub cliver: u32,
    pub force: u32,
    pub mac: [u8; 16],
}
