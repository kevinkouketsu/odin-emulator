use crate::{messages::header::Header, WritableResource};
use bytes::Bytes;
use deku::prelude::*;
use rand::Rng;
use std::rc::Rc;
use thiserror::Error;

const KEYTABLE_LENGHT: usize = 512;
const HALF_KEYTABLE_LENGTH: usize = 255;

#[derive(Debug)]
pub struct EncDecSession {
    keytable: Rc<[u8; 512]>,
    id: u16,
}
impl EncDecSession {
    pub fn new(id: u16, keytable: Rc<[u8; KEYTABLE_LENGHT]>) -> Self {
        Self { keytable, id }
    }

    pub fn encrypt<R: WritableResource>(&self, data: &[u8]) -> Result<Bytes, DecryptError> {
        let mut rng = rand::thread_rng();
        let keyword_index = rng.gen_range::<u8, _>(0u8..HALF_KEYTABLE_LENGTH as u8);
        let header = Header {
            size: (std::mem::size_of::<R::Output>() + std::mem::size_of::<Header>()) as u16,
            keyword: keyword_index,
            checksum: 0,
            typ: R::IDENTIFIER as u16,
            id: self.id,
            tick: 0,
        };

        let mut header: Vec<u8> = header.try_into()?;
        header.extend_from_slice(data);

        let keyword = self.keytable[keyword_index.wrapping_mul(2) as usize];
        let mut data = header;
        let mut checksum: [u8; 2] = [0; 2];
        let mut key_increment = keyword;
        (4..data.len()).for_each(|i| {
            let key = self.keytable[(key_increment.wrapping_mul(2).wrapping_add(1)) as usize];

            let old_data = data[i];
            let result = match i & 3 {
                0 => old_data.wrapping_add(key.wrapping_shr(1)),
                1 => old_data.wrapping_sub(key.wrapping_shl(3)),
                2 => old_data.wrapping_add(key.wrapping_shr(2)),
                3 => old_data.wrapping_sub(key.wrapping_shl(5)),
                _ => unreachable!(),
            } as u8;

            data[i] = result;

            checksum[0] = checksum[0].wrapping_add(old_data);
            checksum[1] = checksum[1].wrapping_add(data[i]);
            key_increment = key_increment.wrapping_add(1);
        });

        data[3] = checksum[1].wrapping_sub(checksum[0]);

        Ok(data.into())
    }

    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), DecryptError> {
        let (_, header) = Header::from_bytes((data, 0))?;
        assert_eq!(data.len(), header.size as usize);

        let keyword = header.keyword;
        let mut pos = self.keytable[keyword.wrapping_mul(2) as usize];

        let mut checksum: [u8; 2] = [0; 2];
        (4..data.len()).for_each(|i| {
            let key = self.keytable[(pos.wrapping_mul(2).wrapping_add(1)) as usize];
            checksum[0] = checksum[0].wrapping_add(data[i]);

            let encoded = data[i];
            let decoded = match i & 3 {
                0 => encoded.wrapping_sub(key.wrapping_shr(1)),
                1 => encoded.wrapping_add(key.wrapping_shl(3)),
                2 => encoded.wrapping_sub(key.wrapping_shr(2)),
                3 => encoded.wrapping_add(key.wrapping_shl(5)),
                _ => unreachable!(),
            };

            data[i] = decoded;
            checksum[1] = checksum[1].wrapping_add(decoded);

            pos = pos.wrapping_add(1);
        });

        let checksum = checksum[0].wrapping_sub(checksum[1]);
        match checksum == header.checksum {
            true => Ok(()),
            false => Err(DecryptError::InvalidChecksum(checksum, header.checksum)),
        }
    }
}

#[derive(Debug, Error)]
pub enum DecryptError {
    #[error("Invalid checksum {0} {1}")]
    InvalidChecksum(u8, u8),

    #[error(transparent)]
    DekuError(#[from] deku::DekuError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{messages::MessageIdentifier, ReadableResource, ReadableResourceError};

    #[derive(Debug, Clone, PartialEq, Eq, DekuWrite, DekuRead)]
    struct PayloadTest {
        a: i32,
        b: i32,
    }
    impl WritableResource for PayloadTest {
        const IDENTIFIER: MessageIdentifier = MessageIdentifier::Login;
        type Output = PayloadTest;

        fn write(self) -> Self::Output {
            self
        }
    }
    impl ReadableResource for PayloadTest {
        type Output = PayloadTest;

        fn read(self, data: &[u8]) -> Result<Self::Output, ReadableResourceError> {
            Ok(PayloadTest::from_bytes((data, 0))?.1)
        }
    }

    #[test]
    fn appends_the_header_at_beginning_of_message() {
        let mut rng = rand::thread_rng();

        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(0, Rc::new(array));
        let payload: Vec<u8> = PayloadTest { a: 1, b: 2 }.try_into().unwrap();
        let message = enc_session.encrypt::<PayloadTest>(&payload).unwrap();

        assert_ne!(message.len(), std::mem::size_of::<PayloadTest>());
    }

    #[test]
    fn sums_the_size_of_header_with_the_payload() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(0, Rc::new(array));
        let payload: Vec<u8> = PayloadTest { a: 1, b: 2 }.try_into().unwrap();
        let message = enc_session.encrypt::<PayloadTest>(&payload).unwrap();

        assert_eq!(
            u16::from_le_bytes([message[0], message[1]]) as usize,
            std::mem::size_of::<PayloadTest>() + std::mem::size_of::<Header>()
        );
    }

    #[test]
    fn encrypt_decrypt() {
        let mut rng = rand::thread_rng();
        let mut array = [0u8; 512];
        rng.fill(&mut array);

        let enc_session = EncDecSession::new(255, Rc::new(array));
        let payload = PayloadTest { a: 1, b: 2 };
        let data: Vec<u8> = payload.clone().try_into().unwrap();
        let mut message = enc_session.encrypt::<PayloadTest>(&data).unwrap().to_vec();

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
        let payload: Vec<u8> = PayloadTest { a: 1, b: 2 }.try_into().unwrap();
        let mut message = enc_session
            .encrypt::<PayloadTest>(&payload)
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
        let payload: Vec<u8> = PayloadTest { a: 1, b: 2 }.try_into().unwrap();
        let mut message = enc_session
            .encrypt::<PayloadTest>(&payload)
            .unwrap()
            .to_vec();

        enc_session.decrypt(&mut message).unwrap();
        assert_eq!(
            u16::from_le_bytes([message[4], message[5]]),
            PayloadTest::IDENTIFIER as u16
        );
    }
}
