use crate::{
    session::{SendError, Session},
    GameServerSignals,
};
use deku::prelude::*;
use message_io::{
    network::{Endpoint, ResourceId},
    node::NodeHandler,
};
use odin_networking::{
    enc_session::{DecryptError, EncDecSession},
    framed_message::HandshakeState,
    WritableResource,
};

pub struct UserSession {
    handler: NodeHandler<GameServerSignals>,
    endpoint: Endpoint,
    encdec_session: EncDecSession,
    framed_message: HandshakeState,
}
impl UserSession {
    pub fn new(
        handler: NodeHandler<GameServerSignals>,
        endpoint: Endpoint,
        encdec_session: EncDecSession,
    ) -> Self {
        Self {
            handler,
            endpoint,
            encdec_session,
            framed_message: Default::default(),
        }
    }

    pub fn get_resource_id(&self) -> ResourceId {
        self.endpoint.resource_id()
    }

    pub fn feed_with_message(&mut self, data: &[u8]) {
        self.framed_message.update(data);
    }

    pub fn next_message(&mut self) -> Option<Vec<u8>> {
        self.framed_message.next_message()
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, DecryptError> {
        let mut output = vec![0; data.len()];
        output.copy_from_slice(data);
        self.encdec_session.decrypt(&mut output)?;

        Ok(output)
    }
}
impl Session for UserSession {
    fn send<R: WritableResource>(&self, message: R) -> Result<(), SendError> {
        let payload: Vec<u8> = message.write()?.to_bytes()?;
        let bytes = self.encdec_session.encrypt::<R>(&payload)?;

        self.handler.network().send(self.endpoint, &bytes);
        Ok(())
    }
}
