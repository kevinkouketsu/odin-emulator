use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "player")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub account_id: Uuid,
    pub slot: i32,
    pub name: String,
    pub merchant: i16,
    pub guild: Option<i16>,
    pub class: i16,
    pub affect_info: i16,
    pub quest_info: i16,
    pub gold: i64,
    pub experience: i64,
    pub last_pos: String,
    pub level: i32,
    pub reserved: i32,
    pub strength: i32,
    pub intelligence: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub special_0: i32,
    pub special_1: i32,
    pub special_2: i32,
    pub special_3: i32,
    pub current_hp: i32,
    pub current_mp: i32,
    pub learned_1: i32,
    pub learned_2: i32,
    pub guild_level: Option<i16>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Guild,
}
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Guild => Entity::belongs_to(super::guilds::Entity)
                .from(Column::Guild)
                .to(super::guilds::Column::Id)
                .into(),
        }
    }
}
impl Related<super::guilds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Guild.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
