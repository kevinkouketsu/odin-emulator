use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "account_ban")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub r#type: BanType,
    pub banned_at: DateTime,
    pub expires_at: DateTime,
    pub account_banned_by: Uuid,
    pub reason: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountBannedBy",
        to = "super::account::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    BannedBy,
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AccountBanned,
}
impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountBanned.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ban_type")]
pub enum BanType {
    #[sea_orm(string_value = "0")]
    Analysis,
    #[sea_orm(string_value = "1")]
    Blocked,
}
