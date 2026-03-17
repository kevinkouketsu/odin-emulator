pub mod base;
pub mod critical;
mod equipment;

use critical::Critical;
use odin_models::{
    EquipmentSlots,
    character::{Class, Evolution},
    effect::Effect,
    item_data::ItemDatabase,
    status::Score,
};

const SPECIAL_EFFECTS: [Effect; 4] = [
    Effect::Special1,
    Effect::Special2,
    Effect::Special3,
    Effect::Special4,
];

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ComputedScore {
    pub score: Score,

    pub critical: Critical,
    pub magic: i32,
    pub parry: i32,
    pub hit_rate: i32,
    pub attack_speed: i32,
    pub weapon_damage: i32,

    pub resist: [i32; 4],
    pub regen_hp: i32,
    pub regen_mp: i32,
    pub save_mana: i32,

    pub attack_pvp: i32,
    pub defense_pvp: i32,
    pub force_damage: i32,
    pub reflect_damage: i32,

    pub life_steal: i32,
    pub vampirism: i32,
    pub potion_bonus: i32,

    pub ignore_resistance: i32,
    pub slow_chance: i32,
    pub resistance_chance: i32,

    pub exp_bonus: i32,
    pub drop_bonus: i32,
    pub individual_exp_bonus: i32,
}

pub struct StatBuilder<'a> {
    item_db: &'a ItemDatabase,
    #[allow(dead_code)]
    class: Class,

    level: i32,
    damage: i32,
    defense: i32,
    max_hp: i32,
    max_mp: i32,
    strength: i32,
    intelligence: i32,
    dexterity: i32,
    constitution: i32,
    specials: [i32; 4],

    inc_damage: i32,
    inc_defense: i32,
    inc_hp: i32,
    inc_mp: i32,

    critical: Critical,
    magic: i32,
    parry: i32,
    hit_rate: i32,
    attack_speed: i32,
    resist: [i32; 4],
    regen_hp: i32,
    regen_mp: i32,
    save_mana: i32,
    attack_pvp: i32,
    defense_pvp: i32,
    force_damage: i32,
    reflect_damage: i32,
    life_steal: i32,
    vampirism: i32,
    potion_bonus: i32,
    ignore_resistance: i32,
    slow_chance: i32,
    resistance_chance: i32,
    exp_bonus: i32,
    drop_bonus: i32,
    individual_exp_bonus: i32,
    weapon_damage: i32,

    reserved: i8,
    attack_run: i8,
}

impl<'a> StatBuilder<'a> {
    pub fn from_base(
        base: &Score,
        class: Class,
        evolution: Evolution,
        item_db: &'a ItemDatabase,
    ) -> Self {
        let base = base::calculate_base_score(base, class, evolution);
        Self {
            item_db,
            class,
            level: base.level as i32,
            damage: base.damage as i32,
            defense: base.defense as i32,
            max_hp: base.max_hp as i32,
            max_mp: base.max_mp as i32,
            strength: base.strength as i32,
            intelligence: base.intelligence as i32,
            dexterity: base.dexterity as i32,
            constitution: base.constitution as i32,
            specials: base.specials.map(|s| s as i32),
            inc_damage: 100,
            inc_defense: 100,
            inc_hp: 100,
            inc_mp: 100,
            critical: Critical::default(),
            magic: 0,
            parry: 0,
            hit_rate: 0,
            attack_speed: 0,
            resist: [0; 4],
            regen_hp: 0,
            regen_mp: 0,
            save_mana: 0,
            attack_pvp: 0,
            defense_pvp: 0,
            force_damage: 0,
            reflect_damage: 0,
            life_steal: 0,
            vampirism: 0,
            potion_bonus: 0,
            ignore_resistance: 0,
            slow_chance: 0,
            resistance_chance: 0,
            exp_bonus: 0,
            drop_bonus: 0,
            individual_exp_bonus: 0,
            weapon_damage: 0,
            reserved: base.reserved,
            attack_run: base.attack_run,
        }
    }

    pub fn apply_equipment(mut self, equipments: &EquipmentSlots) -> Self {
        self.damage += equipment::mob_ability(equipments, Effect::Damage, self.item_db);
        self.defense += equipment::mob_ability(equipments, Effect::Defense, self.item_db);
        self.max_hp += equipment::mob_ability(equipments, Effect::Hp, self.item_db);
        self.max_mp += equipment::mob_ability(equipments, Effect::Mp, self.item_db);
        self.strength += equipment::mob_ability(equipments, Effect::Str, self.item_db);
        self.intelligence += equipment::mob_ability(equipments, Effect::Int, self.item_db);
        self.dexterity += equipment::mob_ability(equipments, Effect::Dex, self.item_db);
        self.constitution += equipment::mob_ability(equipments, Effect::Con, self.item_db);
        for (i, effect) in SPECIAL_EFFECTS.iter().enumerate() {
            self.specials[i] += equipment::mob_ability(equipments, *effect, self.item_db);
        }
        self
    }

