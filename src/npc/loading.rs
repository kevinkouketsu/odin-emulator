use crate::npc::spawn_group::{Formation, RouteType, SpawnGroupConfig, WaypointConfig};
use odin_models::character::Class;
use odin_models::item::{Item, ItemBonusEffect};
use odin_models::npc_mob::NpcMob;
use odin_models::position::Position;
use odin_models::status::Score;
use odin_models::{EquipmentSlot, EquipmentSlots, InventorySlots, MAX_ITEM_EFFECT};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error in {file}: {source}")]
    TomlParse {
        file: String,
        source: toml::de::Error,
    },
    #[error("Invalid equipment slot: {0}")]
    InvalidSlot(String),
    #[error("Invalid class: {0}")]
    InvalidClass(String),
    #[error("Invalid route type: {0}")]
    InvalidRouteType(String),
    #[error("Invalid formation: {0}")]
    InvalidFormation(String),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Too many item effects (max {max}): got {got}")]
    TooManyEffects { max: usize, got: usize },
}

#[derive(Deserialize)]
pub struct MobTemplateToml {
    pub name: String,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub clan: i8,
    #[serde(default)]
    pub merchant: i16,
    #[serde(default)]
    pub guild: Option<i16>,
    #[serde(default)]
    pub affect_info: i16,
    #[serde(default)]
    pub quest_info: i16,
    #[serde(default)]
    pub coin: i32,
    #[serde(default)]
    pub experience: i64,
    #[serde(default)]
    pub score: ScoreToml,
    #[serde(default)]
    pub equipment: Vec<EquipmentEntryToml>,
    #[serde(default)]
    pub inventory: Vec<InventoryEntryToml>,
}

impl MobTemplateToml {
    pub fn into_npc_mob(self) -> Result<NpcMob, LoadError> {
        let class = match &self.class {
            Some(s) => parse_class(s)?,
            None => Class::default(),
        };

        let score = Score {
            level: self.score.level,
            defense: self.score.defense,
            damage: self.score.damage,
            reserved: self.score.reserved,
            attack_run: self.score.speed as i8,
            max_hp: self.score.max_hp,
            max_mp: self.score.max_mp,
            hp: self.score.hp,
            mp: self.score.mp,
            strength: self.score.strength,
            intelligence: self.score.intelligence,
            dexterity: self.score.dexterity,
            constitution: self.score.constitution,
            specials: self.score.specials,
        };

        let mut equipments = EquipmentSlots::default();
        for entry in &self.equipment {
            let slot = parse_equipment_slot(&entry.slot)?;
            let effects = convert_item_effects(&entry.effects)?;
            equipments.set(
                slot,
                Item {
                    id: entry.id,
                    effects,
                },
            );
        }

        let mut inventory = InventorySlots::default();
        for entry in &self.inventory {
            let effects = convert_item_effects(&entry.effects)?;
            inventory.set(
                entry.slot,
                Item {
                    id: entry.id,
                    effects,
                },
            );
        }

        Ok(NpcMob {
            name: self.name,
            clan: self.clan,
            merchant: self.merchant,
            guild: self.guild,
            guild_level: None,
            class,
            affect_info: self.affect_info,
            quest_info: self.quest_info,
            coin: self.coin,
            experience: self.experience,
            score,
            equipments,
            inventory,
        })
    }
}

#[derive(Deserialize, Default)]
pub struct ScoreToml {
    #[serde(default)]
    pub level: u16,
    #[serde(default)]
    pub defense: u32,
    #[serde(default)]
    pub damage: u32,
    #[serde(default)]
    pub reserved: i8,
    #[serde(default)]
    pub speed: u8,
    #[serde(default)]
    pub max_hp: u32,
    #[serde(default)]
    pub max_mp: u32,
    #[serde(default)]
    pub hp: u32,
    #[serde(default)]
    pub mp: u32,
    #[serde(default)]
    pub strength: u16,
    #[serde(default)]
    pub intelligence: u16,
    #[serde(default)]
    pub dexterity: u16,
    #[serde(default)]
    pub constitution: u16,
    #[serde(default)]
    pub specials: [u16; 4],
}

#[derive(Deserialize)]
pub struct EquipmentEntryToml {
    pub slot: String,
    pub id: u16,
    #[serde(default)]
    pub effects: Vec<ItemEffectToml>,
}

#[derive(Deserialize)]
pub struct ItemEffectToml {
    pub r#type: u8,
    pub value: u8,
}

#[derive(Deserialize)]
pub struct InventoryEntryToml {
    pub slot: usize,
    pub id: u16,
    #[serde(default)]
    pub effects: Vec<ItemEffectToml>,
}

