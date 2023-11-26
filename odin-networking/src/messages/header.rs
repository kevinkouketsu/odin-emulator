use deku::prelude::*;

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct Header {
    pub size: u16,
    pub keyword: u8,
    pub checksum: u8,
    pub typ: u16,
    pub id: u16,
    pub tick: u32,
}
