use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "guild")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub fame: i32,
    pub kingdom: i32,
    pub wins: i32,
    pub sub_guild_1: String,
    pub sub_guild_2: String,
    pub sub_guild_3: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Character,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Character => Entity::has_many(crate::character::Entity)
                .from(Column::Id)
                .to(super::character::Column::GuildId)
                .into(),
        }
    }
}
impl Related<super::character::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Character.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
