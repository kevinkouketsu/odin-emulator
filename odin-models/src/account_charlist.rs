use crate::{
    account::{AccessLevel, Ban},
    character::Class,
    item::Item,
    position::Position,
    status::Score,
    storage::Storage,
};
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct AccountCharlist {
    pub identifier: Uuid,
    pub username: String,
    pub password: String,
    pub ban: Option<Ban>,
    pub access: Option<AccessLevel>,
    pub storage: Storage,
    pub charlist: Vec<(usize, CharacterInfo)>,
}

#[derive(Debug, Default, Clone)]
pub struct CharacterInfo {
    pub identifier: Uuid,
    pub name: String,
    pub status: Score,
    pub guild: Option<u16>,
    pub class: Class,
    pub equipments: Vec<(usize, Item)>,
    pub coin: u32,
    pub experience: i64,
    pub position: Position,
}
