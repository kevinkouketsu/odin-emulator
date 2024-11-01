pub mod charlist;
pub mod message_panel;
pub mod numeric_token;

use super::ServerMessage;
use crate::WritableResource;
use deku::prelude::*;
use std::marker::PhantomData;

#[derive(Default, DekuRead, DekuWrite)]
pub struct ZeroedRaw;

#[derive(Default, DekuWrite, DekuRead)]
pub struct MessageSignal<T: WritableResource> {
    #[deku(
        reader = "MessageSignal::read(deku::reader)",
        writer = "MessageSignal::write(deku::writer, &self._phantom)"
    )]
    _phantom: PhantomData<T>,
}
impl<T> MessageSignal<T>
where
    T: WritableResource,
{
    fn read<R: std::io::Read + std::io::Seek>(
        _rest: &mut deku::reader::Reader<R>,
    ) -> Result<PhantomData<T>, DekuError> {
        Ok(Default::default())
    }

    fn write<W: std::io::Write + std::io::Seek>(
        _writer: &mut Writer<W>,
        _field: &PhantomData<T>,
    ) -> Result<(), DekuError> {
        Ok(())
    }
}
impl<T> WritableResource for MessageSignal<T>
where
    T: WritableResource,
{
    const IDENTIFIER: ServerMessage = T::IDENTIFIER;
    type Output = ZeroedRaw;

    fn write(self) -> Result<Self::Output, crate::WritableResourceError> {
        Ok(ZeroedRaw)
    }
}
