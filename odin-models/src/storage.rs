use crate::StorageSlots;

#[derive(Debug, Default, Clone)]
pub struct Storage {
    pub items: StorageSlots,
    pub coin: u64,
}
