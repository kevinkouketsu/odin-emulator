use crate::item::Item;

#[derive(Debug, Default, Clone)]
pub struct Storage {
    pub items: Vec<(usize, Item)>,
    pub coin: u64,
}
