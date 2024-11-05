use crate::{character::Character, storage::Storage};
use chrono::NaiveDateTime;

#[derive(Debug, Default, Clone)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub ban: Option<Ban>,
    pub access: Option<AccessLevel>,
    pub storage: Storage,
    pub token: Option<String>,
    pub characters: Vec<(usize, Character)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessLevel {
    Administrator,
    GameMaster(u32),
}
impl AccessLevel {
    pub fn get_level(&self) -> u32 {
        match self {
            AccessLevel::Administrator => 100,
            AccessLevel::GameMaster(level) => *level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ban {
    pub expiration: NaiveDateTime,
    pub r#type: BanType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BanType {
    Analysis,
    Blocked,
}
