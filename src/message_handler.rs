use crate::GameServerContext;
use odin_networking::messages::login::LoginMessage;
use thiserror::Error;

pub trait MessageHandler {
    type Context;
    type Error: std::error::Error;

    fn handle(&mut self, ctx: Self::Context, message: Self) -> Result<(), Self::Error>;
}

impl MessageHandler for LoginMessage {
    type Context = GameServerContext;
    type Error = LoginError;

    fn handle(&mut self, _ctx: Self::Context, _message: Self) -> Result<(), Self::Error> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum LoginError {}
