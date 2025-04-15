use odin_models::{item::Item, MAX_INVENTORY_VISIBLE};

pub struct Inventory {
    items: [Option<Item>; MAX_INVENTORY_VISIBLE],
}
impl Inventory {
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Item)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| item.as_ref().map(|item| (slot, item)))
    }
}

impl<T, U> From<T> for Inventory
where
    T: IntoIterator<Item = (U, Item)>,
    U: Into<usize>,
{
    fn from(value: T) -> Self {
        let mut items = [None; MAX_INVENTORY_VISIBLE];

        for (item_slot, item) in value {
            let item_slot = item_slot.into();
            if item_slot < MAX_INVENTORY_VISIBLE {
                items[item_slot] = Some(item);
            }
        }

        Inventory { items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odin_models::item::Item;

    #[test]
    fn iter_only_iterates_on_non_empty_items() {
        let item1 = Item::from(737);
        let item2 = Item::from(738);
        let item3 = Item::from(739);
        let item4 = Item::from(740);

        let inventory: [(usize, Item); 4] = [(0, item1), (2, item2), (3, item3), (9, item4)];
        let inventory_with_empty = Inventory::from(inventory);

        let iterated_items: Vec<_> = inventory_with_empty
            .iter()
            .map(|(slot, item)| (slot, item.to_owned()))
            .collect();

        assert_eq!(
            iterated_items.len(),
            4,
            "Should only iterate over non-empty slots"
        );
        assert_eq!(iterated_items[0], (0, item1), "First slot should match");
        assert_eq!(iterated_items[1], (2, item2), "Second slot should match");
        assert_eq!(iterated_items[2], (3, item3), "Third slot should match");
        assert_eq!(iterated_items[3], (9, item4), "Fourth slot should match");
    }
}
