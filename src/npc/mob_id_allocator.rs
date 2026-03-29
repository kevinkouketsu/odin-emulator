use std::collections::VecDeque;

pub const MAX_MOBS: usize = 30_000;
pub const MOB_ID_START: usize = 1000;

#[derive(Debug)]
pub struct MobIdAllocator {
    available: VecDeque<usize>,
}

impl Default for MobIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl MobIdAllocator {
    pub fn new() -> Self {
        Self {
            available: (MOB_ID_START..MOB_ID_START + MAX_MOBS).collect(),
        }
    }

    pub fn allocate(&mut self) -> Option<usize> {
        self.available.pop_front()
    }

    pub fn release(&mut self, id: usize) -> Result<(), MobIdAllocatorError> {
        if self.available.contains(&id) {
            return Err(MobIdAllocatorError::AlreadyAvailable(id));
        }
        self.available.push_back(id);
        Ok(())
    }

    pub fn available_count(&self) -> usize {
        self.available.len()
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MobIdAllocatorError {
    #[error("Mob ID {0} is already available (double-release)")]
    AlreadyAvailable(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_returns_sequential_ids_from_offset() {
        let mut allocator = MobIdAllocator::new();

        assert_eq!(allocator.allocate(), Some(MOB_ID_START));
        assert_eq!(allocator.allocate(), Some(MOB_ID_START + 1));
        assert_eq!(allocator.allocate(), Some(MOB_ID_START + 2));
    }

    #[test]
    fn allocate_exhausts_at_max() {
        let mut allocator = MobIdAllocator::new();

        for _ in 0..MAX_MOBS {
            assert!(allocator.allocate().is_some());
        }
        assert_eq!(allocator.allocate(), None);
    }

    #[test]
    fn release_makes_id_reusable() {
        let mut allocator = MobIdAllocator::new();

        for _ in 0..MAX_MOBS {
            allocator.allocate();
        }
        assert_eq!(allocator.allocate(), None);

        allocator.release(MOB_ID_START + 5).unwrap();
        assert_eq!(allocator.allocate(), Some(MOB_ID_START + 5));
    }

    #[test]
    fn release_already_available_errors() {
        let mut allocator = MobIdAllocator::new();
        assert_eq!(
            allocator.release(MOB_ID_START),
            Err(MobIdAllocatorError::AlreadyAvailable(MOB_ID_START))
        );
    }

    #[test]
    fn available_count_tracks_correctly() {
        let mut allocator = MobIdAllocator::new();

        assert_eq!(allocator.available_count(), MAX_MOBS);

        allocator.allocate();
        assert_eq!(allocator.available_count(), MAX_MOBS - 1);

        allocator.allocate();
        assert_eq!(allocator.available_count(), MAX_MOBS - 2);

        allocator.release(MOB_ID_START).unwrap();
        assert_eq!(allocator.available_count(), MAX_MOBS - 1);
    }

    #[test]
    fn ids_do_not_overlap_with_player_range() {
        let mut allocator = MobIdAllocator::new();
        let first = allocator.allocate().unwrap();
        assert!(first >= MOB_ID_START);
    }
}
