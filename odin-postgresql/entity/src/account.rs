use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password: String,
    pub cash: i32,
    pub access: Option<i32>,
    pub storage_coin: Option<i64>,
    pub token: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AccountBan,
    BannedByMe,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::AccountBan => Entity::has_many(super::account_ban::Entity).into(),
            Relation::BannedByMe => Entity::has_many(super::account_ban::Entity).into(),
        }
    }
}

impl Related<super::account_ban::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountBan.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
