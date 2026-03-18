use crate::{
    WritableResource, WritableResourceError,
    messages::{ServerMessage, common::ScoreRaw, string::FixedSizeString},
};
use deku::prelude::*;
use odin_models::{
    EquipmentSlot, EquipmentSlots, MAX_AFFECT, MAX_EQUIPS, character::GuildLevel, item::Item,
    position::Position, status::Score,
};

#[derive(Clone)]
pub struct CreateMob {
    pub position: Position,
    pub mob_id: u16,
    pub name: String,
    pub score: Score,
    pub equipments: EquipmentSlots,
    pub guild: Option<i16>,
    pub guild_level: Option<GuildLevel>,
    pub create_type: u16,
    pub affect: [u8; MAX_AFFECT],
}

impl WritableResource for CreateMob {
    const IDENTIFIER: ServerMessage = ServerMessage::CreateMob;
    type Output = CreateMobRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        let equip = self.equipments.map_slots(VisualEquipRaw::from_equipment);
        let face_index = self
            .equipments
            .get(EquipmentSlot::Face)
            .map(|item| item.id as i16)
            .unwrap_or(0);

        let guild_level = self.guild_level.map(|g| g.as_raw() as i8).unwrap_or(0);

        Ok(CreateMobRaw {
            pos_x: self.position.x as i16,
            pos_y: self.position.y as i16,
            mob_id: self.mob_id,
            mob_name: self.name.as_str().try_into()?,
            chaos_points: 0,
            current_kill: 0,
            total_kill: 0,
            equip,
            affect: self.affect,
            guild: self.guild.unwrap_or(0) as u16,
            guild_level,
            score: self.score.into(),
            create_type: self.create_type,
            face_index,
            nick: self.name.as_str().try_into()?,
            server: 0,
            life: self.score.hp as i32,
            coat_index: 0,
            royal_arena_level: 0,
        })
    }
}

#[derive(Debug, Clone, PartialEq, DekuRead, DekuWrite)]
pub struct CreateMobRaw {
    pub pos_x: i16,
    pub pos_y: i16,
    pub mob_id: u16,
    pub mob_name: FixedSizeString<16>,
    pub chaos_points: u8,
    pub current_kill: u8,
    pub total_kill: u16,
    pub equip: [VisualEquipRaw; MAX_EQUIPS],
    pub affect: [u8; MAX_AFFECT],
    pub guild: u16,
    pub guild_level: i8,
    pub score: ScoreRaw,
    pub create_type: u16,
    pub face_index: i16,
    pub nick: FixedSizeString<26>,
    pub server: i8,
    pub life: i32,
    pub coat_index: i32,
    pub royal_arena_level: i32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct VisualEquipRaw {
    pub index: u16,
    pub effect_index: u8,
    pub effect_value: u8,
}
impl VisualEquipRaw {
    pub fn from_equipment(slot: EquipmentSlot, item: &Item) -> Self {
        let slot_index = slot.as_index();

        if slot_index == EquipmentSlot::Mount.as_index() {
            if item.id >= 2360 && item.id <= 2390 {
                let combined = i16::from_le_bytes([item.effects[0].index, item.effects[0].value]);
                if combined <= 0 {
                    return VisualEquipRaw::default();
                }
            }
            return VisualEquipRaw {
                index: item.id,
                effect_index: 0,
                effect_value: item.effects[1].index / 10,
            };
        }

        let mut value: u8 = 0;
        let mut colored = false;

        for effect in &item.effects {
            if effect.index >= 116 && effect.index <= 125 {
                value = effect.index;
                colored = true;
                break;
            }
        }

        if !colored {
            for effect in &item.effects {
                if effect.index == 43 {
                    value = effect.value;
                    break;
                }
            }
        }

        if value > 9 && !colored {
            value = match value {
                v if v < 230 => v % 10,
                v if v < 234 => 10,
                v if v < 238 => 11,
                v if v < 242 => 12,
                v if v < 246 => 13,
                v if v < 250 => 14,
                v if v < 254 => 15,
                _ => 16,
            };
        } else if colored && value >= 9 {
            value = 9;
        }

        VisualEquipRaw {
            index: item.id,
            effect_index: 0,
            effect_value: value,
        }
    }
}
