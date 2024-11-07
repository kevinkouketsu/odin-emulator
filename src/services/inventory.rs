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
        let item = Item::from(737);

        let inventory = [(0, item), (2, item), (3, item), (9, item)];

        assert_eq!(
            Inventory::from(inventory)
                .iter()
                .map(|(i, item)| (i, item.to_owned()))
                .collect::<Vec<_>>(),
            inventory.to_vec()
        )
    }
}
