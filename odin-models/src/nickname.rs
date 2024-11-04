use std::ops::Deref;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nickname(String);
impl TryFrom<&str> for Nickname {
    type Error = InvalidNicknameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Nickname::try_from(value.to_string())
    }
}
impl TryFrom<String> for Nickname {
    type Error = InvalidNicknameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() < 4 {
            return Err(InvalidNicknameError::MinimumCharacters);
        }

        // We need to have the null terminated character in the end
        if value.len() >= 12 {
            return Err(InvalidNicknameError::MaximumCharacters);
        }

        value
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
            .then_some(Nickname(value))
            .ok_or(InvalidNicknameError::InvalidCharacter)
    }
}
impl Deref for Nickname {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum InvalidNicknameError {
    #[error("Nickname does not have a minimum of four characters")]
    MinimumCharacters,

    #[error("Nickname has more than 11 characters")]
    MaximumCharacters,

    #[error("Nickname is invalid")]
    InvalidCharacter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nickname_can_only_have_alphanumeric() {
        assert!(Nickname::try_from("admin").is_ok());
        assert!(Nickname::try_from("admin123").is_ok());
        assert!(Nickname::try_from("Admin123").is_ok());
        assert!(matches!(
            Nickname::try_from("Admin@123"),
            Err(InvalidNicknameError::InvalidCharacter),
        ));
        assert!(matches!(
            Nickname::try_from("!123a"),
            Err(InvalidNicknameError::InvalidCharacter),
        ));
        assert!(matches!(
            Nickname::try_from("admin_"),
            Err(InvalidNicknameError::InvalidCharacter),
        ));
        assert!(matches!(
            Nickname::try_from("ad min"),
            Err(InvalidNicknameError::InvalidCharacter),
        ));
    }

    #[test]
    fn nickname_within_a_range() {
        assert!(matches!(
            Nickname::try_from("a"),
            Err(InvalidNicknameError::MinimumCharacters)
        ));
        assert!(matches!(
            Nickname::try_from("123456789123"),
            Err(InvalidNicknameError::MaximumCharacters)
        ));
        assert!(Nickname::try_from("12345678912").is_ok());
    }
}
