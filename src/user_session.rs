use crate::{
    game_server_context::GameServerContext,
    message::Message,
    session::{SessionError, SessionTrait},
    GameServerSignals,
};
use message_io::{
    network::{Endpoint, ResourceId},
    node::NodeHandler,
};
use odin_models::account_charlist::AccountCharlist;
use odin_networking::{
    enc_session::{EncDecError, EncDecSession},
    framed_message::HandshakeState,
    WritableResource,
};
use odin_repositories::account_repository::AccountRepository;

#[derive(Default)]
pub enum Session {
    #[default]
    LoggingIn,
    Charlist {
        account_charlist: AccountCharlist,
        token: bool,
    },
}

pub struct UserSession {
    handler: NodeHandler<GameServerSignals>,
    endpoint: Endpoint,
    encdec_session: EncDecSession,
    framed_message: HandshakeState,
    session: Session,
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
            session: Session::default(),
        }
    }

    pub async fn handle<A: AccountRepository>(
        &mut self,
        context: &GameServerContext<A>,
        message: Message,
    ) {
        let session = self.get_sender();
        match &mut self.session {
            Session::LoggingIn => {
                if let Message::Login(login_message) = message {
                    if let Ok(account_charlist) = login_message
                        .handle(&session, context, context.account_repository.clone())
                        .await
                    {
                        self.session = Session::Charlist {
                            account_charlist,
                            token: false,
                        }
                    }
                }
            }
            Session::Charlist {
                account_charlist,
                token,
            } => match message {
                Message::Token(token_message) => {
                    let r = token_message
                        .handle(
                            &session,
                            account_charlist.identifier,
                            *token,
                            context.account_repository.clone(),
                        )
                        .await;

                    if r.is_ok() {
                        *token = true;
                    }
                }
                message => log::error!("Got a message in incorrect state: {:?}", message),
            },
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

    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), EncDecError> {
        self.encdec_session.decrypt(data)?;

        Ok(())
    }

    fn get_sender(&self) -> SenderSession {
        SenderSession {
            encdec_session: self.encdec_session.clone(),
            endpoint: self.endpoint,
            handler: self.handler.clone(),
        }
    }
}

pub struct SenderSession {
    handler: NodeHandler<GameServerSignals>,
    encdec_session: EncDecSession,
    endpoint: Endpoint,
}
impl SessionTrait for SenderSession {
    fn send<R: WritableResource>(&self, message: R) -> Result<(), SessionError> {
        let bytes = self.encdec_session.encrypt::<R>(message)?;

        self.handler.network().send(self.endpoint, &bytes);
        Ok(())
    }
}
