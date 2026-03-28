use crate::character::{Class, GuildLevel};
use crate::status::Score;
use crate::{EquipmentSlots, InventorySlots};

#[derive(Debug, Clone, Default)]
pub struct NpcMob {
    pub name: String,
    pub clan: i8,
    pub merchant: i16,
    pub guild: Option<i16>,
    pub guild_level: Option<GuildLevel>,
    pub class: Class,
    pub affect_info: i16,
    pub quest_info: i16,
    pub coin: i32,
    pub experience: i64,
    pub score: Score,
    pub equipments: EquipmentSlots,
    pub inventory: InventorySlots,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_mob_default() {
        let mob = NpcMob::default();
        assert!(mob.name.is_empty());
        assert_eq!(mob.clan, 0);
        assert_eq!(mob.coin, 0);
        assert_eq!(mob.guild, None);
    }
}
