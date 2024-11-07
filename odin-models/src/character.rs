use crate::{item::Item, position::Position, status::Score, EquipmentSlot};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct Character {
    pub identifier: Uuid,
    pub name: String,
    // TODO maybe temporary
    pub slot: i32,
    pub score: Score,
    pub evolution: Evolution,
    // TODO i guess this is a bitfield so we could use a bitflag type here
    pub merchant: i16,
    pub guild: Option<i16>,
    pub guild_level: Option<GuildLevel>,
    pub class: Class,
    // TODO i guess this is a bitfield so we could use a bitflag type here
    pub affect_info: i16,
    // TODO i guess this is a bitfield so we could use a bitflag type here
    pub quest_info: i16,
    // TODO change to Wallet type
    pub coin: i32,
    pub experience: i64,
    pub last_pos: Position,
    pub inventory: Vec<(usize, Item)>,
    pub equipments: Vec<(EquipmentSlot, Item)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuildLevel {
    Participant,
    FirstCommander,
    SecondCommander,
    ThirdCommander,
    Leader,
}
impl GuildLevel {
    pub fn new<I: Into<i32>>(level: I) -> Option<Self> {
        let level = level.into();
        match level {
            0 => None,
            1 => Some(GuildLevel::Participant),
            3 => Some(GuildLevel::FirstCommander),
            4 => Some(GuildLevel::SecondCommander),
            5 => Some(GuildLevel::ThirdCommander),
            9 => Some(GuildLevel::Leader),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
#[error("Invalid guild level: {0}")]
pub struct InvalidGuildLevelError(i32);

#[derive(Debug, Copy, Clone, Default)]
pub enum Class {
    // check if this makes sense
    #[default]
    TransKnight,
    Foema,
    BeastMaster,
    Huntress,
}
impl From<Class> for i32 {
    fn from(value: Class) -> Self {
        match value {
            Class::TransKnight => 0,
            Class::Foema => 1,
            Class::BeastMaster => 2,
            Class::Huntress => 3,
        }
    }
}
impl TryFrom<i32> for Class {
    type Error = FailToParseClass;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Class::TransKnight,
            1 => Class::Foema,
            2 => Class::BeastMaster,
            3 => Class::Huntress,
            _ => return Err(FailToParseClass(value)),
        })
    }
}

#[derive(Debug, Error)]
#[error("Fail to parse to class: {0}")]
pub struct FailToParseClass(i32);

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Evolution {
    #[default]
    Mortal = 1,
    Arch = 2,
    Celestial = 3,
    SubCelestial = 4,
}
impl Evolution {
    pub fn as_index(self) -> usize {
        self as usize
    }
}
impl TryFrom<i32> for Evolution {
    type Error = FailToParseEvolution;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Evolution::Mortal,
            2 => Evolution::Arch,
            3 => Evolution::Celestial,
            4 => Evolution::SubCelestial,
            _ => return Err(FailToParseEvolution(value)),
        })
    }
}
impl std::cmp::Ord for Evolution {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_index().cmp(&other.as_index())
    }
}
impl std::cmp::PartialOrd for Evolution {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Error)]
#[error("Fail to parse evolution: {0}")]
pub struct FailToParseEvolution(i32);
