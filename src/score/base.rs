use odin_models::{
    character::{Class, Evolution},
    status::Score,
};

struct ClassStats {
    base_hp: i32,
    base_mp: i32,
}

#[rustfmt::skip]
const CLASS_STATS: [ClassStats; 4] = [
    ClassStats { base_hp: 80, base_mp: 45 }, // TransKnight
    ClassStats { base_hp: 60, base_mp: 65 }, // Foema
    ClassStats { base_hp: 70, base_mp: 55 }, // BeastMaster
    ClassStats { base_hp: 75, base_mp: 60 }, // Huntress
];

struct HpMpPerLevel {
    hp: i32,
    mp: i32,
}

const HP_MP_PER_LEVEL: [HpMpPerLevel; 4] = [
    HpMpPerLevel { hp: 3, mp: 1 },
    HpMpPerLevel { hp: 1, mp: 3 },
    HpMpPerLevel { hp: 1, mp: 2 },
    HpMpPerLevel { hp: 2, mp: 1 },
];

pub fn score_points(level: u16, evolution: Evolution) -> i32 {
    let level = level as i32;
    match evolution {
        Evolution::Mortal => {
            if level < 255 {
                level * 5
            } else if level < 301 {
                1270 + (level - 254) * 10
            } else if level < 354 {
                1270 + 450 + (level - 300) * 20
            } else {
                1270 + 450 + 1080 + (level - 354) * 12
            }
        }
        _ => 0, // TODO: Arch, Celestial, SubCelestial
    }
}

pub fn master_points(level: u16, evolution: Evolution) -> i32 {
    let level = level as i32;
    match evolution {
        Evolution::Mortal => level * 2,
        _ => 0, // TODO: Arch, Celestial, SubCelestial
    }
}

pub(super) fn calculate_base_score(score: &Score, class: Class, evolution: Evolution) -> Score {
    let class_index = i32::from(class) as usize;
    let stats = &CLASS_STATS[class_index];
    let hp_mp = &HP_MP_PER_LEVEL[class_index];

    let level = score.level as i32;
    let eff_level = effective_level(level, evolution);
    let mult = evolution_multiplier(evolution);

    let damage = (5 + mult * eff_level) as u32;
    let defense = (4 + mult * eff_level) as u32;
    let max_hp = (hp_mp.hp * eff_level + stats.base_hp) as u32;
    let max_mp = (hp_mp.mp * eff_level + stats.base_mp) as u32;

    Score {
        damage,
        defense,
        max_hp,
        max_mp,
        ..*score
    }
}

fn evolution_multiplier(evolution: Evolution) -> i32 {
    match evolution {
        Evolution::Mortal => 1,
        Evolution::Arch => 2,
        Evolution::Celestial | Evolution::SubCelestial => 3,
    }
}

