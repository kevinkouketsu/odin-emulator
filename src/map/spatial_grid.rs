use crate::map::{EntityId, MAP_SIZE};
use std::collections::HashMap;

const CHUNK_SIZE: u16 = 32;
const CHUNKS_PER_AXIS: usize = (MAP_SIZE / CHUNK_SIZE) as usize;

pub struct SpatialGrid {
    chunks: Vec<Vec<(EntityId, u16, u16)>>,
    occupancy: HashMap<(u16, u16), EntityId>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            chunks: vec![Vec::new(); CHUNKS_PER_AXIS * CHUNKS_PER_AXIS],
            occupancy: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: EntityId, x: u16, y: u16) {
        self.occupancy.insert((x, y), id);
        self.chunks[Self::chunk_index(x, y)].push((id, x, y));
    }

    pub fn remove(&mut self, id: EntityId, x: u16, y: u16) {
        self.occupancy.remove(&(x, y));
        let chunk = &mut self.chunks[Self::chunk_index(x, y)];
        if let Some(i) = chunk.iter().position(|(eid, _, _)| *eid == id) {
            chunk.swap_remove(i);
        }
    }

    pub fn move_entity(&mut self, id: EntityId, old_x: u16, old_y: u16, new_x: u16, new_y: u16) {
        self.occupancy.remove(&(old_x, old_y));
        self.occupancy.insert((new_x, new_y), id);

        let old_ci = Self::chunk_index(old_x, old_y);
        let new_ci = Self::chunk_index(new_x, new_y);

        if old_ci == new_ci {
            let chunk = &mut self.chunks[old_ci];
            if let Some(entry) = chunk.iter_mut().find(|(eid, _, _)| *eid == id) {
                entry.1 = new_x;
                entry.2 = new_y;
            }
        } else {
            let old_chunk = &mut self.chunks[old_ci];
            if let Some(i) = old_chunk.iter().position(|(eid, _, _)| *eid == id) {
                old_chunk.swap_remove(i);
            }
            self.chunks[new_ci].push((id, new_x, new_y));
        }
    }

    pub fn is_occupied(&self, x: u16, y: u16) -> Option<EntityId> {
        self.occupancy.get(&(x, y)).copied()
    }

    pub fn get_spectators(
        &self,
        min_x: u16,
        min_y: u16,
        max_x: u16,
        max_y: u16,
        exclude: EntityId,
    ) -> Vec<EntityId> {
        let chunk_min_x = (min_x / CHUNK_SIZE) as usize;
        let chunk_max_x = (max_x / CHUNK_SIZE) as usize;
        let chunk_min_y = (min_y / CHUNK_SIZE) as usize;
        let chunk_max_y = (max_y / CHUNK_SIZE) as usize;

        let mut result = Vec::new();
        for cy in chunk_min_y..=chunk_max_y {
            let row = cy * CHUNKS_PER_AXIS;
            for cx in chunk_min_x..=chunk_max_x {
                for &(id, ex, ey) in &self.chunks[row + cx] {
                    if id != exclude && ex >= min_x && ex <= max_x && ey >= min_y && ey <= max_y {
                        result.push(id);
                    }
                }
            }
        }
        result
    }

    fn chunk_index(x: u16, y: u16) -> usize {
        (y / CHUNK_SIZE) as usize * CHUNKS_PER_AXIS + (x / CHUNK_SIZE) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(id: usize) -> EntityId {
        EntityId::Player(id)
    }

    #[test]
    fn spatial_chunk_index_origin() {
        assert_eq!(SpatialGrid::chunk_index(0, 0), 0);
    }

    #[test]
    fn spatial_chunk_index_within_chunk() {
        assert_eq!(
            SpatialGrid::chunk_index(0, 0),
            SpatialGrid::chunk_index(31, 31)
        );
    }

    #[test]
    fn spatial_chunk_index_next_chunk() {
        assert_ne!(
            SpatialGrid::chunk_index(31, 0),
            SpatialGrid::chunk_index(32, 0)
        );
    }

    #[test]
    fn spatial_insert_and_occupancy() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 100, 100);
        assert_eq!(grid.is_occupied(100, 100), Some(player(1)));
        assert_eq!(grid.is_occupied(100, 101), None);
    }

    #[test]
    fn spatial_remove() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 100, 100);
        grid.remove(player(1), 100, 100);
        assert_eq!(grid.is_occupied(100, 100), None);
        assert!(grid.chunks[SpatialGrid::chunk_index(100, 100)].is_empty());
    }

    #[test]
    fn spatial_move_same_chunk() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 100, 100);
        grid.move_entity(player(1), 100, 100, 101, 101);
        assert_eq!(grid.is_occupied(100, 100), None);
        assert_eq!(grid.is_occupied(101, 101), Some(player(1)));
        assert_eq!(grid.chunks[SpatialGrid::chunk_index(100, 100)].len(), 1);
    }

    #[test]
    fn spatial_move_across_chunks() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 31, 0);
        let old_ci = SpatialGrid::chunk_index(31, 0);
        let new_ci = SpatialGrid::chunk_index(32, 0);
        assert_ne!(old_ci, new_ci);

        grid.move_entity(player(1), 31, 0, 32, 0);
        assert_eq!(grid.is_occupied(31, 0), None);
        assert_eq!(grid.is_occupied(32, 0), Some(player(1)));
        assert!(grid.chunks[old_ci].is_empty());
        assert_eq!(grid.chunks[new_ci].len(), 1);
    }

    #[test]
    fn spatial_get_spectators_filters_by_viewport() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 100, 100);
        grid.insert(player(2), 110, 110);
        grid.insert(player(3), 200, 200);

        let specs = grid.get_spectators(84, 84, 116, 116, player(99));
        assert!(specs.contains(&player(1)));
        assert!(specs.contains(&player(2)));
        assert!(!specs.contains(&player(3)));
    }

    #[test]
    fn spatial_get_spectators_excludes_self() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 100, 100);
        let specs = grid.get_spectators(84, 84, 116, 116, player(1));
        assert!(!specs.contains(&player(1)));
    }

    #[test]
    fn spatial_get_spectators_across_chunk_boundary() {
        let mut grid = SpatialGrid::new();
        grid.insert(player(1), 30, 30);
        grid.insert(player(2), 34, 34);
        let ci_1 = SpatialGrid::chunk_index(30, 30);
        let ci_2 = SpatialGrid::chunk_index(34, 34);
        assert_ne!(ci_1, ci_2);

        let specs = grid.get_spectators(14, 14, 50, 50, player(99));
        assert!(specs.contains(&player(1)));
        assert!(specs.contains(&player(2)));
    }

    #[test]
    fn spatial_many_entities_same_chunk() {
        let mut grid = SpatialGrid::new();
        for i in 0..20 {
            grid.insert(player(i), 100 + i as u16, 100);
        }
        let ci = SpatialGrid::chunk_index(100, 100);
        assert_eq!(grid.chunks[ci].len(), 20);

        grid.remove(player(10), 110, 100);
        assert_eq!(grid.chunks[ci].len(), 19);
        assert_eq!(grid.is_occupied(110, 100), None);
    }
}
