use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "character")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub slot: i32,
    pub name: String,
    pub merchant: i16,
    pub guild_id: Option<i16>,
    pub class: i16,
    pub affect_info: i16,
    pub quest_info: i16,
    pub coin: i32,
    pub experience: i64,
    pub last_pos: String,
    pub level: i32,
    pub reserved: i32,
    pub strength: i32,
    pub intelligence: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub special0: i32,
    pub special1: i32,
    pub special2: i32,
    pub special3: i32,
    pub current_hp: i32,
    pub current_mp: i32,
    pub learned1: i32,
    pub learned2: i32,
    pub guild_level: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Guild,
    Items,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Relation::Guild => Entity::has_one(super::guilds::Entity)
                .from(Column::GuildId)
                .to(super::guilds::Column::Id)
                .into(),
            Relation::Items => Entity::has_many(super::item::Entity)
                .from(Column::Id)
                .to(super::item::Column::CharacterId)
                .into(),
        }
    }
}
impl Related<super::guilds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guild.def()
    }
}
impl Related<super::item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Items.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