#[derive(Deserialize)]
pub struct SpawnFileToml {
    pub group: Vec<SpawnGroupToml>,
}

#[derive(Deserialize)]
pub struct SpawnGroupToml {
    pub leader: String,
    #[serde(default)]
    pub follower: Option<String>,
    #[serde(default)]
    pub min_group: u32,
    #[serde(default)]
    pub max_group: u32,
    #[serde(default = "default_max_alive")]
    pub max_alive: u32,
    #[serde(default)]
    pub respawn_ticks: u32,
    pub route_type: RouteTypeToml,
    #[serde(default)]
    pub formation: Option<String>,
    #[serde(default)]
    pub waypoints: Vec<WaypointToml>,
}

fn default_max_alive() -> u32 {
    1
}

impl SpawnGroupToml {
    pub fn into_config(
        self,
        templates: &HashMap<String, NpcMob>,
    ) -> Result<SpawnGroupConfig, LoadError> {
        let leader_template = templates
            .get(&self.leader)
            .ok_or_else(|| LoadError::TemplateNotFound(self.leader.clone()))?
            .clone();

        let follower_template = match &self.follower {
            Some(name) => Some(
                templates
                    .get(name)
                    .ok_or_else(|| LoadError::TemplateNotFound(name.clone()))?
                    .clone(),
            ),
            None => None,
        };

        let route_type = parse_route_type(&self.route_type)?;
        let formation = match &self.formation {
            Some(s) => parse_formation(s)?,
            None => Formation::None,
        };

        let waypoints = self
            .waypoints
            .into_iter()
            .map(|w| WaypointConfig {
                position: Position { x: w.x, y: w.y },
                range: w.range,
                wait_ticks: w.wait_ticks,
            })
            .collect();

        Ok(SpawnGroupConfig {
            leader_template,
            follower_template,
            min_group: self.min_group,
            max_group: self.max_group,
            formation,
            route_type,
            waypoints,
            respawn_ticks: self.respawn_ticks,
            max_alive: self.max_alive,
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RouteTypeToml {
    Simple(String),
    Random { random: RandomConfig },
}

#[derive(Deserialize)]
pub struct RandomConfig {
    pub radius: u16,
}

#[derive(Deserialize)]
pub struct WaypointToml {
    pub x: u16,
    pub y: u16,
    #[serde(default)]
    pub range: u16,
    #[serde(default)]
    pub wait_ticks: u32,
}

fn parse_equipment_slot(s: &str) -> Result<EquipmentSlot, LoadError> {
    match s {
        "face" => Ok(EquipmentSlot::Face),
        "helmet" => Ok(EquipmentSlot::Helmet),
        "armor" => Ok(EquipmentSlot::Armor),
        "armor_pants" => Ok(EquipmentSlot::ArmorPants),
        "gloves" => Ok(EquipmentSlot::Gloves),
        "boots" => Ok(EquipmentSlot::Boots),
        "left_weapon" => Ok(EquipmentSlot::LeftWeapon),
        "right_weapon" => Ok(EquipmentSlot::RightWeapon),
        "amulet1" => Ok(EquipmentSlot::Amulet1),
        "amulet2" => Ok(EquipmentSlot::Amulet2),
        "amulet3" => Ok(EquipmentSlot::Amulet3),
        "amulet4" => Ok(EquipmentSlot::Amulet4),
        "familiar" => Ok(EquipmentSlot::Familiar),
        "costume" => Ok(EquipmentSlot::Costume),
        "mount" => Ok(EquipmentSlot::Mount),
        "mantle" => Ok(EquipmentSlot::Mantle),
        "reserved1" => Ok(EquipmentSlot::Reserved1),
        "reserved2" => Ok(EquipmentSlot::Reserved2),
        _ => Err(LoadError::InvalidSlot(s.to_string())),
    }
}

fn parse_class(s: &str) -> Result<Class, LoadError> {
    match s {
        "trans_knight" | "transknight" => Ok(Class::TransKnight),
        "foema" => Ok(Class::Foema),
        "beast_master" | "beastmaster" => Ok(Class::BeastMaster),
        "huntress" => Ok(Class::Huntress),
        _ => Err(LoadError::InvalidClass(s.to_string())),
    }
}

fn parse_formation(s: &str) -> Result<Formation, LoadError> {
    match s {
        "none" => Ok(Formation::None),
        "line" => Ok(Formation::Line),
        "wedge" => Ok(Formation::Wedge),
        "ring" => Ok(Formation::Ring),
        "cross" => Ok(Formation::Cross),
        _ => Err(LoadError::InvalidFormation(s.to_string())),
    }
}

fn parse_route_type(rt: &RouteTypeToml) -> Result<RouteType, LoadError> {
    match rt {
        RouteTypeToml::Simple(s) => match s.as_str() {
            "stationary" => Ok(RouteType::Stationary),
            "walk_to_end" => Ok(RouteType::WalkToEnd),
            "walk_and_despawn" => Ok(RouteType::WalkAndDespawn),
            "ping_pong" => Ok(RouteType::PingPong),
            "ping_pong_despawn" => Ok(RouteType::PingPongDespawn),
            "loop" => Ok(RouteType::Loop),
            _ => Err(LoadError::InvalidRouteType(s.clone())),
        },
        RouteTypeToml::Random { random } => Ok(RouteType::Random {
            radius: random.radius,
        }),
    }
}

fn convert_item_effects(
    effects: &[ItemEffectToml],
) -> Result<[ItemBonusEffect; MAX_ITEM_EFFECT], LoadError> {
    if effects.len() > MAX_ITEM_EFFECT {
        return Err(LoadError::TooManyEffects {
            max: MAX_ITEM_EFFECT,
            got: effects.len(),
        });
    }
    let mut result = [ItemBonusEffect::default(); MAX_ITEM_EFFECT];
    for (i, e) in effects.iter().enumerate() {
        result[i] = ItemBonusEffect {
            index: e.r#type,
            value: e.value,
        };
    }
    Ok(result)
}

pub fn load_mob_templates(dir: &Path) -> Result<HashMap<String, NpcMob>, LoadError> {
    let mut templates = HashMap::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let contents = std::fs::read_to_string(&path)?;
            let template: MobTemplateToml =
                toml::from_str(&contents).map_err(|e| LoadError::TomlParse {
                    file: path.display().to_string(),
                    source: e,
                })?;
            let npc_mob = template.into_npc_mob()?;
            templates.insert(stem, npc_mob);
        }
    }
    Ok(templates)
}

pub fn load_spawn_groups(
    dir: &Path,
    templates: &HashMap<String, NpcMob>,
) -> Result<Vec<SpawnGroupConfig>, LoadError> {
    let mut configs = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let contents = std::fs::read_to_string(&path)?;
            let spawn_file: SpawnFileToml =
                toml::from_str(&contents).map_err(|e| LoadError::TomlParse {
                    file: path.display().to_string(),
                    source: e,
                })?;
            for group in spawn_file.group {
                let leader_name = group.leader.clone();
                let Ok(config) = group.into_config(templates) else {
                    log::error!("Failed to load spawn group from {}", leader_name);

                    continue;
                };

                configs.push(config);
            }
        }
    }
    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_minimal_mob_template() {
        let toml_str = r#"
            name = "TestMob"

            [score]
            level = 10
        "#;
        let template: MobTemplateToml = toml::from_str(toml_str).unwrap();
        let mob = template.into_npc_mob().unwrap();
        assert_eq!(mob.name, "TestMob");
        assert_eq!(mob.score.level, 10);
        assert_eq!(mob.score.hp, 0);
        assert_eq!(mob.clan, 0);
        assert_eq!(mob.coin, 0);
        assert_eq!(mob.experience, 0);
        assert!(mob.equipments.iter().next().is_none());
    }

