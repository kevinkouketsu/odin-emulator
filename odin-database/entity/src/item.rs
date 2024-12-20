use crate::ActiveValueExt;
use async_trait::async_trait;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "item")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub r#type: ItemCategory,
    pub item_id: i16,
    pub ef1: i16,
    pub efv1: i16,
    pub ef2: i16,
    pub efv2: i16,
    pub ef3: i16,
    pub efv3: i16,
    pub ef4: i16,
    pub efv4: i16,
    pub ef5: i16,
    pub efv5: i16,
    pub slot: i16,
    pub character_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::character::Entity",
        from = "Column::CharacterId",
        to = "super::character::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Character,
}
impl Related<super::character::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Character.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.id.generate_new_uuid(insert);
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "item_category")]
pub enum ItemCategory {
    #[sea_orm(string_value = "equip")]
    Equip,
    #[sea_orm(string_value = "inventory")]
    Inventory,
}

impl From<Model> for odin_models::item::Item {
    fn from(value: Model) -> Self {
        odin_models::item::Item {
            id: value.item_id as u16,
            effects: [
                (value.ef1 as u8, value.efv1 as u8).into(),
                (value.ef2 as u8, value.efv2 as u8).into(),
                (value.ef3 as u8, value.efv3 as u8).into(),
            ],
        }
    }
}
