use deku::prelude::*;
use odin_models::{
    item::{Item, ItemBonusEffect},
    status::Score,
    MAX_ITEM_EFFECT,
};
use std::array;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct ItemRaw {
    pub id: u16,
    pub effects: [ItemBonusEffectRaw; MAX_ITEM_EFFECT],
}
impl From<Item> for ItemRaw {
    fn from(value: Item) -> Self {
        ItemRaw {
            id: value.id,
            effects: array::from_fn(|i| value.effects[i].into()),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct ItemBonusEffectRaw {
    // TODO: change this to an enum
    pub index: u8,
    pub value: u8,
}
impl From<ItemBonusEffect> for ItemBonusEffectRaw {
    fn from(value: ItemBonusEffect) -> Self {
        ItemBonusEffectRaw {
            index: value.index,
            value: value.value,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct PositionRaw {
    pub x: u16,
    pub y: u16,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, DekuRead, DekuWrite)]
#[repr(C)]
pub struct ScoreRaw {
    pub level: u16,
    pub defense: u32,
    pub damage: u32,
    pub reserved: i8,
    pub attack_run: i8,
    pub max_hp: u32,
    pub max_mp: u32,
    pub hp: u32,
    pub mp: u32,
    pub strength: u16,
    pub intelligence: u16,
    pub dexterity: u16,
    pub constitution: u16,
    pub specials: [u16; 4],
}
impl From<Score> for ScoreRaw {
    fn from(value: Score) -> Self {
        ScoreRaw {
            level: value.level,
            defense: value.defense,
            damage: value.damage,
            reserved: value.reserved,
            attack_run: value.attack_run,
            max_hp: value.max_hp,
            max_mp: value.max_mp,
            hp: value.hp,
            mp: value.mp,
            strength: value.strength,
            intelligence: value.intelligence,
            dexterity: value.dexterity,
            constitution: value.constitution,
            specials: value.specials,
        }
    }
}

#[test]
fn size_of() {
    assert_eq!(std::mem::size_of::<ItemRaw>(), 8);
    assert_eq!(std::mem::size_of::<ScoreRaw>(), 48);
}
