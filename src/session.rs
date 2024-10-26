use deku::prelude::*;
use odin_networking::{enc_session::DecryptError, WritableResource, WritableResourceError};
use thiserror::Error;

pub trait Session {
    fn send<R: WritableResource>(&self, message: R) -> Result<(), SendError>;
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error(transparent)]
    DekuError(#[from] DekuError),

    #[error(transparent)]
    EncryptionError(#[from] DecryptError),

    #[error(transparent)]
    WritableResourceError(#[from] WritableResourceError),
}
