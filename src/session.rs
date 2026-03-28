use crate::map::EntityId;
use deku::prelude::*;
use odin_networking::{WritableResource, WritableResourceError, enc_session::EncDecError};
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

    #[error("Client disconnected")]
    Disconnected,
}

pub trait PacketSender {
    fn send_to<W: WritableResource>(
        &self,
        target: EntityId,
        message: W,
    ) -> Result<(), SessionError>;
}
