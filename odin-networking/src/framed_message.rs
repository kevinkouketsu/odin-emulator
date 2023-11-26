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

#[cfg(test)]
mod tests {
    use super::*;

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
}