    #[test]
    fn parse_full_mob_template() {
        let toml_str = r#"
            name = "HellAmonChief"
            class = "trans_knight"
            clan = 3
            merchant = 1
            guild = 100
            affect_info = 5
            quest_info = 7
            coin = 5000
            experience = 50000

            [score]
            level = 85
            defense = 200
            damage = 300
            reserved = 1
            speed = 3
            max_hp = 12000
            max_mp = 500
            hp = 12000
            mp = 500
            strength = 150
            intelligence = 50
            dexterity = 80
            constitution = 120
            specials = [10, 20, 30, 40]

            [[equipment]]
            slot = "armor"
            id = 1506
            effects = [{ type = 2, value = 30 }, { type = 5, value = 10 }]

            [[equipment]]
            slot = "helmet"
            id = 1200

            [[inventory]]
            slot = 0
            id = 3000
            effects = [{ type = 1, value = 50 }]
        "#;
        let template: MobTemplateToml = toml::from_str(toml_str).unwrap();
        let mob = template.into_npc_mob().unwrap();

        assert_eq!(mob.name, "HellAmonChief");
        assert_eq!(mob.clan, 3);
        assert_eq!(mob.merchant, 1);
        assert_eq!(mob.guild, Some(100));
        assert_eq!(mob.affect_info, 5);
        assert_eq!(mob.quest_info, 7);
        assert_eq!(mob.coin, 5000);
        assert_eq!(mob.experience, 50000);

        assert_eq!(mob.score.level, 85);
        assert_eq!(mob.score.defense, 200);
        assert_eq!(mob.score.damage, 300);
        assert_eq!(mob.score.reserved, 1);
        assert_eq!(mob.score.attack_run, 3);
        assert_eq!(mob.score.max_hp, 12000);
        assert_eq!(mob.score.max_mp, 500);
        assert_eq!(mob.score.hp, 12000);
        assert_eq!(mob.score.mp, 500);
        assert_eq!(mob.score.strength, 150);
        assert_eq!(mob.score.intelligence, 50);
        assert_eq!(mob.score.dexterity, 80);
        assert_eq!(mob.score.constitution, 120);
        assert_eq!(mob.score.specials, [10, 20, 30, 40]);

        let armor = mob.equipments.get(EquipmentSlot::Armor).unwrap();
        assert_eq!(armor.id, 1506);
        assert_eq!(
            armor.effects[0],
            ItemBonusEffect {
                index: 2,
                value: 30
            }
        );
        assert_eq!(
            armor.effects[1],
            ItemBonusEffect {
                index: 5,
                value: 10
            }
        );
        assert_eq!(armor.effects[2], ItemBonusEffect::default());

        let helmet = mob.equipments.get(EquipmentSlot::Helmet).unwrap();
        assert_eq!(helmet.id, 1200);
        assert_eq!(
            helmet.effects,
            [ItemBonusEffect::default(); MAX_ITEM_EFFECT]
        );

        let inv_item = mob.inventory.get(0).unwrap();
        assert_eq!(inv_item.id, 3000);
        assert_eq!(
            inv_item.effects[0],
            ItemBonusEffect {
                index: 1,
                value: 50
            }
        );
    }

