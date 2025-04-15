use std::ops::Deref;
use thiserror::Error;

const MIN_NICKNAME_LENGTH: usize = 4;
const MAX_NICKNAME_LENGTH: usize = 12;

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
        if value.len() < MIN_NICKNAME_LENGTH {
            return Err(InvalidNicknameError::MinimumCharacters);
        }

        if value.len() >= MAX_NICKNAME_LENGTH {
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
    #[error("Nickname must be at least 4 characters long")]
    MinimumCharacters,

    #[error("Nickname cannot be longer than 11 characters")]
    MaximumCharacters,

    #[error("Nickname can only contain letters and numbers")]
    InvalidCharacter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_alphanumeric_nicknames() {
        assert!(Nickname::try_from("admin").is_ok());
        assert!(Nickname::try_from("admin123").is_ok());
        assert!(Nickname::try_from("Admin123").is_ok());
    }

    #[test]
    fn rejects_nicknames_with_special_characters() {
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
    fn enforces_nickname_length_limits() {
        // Test minimum length
        let too_short = "a".repeat(MIN_NICKNAME_LENGTH - 1);
        assert!(matches!(
            Nickname::try_from(too_short),
            Err(InvalidNicknameError::MinimumCharacters)
        ));

        // Test maximum length
        let too_long = "a".repeat(MAX_NICKNAME_LENGTH);
        assert!(matches!(
            Nickname::try_from(too_long),
            Err(InvalidNicknameError::MaximumCharacters)
        ));

        // Test valid length
        let valid_length = "a".repeat(MAX_NICKNAME_LENGTH - 1);
        assert!(Nickname::try_from(valid_length).is_ok());
    }
}
