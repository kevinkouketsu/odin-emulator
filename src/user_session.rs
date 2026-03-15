use bytes::Bytes;
use crate::{
    game_server_context::GameServerContext,
    message::Message,
    session::{SessionError, SessionTrait},
};
use odin_models::account_charlist::AccountCharlist;
use odin_networking::{
    enc_session::{EncDecError, EncDecSession},
    WritableResource,
};
use odin_repositories::account_repository::AccountRepository;
use tokio::sync::mpsc;

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
    writer: mpsc::UnboundedSender<Bytes>,
    encdec_session: EncDecSession,
    session: Session,
}
impl UserSession {
    pub fn new(
        writer: mpsc::UnboundedSender<Bytes>,
        encdec_session: EncDecSession,
    ) -> Self {
        Self {
            writer,
            encdec_session,
            session: Session::default(),
        }
    }

    pub async fn handle<A: AccountRepository>(
        &mut self,
        context: &GameServerContext<A>,
        message: Message,
    ) {
        let sender = self.get_sender();
        let repo = context.account_repository.clone();

        match &mut self.session {
            Session::LoggingIn => {
                let Message::Login(msg) = message else {
                    log::error!("Got a message in incorrect state: {:?}", message);
                    return;
                };

                match msg.handle(&sender, context, repo).await {
                    Ok(account_charlist) => {
                        self.session = Session::Charlist {
                            account_charlist,
                            token: false,
                        };
                    }
                    Err(e) => log::warn!("Login failed: {e:?}"),
                }
            }
            Session::Charlist {
                account_charlist,
                token,
            } => {
                let account_id = account_charlist.identifier;

                match message {
                    Message::Token(msg) => {
                        match msg.handle(&sender, account_id, *token, repo).await {
                            Ok(()) => *token = true,
                            Err(e) => log::warn!("Token failed: {e:?}"),
                        }
                    }
                    Message::CreateCharacter(msg) if *token => {
                        match msg.handle(&sender, account_id, repo).await {
                            Ok(new_charlist) => account_charlist.charlist = new_charlist,
                            Err(e) => log::warn!("CreateCharacter failed: {e:?}"),
                        }
                    }
                    Message::DeleteCharacter(msg) if *token => {
                        match msg.handle(&sender, account_id, repo).await {
                            Ok(new_charlist) => account_charlist.charlist = new_charlist,
                            Err(e) => log::warn!("DeleteCharacter failed: {e:?}"),
                        }
                    }
                    message => log::error!("Got a message in incorrect state: {:?}", message),
                }
            }
        }
    }

    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), EncDecError> {
        self.encdec_session.decrypt(data)?;
        Ok(())
    }

    fn get_sender(&self) -> SenderSession {
        SenderSession {
            encdec_session: self.encdec_session.clone(),
            writer: self.writer.clone(),
        }
    }
}

pub struct SenderSession {
    encdec_session: EncDecSession,
    writer: mpsc::UnboundedSender<Bytes>,
}
impl SessionTrait for SenderSession {
    fn send<R: WritableResource>(&self, message: R) -> Result<(), SessionError> {
        let bytes = self.encdec_session.encrypt::<R>(message)?;
        self.writer.send(bytes).map_err(|_| SessionError::Disconnected)?;
        Ok(())
    }
}
