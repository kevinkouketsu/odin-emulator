use deku::{ctx::Limit, prelude::*};
use std::ffi::{CString, NulError};
use thiserror::Error;

#[derive(Clone, Default, Debug, PartialEq, Eq, DekuWrite, DekuRead)]
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
            true => Err(FixedSizeStringError::InvalidSize(value, len)),
            false => Ok(FixedSizeString {
                str: CString::new(value).map_err(FixedSizeStringError::NulError)?,
            }),
        }
    }
}
impl<const N: usize> TryFrom<&str> for FixedSizeString<N> {
    type Error = FixedSizeStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let len = value.len();
        match len > N {
            true => Err(FixedSizeStringError::InvalidSize(value.to_string(), len)),
            false => Ok(FixedSizeString {
                str: CString::new(value).unwrap(),
            }),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FixedSizeStringError {
    #[error("The string size is bigger than the fixed size: {0} (size: {1})")]
    InvalidSize(String, usize),

    #[error(transparent)]
    NulError(#[from] NulError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_string_with_padding() {
        let string: FixedSizeString<10> = "hello".to_string().try_into().unwrap();
        let encoded_string: Vec<u8> = string.try_into().unwrap();

        assert_eq!(encoded_string, b"hello\0\0\0\0\0");
    }

    #[test]
    fn decodes_padded_string() {
        let string: FixedSizeString<10> = "hello".to_string().try_into().unwrap();
        let encoded_string: Vec<u8> = string.try_into().unwrap();
        let (_, decoded) = FixedSizeString::<10>::from_bytes((&encoded_string, 0)).unwrap();

        assert_eq!(decoded.str.as_bytes().len(), 5);
        assert_eq!(decoded.str.to_str().unwrap(), "hello");
    }

    #[test]
    fn requires_space_for_null_terminator() {
        let result = FixedSizeString::<5>::try_from("hello".to_string());
        assert!(matches!(
            result,
            Err(FixedSizeStringError::InvalidSize(_, 5))
        ));
    }

    #[test]
    fn handles_nul_bytes_in_string() {
        let result = FixedSizeString::<10>::try_from("hello\0world".to_string());
        assert!(matches!(result, Err(FixedSizeStringError::NulError(_))));
    }

    #[derive(Clone, Debug, PartialEq, Eq, DekuWrite, DekuRead)]
    struct TwoAlignedStrings {
        a: FixedSizeString<10>,
        b: u32,
        c: FixedSizeString<7>,
        d: u32,
    }

    #[test]
    fn maintains_alignment_in_complex_struct() {
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
    fn rejects_strings_exceeding_fixed_size() {
        let bytes = b"more than two caracteres\0";
        match FixedSizeString::<10>::from_bytes((bytes.as_slice(), 0)) {
            Err(DekuError::Parse(x)) => assert!(x.contains("String bigger than expected")),
            _ => panic!("Should fail"),
        }
    }

    #[test]
    fn validates_size_during_conversion() {
        let str = "bigger than two bytes".to_string();
        assert!(FixedSizeString::<2>::try_from(str).is_err());
    }
}
