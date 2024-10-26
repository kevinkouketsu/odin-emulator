use deku::{ctx::Limit, prelude::*};
use std::ffi::CString;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, DekuWrite, DekuRead)]
pub struct FixedSizeString<const N: usize> {
    #[deku(
        reader = "FixedSizeString::<N>::read(deku::reader)",
        writer = "FixedSizeString::<N>::write(deku::writer, &self.str)"
    )]
    str: CString,
}
impl<const N: usize> FixedSizeString<N> {
    fn read<R: std::io::Read + std::io::Seek>(
        rest: &mut deku::reader::Reader<R>,
    ) -> Result<CString, DekuError> {
        let value = CString::from_reader_with_ctx(rest, ())?;

        let bytes_with_nul = value.as_bytes_with_nul();
        if bytes_with_nul.len() > N {
            return Err(DekuError::Parse(
                format!(
                    "String bigger than expected. Expected max of: {} Size: {}, {}",
                    N,
                    bytes_with_nul.len(),
                    value.to_str().unwrap()
                )
                .into(),
            ));
        };

        let remaining = N - bytes_with_nul.len();
        match remaining > 0 {
            true => {
                let _consuming_rest =
                    Vec::<u8>::from_reader_with_ctx(rest, (Limit::from(remaining), ()))?;
                Ok(value)
            }
            false => Ok(value),
        }
    }

    fn write<W: std::io::Write + std::io::Seek>(
        writer: &mut Writer<W>,
        field: &CString,
    ) -> Result<(), DekuError> {
        let bytes = field.as_bytes_with_nul();
        if bytes.len() > N {
            return Err(DekuError::Parse(
                format!(
                    "Value bigger than expected. Expected: {} Size: {}, {}",
                    field.as_bytes().len(),
                    N,
                    field.to_str().unwrap()
                )
                .into(),
            ));
        }

        writer.write_bytes(bytes)?;

        let remaining = N - bytes.len();
        if remaining > 0 {
            writer.write_bytes(&vec![0u8; remaining])?;
        }

        Ok(())
    }
}
impl<const N: usize> TryInto<String> for FixedSizeString<N> {
    type Error = std::ffi::IntoStringError;

    fn try_into(self) -> Result<String, Self::Error> {
        self.str.into_string()
    }
}
impl<const N: usize> TryFrom<String> for FixedSizeString<N> {
    type Error = FixedSizeStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let len = value.len();
        match len > N {
            true => Err(FixedSizeStringError(value, len)),
            false => Ok(FixedSizeString {
                str: CString::new(value).unwrap(),
            }),
        }
    }
}
#[derive(Debug, Error)]
#[error("The string size is bigger than the fixed size: {0} {0}")]
pub struct FixedSizeStringError(String, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enc_fixed_sized_string() {
        let string: FixedSizeString<10> = "hello".to_string().try_into().unwrap();
        let encoded_string: Vec<u8> = string.try_into().unwrap();

        assert_eq!(encoded_string, b"hello\0\0\0\0\0");
    }

    #[test]
    fn decode_fixed_size_string() {
        let string: FixedSizeString<10> = "hello".to_string().try_into().unwrap();
        let encoded_string: Vec<u8> = string.try_into().unwrap();
        let (_, decoded) = FixedSizeString::<10>::from_bytes((&encoded_string, 0)).unwrap();

        assert_eq!(decoded.str.as_bytes().len(), 5);
    }

    #[test]
    fn write_string_bigger_than_buffer() {
        let string: FixedSizeString<2> = "hello".to_string().try_into().unwrap();
        let encoded_string: Result<Vec<u8>, _> = string.try_into();

        assert!(encoded_string.is_err());
    }

    #[test]
    fn must_have_space_for_null_terminate() {
        let string: FixedSizeString<5> = "hello".to_string().try_into().unwrap();
        let encoded_string: Result<Vec<u8>, _> = string.try_into();

        assert!(encoded_string.is_err());
    }

    #[derive(Clone, Debug, PartialEq, Eq, DekuWrite, DekuRead)]
    struct TwoAlignedStrings {
        a: FixedSizeString<10>,
        b: u32,
        c: FixedSizeString<7>,
        d: u32,
    }

    #[test]
    fn must_consume_the_entire_length() {
        let value = TwoAlignedStrings {
            a: "wyd".to_string().try_into().unwrap(),
            b: 10,
            c: "rulez".to_string().try_into().unwrap(),
            d: 15,
        };
        let encoded_string: Vec<u8> = value.clone().try_into().unwrap();
        let (rest, decoded) = TwoAlignedStrings::from_bytes((&encoded_string, 0)).unwrap();

        assert_eq!(value, decoded);
        assert_eq!(rest.1, 0);
    }

    #[test]
    fn cstring_bigger_than_the_fixed_size() {
        let bytes = b"more than two caracteres\0";
        match FixedSizeString::<10>::from_bytes((bytes.as_slice(), 0)) {
            Err(DekuError::Parse(x)) => assert!(x.contains("String bigger than expected")),
            _ => panic!("Should fail"),
        }
    }

    #[test]
    fn try_from_string_must_check_size() {
        let str = "bigger than two bytes".to_string();
        assert!(FixedSizeString::<2>::try_from(str).is_err());
    }
}
