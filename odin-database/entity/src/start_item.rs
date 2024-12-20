use crate::{character::Class, item::ItemCategory};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "start_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub r#type: ItemCategory,
    pub class: Class,
    pub slot: i16,
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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
