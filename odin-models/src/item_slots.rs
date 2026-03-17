use crate::item::Item;
use std::fmt;
use std::marker::PhantomData;

pub trait SlotIndex: Copy {
    fn to_index(self) -> usize;
    fn from_index(index: usize) -> Option<Self>;
}

impl SlotIndex for usize {
    fn to_index(self) -> usize {
        self
    }
    fn from_index(index: usize) -> Option<Self> {
        Some(index)
    }
}

pub struct ItemSlots<K: SlotIndex, const N: usize> {
    items: [Option<Item>; N],
    _key: PhantomData<K>,
}

impl<K: SlotIndex, const N: usize> ItemSlots<K, N> {
    pub fn get(&self, key: K) -> Option<&Item> {
        let idx = key.to_index();
        self.items.get(idx)?.as_ref()
    }

    pub fn set(&mut self, key: K, item: Item) {
        let idx = key.to_index();
        if idx < N {
            self.items[idx] = Some(item);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &Item)> {
        self.items.iter().enumerate().filter_map(|(i, item)| {
            let key = K::from_index(i)?;
            item.as_ref().map(|item| (key, item))
        })
    }

    pub fn map_slots<T: Default + Copy>(&self, f: impl Fn(K, &Item) -> T) -> [T; N] {
        std::array::from_fn(|i| {
            K::from_index(i)
                .and_then(|key| self.items[i].as_ref().map(|item| f(key, item)))
                .unwrap_or_default()
        })
    }
}

impl<K: SlotIndex, const N: usize> Default for ItemSlots<K, N> {
    fn default() -> Self {
        Self {
            items: [None; N],
            _key: PhantomData,
        }
    }
}

impl<K: SlotIndex, const N: usize> Clone for ItemSlots<K, N> {
    fn clone(&self) -> Self {
        Self {
            items: self.items,
            _key: PhantomData,
        }
    }
}

impl<K: SlotIndex, const N: usize> fmt::Debug for ItemSlots<K, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(
                self.items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| item.as_ref().map(|item| (i, item))),
            )
            .finish()
    }
}

impl<K: SlotIndex, T, const N: usize> From<T> for ItemSlots<K, N>
where
    T: IntoIterator<Item = (K, Item)>,
{
    fn from(value: T) -> Self {
        let mut items = [None; N];
        for (key, item) in value {
            let idx = key.to_index();
            if idx < N {
                items[idx] = Some(item);
            }
        }
        Self {
            items,
            _key: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EquipmentSlot;

    type TestSlots = ItemSlots<usize, 4>;
    type EquipSlots = ItemSlots<EquipmentSlot, 18>;

    #[test]
    fn default_is_all_empty() {
        let slots = TestSlots::default();
        assert_eq!(slots.iter().count(), 0);
    }

    #[test]
    fn from_iterator() {
        let item_a = Item::from(100u16);
        let item_b = Item::from(200u16);
        let slots = TestSlots::from([(0, item_a), (2, item_b)]);

        assert_eq!(slots.get(0).unwrap().id, 100);
        assert!(slots.get(1).is_none());
        assert_eq!(slots.get(2).unwrap().id, 200);
        assert!(slots.get(3).is_none());
    }

    #[test]
    fn iter_skips_empty_slots() {
        let item_a = Item::from(100u16);
        let item_b = Item::from(200u16);
        let slots = TestSlots::from([(0, item_a), (3, item_b)]);

        let collected: Vec<_> = slots.iter().map(|(k, item)| (k, item.id)).collect();
        assert_eq!(collected, vec![(0, 100), (3, 200)]);
    }

    #[test]
    fn set_and_get() {
        let mut slots = TestSlots::default();
        let item = Item::from(42u16);
        slots.set(2, item);

        assert_eq!(slots.get(2).unwrap().id, 42);
        assert!(slots.get(0).is_none());
    }

    #[test]
    fn out_of_bounds_set_is_noop() {
        let mut slots = TestSlots::default();
        slots.set(10, Item::from(1u16));
        assert_eq!(slots.iter().count(), 0);
    }

    #[test]
    fn equipment_slot_round_trip() {
        let item = Item::from(500u16);
        let slots = EquipSlots::from([(EquipmentSlot::Armor, item)]);

        assert_eq!(slots.get(EquipmentSlot::Armor).unwrap().id, 500);
        assert!(slots.get(EquipmentSlot::Face).is_none());
    }

    #[test]
    fn map_slots_produces_correct_array() {
        let slots = TestSlots::from([(1, Item::from(10u16)), (3, Item::from(30u16))]);
        let result = slots.map_slots(|_, item| item.id);

        assert_eq!(result, [0, 10, 0, 30]);
    }

    #[test]
    fn iter_with_equipment_slot_covers_all_indices() {
        let item = Item::from(1u16);
        let mut slots = EquipSlots::default();
        for i in 0..18 {
            slots.items[i] = Some(item);
        }

        let keys: Vec<_> = slots.iter().map(|(k, _)| k.to_index()).collect();
        assert_eq!(keys, (0..18).collect::<Vec<_>>());
    }
}
