pub mod enc_session;
pub mod framed_message;
pub mod messages;

use deku::prelude::*;
use messages::{string::FixedSizeStringError, ServerMessage};
use std::ffi::IntoStringError;
use thiserror::Error;

pub trait WritableResource {
    const IDENTIFIER: ServerMessage;
    type Output: DekuWriter + deku::DekuContainerWrite;

    fn write(self) -> Result<Self::Output, WritableResourceError>;
    fn client_id(&self) -> Option<u16> {
        None
    }
}

#[derive(Debug, Error)]
pub enum WritableResourceError {
    #[error(transparent)]
    InvalidCString(#[from] IntoStringError),

    #[error(transparent)]
    FixedSizeStringError(#[from] FixedSizeStringError),

    #[error(transparent)]
    Deku(#[from] DekuError),
}
