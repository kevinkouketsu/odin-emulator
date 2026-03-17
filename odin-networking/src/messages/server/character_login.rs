use crate::{
    WritableResource, WritableResourceError,
    messages::{
        ServerMessage,
        common::{ItemRaw, PositionRaw, ScoreRaw},
        string::FixedSizeString,
    },
};
use deku::prelude::*;
use odin_models::{
    MAX_EQUIPS, MAX_INVENTORY,
    character::{Character, GuildLevel},
    position::Position,
};
use std::array;

const EXT1_SIZE: usize = 288;
const EXT2_SIZE: usize = 360;

pub struct CharacterLogin {
    pub position: Position,
    pub character: Character,
    pub client_id: u16,
}

impl WritableResource for CharacterLogin {
    const IDENTIFIER: ServerMessage = ServerMessage::CharacterLogin;
    type Output = CharacterLoginRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        let mob = StructMobRaw::from_character(&self.character);

        Ok(CharacterLoginRaw {
            pos_x: self.position.x as i16,
            pos_y: self.position.y as i16,
            mob,
            slot: 0,
            client_id: self.client_id,
            weather: 0,
            short_skill: [0xFF; 16],
            ext1: [0; EXT1_SIZE],
            ext2: [0; EXT2_SIZE],
        })
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct StructMobRaw {
    pub mob_name: FixedSizeString<16>,
    pub clan: i8,
    pub merchant: i8,
    pub guild: u16,
    pub class: i8,
    pub affect_info: u8,
    pub quest_info: u16,
    pub coin: i32,
    pub exp: i64,
    pub last_position: PositionRaw,
    pub base_score: ScoreRaw,
    pub current_score: ScoreRaw,
    pub equip: [ItemRaw; MAX_EQUIPS],
    pub carry: [ItemRaw; MAX_INVENTORY],
    pub learned_skill: [u32; 2],
    pub score_bonus: i16,
    pub special_bonus: i16,
    pub skill_bonus: i16,
    pub critical: u8,
    pub save_mana: u8,
    pub short_skill: [u8; 4],
    pub guild_level: i8,
    pub magic: u32,
    pub regen_hp: u8,
    pub regen_mp: u8,
    pub resist: [i8; 4],
}

impl StructMobRaw {
    fn from_character(character: &Character) -> Self {
        let equip = character
            .equipments
            .map_slots(|_, item| ItemRaw::from(*item));

        let carry: [ItemRaw; MAX_INVENTORY] = array::from_fn(|i| {
            character
                .inventory
                .get(i)
                .map(|item| ItemRaw::from(*item))
                .unwrap_or_default()
        });

        let guild_level = match &character.guild_level {
            Some(GuildLevel::Participant) => 1,
            Some(GuildLevel::FirstCommander) => 3,
            Some(GuildLevel::SecondCommander) => 4,
            Some(GuildLevel::ThirdCommander) => 5,
            Some(GuildLevel::Leader) => 9,
            None => 0,
        };

        StructMobRaw {
            mob_name: character.name.as_str().try_into().unwrap_or_default(),
            clan: 0,
            merchant: character.merchant as i8,
            guild: character.guild.unwrap_or(0) as u16,
            class: i32::from(character.class) as i8,
            affect_info: character.affect_info as u8,
            quest_info: character.quest_info as u16,
            coin: character.coin,
            exp: character.experience,
            last_position: PositionRaw {
                x: character.last_pos.x,
                y: character.last_pos.y,
            },
            base_score: character.score.into(),
            current_score: character.score.into(),
            equip,
            carry,
            learned_skill: [0; 2],
            score_bonus: 0,
            special_bonus: 0,
            skill_bonus: 0,
            critical: 0,
            save_mana: 0,
            short_skill: [0xFF; 4],
            guild_level,
            magic: 0,
            regen_hp: 0,
            regen_mp: 0,
            resist: [0; 4],
        }
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct CharacterLoginRaw {
    pub pos_x: i16,
    pub pos_y: i16,
    pub mob: StructMobRaw,
    pub slot: u16,
    pub client_id: u16,
    pub weather: u16,
    pub short_skill: [u8; 16],
    pub ext1: [u8; EXT1_SIZE],
    pub ext2: [u8; EXT2_SIZE],
}

#[cfg(test)]
mod tests {
    use super::*;
    use deku::DekuContainerWrite;

    fn default_struct_mob_raw() -> StructMobRaw {
        StructMobRaw::from_character(&Character::default())
    }

    #[test]
    fn struct_mob_raw_serialized_size() {
        let mob = default_struct_mob_raw();
        let bytes = mob.to_bytes().unwrap();
        assert_eq!(bytes.len(), 815);
    }

    #[test]
    fn character_login_raw_serialized_size() {
        let raw = CharacterLoginRaw {
            pos_x: 0,
            pos_y: 0,
            mob: default_struct_mob_raw(),
            slot: 0,
            client_id: 0,
            weather: 0,
            short_skill: [0; 16],
            ext1: [0; EXT1_SIZE],
            ext2: [0; EXT2_SIZE],
        };
        let bytes = raw.to_bytes().unwrap();
        assert_eq!(bytes.len(), 1489);
    }
}
