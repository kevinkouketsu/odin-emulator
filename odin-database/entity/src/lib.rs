pub mod account;
pub mod account_ban;
pub mod character;
pub mod guilds;
pub mod item;
pub mod start_item;
pub mod storage_start_items;

use sea_orm::ActiveValue;
use uuid::Uuid;

pub trait ActiveValueExt {
    fn generate_new_uuid(&mut self, insert: bool);
}
impl ActiveValueExt for ActiveValue<Uuid> {
    fn generate_new_uuid(&mut self, insert: bool) {
        if insert {
            match self {
                sea_orm::ActiveValue::Unchanged(id) if *id == Uuid::default() => {
                    *self = sea_orm::ActiveValue::Set(Uuid::new_v4())
                }
                sea_orm::ActiveValue::NotSet => *self = sea_orm::ActiveValue::Set(Uuid::new_v4()),
                _ => {}
            }
        }
    }
}
