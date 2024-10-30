use crate::{
    configuration::Configuration,
    handlers::authentication::{AuthenticationError, LoginMessage},
    user_session::UserSession,
};
use deku::prelude::*;
use odin_networking::{
    messages::{client::login::LoginMessageRaw, header::Header, ClientMessage},
    WritableResourceError,
};
use odin_repositories::account_repository::AccountRepository;
use thiserror::Error;

#[derive(Debug)]
pub enum Message {
    Login(LoginMessage),
    Token,
}
impl Message {
    pub async fn handle<A: AccountRepository, C: Configuration>(
        &self,
        user_session: &UserSession,
        configuration: &C,
        account_repository: A,
    ) -> Result<(), MessageError> {
        match self {
            Message::Login(login_message) => {
                login_message
                    .handle(user_session, configuration, account_repository)
                    .await
            }
            Message::Token => todo!(),
        };

        Ok(())
    }
}
impl TryFrom<((&[u8], usize), Header)> for Message {
    type Error = MessageError;

    fn try_from((rest, header): ((&[u8], usize), Header)) -> Result<Self, Self::Error> {
        let message_type = ClientMessage::try_from(header.typ)
            .map_err(|_| MessageError::NotRecognized(header.clone()))?;

        Ok(match message_type {
            ClientMessage::Login => {
                Message::Login(LoginMessageRaw::from_bytes(rest)?.1.try_into()?)
            }
            ClientMessage::Token => return Err(MessageError::NotImplemented(header)),
        })
    }
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("The packet is not implemented yet: {0:?}")]
    NotImplemented(Header),

    #[error("Invalid packet, not recognized: {0:?}")]
    NotRecognized(Header),

    #[error("Invalid packet structure")]
    InvalidStructure(#[from] DekuError),

    #[error("Invalid conversion for rust type")]
    InvalidToRust(#[from] WritableResourceError),

    #[error(transparent)]
    AuthenticationError(#[from] AuthenticationError),
}
