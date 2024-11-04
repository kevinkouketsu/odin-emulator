use crate::{
    messages::{
        common::{ItemRaw, ScoreRaw},
        server::MessageSignal,
        string::FixedSizeString,
        ServerMessage,
    },
    WritableResource, WritableResourceError,
};
use deku::prelude::*;
use odin_macros::MessageSignalDerive;
use odin_models::{
    account_charlist::CharacterInfo, item::Item, position::Position, status::Score,
    storage::Storage, MAX_EQUIPS, MAX_STORAGE_ITEMS,
};
use std::array;

#[derive(Debug, Clone)]
pub struct FirstCharlist {
    pub token: Vec<u8>,
    pub character_info: Vec<(usize, CharlistInfo)>,
    pub storage: Storage,
    pub account_name: String,
}
impl WritableResource for FirstCharlist {
    const IDENTIFIER: ServerMessage = ServerMessage::FirstCharlist;
    type Output = FirstCharlistRaw;

    fn write(self) -> Result<Self::Output, crate::WritableResourceError> {
        let character_info = CharlistInfoRaw::from(self.character_info.as_slice());
        Ok(FirstCharlistRaw {
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
pub struct UpdateCharlist {
    pub character_info: Vec<(usize, CharlistInfo)>,
}
impl WritableResource for UpdateCharlist {
    const IDENTIFIER: ServerMessage = ServerMessage::UpdateCharlist;
    type Output = UpdateCharlistRaw;

    fn write(self) -> Result<Self::Output, crate::WritableResourceError> {
        Ok(UpdateCharlistRaw {
            data: CharlistInfoRaw::from(self.character_info.as_slice()),
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct CharlistInfo {
    pub position: Position,
    pub name: String,
    pub status: Score,
    pub equips: Vec<(usize, Item)>,
    pub guild: Option<u16>,
    pub coin: u32,
    pub experience: i64,
}
impl From<CharacterInfo> for CharlistInfo {
    fn from(character: CharacterInfo) -> Self {
        CharlistInfo {
            position: character.position,
            name: character.name.clone(),
            status: character.status,
            equips: character.equipments.clone(),
            guild: character.guild,
            coin: character.coin,
            experience: character.experience,
        }
    }
}
impl From<odin_models::account_charlist::AccountCharlist> for FirstCharlist {
    fn from(value: odin_models::account_charlist::AccountCharlist) -> Self {
        let characters = value
            .charlist
            .into_iter()
            .map(|(slot, character)| (slot, character.into()))
            .collect();

        FirstCharlist {
            token: vec![0; 16],
            character_info: characters,
            storage: Storage::default(),
            account_name: value.username.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct FirstCharlistRaw {
    pub token: [u8; 16],
    pub data: CharlistInfoRaw,
    pub storage_items: [ItemRaw; MAX_STORAGE_ITEMS],
    pub storage_coin: u64,
    pub account_name: FixedSizeString<16>,
    pub ssn1: u32,
    pub ssn2: u32,
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct UpdateCharlistRaw {
    pub data: CharlistInfoRaw,
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
    pub experience: [i64; 4],
}
impl From<&[(usize, CharlistInfo)]> for CharlistInfoRaw {
    fn from(value: &[(usize, CharlistInfo)]) -> Self {
        CharlistInfoRaw {
            home_town_x: map_character_info(value, |char| char.position.x),
            home_town_y: map_character_info(value, |char| char.position.y),
            name: map_character_info(value, |char| {
                char.name.as_str().try_into().unwrap_or_default()
            }),
            score: map_character_info(value, |char| char.status.into()),
            equipments: map_character_info(value, |char| {
                array::from_fn(|i| {
                    char.equips
                        .iter()
                        .find_map(|(index, item)| (*index == i).then_some(item.to_owned()))
                        .unwrap_or_default()
                        .into()
                })
            }),
            guilds: map_character_info(value, |char| char.guild.unwrap_or_default()),
            coin: map_character_info(value, |char| char.coin),
            experience: map_character_info(value, |char| char.experience),
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

#[derive(Default, MessageSignalDerive)]
#[identifier = "ServerMessage::CharacterNameAlreadyExists"]
pub struct NameAlreadyExistsError;
