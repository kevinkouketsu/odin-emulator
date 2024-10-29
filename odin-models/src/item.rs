use crate::MAX_ITEM_EFFECT;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub id: u16,
    pub effects: [ItemBonusEffect; MAX_ITEM_EFFECT],
}
impl From<u16> for Item {
    fn from(value: u16) -> Self {
        Item {
            id: value,
            ..Default::default()
        }
    }
}
impl From<(u16, BonusEffect, BonusEffect)> for Item {
    fn from((id, ef1, efv1): (u16, BonusEffect, BonusEffect)) -> Self {
        Item {
            id,
            effects: [
                (ef1, efv1).into(),
                ItemBonusEffect::default(),
                ItemBonusEffect::default(),
            ],
        }
    }
}
impl From<(u16, BonusEffect, BonusEffect, BonusEffect, BonusEffect)> for Item {
    fn from(
        (id, ef1, efv1, ef2, efv2): (u16, BonusEffect, BonusEffect, BonusEffect, BonusEffect),
    ) -> Self {
        Item {
            id,
            effects: [
                (ef1, efv1).into(),
                (ef2, efv2).into(),
                ItemBonusEffect::default(),
            ],
        }
    }
}
impl
    From<(
        u16,
        BonusEffect,
        BonusEffect,
        BonusEffect,
        BonusEffect,
        BonusEffect,
        BonusEffect,
    )> for Item
{
    fn from(
        (id, ef1, efv1, ef2, efv2, ef3, efv3): (
            u16,
            BonusEffect,
            BonusEffect,
            BonusEffect,
            BonusEffect,
            BonusEffect,
            BonusEffect,
        ),
    ) -> Self {
        Item {
            id,
            effects: [(ef1, efv1).into(), (ef2, efv2).into(), (ef3, efv3).into()],
        }
    }
}

pub type BonusEffect = u8;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ItemBonusEffect {
    // TODO: change this to an enum
    pub index: BonusEffect,
    pub value: BonusEffect,
}
impl From<(BonusEffect, BonusEffect)> for ItemBonusEffect {
    fn from((effect_index, effect_value): (BonusEffect, BonusEffect)) -> Self {
        Self {
            index: effect_index,
            value: effect_value,
        }
    }
}
