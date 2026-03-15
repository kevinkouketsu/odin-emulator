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
                Message::CreateCharacter(message) if *token => {
                    if let Ok(new_charlist) = message
                        .handle(
                            &session,
                            account_charlist.identifier,
                            context.account_repository.clone(),
                        )
                        .await
                    {
                        account_charlist.charlist = new_charlist;
                    }
                }
                Message::DeleteCharacter(message) if *token => {
                    if let Ok(new_charlist) = message
                        .handle(
                            account_charlist.identifier,
                            &session,
                            context.account_repository.clone(),
                        )
                        .await
                    {
                        account_charlist.charlist = new_charlist;
                    }
                }
                message => log::error!("Got a message in incorrect state: {:?}", message),
            },
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
