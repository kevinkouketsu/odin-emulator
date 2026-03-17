use odin_models::{EquipmentSlots, effect::Effect, item::Item, item_data::ItemDatabase};

pub(super) fn item_ability(item: &Item, effect: Effect, item_db: &ItemDatabase) -> i32 {
    let mut total: i32 = 0;
    let effect_index = effect as u8;

    if let Some(data) = item_db.get(item.id) {
        for ef in &data.effects {
            if ef.index == effect_index && ef.index != 0 {
                total += ef.value as i32;
            }
        }
    }

    for ef in &item.effects {
        if ef.index == effect_index && ef.index != 0 {
            total += ef.value as i32;
        }
    }

    total
}

pub(super) fn mob_ability(
    equipments: &EquipmentSlots,
    effect: Effect,
    item_db: &ItemDatabase,
) -> i32 {
    equipments
        .iter()
        .map(|(_, item)| item_ability(item, effect, item_db))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use odin_models::{
        EquipmentSlot,
        item_data::{ItemData, ItemDataEffect, MAX_ITEM_DATA_EFFECTS},
    };

    fn make_item_data(id: u16, effect: Effect, value: i16) -> ItemData {
        let mut effects = [ItemDataEffect::default(); MAX_ITEM_DATA_EFFECTS];
        effects[0] = ItemDataEffect {
            index: effect as u8,
            value,
        };

        ItemData {
            id,
            name: "Test".to_string(),
            mesh: (0, 0),
            level: 0,
            str_req: 0,
            int_req: 0,
            dex_req: 0,
            con_req: 0,
            effects,
            price: 0,
            unique: 0,
            pos: 0,
            extreme: 0,
            grade: 0,
        }
    }

    #[test]
    fn item_ability_returns_template_effect() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let item = Item::from(100u16);
        assert_eq!(item_ability(&item, Effect::Damage, &db), 50);
    }

    #[test]
    fn item_ability_returns_zero_for_non_matching_effect() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let item = Item::from(100u16);
        assert_eq!(item_ability(&item, Effect::Defense, &db), 0);
    }

    #[test]
    fn item_ability_combines_template_and_runtime() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let item = Item::from((100u16, Effect::Damage as u8, 10u8));
        assert_eq!(item_ability(&item, Effect::Damage, &db), 60);
    }

    #[test]
    fn item_ability_unknown_item_uses_runtime_only() {
        let db = ItemDatabase::default();
        let item = Item::from((999u16, Effect::Str as u8, 5u8));
        assert_eq!(item_ability(&item, Effect::Str, &db), 5);
    }

    #[test]
    fn mob_ability_sums_all_equipment() {
        let db = ItemDatabase::from_items([
            make_item_data(100, Effect::Damage, 30),
            make_item_data(200, Effect::Damage, 20),
        ]);

        let equipments = EquipmentSlots::from([
            (EquipmentSlot::LeftWeapon, Item::from(100u16)),
            (EquipmentSlot::RightWeapon, Item::from(200u16)),
        ]);

        assert_eq!(mob_ability(&equipments, Effect::Damage, &db), 50);
    }
}