    pub fn finalize(self, current_hp: u32, current_mp: u32) -> ComputedScore {
        let damage = (self.damage * self.inc_damage / 100).max(0) as u32;
        let defense = (self.defense * self.inc_defense / 100).max(0) as u32;
        let max_hp = (self.max_hp * self.inc_hp / 100).max(1) as u32;
        let max_mp = (self.max_mp * self.inc_mp / 100).max(0) as u32;

        ComputedScore {
            score: Score {
                level: self.level.max(0) as u16,
                damage,
                defense,
                max_hp,
                max_mp,
                hp: current_hp.min(max_hp),
                mp: current_mp.min(max_mp),
                strength: self.strength.max(0) as u16,
                intelligence: self.intelligence.max(0) as u16,
                dexterity: self.dexterity.max(0) as u16,
                constitution: self.constitution.max(0) as u16,
                specials: [
                    self.specials[0].max(0) as u16,
                    self.specials[1].max(0) as u16,
                    self.specials[2].max(0) as u16,
                    self.specials[3].max(0) as u16,
                ],
                reserved: self.reserved,
                attack_run: self.attack_run,
            },
            critical: self.critical,
            magic: self.magic,
            parry: self.parry,
            hit_rate: self.hit_rate,
            attack_speed: self.attack_speed,
            weapon_damage: self.weapon_damage,
            resist: self.resist,
            regen_hp: self.regen_hp,
            regen_mp: self.regen_mp,
            save_mana: self.save_mana,
            attack_pvp: self.attack_pvp,
            defense_pvp: self.defense_pvp,
            force_damage: self.force_damage,
            reflect_damage: self.reflect_damage,
            life_steal: self.life_steal,
            vampirism: self.vampirism,
            potion_bonus: self.potion_bonus,
            ignore_resistance: self.ignore_resistance,
            slow_chance: self.slow_chance,
            resistance_chance: self.resistance_chance,
            exp_bonus: self.exp_bonus,
            drop_bonus: self.drop_bonus,
            individual_exp_bonus: self.individual_exp_bonus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odin_models::{
        EquipmentSlot,
        character::Evolution,
        item::Item,
        item_data::{ItemData, ItemDataEffect, MAX_ITEM_DATA_EFFECTS},
    };

    const E: Evolution = Evolution::Mortal;

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
    fn from_base_preserves_non_computed_fields() {
        let db = ItemDatabase::default();
        let base = Score {
            level: 5,
            strength: 30,
            intelligence: 20,
            dexterity: 15,
            constitution: 25,
            specials: [1, 2, 3, 4],
            reserved: -1,
            attack_run: 2,
            ..Default::default()
        };

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(400, 150);

        assert_eq!(result.score.level, 5);
        assert_eq!(result.score.strength, 30);
        assert_eq!(result.score.intelligence, 20);
        assert_eq!(result.score.dexterity, 15);
        assert_eq!(result.score.constitution, 25);
        assert_eq!(result.score.specials, [1, 2, 3, 4]);
        assert_eq!(result.score.reserved, -1);
        assert_eq!(result.score.attack_run, 2);
    }

    #[test]
    fn from_base_computes_base_damage_and_defense() {
        let db = ItemDatabase::default();
        // Level 10 mortal TK: damage = 5 + 1*10 = 15, defense = 4 + 1*10 = 14
        let base = Score {
            level: 10,
            ..Default::default()
        };

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(0, 0);

        assert_eq!(result.score.damage, 15);
        assert_eq!(result.score.defense, 14);
    }

    #[test]
    fn from_base_computes_base_hp_mp() {
        let db = ItemDatabase::default();
        // Level 10 mortal TK: max_hp = 3*10 + 80 = 110, max_mp = 1*10 + 45 = 55
        let base = Score {
            level: 10,
            ..Default::default()
        };

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(0, 0);

        assert_eq!(result.score.max_hp, 110);
        assert_eq!(result.score.max_mp, 55);
    }

    #[test]
    fn finalize_clamps_negative_stats_to_zero() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, -999)]);
        let equips = EquipmentSlots::from([(EquipmentSlot::LeftWeapon, Item::from(100u16))]);
        // level 0 mortal TK: base damage = 5
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(1, 0);

        assert_eq!(result.score.damage, 0);
    }

    #[test]
    fn finalize_max_hp_minimum_is_one() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Hp, -9999)]);
        let equips = EquipmentSlots::from([(EquipmentSlot::LeftWeapon, Item::from(100u16))]);
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(0, 0);

        assert_eq!(result.score.max_hp, 1);
    }

    #[test]
    fn finalize_clamps_hp_to_max() {
        let db = ItemDatabase::default();
        // level 0 mortal TK: max_hp = 80
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(9999, 0);

        assert_eq!(result.score.hp, 80);
    }

    #[test]
    fn finalize_clamps_mp_to_max() {
        let db = ItemDatabase::default();
        // level 0 mortal TK: max_mp = 45
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(0, 9999);

        assert_eq!(result.score.mp, 45);
    }

    #[test]
    fn percentage_multipliers_default_to_identity() {
        let db = ItemDatabase::default();
        // level 10 mortal TK: damage = 15, max_hp = 110
        let base = Score {
            level: 10,
            ..Default::default()
        };

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(200, 0);

        assert_eq!(result.score.damage, 15);
        assert_eq!(result.score.max_hp, 110);
    }

    #[test]
    fn extra_fields_default_to_zero() {
        let db = ItemDatabase::default();
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db).finalize(0, 0);

        assert_eq!(result.critical, Critical::default());
        assert_eq!(result.magic, 0);
        assert_eq!(result.resist, [0; 4]);
        assert_eq!(result.life_steal, 0);
        assert_eq!(result.exp_bonus, 0);
    }

    #[test]
    fn apply_equipment_adds_damage_bonus() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let equips = EquipmentSlots::from([(EquipmentSlot::LeftWeapon, Item::from(100u16))]);
        // level 0 mortal TK: base damage = 5
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(100, 0);

        assert_eq!(result.score.damage, 55); // 5 base + 50 equip
    }

    #[test]
    fn apply_equipment_multiple_slots_stack() {
        let db = ItemDatabase::from_items([
            make_item_data(100, Effect::Damage, 30),
            make_item_data(200, Effect::Damage, 20),
        ]);
        let equips = EquipmentSlots::from([
            (EquipmentSlot::LeftWeapon, Item::from(100u16)),
            (EquipmentSlot::RightWeapon, Item::from(200u16)),
        ]);
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(0, 0);

        assert_eq!(result.score.damage, 55); // 5 base + 30 + 20
    }

    #[test]
    fn apply_equipment_empty_is_noop() {
        let db = ItemDatabase::default();
        let equips = EquipmentSlots::default();
        // level 10 mortal TK: damage = 15, max_hp = 110
        let base = Score {
            level: 10,
            ..Default::default()
        };

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(110, 0);

        assert_eq!(result.score.damage, 15);
        assert_eq!(result.score.max_hp, 110);
    }

    #[test]
    fn apply_equipment_combines_template_and_runtime_effects() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let item = Item::from((100u16, Effect::Damage as u8, 10u8));
        let equips = EquipmentSlots::from([(EquipmentSlot::LeftWeapon, item)]);
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(0, 0);

        assert_eq!(result.score.damage, 65); // 5 base + 50 template + 10 runtime
    }

    #[test]
    fn full_pipeline_with_equipment() {
        let db = ItemDatabase::from_items([make_item_data(100, Effect::Damage, 50)]);
        let equips = EquipmentSlots::from([(EquipmentSlot::LeftWeapon, Item::from(100u16))]);
        // level 10 mortal TK: base damage = 15, max_hp = 110
        let base = Score {
            level: 10,
            ..Default::default()
        };

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(100, 0);

        assert_eq!(result.score.damage, 65); // 15 base + 50 equip
        assert_eq!(result.score.max_hp, 110);
        assert_eq!(result.score.hp, 100);
    }

    #[test]
    fn full_pipeline_clamps_hp_to_new_max() {
        let db = ItemDatabase::default();
        let equips = EquipmentSlots::default();
        // level 0 mortal TK: max_hp = 80
        let base = Score::default();

        let result = StatBuilder::from_base(&base, Class::TransKnight, E, &db)
            .apply_equipment(&equips)
            .finalize(9999, 0);

        assert_eq!(result.score.hp, 80);
    }
}
