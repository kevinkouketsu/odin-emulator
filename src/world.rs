use crate::map::{EntityId, InsertResult, Map, MapError, RemoveResult};
use crate::score::base::{base_class_stats, master_points, score_points};
use crate::score::{ComputedScore, StatBuilder};
use odin_models::character::Character;
use odin_models::character::{Class, Evolution, GuildLevel};
use odin_models::item_data::ItemDatabase;
use odin_models::position::Position;
use odin_models::status::Score;
use odin_models::uuid::Uuid;
use odin_models::{EquipmentSlots, InventorySlots};
use std::collections::HashMap;

pub struct World {
    map: Map,
    entities: HashMap<EntityId, Mob>,
    item_db: ItemDatabase,
}

impl World {
    pub fn new(item_db: ItemDatabase) -> Self {
        Self {
            map: Map::new(),
            entities: HashMap::new(),
            item_db,
        }
    }

    pub fn item_db(&self) -> &ItemDatabase {
        &self.item_db
    }

    pub fn recalculate_score(&mut self, entity_id: EntityId) -> bool {
        let Some(mob) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        match mob {
            Mob::Player(player) => {
                let hp = player.computed.score.hp;
                let mp = player.computed.score.mp;
                let new_computed = StatBuilder::from_base(
                    &player.score,
                    player.class,
                    player.evolution,
                    &self.item_db,
                )
                .apply_equipment(&player.equipments)
                .finalize(hp, mp);
                let changed = player.computed != new_computed;
                player.computed = new_computed;
                changed
            }
        }
    }

    pub fn add_player(
        &mut self,
        entity_id: EntityId,
        player: Player,
        position: Position,
    ) -> Result<InsertResult, MapError> {
        let result = self.map.force_insert(entity_id, position)?;
        self.entities.insert(entity_id, Mob::Player(player));
        Ok(result)
    }

    pub fn remove_entity(&mut self, id: EntityId) -> Result<RemoveResult, MapError> {
        let result = self.map.remove(id)?;
        self.entities.remove(&id);
        Ok(result)
    }

    pub fn get_mob(&self, id: EntityId) -> Option<&Mob> {
        self.entities.get(&id)
    }

    pub fn get_mob_mut(&mut self, id: EntityId) -> Option<&mut Mob> {
        self.entities.get_mut(&id)
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut Map {
        &mut self.map
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new(ItemDatabase::default())
    }
}

pub enum Mob {
    Player(Player),
}
impl Mob {
    pub fn entity_id(&self) -> EntityId {
        match self {
            Mob::Player(player) => player.entity_id(),
        }
    }

    pub fn revive(&mut self) -> bool {
        match self {
            Mob::Player(player) => player.revive(),
        }
    }
}

pub struct Player {
    pub entity_id: EntityId,
    pub identifier: Uuid,
    pub name: String,
    pub slot: i32,
    pub score: Score,
    pub evolution: Evolution,
    pub merchant: i16,
    pub guild: Option<i16>,
    pub guild_level: Option<GuildLevel>,
    pub class: Class,
    pub affect_info: i16,
    pub quest_info: i16,
    pub coin: i32,
    pub experience: i64,
    pub last_pos: Position,
    pub inventory: InventorySlots,
    pub equipments: EquipmentSlots,
    pub computed: ComputedScore,
    pub score_bonus: i16,
    pub special_bonus: i16,
    pub skill_bonus: i16,
}

impl Player {
    pub fn from_character(entity_id: EntityId, character: Character) -> Self {
        let hp = character.score.hp;
        let mp = character.score.mp;
        Self {
            entity_id,
            identifier: character.identifier,
            name: character.name,
            slot: character.slot,
            score: character.score,
            evolution: character.evolution,
            merchant: character.merchant,
            guild: character.guild,
            guild_level: character.guild_level,
            class: character.class,
            affect_info: character.affect_info,
            quest_info: character.quest_info,
            coin: character.coin,
            experience: character.experience,
            last_pos: character.last_pos,
            inventory: character.inventory,
            equipments: character.equipments,
            computed: ComputedScore {
                score: Score {
                    hp,
                    mp,
                    ..Default::default()
                },
                ..Default::default()
            },
            score_bonus: 0,
            special_bonus: 0,
            skill_bonus: 0,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn revive(&mut self) -> bool {
        if self.computed.score.hp > 0 {
            return false;
        }

        self.computed.score.hp = 2;
        self.computed.score.mp = 2.max(self.computed.score.mp);
        true
    }

    pub fn current_score(&self) -> &Score {
        &self.computed.score
    }

    pub fn calculate_bonus_points(&mut self) {
        let (base_str, base_int, base_dex, base_con) = base_class_stats(self.class);

        let total_score = score_points(self.score.level, self.evolution);
        let spent_score = (self.score.strength as i32 - base_str)
            + (self.score.intelligence as i32 - base_int)
            + (self.score.dexterity as i32 - base_dex)
            + (self.score.constitution as i32 - base_con);
        self.score_bonus = (total_score - spent_score) as i16;

        let total_master = master_points(self.score.level, self.evolution);
        let spent_master: i32 = self.score.specials.iter().map(|&s| s as i32).sum();
        self.special_bonus = (total_master - spent_master) as i16;
    }
}
