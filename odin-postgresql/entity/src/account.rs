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
    pub divina: Option<DateTime>,
    pub sephira: Option<DateTime>,
    pub saude: Option<DateTime>,
    pub token: Option<String>,
    pub unique_field: Option<i64>,
    pub daily_last_year_day: Option<i32>,
    pub daily_consecutive_days: Option<i32>,
    pub water_last_year_day: Option<i32>,
    pub water_total_entries: Option<i32>,
    pub single_gift: Option<i32>,
    pub telegram_token: Option<String>,
    pub change_server_key: Option<String>,
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
