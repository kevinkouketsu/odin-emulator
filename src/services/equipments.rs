use odin_models::{item::Item, EquipmentSlot, MAX_EQUIPS};

#[derive(Debug)]
pub struct Equipments {
    items: [Option<Item>; MAX_EQUIPS],
}
impl Equipments {
    pub fn iter(&self) -> impl Iterator<Item = (EquipmentSlot, &Item)> {
        self.items.iter().enumerate().filter_map(|(slot, item)| {
            item.as_ref().map(|item| {
                (
                    slot.try_into().expect(
                        "This is a array with fixed size therefore it is always valid to convert",
                    ),
                    item,
                )
            })
        })
    }

    pub fn get_item(&self, slot: EquipmentSlot) -> Option<&Item> {
        self.items
            .get(slot.as_index())
            .and_then(|slot| slot.as_ref())
    }
}
impl<T, U> From<T> for Equipments
where
    T: IntoIterator<Item = (U, Item)>,
    U: Into<usize>,
{
    fn from(value: T) -> Self {
        let mut items = [None; MAX_EQUIPS];

        for (item_slot, item) in value {
            let item_slot = item_slot.into();
            if item_slot < MAX_EQUIPS {
                items[item_slot] = Some(item);
            }
        }

        Equipments { items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odin_models::item::Item;

    #[test]
    fn iter_only_iterates_on_non_empty_items() {
        let item = Item::from(737);

        let equipments = [
            (EquipmentSlot::Face, item),
            (EquipmentSlot::Armor, item),
            (EquipmentSlot::LeftWeapon, item),
            (EquipmentSlot::Amulet1, item),
        ];

        assert_eq!(
            Equipments::from(equipments)
                .iter()
                .map(|(i, item)| (i, item.to_owned()))
                .collect::<Vec<_>>(),
            equipments.to_vec()
        )
    }
}
