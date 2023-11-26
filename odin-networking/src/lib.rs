pub mod enc_session;
pub mod framed_message;
pub mod messages;

use deku::DekuError;
use messages::MessageIdentifier;
use std::{ffi::IntoStringError, hash, io, net::ToSocketAddrs};
use thiserror::Error;

pub trait Listener<S: Send + 'static> {
    type Controller: Server<S>;

    fn listen<T: ToSocketAddrs>(&self, addr: T) -> io::Result<Self::Controller>;
}

pub trait Server<S: Send + 'static>: Send {
    type Endpoint: hash::Hash;

    fn send(&self, endpoint: Self::Endpoint, data: &[u8]);
    fn close(&self, endpoint: Self::Endpoint) -> io::Result<()>;
}

pub trait WritableResource {
    const IDENTIFIER: MessageIdentifier;
    type Output: deku::DekuWrite + deku::DekuContainerWrite;

    fn write(self) -> Self::Output;
}
pub trait ReadableResource {
    type Output;

    fn read(self, data: &[u8]) -> Result<Self::Output, ReadableResourceError>;
}

#[derive(Debug, Error)]
pub enum ReadableResourceError {
    #[error(transparent)]
    InvalidCString(#[from] IntoStringError),

    #[error(transparent)]
    Deku(#[from] DekuError),
}
// pub trait OutgoingClient {
//     fn send(&self, endpoint: Endpoint, resource: &[u8]);
// }
// impl<S: Send + 'static> OutgoingClient for NodeHandler<S> {
//     fn send(&self, endpoint: Endpoint, data: &[u8]) {
//         self.network().send(endpoint, data);
//     }
// }
