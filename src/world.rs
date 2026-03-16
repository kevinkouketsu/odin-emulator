use crate::map::{EntityId, InsertResult, Map, MapError, RemoveResult};
use crate::score::calculate_score;
use crate::services::equipments::Equipments;
use odin_models::{
    character::Character, item_data::ItemDatabase, position::Position, status::Score,
};
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
                let equipments = Equipments::from(player.base_character.equipments.clone());
                player.current_score = calculate_score(
                    &player.base_character.score,
                    player.current_score.hp,
                    player.current_score.mp,
                    &equipments,
                    &self.item_db,
                );

                log::debug!(
                    "Recalculated score for player {:?}: {:?}",
                    entity_id,
                    player.current_score
                );
            }
        }
    }

    pub fn add_player(
        &mut self,
        client_id: usize,
        character: Player,
        position: Position,
    ) -> Result<InsertResult, MapError> {
        let entity_id = EntityId::Player(client_id);
        let result = self.map.force_insert(entity_id, position)?;
        self.entities.insert(entity_id, Mob::Player(character));
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
    pub fn revive(&mut self) -> bool {
        match self {
            Mob::Player(player) => player.revive(),
        }
    }
}

#[derive(Default)]
pub struct Player {
    pub base_character: Character,
    pub current_score: Score,
}
impl Player {
    pub fn base_character(&self) -> &Character {
        &self.base_character
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
