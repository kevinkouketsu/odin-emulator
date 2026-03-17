use crate::map::{EntityId, InsertResult, Map, MapError, RemoveResult};
use crate::score::calculate_score;
use crate::services::equipments::Equipments;
use crate::services::inventory::Inventory;
use odin_models::character::Character;
use odin_models::character::{Class, Evolution, GuildLevel};
use odin_models::item_data::ItemDatabase;
use odin_models::position::Position;
use odin_models::status::Score;
use odin_models::uuid::Uuid;
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

    pub fn recalculate_score(&mut self, entity_id: EntityId) {
        let Some(mob) = self.entities.get_mut(&entity_id) else {
            return;
        };
        match mob {
            Mob::Player(player) => {
                player.current_score = calculate_score(
                    &player.score,
                    player.current_score.hp,
                    player.current_score.mp,
                    &player.equipments,
                    &self.item_db,
                );
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
    pub inventory: Inventory,
    pub equipments: Equipments,
    pub current_score: Score,
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
            inventory: Inventory::from(character.inventory),
            equipments: Equipments::from(character.equipments),
            current_score: Score {
                hp,
                mp,
                ..Default::default()
            },
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn revive(&mut self) -> bool {
        if self.current_score.hp > 0 {
            return false;
        }

        self.current_score.hp = 2;
        self.current_score.mp = 2.max(self.current_score.mp);
        true
    }

    pub fn current_score(&self) -> &Score {
        &self.current_score
    }
}

impl From<&Player> for Character {
    fn from(player: &Player) -> Self {
        Character {
            identifier: player.identifier,
            name: player.name.clone(),
            slot: player.slot,
            score: player.score,
            evolution: player.evolution,
            merchant: player.merchant,
            guild: player.guild,
            guild_level: player.guild_level,
            class: player.class,
            affect_info: player.affect_info,
            quest_info: player.quest_info,
            coin: player.coin,
            experience: player.experience,
            last_pos: player.last_pos,
            inventory: player.inventory.iter().map(|(i, item)| (i, *item)).collect(),
            equipments: player.equipments.iter().map(|(s, item)| (s, *item)).collect(),
        }
    }
}
