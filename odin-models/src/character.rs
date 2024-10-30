use crate::{item::Item, position::Position, status::Score};

#[derive(Debug, Default, Clone)]
pub struct Character {
    pub name: String,
    // TODO maybe temporary
    pub slot: i32,
    pub score: Score,
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
    pub gold: i64,
    pub experience: i64,
    pub last_pos: Position,
    pub inventory: Vec<(usize, Item)>,
    pub equipments: Vec<(usize, Item)>,
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

#[derive(Debug, Clone, Default)]
pub enum Class {
    // check if this makes sense
    #[default]
    TransKnight,
    Foema,
    BeastMaster,
    Huntress,
}
impl Class {
    pub fn new<I: Into<i32>>(class: I) -> Option<Self> {
        let class = class.into();
        Some(match class {
            0 => Class::TransKnight,
            1 => Class::Foema,
            2 => Class::BeastMaster,
            3 => Class::Huntress,
            _ => return None,
        })
    }
}