    #[test]
    fn parse_equipment_slots_all_variants() {
        let slots = [
            ("face", EquipmentSlot::Face),
            ("helmet", EquipmentSlot::Helmet),
            ("armor", EquipmentSlot::Armor),
            ("armor_pants", EquipmentSlot::ArmorPants),
            ("gloves", EquipmentSlot::Gloves),
            ("boots", EquipmentSlot::Boots),
            ("left_weapon", EquipmentSlot::LeftWeapon),
            ("right_weapon", EquipmentSlot::RightWeapon),
            ("amulet1", EquipmentSlot::Amulet1),
            ("amulet2", EquipmentSlot::Amulet2),
            ("amulet3", EquipmentSlot::Amulet3),
            ("amulet4", EquipmentSlot::Amulet4),
            ("familiar", EquipmentSlot::Familiar),
            ("costume", EquipmentSlot::Costume),
            ("mount", EquipmentSlot::Mount),
            ("mantle", EquipmentSlot::Mantle),
            ("reserved1", EquipmentSlot::Reserved1),
            ("reserved2", EquipmentSlot::Reserved2),
        ];
        for (name, expected) in slots {
            assert_eq!(
                parse_equipment_slot(name).unwrap(),
                expected,
                "failed for {name}"
            );
        }
    }

    #[test]
    fn parse_class_variants() {
        assert!(matches!(
            parse_class("trans_knight").unwrap(),
            Class::TransKnight
        ));
        assert!(matches!(
            parse_class("transknight").unwrap(),
            Class::TransKnight
        ));
        assert!(matches!(parse_class("foema").unwrap(), Class::Foema));
        assert!(matches!(
            parse_class("beast_master").unwrap(),
            Class::BeastMaster
        ));
        assert!(matches!(
            parse_class("beastmaster").unwrap(),
            Class::BeastMaster
        ));
        assert!(matches!(parse_class("huntress").unwrap(), Class::Huntress));
    }

    #[test]
    fn parse_route_type_strings() {
        let cases = [
            ("stationary", RouteType::Stationary),
            ("walk_to_end", RouteType::WalkToEnd),
            ("walk_and_despawn", RouteType::WalkAndDespawn),
            ("ping_pong", RouteType::PingPong),
            ("ping_pong_despawn", RouteType::PingPongDespawn),
            ("loop", RouteType::Loop),
        ];
        for (name, expected) in cases {
            let rt = RouteTypeToml::Simple(name.to_string());
            assert_eq!(
                parse_route_type(&rt).unwrap(),
                expected,
                "failed for {name}"
            );
        }
    }

