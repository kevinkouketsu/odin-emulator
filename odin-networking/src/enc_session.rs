use crate::{messages::header::Header, WritableResource, WritableResourceError};
use bytes::Bytes;
use deku::prelude::*;
use rand::Rng;
use std::rc::Rc;
use thiserror::Error;

const KEYTABLE_LENGHT: usize = 512;
const HALF_KEYTABLE_LENGTH: usize = 255;

#[derive(Debug, Clone)]
pub struct EncDecSession {
    keytable: Rc<[u8; 512]>,
    id: u16,
}
impl EncDecSession {
    pub fn new(id: u16, keytable: Rc<[u8; KEYTABLE_LENGHT]>) -> Self {
        Self { keytable, id }
    }

    pub fn encrypt<R: WritableResource>(&self, data: R) -> Result<Bytes, EncDecError> {
        let mut rng = rand::thread_rng();
        let keyword_index = rng.gen_range::<u8, _>(0u8..HALF_KEYTABLE_LENGTH as u8);
        let client_id = data.client_id().unwrap_or(self.id);
        let data = data.write()?.to_bytes()?;
        let header = Header {
            size: (data.len() + std::mem::size_of::<Header>()) as u16,
            keyword: keyword_index,
            checksum: 0,
            typ: u16::try_from(R::IDENTIFIER).expect("Message identifier must be valid"),
            id: client_id,
            tick: 0,
        };

        let mut buffer: Vec<u8> = header.to_bytes()?;
        buffer.extend(data);

        log::debug!("Sending packet {:?} {:?}", R::IDENTIFIER, buffer);

        let mut checksum: [u8; 2] = [0; 2];
        let key_index = keyword_index as usize * 2;
        let mut pos = self.keytable[key_index] as i32;

        (4..buffer.len()).for_each(|i| {
            checksum[0] = checksum[0].wrapping_add(buffer[i]);
            let rst = pos % 256;
            let key = self.keytable[(rst * 2 + 1) as usize];

            buffer[i] = match i & 3 {
                0 => buffer[i].wrapping_add(key.wrapping_shl(1)),
                1 => buffer[i].wrapping_sub(key.wrapping_shr(3)),
                2 => buffer[i].wrapping_add(key.wrapping_shl(2)),
                3 => buffer[i].wrapping_sub(key.wrapping_shr(5)),
                _ => unreachable!(),
            };

            checksum[1] = checksum[1].wrapping_add(buffer[i]);
            pos += 1;
        });

        buffer[3] = checksum[1].wrapping_sub(checksum[0]);
        Ok(buffer.into())
    }

    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), EncDecError> {
        let (_, header) = Header::from_bytes((data, 0))?;
        assert_eq!(data.len(), header.size as usize);

        let keyword = header.keyword as u32;
        let mut pos = self.keytable[(keyword * 2) as usize] as i32;

        let mut checksum: [i32; 2] = [0; 2];
        (4..data.len()).for_each(|i| {
            let key = self.keytable[(pos % 256).wrapping_mul(2).wrapping_add(1) as usize];
            let encoded = data[i] as i8;

            checksum[0] += encoded as i32;
            let decoded = match i & 3 {
                0 => encoded.wrapping_sub(key.wrapping_shl(1) as i8),
                1 => encoded.wrapping_add(key.wrapping_shr(3) as i8),
                2 => encoded.wrapping_sub(key.wrapping_shl(2) as i8),
                3 => encoded.wrapping_add(key.wrapping_shr(5) as i8),
                _ => unreachable!(),
            };

            data[i] = decoded as u8;
            checksum[1] += decoded as i32;
            pos += 1;
        });

        match (checksum[1].wrapping_sub(checksum[0]) & 255) as u8 != header.checksum {
            true => Ok(()),
            false => Err(EncDecError::InvalidChecksum(
                (checksum[0] & 255) as u8,
                header.checksum,
            )),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncDecError {
    #[error("Invalid checksum {0} {1}")]
    InvalidChecksum(u8, u8),

    #[error(transparent)]
    DekuError(#[from] deku::DekuError),

    #[error(transparent)]
    WritableResourceError(#[from] WritableResourceError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{messages::ServerMessage, WritableResourceError};

    #[derive(Debug, Clone, PartialEq, Eq, DekuWrite, DekuRead)]
    struct PayloadTest {
        a: i32,
        b: i32,
    }
    impl WritableResource for PayloadTest {
        const IDENTIFIER: ServerMessage = ServerMessage::MessagePanel;
        type Output = PayloadTest;

        fn write(self) -> Result<Self::Output, WritableResourceError> {
            Ok(self)
        }
    }

    #[test]
    fn appends_the_header_at_beginning_of_message() {
        let mut rng = rand::thread_rng();

        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(0, Rc::new(array));
        let message = enc_session.encrypt(PayloadTest { a: 1, b: 2 }).unwrap();

        assert_ne!(message.len(), std::mem::size_of::<PayloadTest>());
    }

    #[test]
    fn sums_the_size_of_header_with_the_payload() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(0, Rc::new(array));
        let message = enc_session.encrypt(PayloadTest { a: 1, b: 2 }).unwrap();

        assert_eq!(
            u16::from_le_bytes([message[0], message[1]]) as usize,
            std::mem::size_of::<PayloadTest>() + std::mem::size_of::<Header>()
        );
    }

    #[test]
    fn encryption_roundtrip() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(255, Rc::new(array));
        let payload = PayloadTest { a: 1, b: 2 };
        let mut message = enc_session.encrypt(payload.clone()).unwrap().to_vec();
        assert_ne!(
            PayloadTest::from_bytes((&message[12..], 0)).unwrap().1,
            payload
        );
        enc_session.decrypt(&mut message).unwrap();

        let (_, decrypted) = PayloadTest::from_bytes((&message[12..], 0)).unwrap();
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn client_id() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(255, Rc::new(array));
        let mut message = enc_session
            .encrypt(PayloadTest { a: 1, b: 2 })
            .unwrap()
            .to_vec();

        enc_session.decrypt(&mut message).unwrap();
        assert_eq!(u16::from_le_bytes([message[6], message[7]]), 255);
    }

    #[test]
    fn message_identifier() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(255, Rc::new(array));
        let mut message = enc_session
            .encrypt(PayloadTest { a: 1, b: 2 })
            .unwrap()
            .to_vec();

        enc_session.decrypt(&mut message).unwrap();
        assert_eq!(
            u16::from_le_bytes([message[4], message[5]]),
            u16::try_from(PayloadTest::IDENTIFIER).unwrap()
        );
    }

    #[derive(Debug, DekuRead, DekuWrite)]
    pub struct PayloadWithClientId(u32);

    impl WritableResource for PayloadWithClientId {
        const IDENTIFIER: ServerMessage = ServerMessage::MessagePanel;
        type Output = PayloadWithClientId;

        fn write(self) -> Result<Self::Output, WritableResourceError> {
            Ok(self)
        }

        fn client_id(&self) -> Option<u16> {
            Some(1000)
        }
    }

    #[test]
    fn the_packet_can_decide_the_client_id() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(255, Rc::new(array));
        let mut message = enc_session
            .encrypt(PayloadWithClientId(1))
            .unwrap()
            .to_vec();

        enc_session.decrypt(&mut message).unwrap();
        assert_eq!(u16::from_le_bytes([message[6], message[7]]), 1000);
    }
}
