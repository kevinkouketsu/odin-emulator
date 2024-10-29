use crate::{
    messages::{string::FixedSizeString, ServerMessage},
    WritableResource, WritableResourceError,
};
use deku::prelude::*;

pub struct MessagePanel(String);
impl From<String> for MessagePanel {
    fn from(value: String) -> Self {
        MessagePanel(value)
    }
}
impl From<&str> for MessagePanel {
    fn from(value: &str) -> Self {
        MessagePanel(value.to_string())
    }
}
impl WritableResource for MessagePanel {
    const IDENTIFIER: ServerMessage = ServerMessage::MessagePanel;
    type Output = MessagePanelRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(MessagePanelRaw(self.0.try_into()?))
    }

    fn client_id(&self) -> Option<u16> {
        Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct MessagePanelRaw(FixedSizeString<128>);
