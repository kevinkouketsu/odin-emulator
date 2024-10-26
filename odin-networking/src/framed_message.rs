pub const HANDSHAKE_VALUE: u32 = 0x1F11F311;

#[derive(Default)]
pub struct FramedMessage {
    cache: Vec<u8>,
}
impl FramedMessage {
    pub fn update(&mut self, data: &[u8]) {
        self.cache.extend_from_slice(data);
    }

    pub fn next_message(&mut self) -> Option<Vec<u8>> {
        match Self::decode_size(&self.cache) {
            Some(size) if self.cache.len() >= size => {
                let result_data = self.cache[..size].to_vec();
                self.cache.drain(..size);

                Some(result_data)
            }
            _ => None,
        }
    }

    fn decode_size(data: &[u8]) -> Option<usize> {
        match data.len() > std::mem::size_of::<u16>() {
            true => Some(u16::from_le_bytes([data[0], data[1]]) as usize),
            false => None,
        }
    }
}

#[derive(Default)]
pub enum HandshakeState {
    #[default]
    Handshaking,
    Done(FramedMessage),
}
impl HandshakeState {
    pub fn update(&mut self, data: &[u8]) {
        match self {
            HandshakeState::Handshaking => {
                match u32::from_le_bytes(data.try_into().unwrap()) == HANDSHAKE_VALUE {
                    true => *self = HandshakeState::Done(Default::default()),
                    false => {}
                }
            }
            HandshakeState::Done(framed_message) => framed_message.update(data),
        }
    }

    pub fn next_message(&mut self) -> Option<Vec<u8>> {
        match self {
            HandshakeState::Handshaking => None,
            HandshakeState::Done(framed_message) => framed_message.next_message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deku::prelude::*;

    #[test]
    fn two_first_bytes_are_the_length() {
        let mut framed_message = FramedMessage::default();

        let message: [u8; 4] = [4, 0, 0, 1];
        framed_message.update(&message);
        let message = framed_message.next_message();

        assert!(message.is_some());
    }

    #[test]
    fn returns_none_if_there_is_no_enough_data() {
        let mut framed_message = FramedMessage::default();

        let message: [u8; 4] = [6, 0, 0, 1];
        framed_message.update(&message);
        let message = framed_message.next_message();

        assert!(message.is_none());
    }

    #[test]
    fn two_messages_in_a_single_update() {
        let mut framed_message = FramedMessage::default();
        framed_message.update(&[6]);

        assert!(framed_message.next_message().is_none());
        framed_message.update(&[0, 0, 0, 0, 0, 4]);

        let message = framed_message.next_message().unwrap();
        assert_eq!(message, vec![6, 0, 0, 0, 0, 0]);

        framed_message.update(&[0, 0]);

        assert!(framed_message.next_message().is_none());
        framed_message.update(&[0]);

        assert!(framed_message.next_message().is_some());
    }

    #[test]
    fn receiving_valid_handshake_updates_current_state() {
        let mut state = HandshakeState::default();

        state.update(&u32::to_le_bytes(HANDSHAKE_VALUE));
        match state {
            HandshakeState::Handshaking => panic!("Invalid state, expected Done"),
            HandshakeState::Done(_) => {}
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, DekuWrite, DekuRead)]
    struct PayloadTest(#[deku(bytes = 4)] u32);

    #[test]
    fn ignore_packet_until_it_receives_the_handshake_packet() {
        let mut state = HandshakeState::default();

        let payload_test = PayloadTest(1);
        state.update(&payload_test.to_bytes().unwrap());
        assert!(state.next_message().is_none());
        state.update(&u32::to_le_bytes(HANDSHAKE_VALUE));

        // Prefix the message with the size
        state.update(&u16::to_le_bytes(
            (std::mem::size_of::<PayloadTest>() + std::mem::size_of::<u16>()) as u16,
        ));

        // Put the actually message
        state.update(&payload_test.to_bytes().unwrap());

        let message = state.next_message().unwrap();
        assert_eq!(
            PayloadTest::from_bytes((&message[2..], 0)).unwrap().1,
            payload_test
        );
    }
}
