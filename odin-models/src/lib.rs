pub mod account;
pub mod account_charlist;
pub mod character;
pub mod direction;
pub mod effect;
pub mod height_map;
pub mod item;
pub mod item_data;
pub mod item_slots;
pub mod nickname;
pub mod npc_mob;
pub mod position;
pub mod status;
pub mod storage;

pub const MAX_CHARACTERS: usize = 4;
pub const MAX_EQUIPS: usize = 18;
pub const MAX_STORAGE_ITEMS: usize = 160;
pub const MAX_ITEM_EFFECT: usize = 3;
pub const MAX_INVENTORY: usize = 64;
pub const MAX_INVENTORY_VISIBLE: usize = MAX_INVENTORY - 4;
pub const MAX_AFFECT: usize = 32;

pub use uuid;

pub use item_slots::{ItemSlots, SlotIndex};

pub type InventorySlots = ItemSlots<usize, MAX_INVENTORY_VISIBLE>;
pub type EquipmentSlots = ItemSlots<EquipmentSlot, MAX_EQUIPS>;
pub type StorageSlots = ItemSlots<usize, MAX_STORAGE_ITEMS>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Face = 0,
    Helmet = 1,
    Armor = 2,
    ArmorPants = 3,
    Gloves = 4,
    Boots = 5,
    LeftWeapon = 6,
    RightWeapon = 7,
    Amulet1 = 8,
    Amulet2 = 9,
    Amulet3 = 10,
    Amulet4 = 11,
    Familiar = 12,
    Costume = 13,
    Mount = 14,
    Mantle = 15,
    Reserved1 = 16,
    Reserved2 = 17,
}
impl EquipmentSlot {
    pub fn as_index(self) -> usize {
        self as usize
    }
}
impl From<EquipmentSlot> for usize {
    fn from(val: EquipmentSlot) -> Self {
        val.as_index()
    }
}
impl SlotIndex for EquipmentSlot {
    fn to_index(self) -> usize {
        self.as_index()
    }
    fn from_index(index: usize) -> Option<Self> {
        Self::try_from(index).ok()
    }
}

impl TryFrom<usize> for EquipmentSlot {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let slot = match value {
            0 => EquipmentSlot::Face,
            1 => EquipmentSlot::Helmet,
            2 => EquipmentSlot::Armor,
            3 => EquipmentSlot::ArmorPants,
            4 => EquipmentSlot::Gloves,
            5 => EquipmentSlot::Boots,
            6 => EquipmentSlot::LeftWeapon,
            7 => EquipmentSlot::RightWeapon,
            8 => EquipmentSlot::Amulet1,
            9 => EquipmentSlot::Amulet2,
            10 => EquipmentSlot::Amulet3,
            11 => EquipmentSlot::Amulet4,
            12 => EquipmentSlot::Familiar,
            13 => EquipmentSlot::Costume,
            14 => EquipmentSlot::Mount,
            15 => EquipmentSlot::Mantle,
            16 => EquipmentSlot::Reserved1,
            17 => EquipmentSlot::Reserved2,
            _ => return Err(format!("Can't convert {value} to EquipmentSlot")),
        };

        Ok(slot)
    }
}
