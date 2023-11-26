use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::Limit,
    prelude::*,
};
use std::ffi::CString;

#[derive(Clone, Debug, PartialEq, Eq, DekuWrite, DekuRead)]
pub struct FixedSizeString<const N: usize> {
    #[deku(
        reader = "FixedSizeString::<N>::read(deku::rest)",
        writer = "FixedSizeString::<N>::write(deku::output, &self.str)"
    )]
    str: CString,
}
impl<const N: usize> FixedSizeString<N> {
    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, CString), DekuError> {
        let (rest, value) = CString::read(rest, ())?;

        let remaining = N - value.as_bytes_with_nul().len();
        match remaining > 0 {
            true => {
                let (rest, _) = Vec::<u8>::read(rest, (Limit::from(remaining), ()))?;
                Ok((rest, value))
            }
            false => Ok((rest, value)),
        }
    }

    fn write(output: &mut BitVec<u8, Msb0>, field: &CString) -> Result<(), DekuError> {
        let bytes = field.as_bytes_with_nul();
        if bytes.len() > N {
            return Err(DekuError::Parse(format!(
                "Value bigger than expected. Expected: {} Size: {}, {}",
                field.as_bytes().len(),
                N,
                field.to_str().unwrap()
            )));
        }

        bytes.write(output, ())?;

        let remaining = N - bytes.len();
        if remaining > 0 {
            vec![0u8; remaining].write(output, ())?;
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
impl<const N: usize> From<String> for FixedSizeString<N> {
    fn from(value: String) -> Self {
        FixedSizeString {
            str: CString::new(value).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enc_fixed_sized_string() {
        let string: FixedSizeString<10> = "hello".to_string().into();
        let encoded_string: Vec<u8> = string.try_into().unwrap();

        assert_eq!(encoded_string, b"hello\0\0\0\0\0");
    }

    #[test]
    fn decode_fixed_size_string() {
        let string: FixedSizeString<10> = "hello".to_string().into();
        let encoded_string: Vec<u8> = string.try_into().unwrap();
        let (_, decoded) = FixedSizeString::<10>::from_bytes((&encoded_string, 0)).unwrap();

        assert_eq!(decoded.str.as_bytes().len(), 5);
    }

    #[test]
    fn write_string_bigger_than_buffer() {
        let string: FixedSizeString<2> = "hello".to_string().into();
        let encoded_string: Result<Vec<u8>, _> = string.try_into();

        assert!(encoded_string.is_err());
    }

    #[test]
    fn must_have_space_for_null_terminate() {
        let string: FixedSizeString<5> = "hello".to_string().into();
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
            a: "wyd".to_string().into(),
            b: 10,
            c: "rulez".to_string().into(),
            d: 15,
        };
        let encoded_string: Vec<u8> = value.clone().try_into().unwrap();
        let (rest, decoded) = TwoAlignedStrings::from_bytes((&encoded_string, 0)).unwrap();

        assert_eq!(value, decoded);
        assert_eq!(rest.1, 0);
    }
}
