use crate::{
    account::{AccessLevel, Ban},
    character::Class,
    item::Item,
    position::Position,
    status::Score,
    storage::Storage,
};

#[derive(Debug, Default, Clone)]
pub struct AccountCharlist {
    pub username: String,
    pub password: String,
    pub ban: Option<Ban>,
    pub access: Option<AccessLevel>,
    pub storage: Storage,
    pub token: Option<String>,
    pub charlist: Vec<(usize, CharacterInfo)>,
}

#[derive(Debug, Default, Clone)]
pub struct CharacterInfo {
    pub name: String,
    pub status: Score,
    pub guild: Option<u16>,
    pub class: Class,
    pub equipments: Vec<(usize, Item)>,
    pub coin: u32,
    pub experience: i64,
    pub position: Position,
}
