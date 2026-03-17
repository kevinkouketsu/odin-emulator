use odin_models::{
    EquipmentSlots,
    effect::Effect,
    item::Item,
    item_data::ItemDatabase,
    status::Score,
};

pub fn item_ability(item: &Item, effect: Effect, item_db: &ItemDatabase) -> i32 {
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

pub fn mob_ability(equipments: &EquipmentSlots, effect: Effect, item_db: &ItemDatabase) -> i32 {
    equipments
        .iter()
        .map(|(_, item)| item_ability(item, effect, item_db))
        .sum()
}

pub fn calculate_score(
    base_score: &Score,
    current_hp: u32,
    current_mp: u32,
    equipments: &EquipmentSlots,
    item_db: &ItemDatabase,
) -> Score {
    let damage = (base_score.damage as i32 + mob_ability(equipments, Effect::Damage, item_db))
        .max(0) as u32;
    let defense = (base_score.defense as i32 + mob_ability(equipments, Effect::Defense, item_db))
        .max(0) as u32;
    let max_hp =
        (base_score.max_hp as i32 + mob_ability(equipments, Effect::Hp, item_db)).max(1) as u32;
    let max_mp =
        (base_score.max_mp as i32 + mob_ability(equipments, Effect::Mp, item_db)).max(0) as u32;
    let strength = (base_score.strength as i32 + mob_ability(equipments, Effect::Str, item_db))
        .max(0) as u16;
    let intelligence =
        (base_score.intelligence as i32 + mob_ability(equipments, Effect::Int, item_db)).max(0)
            as u16;
    let dexterity = (base_score.dexterity as i32 + mob_ability(equipments, Effect::Dex, item_db))
        .max(0) as u16;
    let constitution =
        (base_score.constitution as i32 + mob_ability(equipments, Effect::Con, item_db)).max(0)
            as u16;

    let mut specials = base_score.specials;
    let special_effects = [
        Effect::Special1,
        Effect::Special2,
        Effect::Special3,
        Effect::Special4,
    ];
    for (i, effect) in special_effects.iter().enumerate() {
        specials[i] =
            (specials[i] as i32 + mob_ability(equipments, *effect, item_db)).max(0) as u16;
    }

    Score {
        level: base_score.level,
        damage,
        defense,
        max_hp,
        max_mp,
        hp: current_hp.min(max_hp),
        mp: current_mp.min(max_mp),
        strength,
        intelligence,
        dexterity,
        constitution,
        specials,
        reserved: base_score.reserved,
        attack_run: base_score.attack_run,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odin_models::{
        EquipmentSlot,
        effect::Effect,
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

    #[test]
    fn calculate_score_applies_equipment_bonuses() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let equipments = EquipmentSlots::from([(EquipmentSlot::LeftWeapon, Item::from(100u16))]);
        let base = Score {
            damage: 10,
            max_hp: 100,
            hp: 100,
            ..Default::default()
        };

        let result = calculate_score(&base, base.hp, base.mp, &equipments, &db);
        assert_eq!(result.damage, 60);
        assert_eq!(result.max_hp, 100);
    }

    #[test]
    fn calculate_score_clamps_hp_to_new_max() {
        let db = ItemDatabase::default();
        let equipments = EquipmentSlots::from([] as [(EquipmentSlot, Item); 0]);
        let base = Score {
            max_hp: 50,
            hp: 100,
            ..Default::default()
        };

        let result = calculate_score(&base, 100, 0, &equipments, &db);
        assert_eq!(result.hp, 50);
    }
}
