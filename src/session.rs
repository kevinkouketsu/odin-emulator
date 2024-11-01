use deku::prelude::*;
use odin_networking::{enc_session::EncDecError, WritableResource, WritableResourceError};
use thiserror::Error;

pub trait SessionTrait {
    fn send<R: WritableResource>(&self, message: R) -> Result<(), SessionError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessionError {
    #[error(transparent)]
    DekuError(#[from] DekuError),

    #[error(transparent)]
    EncryptionError(#[from] EncDecError),

    #[error(transparent)]
    WritableResourceError(#[from] WritableResourceError),
}
