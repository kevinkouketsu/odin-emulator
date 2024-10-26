pub mod enc_session;
pub mod framed_message;
pub mod messages;

use deku::prelude::*;
use messages::{string::FixedSizeStringError, MessageIdentifier};
use std::ffi::IntoStringError;
use thiserror::Error;

pub trait WritableResource {
    const IDENTIFIER: MessageIdentifier;
    type Output: DekuWriter + deku::DekuContainerWrite;

    fn write(self) -> Result<Self::Output, WritableResourceError>;
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