    #[test]
    fn parse_route_type_random() {
        let toml_str = r#"
            [[group]]
            leader = "Mob"
            route_type = { random = { radius = 15 } }

            [[group.waypoints]]
            x = 100
            y = 200
        "#;
        let spawn_file: SpawnFileToml = toml::from_str(toml_str).unwrap();
        let rt = &spawn_file.group[0].route_type;
        assert_eq!(
            parse_route_type(rt).unwrap(),
            RouteType::Random { radius: 15 }
        );
    }

    #[test]
    fn parse_spawn_group_resolves_templates() {
        let toml_str = r#"
            [[group]]
            leader = "Chief"
            follower = "Minion"
            min_group = 3
            max_group = 5
            max_alive = 5
            respawn_ticks = 120
            route_type = "ping_pong"
            formation = "line"

            [[group.waypoints]]
            x = 3414
            y = 1462
            range = 0
            wait_ticks = 0

            [[group.waypoints]]
            x = 3500
            y = 1500
            range = 5
            wait_ticks = 10
        "#;
        let spawn_file: SpawnFileToml = toml::from_str(toml_str).unwrap();

        let mut templates = HashMap::new();
        let chief = NpcMob {
            name: "Chief".to_string(),
            ..Default::default()
        };
        let minion = NpcMob {
            name: "Minion".to_string(),
            ..Default::default()
        };
        templates.insert("Chief".to_string(), chief);
        templates.insert("Minion".to_string(), minion);

        let config = spawn_file
            .group
            .into_iter()
            .next()
            .unwrap()
            .into_config(&templates)
            .unwrap();
        assert_eq!(config.leader_template.name, "Chief");
        assert_eq!(config.follower_template.as_ref().unwrap().name, "Minion");
        assert_eq!(config.min_group, 3);
        assert_eq!(config.max_group, 5);
        assert_eq!(config.max_alive, 5);
        assert_eq!(config.respawn_ticks, 120);
        assert_eq!(config.route_type, RouteType::PingPong);
        assert_eq!(config.formation, Formation::Line);
        assert_eq!(config.waypoints.len(), 2);
        assert_eq!(config.waypoints[0].position, Position { x: 3414, y: 1462 });
        assert_eq!(config.waypoints[0].range, 0);
        assert_eq!(config.waypoints[1].position, Position { x: 3500, y: 1500 });
        assert_eq!(config.waypoints[1].range, 5);
        assert_eq!(config.waypoints[1].wait_ticks, 10);
    }

    #[test]
    fn parse_spawn_group_missing_template_errors() {
        let toml_str = r#"
            [[group]]
            leader = "NonExistent"
            route_type = "stationary"
        "#;
        let spawn_file: SpawnFileToml = toml::from_str(toml_str).unwrap();
        let templates = HashMap::new();
        let err = spawn_file
            .group
            .into_iter()
            .next()
            .unwrap()
            .into_config(&templates)
            .unwrap_err();
        assert!(matches!(err, LoadError::TemplateNotFound(ref name) if name == "NonExistent"));
    }

    #[test]
    fn parse_invalid_slot_errors() {
        let result = parse_equipment_slot("invalid_slot");
        assert!(matches!(result, Err(LoadError::InvalidSlot(ref s)) if s == "invalid_slot"));
    }

    #[test]
    fn parse_too_many_effects_errors() {
        let effects = vec![
            ItemEffectToml {
                r#type: 1,
                value: 10,
            },
            ItemEffectToml {
                r#type: 2,
                value: 20,
            },
            ItemEffectToml {
                r#type: 3,
                value: 30,
            },
            ItemEffectToml {
                r#type: 4,
                value: 40,
            },
        ];
        let result = convert_item_effects(&effects);
        assert!(matches!(
            result,
            Err(LoadError::TooManyEffects { max: 3, got: 4 })
        ));
    }

    #[test]
    fn parse_spawn_group_defaults() {
        let toml_str = r#"
            [[group]]
            leader = "Mob"
            route_type = "stationary"
        "#;
        let spawn_file: SpawnFileToml = toml::from_str(toml_str).unwrap();
        let group = &spawn_file.group[0];
        assert!(group.follower.is_none());
        assert_eq!(group.min_group, 0);
        assert_eq!(group.max_group, 0);
        assert_eq!(group.max_alive, 1);
        assert_eq!(group.respawn_ticks, 0);
        assert!(group.formation.is_none());
        assert!(group.waypoints.is_empty());

        let mut templates = HashMap::new();
        templates.insert("Mob".to_string(), NpcMob::default());
        let config = spawn_file
            .group
            .into_iter()
            .next()
            .unwrap()
            .into_config(&templates)
            .unwrap();
        assert_eq!(config.formation, Formation::None);
        assert_eq!(config.max_alive, 1);
        assert!(config.follower_template.is_none());
    }
}
