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
    EquipmentSlots, InventorySlots, MAX_EQUIPS, MAX_INVENTORY,
    character::{Class, Evolution, GuildLevel},
    position::Position,
    status::Score,
};
use std::array;

const EXT1_SIZE: usize = 288;
const EXT2_SIZE: usize = 360;

pub struct CharacterLogin {
    pub position: Position,
    pub client_id: u16,

    pub name: String,
    pub class: Class,
    pub evolution: Evolution,
    pub merchant: i16,
    pub guild: Option<i16>,
    pub guild_level: Option<GuildLevel>,
    pub affect_info: i16,
    pub quest_info: i16,
    pub coin: i32,
    pub experience: i64,
    pub last_pos: Position,
    pub equipments: EquipmentSlots,
    pub inventory: InventorySlots,

    pub base_score: Score,
    pub current_score: Score,

    pub score_bonus: i16,
    pub special_bonus: i16,
    pub skill_bonus: i16,

    pub critical: u8,
    pub save_mana: i32,
    pub magic: i32,
    pub regen_hp: i32,
    pub regen_mp: i32,
    pub resist: [i32; 4],
}

impl WritableResource for CharacterLogin {
    const IDENTIFIER: ServerMessage = ServerMessage::CharacterLogin;
    type Output = CharacterLoginRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        let equip = self.equipments.map_slots(|_, item| ItemRaw::from(*item));

        let carry: [ItemRaw; MAX_INVENTORY] = array::from_fn(|i| {
            self.inventory
                .get(i)
                .map(|item| ItemRaw::from(*item))
                .unwrap_or_default()
        });

        let guild_level = self.guild_level.map(|g| g.as_raw() as i8).unwrap_or(0);

        let mob = StructMobRaw {
            mob_name: self.name.as_str().try_into().unwrap_or_default(),
            clan: 0,
            merchant: self.merchant as i8,
            guild: self.guild.unwrap_or(0) as u16,
            class: i32::from(self.class) as i8,
            affect_info: self.affect_info as u8,
            quest_info: self.quest_info as u16,
            coin: self.coin,
            exp: self.experience,
            last_position: PositionRaw {
                x: self.last_pos.x,
                y: self.last_pos.y,
            },
            base_score: self.base_score.into(),
            current_score: self.current_score.into(),
            equip,
            carry,
            learned_skill: [0; 2],
            score_bonus: self.score_bonus,
            special_bonus: self.special_bonus,
            skill_bonus: self.skill_bonus,
            critical: self.critical,
            save_mana: self.save_mana.clamp(0, 255) as u8,
            short_skill: [0xFF; 4],
            guild_level,
            magic: self.magic.max(0) as u32,
            regen_hp: self.regen_hp.clamp(0, 255) as u8,
            regen_mp: self.regen_mp.clamp(0, 255) as u8,
            resist: [
                self.resist[0].clamp(-128, 127) as i8,
                self.resist[1].clamp(-128, 127) as i8,
                self.resist[2].clamp(-128, 127) as i8,
                self.resist[3].clamp(-128, 127) as i8,
            ],
        };

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
        StructMobRaw {
            mob_name: FixedSizeString::default(),
            clan: 0,
            merchant: 0,
            guild: 0,
            class: 0,
            affect_info: 0,
            quest_info: 0,
            coin: 0,
            exp: 0,
            last_position: PositionRaw::default(),
            base_score: ScoreRaw::default(),
            current_score: ScoreRaw::default(),
            equip: [ItemRaw::default(); MAX_EQUIPS],
            carry: [ItemRaw::default(); MAX_INVENTORY],
            learned_skill: [0; 2],
            score_bonus: 0,
            special_bonus: 0,
            skill_bonus: 0,
            critical: 0,
            save_mana: 0,
            short_skill: [0; 4],
            guild_level: 0,
            magic: 0,
            regen_hp: 0,
            regen_mp: 0,
            resist: [0; 4],
        }
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
