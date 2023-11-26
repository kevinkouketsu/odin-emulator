pub mod header;
pub mod login;
pub mod string;

use std::io;

trait MessageHandler {
    type Context;

    fn handle(&mut self, context: Self::Context) -> io::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageIdentifier {
    Login = 0x20D,
}
