use odin_models::height_map::HeightMap;
use odin_models::position::Position;
use std::collections::HashMap;

const MAP_SIZE: u16 = 4096;
const HALF_VIEWPORT_X: u16 = 16;
const HALF_VIEWPORT_Y: u16 = 16;
const SEARCH_RANGE: i32 = 5;

pub struct Map {
    grid: HashMap<(u16, u16), EntityId>,
    positions: HashMap<EntityId, Position>,
    height_map: Option<HeightMap>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
            positions: HashMap::new(),
            height_map: None,
        }
    }

    pub fn with_height_map(height_map: HeightMap) -> Self {
        Self {
            grid: HashMap::new(),
            positions: HashMap::new(),
            height_map: Some(height_map),
        }
    }

    pub fn insert(&mut self, id: EntityId, pos: Position) -> Result<InsertResult, MapError> {
        if !Self::is_in_bounds(pos) {
            return Err(MapError::OutOfBounds);
        }
        if self.positions.contains_key(&id) {
            return Err(MapError::AlreadyInserted);
        }
        if self.grid.contains_key(&(pos.x, pos.y)) {
            return Err(MapError::Occupied);
        }

        self.grid.insert((pos.x, pos.y), id);
        self.positions.insert(id, pos);

        let spectators = self.get_spectators(pos, id);
        Ok(InsertResult {
            position: pos,
            spectators,
        })
    }

    pub fn force_insert(&mut self, id: EntityId, pos: Position) -> Result<InsertResult, MapError> {
        if !Self::is_in_bounds(pos) {
            return Err(MapError::OutOfBounds);
        }

        if let Some(&existing_pos) = self.positions.get(&id) {
            if existing_pos == pos {
                let spectators = self.get_spectators(pos, id);
                return Ok(InsertResult {
                    position: pos,
                    spectators,
                });
            }
            return Err(MapError::AlreadyInserted);
        }

        if !self.grid.contains_key(&(pos.x, pos.y)) && self.is_walkable(pos) {
            return self.insert(id, pos);
        }

        if let Some(free_pos) = self.find_nearest_free(pos) {
            return self.insert(id, free_pos);
        }

        Err(MapError::NoFreePosition)
    }

    pub fn remove(&mut self, id: EntityId) -> Result<RemoveResult, MapError> {
        let pos = self.positions.remove(&id).ok_or(MapError::EntityNotFound)?;
        self.grid.remove(&(pos.x, pos.y));
        let spectators = self.get_spectators(pos, id);
        Ok(RemoveResult {
            position: pos,
            spectators,
        })
    }

    pub fn force_move_entity(
        &mut self,
        id: EntityId,
        pos: Position,
    ) -> Result<MoveResult, MapError> {
        match self.move_entity(id, pos) {
            Err(MapError::Occupied) => {
                let free = self
                    .find_nearest_free(pos)
                    .ok_or(MapError::NoFreePosition)?;
                self.move_entity(id, free)
            }
            other => other,
        }
    }

    pub fn move_entity(&mut self, id: EntityId, new_pos: Position) -> Result<MoveResult, MapError> {
        if !Self::is_in_bounds(new_pos) {
            return Err(MapError::OutOfBounds);
        }

        let old_pos = *self.positions.get(&id).ok_or(MapError::EntityNotFound)?;

        if let Some(&occupant) = self.grid.get(&(new_pos.x, new_pos.y))
            && occupant != id
        {
            return Err(MapError::Occupied);
        }

        if old_pos == new_pos {
            let stayed = self.get_spectators(old_pos, id);
            return Ok(MoveResult {
                from: old_pos,
                to: new_pos,
                entered: Vec::new(),
                exited: Vec::new(),
                stayed,
            });
        }

        let old_entities = self.get_spectators(old_pos, id);

        self.grid.remove(&(old_pos.x, old_pos.y));
        self.grid.insert((new_pos.x, new_pos.y), id);
        self.positions.insert(id, new_pos);

        let new_entities = self.get_spectators(new_pos, id);

        let mut entered = Vec::new();
        let mut stayed = Vec::new();
        for &entity in &new_entities {
            if old_entities.contains(&entity) {
                stayed.push(entity);
            } else {
                entered.push(entity);
            }
        }

        let mut exited = Vec::new();
        for &entity in &old_entities {
            if !new_entities.contains(&entity) {
                exited.push(entity);
            }
        }

        Ok(MoveResult {
            from: old_pos,
            to: new_pos,
            entered,
            exited,
            stayed,
        })
    }

    pub fn get_spectators(&self, center: Position, exclude: EntityId) -> Vec<EntityId> {
        let (min_x, min_y, max_x, max_y) = Self::viewport_bounds(center);
        let mut result = Vec::new();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if let Some(&id) = self.grid.get(&(x, y))
                    && id != exclude
                {
                    result.push(id);
                }
            }
        }
        result
    }

    pub fn get_position(&self, id: EntityId) -> Option<Position> {
        self.positions.get(&id).copied()
    }

    fn viewport_bounds(center: Position) -> (u16, u16, u16, u16) {
        let min_x = center.x.saturating_sub(HALF_VIEWPORT_X);
        let min_y = center.y.saturating_sub(HALF_VIEWPORT_Y);
        let max_x = (center.x + HALF_VIEWPORT_X).min(MAP_SIZE - 1);
        let max_y = (center.y + HALF_VIEWPORT_Y).min(MAP_SIZE - 1);
        (min_x, min_y, max_x, max_y)
    }

    pub fn is_occupied_by_other(&self, pos: Position, exclude: EntityId) -> bool {
        match self.grid.get(&(pos.x, pos.y)) {
            Some(&occupant) => occupant != exclude,
            None => false,
        }
    }

    pub fn find_nearest_free(&self, center: Position) -> Option<Position> {
        for distance in 1..=SEARCH_RANGE {
            for dy in -distance..=distance {
                for dx in -distance..=distance {
                    if dx.abs() != distance && dy.abs() != distance {
                        continue;
                    }
                    let nx = center.x as i32 + dx;
                    let ny = center.y as i32 + dy;
                    if nx < 0 || nx >= MAP_SIZE as i32 || ny < 0 || ny >= MAP_SIZE as i32 {
                        continue;
                    }
                    let pos = Position {
                        x: nx as u16,
                        y: ny as u16,
                    };
                    if !self.grid.contains_key(&(pos.x, pos.y)) && self.is_walkable(pos) {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    fn is_in_bounds(pos: Position) -> bool {
        pos.x < MAP_SIZE && pos.y < MAP_SIZE
    }

    fn is_walkable(&self, pos: Position) -> bool {
        match &self.height_map {
            Some(hm) => !hm.is_blocked(pos.x, pos.y),
            None => true,
        }
    }

    pub fn is_terrain_passable(&self, pos: Position) -> bool {
        Self::is_in_bounds(pos) && self.is_walkable(pos)
    }

    pub fn can_step(&self, from: Position, to: Position) -> bool {
        if !Self::is_in_bounds(to) {
            return false;
        }
        if self.grid.contains_key(&(to.x, to.y)) {
            return false;
        }
        match &self.height_map {
            Some(hm) => hm.can_walk(from.x, from.y, to.x, to.y),
            None => true,
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityId {
    Player(usize),
    Mob(usize),
}
impl EntityId {
    pub fn id(&self) -> usize {
        match self {
            EntityId::Player(id) | EntityId::Mob(id) => *id,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MapError {
    #[error("Position is out of map bounds")]
    OutOfBounds,
    #[error("Position is occupied by another entity")]
    Occupied,
    #[error("Entity not found on the map")]
    EntityNotFound,
    #[error("Entity is already on the map")]
    AlreadyInserted,
    #[error("No free position found nearby")]
    NoFreePosition,
}

#[derive(Debug, PartialEq)]
pub struct InsertResult {
    pub position: Position,
    pub spectators: Vec<EntityId>,
}

#[derive(Debug, PartialEq)]
pub struct RemoveResult {
    pub position: Position,
    pub spectators: Vec<EntityId>,
}

#[derive(Debug, PartialEq)]
pub struct MoveResult {
    pub from: Position,
    pub to: Position,
    pub entered: Vec<EntityId>,
    pub exited: Vec<EntityId>,
    pub stayed: Vec<EntityId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(id: usize) -> EntityId {
        EntityId::Player(id)
    }

    fn mob(id: usize) -> EntityId {
        EntityId::Mob(id)
    }

    fn pos(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    #[test]
    fn insert_at_valid_position() {
        let mut map = Map::new();
        let result = map.insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(result.position, pos(100, 100));
        assert!(result.spectators.is_empty());
    }

    #[test]
    fn insert_returns_nearby_spectators() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let result = map.insert(player(2), pos(110, 110)).unwrap();
        assert_eq!(result.spectators, vec![player(1)]);
    }

    #[test]
    fn insert_spectators_excludes_self() {
        let mut map = Map::new();
        let result = map.insert(player(1), pos(100, 100)).unwrap();
        assert!(!result.spectators.contains(&player(1)));
    }

    #[test]
    fn insert_at_occupied_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(
            map.insert(player(2), pos(100, 100)),
            Err(MapError::Occupied)
        );
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut map = Map::new();
        assert_eq!(
            map.insert(player(1), pos(4096, 0)),
            Err(MapError::OutOfBounds)
        );
        assert_eq!(
            map.insert(player(2), pos(0, 4096)),
            Err(MapError::OutOfBounds)
        );
    }

    #[test]
    fn insert_duplicate_entity() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(
            map.insert(player(1), pos(200, 200)),
            Err(MapError::AlreadyInserted)
        );
    }

    #[test]
    fn insert_at_boundary_positions() {
        let mut map = Map::new();
        map.insert(player(1), pos(0, 0)).unwrap();
        map.insert(player(2), pos(4095, 4095)).unwrap();
        map.insert(player(3), pos(0, 4095)).unwrap();
        map.insert(player(4), pos(4095, 0)).unwrap();
    }

    #[test]
    fn force_insert_at_free_position() {
        let mut map = Map::new();
        let result = map.force_insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(result.position, pos(100, 100));
    }

    #[test]
    fn force_insert_at_occupied_finds_nearby() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let result = map.force_insert(player(2), pos(100, 100)).unwrap();
        assert_ne!(result.position, pos(100, 100));
        let dx = (result.position.x as i32 - 100).abs();
        let dy = (result.position.y as i32 - 100).abs();
        assert!(dx <= SEARCH_RANGE && dy <= SEARCH_RANGE);
    }

    #[test]
    fn force_insert_near_map_edge() {
        let mut map = Map::new();
        map.insert(player(1), pos(0, 0)).unwrap();
        let result = map.force_insert(player(2), pos(0, 0)).unwrap();
        assert_ne!(result.position, pos(0, 0));
        assert!(
            result.position.x <= SEARCH_RANGE as u16 || result.position.y <= SEARCH_RANGE as u16
        );
    }

    #[test]
    fn force_insert_fully_surrounded() {
        let mut map = Map::new();
        let center = pos(100, 100);
        let mut id = 1;
        for dy in -SEARCH_RANGE..=SEARCH_RANGE {
            for dx in -SEARCH_RANGE..=SEARCH_RANGE {
                let p = pos((100 + dx) as u16, (100 + dy) as u16);
                map.insert(player(id), p).unwrap();
                id += 1;
            }
        }
        assert_eq!(
            map.force_insert(player(id), center),
            Err(MapError::NoFreePosition)
        );
    }

    #[test]
    fn force_insert_idempotent_same_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let result = map.force_insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(result.position, pos(100, 100));
    }

    #[test]
    fn force_insert_already_on_map_different_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(
            map.force_insert(player(1), pos(200, 200)),
            Err(MapError::AlreadyInserted)
        );
    }

    #[test]
    fn remove_existing_entity() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(110, 110)).unwrap();
        let result = map.remove(player(1)).unwrap();
        assert_eq!(result.position, pos(100, 100));
        assert_eq!(result.spectators, vec![player(2)]);
    }

    #[test]
    fn remove_nonexistent_entity() {
        let mut map = Map::new();
        assert_eq!(map.remove(player(1)), Err(MapError::EntityNotFound));
    }

    #[test]
    fn remove_frees_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.remove(player(1)).unwrap();
        map.insert(player(2), pos(100, 100)).unwrap();
    }

    #[test]
    fn move_to_free_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let result = map.move_entity(player(1), pos(200, 200)).unwrap();
        assert_eq!(result.from, pos(100, 100));
        assert_eq!(result.to, pos(200, 200));
        assert_eq!(map.get_position(player(1)), Some(pos(200, 200)));
    }

    #[test]
    fn move_to_occupied_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(200, 200)).unwrap();
        assert_eq!(
            map.move_entity(player(1), pos(200, 200)),
            Err(MapError::Occupied)
        );
    }

    #[test]
    fn move_nonexistent_entity() {
        let mut map = Map::new();
        assert_eq!(
            map.move_entity(player(1), pos(100, 100)),
            Err(MapError::EntityNotFound)
        );
    }

    #[test]
    fn move_out_of_bounds() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(
            map.move_entity(player(1), pos(4096, 0)),
            Err(MapError::OutOfBounds)
        );
    }

    #[test]
    fn move_to_same_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(110, 110)).unwrap();
        let result = map.move_entity(player(1), pos(100, 100)).unwrap();
        assert!(result.entered.is_empty());
        assert!(result.exited.is_empty());
        assert_eq!(result.stayed, vec![player(2)]);
    }

    #[test]
    fn move_frees_old_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.move_entity(player(1), pos(200, 200)).unwrap();
        map.insert(player(2), pos(100, 100)).unwrap();
    }

    #[test]
    fn move_vision_entered() {
        let mut map = Map::new();
        // Player 1 at (100, 100), player 2 far away at (200, 200)
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(200, 200)).unwrap();

        // Move player 1 close to player 2
        let result = map.move_entity(player(1), pos(195, 195)).unwrap();
        assert!(result.entered.contains(&player(2)));
        assert!(!result.exited.contains(&player(2)));
        assert!(!result.stayed.contains(&player(2)));
    }

    #[test]
    fn move_vision_exited() {
        let mut map = Map::new();
        // Player 1 and 2 near each other
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(110, 110)).unwrap();

        // Move player 1 far away
        let result = map.move_entity(player(1), pos(500, 500)).unwrap();
        assert!(result.exited.contains(&player(2)));
        assert!(!result.entered.contains(&player(2)));
        assert!(!result.stayed.contains(&player(2)));
    }

    #[test]
    fn move_vision_stayed() {
        let mut map = Map::new();
        // Player 1 and 2 near each other
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(105, 105)).unwrap();

        // Move player 1 slightly (player 2 still in range)
        let result = map.move_entity(player(1), pos(102, 102)).unwrap();
        assert!(result.stayed.contains(&player(2)));
        assert!(!result.entered.contains(&player(2)));
        assert!(!result.exited.contains(&player(2)));
    }

    #[test]
    fn move_vision_comprehensive() {
        let mut map = Map::new();
        // Player A at center
        map.insert(player(1), pos(100, 100)).unwrap();
        // Player B: near A, will stay in vision after move
        map.insert(player(2), pos(108, 108)).unwrap();
        // Player C: near A, will exit vision after move
        map.insert(player(3), pos(85, 85)).unwrap();
        // Player D: far from A, will enter vision after move
        map.insert(player(4), pos(130, 130)).unwrap();
        // Player E: far from both old and new position
        map.insert(player(5), pos(500, 500)).unwrap();

        let result = map.move_entity(player(1), pos(115, 115)).unwrap();

        assert!(result.stayed.contains(&player(2)));
        assert!(result.exited.contains(&player(3)));
        assert!(result.entered.contains(&player(4)));
        assert!(!result.entered.contains(&player(5)));
        assert!(!result.exited.contains(&player(5)));
        assert!(!result.stayed.contains(&player(5)));
    }

    #[test]
    fn get_spectators_empty_map() {
        let map = Map::new();
        let result = map.get_spectators(pos(100, 100), player(1));
        assert!(result.is_empty());
    }

    #[test]
    fn get_spectators_excludes_target() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let result = map.get_spectators(pos(100, 100), player(1));
        assert!(!result.contains(&player(1)));
    }

    #[test]
    fn get_spectators_viewport_boundary() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        // Exactly at boundary: center.x + HALF_VIEWPORT_X = 100 + 16 = 116
        map.insert(player(2), pos(116, 100)).unwrap();
        // One past boundary
        map.insert(player(3), pos(117, 100)).unwrap();

        let result = map.get_spectators(pos(100, 100), player(1));
        assert!(result.contains(&player(2)));
        assert!(!result.contains(&player(3)));
    }

    #[test]
    fn get_spectators_clamped_at_edge() {
        let mut map = Map::new();
        map.insert(player(1), pos(0, 0)).unwrap();
        map.insert(player(2), pos(16, 16)).unwrap();
        map.insert(player(3), pos(17, 0)).unwrap();

        let result = map.get_spectators(pos(0, 0), player(1));
        assert!(result.contains(&player(2)));
        assert!(!result.contains(&player(3)));
    }

    #[test]
    fn get_position_after_insert() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert_eq!(map.get_position(player(1)), Some(pos(100, 100)));
    }

    #[test]
    fn get_position_after_move() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.move_entity(player(1), pos(200, 200)).unwrap();
        assert_eq!(map.get_position(player(1)), Some(pos(200, 200)));
    }

    #[test]
    fn get_position_after_remove() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.remove(player(1)).unwrap();
        assert_eq!(map.get_position(player(1)), None);
    }

    #[test]
    fn get_position_nonexistent() {
        let map = Map::new();
        assert_eq!(map.get_position(player(1)), None);
    }

    #[test]
    fn force_move_to_free_position() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let result = map.force_move_entity(player(1), pos(200, 200)).unwrap();
        assert_eq!(result.to, pos(200, 200));
        assert_eq!(map.get_position(player(1)), Some(pos(200, 200)));
    }

    #[test]
    fn force_move_to_occupied_finds_nearby() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        map.insert(player(2), pos(200, 200)).unwrap();
        let result = map.force_move_entity(player(1), pos(200, 200)).unwrap();
        assert_ne!(result.to, pos(200, 200));
        let dx = (result.to.x as i32 - 200).abs();
        let dy = (result.to.y as i32 - 200).abs();
        assert!(dx <= SEARCH_RANGE && dy <= SEARCH_RANGE);
    }

    #[test]
    fn force_move_fully_surrounded() {
        let mut map = Map::new();
        map.insert(player(1), pos(50, 50)).unwrap();
        let mut id = 10;
        let center = pos(100, 100);
        for dy in -SEARCH_RANGE..=SEARCH_RANGE {
            for dx in -SEARCH_RANGE..=SEARCH_RANGE {
                let p = pos((100 + dx) as u16, (100 + dy) as u16);
                map.insert(player(id), p).unwrap();
                id += 1;
            }
        }
        assert_eq!(
            map.force_move_entity(player(1), center),
            Err(MapError::NoFreePosition)
        );
    }

    #[test]
    fn mob_entities_work_same_as_players() {
        let mut map = Map::new();
        map.insert(mob(1), pos(100, 100)).unwrap();
        map.insert(player(1), pos(110, 110)).unwrap();

        let spectators = map.get_spectators(pos(105, 105), player(99));
        assert!(spectators.contains(&mob(1)));
        assert!(spectators.contains(&player(1)));

        let result = map.move_entity(mob(1), pos(105, 105)).unwrap();
        assert_eq!(result.from, pos(100, 100));
        assert_eq!(result.to, pos(105, 105));

        map.remove(mob(1)).unwrap();
        assert_eq!(map.get_position(mob(1)), None);
    }

    #[test]
    fn is_occupied_by_other_different_entity() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert!(map.is_occupied_by_other(pos(100, 100), player(2)));
    }

    #[test]
    fn is_occupied_by_other_self() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert!(!map.is_occupied_by_other(pos(100, 100), player(1)));
    }

    #[test]
    fn is_occupied_by_other_empty() {
        let map = Map::new();
        assert!(!map.is_occupied_by_other(pos(100, 100), player(1)));
    }

    #[test]
    fn find_nearest_free_finds_adjacent() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        let free = map.find_nearest_free(pos(100, 100)).unwrap();
        assert_ne!(free, pos(100, 100));
        let dx = (free.x as i32 - 100).abs();
        let dy = (free.y as i32 - 100).abs();
        assert!(dx <= 1 && dy <= 1);
    }

    #[test]
    fn is_walkable_without_height_map() {
        let map = Map::new();
        assert!(map.is_walkable(pos(0, 0)));
        assert!(map.is_walkable(pos(100, 200)));
        assert!(map.is_walkable(pos(4095, 4095)));
    }

    #[test]
    fn is_walkable_with_height_map_blocked() {
        let mut hm = HeightMap::empty(4096, 4096);
        hm.set(50, 50, 127);
        let map = Map::with_height_map(hm);
        assert!(!map.is_walkable(pos(50, 50)));
        assert!(map.is_walkable(pos(51, 50)));
    }

    #[test]
    fn can_step_to_empty_flat() {
        let hm = HeightMap::empty(4096, 4096);
        let map = Map::with_height_map(hm);
        assert!(map.can_step(pos(100, 100), pos(101, 100)));
    }

    #[test]
    fn can_step_to_occupied() {
        let mut map = Map::new();
        map.insert(player(1), pos(100, 100)).unwrap();
        assert!(!map.can_step(pos(99, 100), pos(100, 100)));
    }

    #[test]
    fn can_step_to_blocked() {
        let mut hm = HeightMap::empty(4096, 4096);
        hm.set(101, 100, 127);
        let map = Map::with_height_map(hm);
        assert!(!map.can_step(pos(100, 100), pos(101, 100)));
    }

    #[test]
    fn can_step_steep() {
        let mut hm = HeightMap::empty(4096, 4096);
        hm.set(100, 100, 10);
        hm.set(101, 100, 20);
        let map = Map::with_height_map(hm);
        assert!(!map.can_step(pos(100, 100), pos(101, 100)));
    }

    #[test]
    fn can_step_out_of_bounds() {
        let map = Map::new();
        assert!(!map.can_step(pos(4095, 4095), pos(4096, 4095)));
    }

    #[test]
    fn force_insert_avoids_blocked_cells() {
        let mut hm = HeightMap::empty(4096, 4096);
        hm.set(100, 100, 127);
        let mut map = Map::with_height_map(hm);
        let result = map.force_insert(player(1), pos(100, 100)).unwrap();
        assert_ne!(result.position, pos(100, 100));
        let dx = (result.position.x as i32 - 100).abs();
        let dy = (result.position.y as i32 - 100).abs();
        assert!(dx <= SEARCH_RANGE && dy <= SEARCH_RANGE);
    }
}