fn effective_level(level: i32, evolution: Evolution) -> i32 {
    if evolution >= Evolution::Celestial {
        level + 400
    } else {
        level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mortal_transknight_level_1() {
        let score = Score {
            level: 1,
            ..Default::default()
        };
        let result = calculate_base_score(&score, Class::TransKnight, Evolution::Mortal);

        // mult=1, eff_level=1 → damage = 5 + 1*1 = 6
        assert_eq!(result.damage, 6);
        // defense = 4 + 1*1 = 5
        assert_eq!(result.defense, 5);
        // hp = 3*1 + 80 = 83
        assert_eq!(result.max_hp, 83);
        // mp = 1*1 + 45 = 46
        assert_eq!(result.max_mp, 46);
    }

    #[test]
    fn mortal_foema_level_100() {
        let score = Score {
            level: 100,
            ..Default::default()
        };
        let result = calculate_base_score(&score, Class::Foema, Evolution::Mortal);

        // mult=1, eff_level=100 → damage = 5 + 100 = 105
        assert_eq!(result.damage, 105);
        assert_eq!(result.defense, 104);
        // hp = 1*100 + 60 = 160
        assert_eq!(result.max_hp, 160);
        // mp = 3*100 + 65 = 365
        assert_eq!(result.max_mp, 365);
    }

    #[test]
    fn arch_beastmaster_level_200() {
        let score = Score {
            level: 200,
            ..Default::default()
        };
        let result = calculate_base_score(&score, Class::BeastMaster, Evolution::Arch);

        // mult=2, eff_level=200 → damage = 5 + 2*200 = 405
        assert_eq!(result.damage, 405);
        assert_eq!(result.defense, 404);
        // hp = 1*200 + 70 = 270
        assert_eq!(result.max_hp, 270);
        // mp = 2*200 + 55 = 455
        assert_eq!(result.max_mp, 455);
    }

    #[test]
    fn celestial_huntress_level_300() {
        let score = Score {
            level: 300,
            ..Default::default()
        };
        let result = calculate_base_score(&score, Class::Huntress, Evolution::Celestial);

        // mult=3, eff_level=300+400=700 → damage = 5 + 3*700 = 2105
        assert_eq!(result.damage, 2105);
        assert_eq!(result.defense, 2104);
        // hp = 2*700 + 75 = 1475
        assert_eq!(result.max_hp, 1475);
        // mp = 1*700 + 60 = 760
        assert_eq!(result.max_mp, 760);
    }

    #[test]
    fn subcelestial_same_as_celestial() {
        let score = Score {
            level: 100,
            ..Default::default()
        };
        let celestial = calculate_base_score(&score, Class::TransKnight, Evolution::Celestial);
        let sub = calculate_base_score(&score, Class::TransKnight, Evolution::SubCelestial);

        assert_eq!(celestial.damage, sub.damage);
        assert_eq!(celestial.defense, sub.defense);
        assert_eq!(celestial.max_hp, sub.max_hp);
        assert_eq!(celestial.max_mp, sub.max_mp);
    }

    #[test]
    fn preserves_other_score_fields() {
        let score = Score {
            level: 10,
            strength: 50,
            intelligence: 30,
            dexterity: 20,
            constitution: 40,
            specials: [1, 2, 3, 4],
            hp: 999,
            mp: 888,
            ..Default::default()
        };
        let result = calculate_base_score(&score, Class::TransKnight, Evolution::Mortal);

        assert_eq!(result.level, 10);
        assert_eq!(result.strength, 50);
        assert_eq!(result.intelligence, 30);
        assert_eq!(result.dexterity, 20);
        assert_eq!(result.constitution, 40);
        assert_eq!(result.specials, [1, 2, 3, 4]);
        assert_eq!(result.hp, 999);
        assert_eq!(result.mp, 888);
    }

    #[test]
    fn score_points_mortal_low_level() {
        assert_eq!(score_points(0, Evolution::Mortal), 0);
        assert_eq!(score_points(1, Evolution::Mortal), 5);
        assert_eq!(score_points(100, Evolution::Mortal), 500);
        assert_eq!(score_points(254, Evolution::Mortal), 1270);
    }

    #[test]
    fn score_points_mortal_tier_boundaries() {
        // tier 2: 255-300
        assert_eq!(score_points(255, Evolution::Mortal), 1270 + 10);
        assert_eq!(score_points(300, Evolution::Mortal), 1270 + 460);
        // tier 3: 301-353
        assert_eq!(score_points(301, Evolution::Mortal), 1270 + 450 + 20);
        assert_eq!(score_points(353, Evolution::Mortal), 1270 + 450 + 1060);
        // tier 4: 354+
        assert_eq!(score_points(354, Evolution::Mortal), 1270 + 450 + 1080);
        assert_eq!(
            score_points(400, Evolution::Mortal),
            1270 + 450 + 1080 + 46 * 12
        );
    }

    #[test]
    fn master_points_mortal() {
        assert_eq!(master_points(0, Evolution::Mortal), 0);
        assert_eq!(master_points(1, Evolution::Mortal), 2);
        assert_eq!(master_points(100, Evolution::Mortal), 200);
        assert_eq!(master_points(400, Evolution::Mortal), 800);
    }

    #[test]
    fn level_zero_gets_base_stats_only() {
        let score = Score {
            level: 0,
            ..Default::default()
        };
        let result = calculate_base_score(&score, Class::Foema, Evolution::Mortal);

        assert_eq!(result.damage, 5);
        assert_eq!(result.defense, 4);
        assert_eq!(result.max_hp, 60);
        assert_eq!(result.max_mp, 65);
    }
}
