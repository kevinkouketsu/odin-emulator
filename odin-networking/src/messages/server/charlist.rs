use crate::{
    messages::{
        common::{ItemRaw, ScoreRaw},
        string::FixedSizeString,
        ServerMessage,
    },
    WritableResource,
};
use deku::prelude::*;
use odin_models::{
    item::Item, position::Position, status::Score, storage::Storage, MAX_EQUIPS, MAX_STORAGE_ITEMS,
};
use std::array;

#[derive(Debug, Clone)]
pub struct Charlist {
    pub token: Vec<u8>,
    pub character_info: Vec<(usize, CharlistInfo)>,
    pub storage: Storage,
    pub account_name: String,
}
impl WritableResource for Charlist {
    const IDENTIFIER: ServerMessage = ServerMessage::FirstCharlist;
    type Output = CharlistRaw;

    fn write(self) -> Result<Self::Output, crate::WritableResourceError> {
        let character_info = CharlistInfoRaw::from(&self);
        Ok(CharlistRaw {
            token: array::from_fn(|i| {
                self.token
                    .get(i)
                    .expect("Generated token must have the same size as packet")
                    .to_owned()
            }),
            data: character_info,
            storage_items: self.storage.items.into_iter().fold(
                [ItemRaw::default(); MAX_STORAGE_ITEMS],
                |mut acc, (index, item)| {
                    acc[index] = item.into();
                    acc
                },
            ),
            storage_coin: self.storage.coin,
            account_name: self.account_name.try_into()?,
            ssn1: 0,
            ssn2: 0,
        })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CharlistInfo {
    pub position: Position,
    pub name: String,
    pub status: Score,
    pub equips: [Item; MAX_EQUIPS],
    pub guild: u16,
    pub coin: u32,
    pub experience: u64,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct CharlistRaw {
    pub token: [u8; 16],
    pub data: CharlistInfoRaw,
    pub storage_items: [ItemRaw; MAX_STORAGE_ITEMS],
    pub storage_coin: u64,
    pub account_name: FixedSizeString<16>,
    pub ssn1: u32,
    pub ssn2: u32,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct CharlistInfoRaw {
    pub home_town_x: [u16; 4],
    pub home_town_y: [u16; 4],
    pub name: [FixedSizeString<16>; 4],
    pub score: [ScoreRaw; 4],
    pub equipments: [[ItemRaw; MAX_EQUIPS]; 4],
    pub guilds: [u16; 4],
    pub coin: [u32; 4],
    pub experience: [u64; 4],
}
impl From<&Charlist> for CharlistInfoRaw {
    fn from(value: &Charlist) -> Self {
        CharlistInfoRaw {
            home_town_x: map_character_info(&value.character_info, |char| char.position.x),
            home_town_y: map_character_info(&value.character_info, |char| char.position.y),
            name: map_character_info(&value.character_info, |char| {
                char.name.as_str().try_into().unwrap_or_default()
            }),
            score: map_character_info(&value.character_info, |char| char.status.into()),
            equipments: map_character_info(&value.character_info, |char| {
                array::from_fn(|i| {
                    char.equips
                        .get(i)
                        .expect("Fixed array size")
                        .to_owned()
                        .into()
                })
            }),
            guilds: map_character_info(&value.character_info, |char| char.guild),
            coin: map_character_info(&value.character_info, |char| char.coin),
            experience: map_character_info(&value.character_info, |char| char.experience),
        }
    }
}
fn map_character_info<F, T, const N: usize>(
    character_info: &[(usize, CharlistInfo)],
    mut callback: F,
) -> [T; N]
where
    F: FnMut(&CharlistInfo) -> T,
    T: Default,
{
    array::from_fn(|i| {
        character_info
            .iter()
            .find_map(|(index, char)| match *index == i {
                true => Some(callback(char)),
                false => None,
            })
            .unwrap_or_default()
    })
}
