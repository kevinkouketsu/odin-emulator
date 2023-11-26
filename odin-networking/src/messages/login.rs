use super::string::FixedSizeString;
use crate::{ReadableResource, ReadableResourceError};
use deku::prelude::*;

pub struct LoginMessage {
    pub username: String,
    pub password: String,
    pub tid: [u8; 52],
    pub cliver: u32,
}
impl ReadableResource for LoginMessageRaw {
    type Output = LoginMessage;

    fn read(self, data: &[u8]) -> Result<Self::Output, ReadableResourceError> {
        let (_, value) = LoginMessageRaw::from_bytes((data, 0))?;
        let username: String = value.username.try_into()?;
        let password: String = value.password.try_into()?;

        Ok(LoginMessage {
            username,
            password,
            tid: self.tid,
            cliver: self.cliver,
        })
    }
}

#[derive(Debug, DekuWrite, DekuRead)]
pub struct LoginMessageRaw {
    pub username: FixedSizeString<12>,
    pub password: FixedSizeString<16>,
    pub tid: [u8; 52],
    pub cliver: u32,
}
