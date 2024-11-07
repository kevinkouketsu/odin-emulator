use entity::{
    character::{Class, Evolution},
    item::ItemCategory,
};
use sea_orm::{prelude::*, ActiveEnum, Set};
use sea_orm_migration::prelude::{ColumnDef, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(StartItem::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StartItem::Id)
                            .integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StartItem::Type)
                            .custom(ItemCategory::name())
                            .not_null(),
                    )
                    .col(ColumnDef::new(StartItem::Class).small_integer().not_null())
                    .col(ColumnDef::new(StartItem::Slot).small_integer().not_null())
                    .col(
                        ColumnDef::new(StartItem::ItemId)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Ef1)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Efv1)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Ef2)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Efv2)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Ef3)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Efv3)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Ef4)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Efv4)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Ef5)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StartItem::Efv5)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .index(
                        Index::create()
                            .col(StartItem::Class)
                            .col(StartItem::Slot)
                            .col(StartItem::Type)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StorageStartItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StorageStartItems::Id)
                            .integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Slot)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::ItemId)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Ef1)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Efv1)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Ef2)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Efv2)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Ef3)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Efv3)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Ef4)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Efv4)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Ef5)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(StorageStartItems::Efv5)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .index(Index::create().col(StorageStartItems::Slot).unique())
                    .to_owned(),
            )
            .await?;

        let characters = vec![
            entity::character::ActiveModel {
                id: Set(Uuid::new_v4()),
                account_id: Set(None),
                slot: Set(0),
                name: Set("TransKnight".to_string()),
                class: Set(Class::TransKnight),
                evolution: Set(Evolution::Mortal),
                strength: Set(8),
                intelligence: Set(9),
                dexterity: Set(13),
                constitution: Set(6),
                current_hp: Set(75),
                current_mp: Set(45),
                last_pos: Set("(2100, 2100)".to_string()),
                ..Default::default()
            },
            entity::character::ActiveModel {
                id: Set(Uuid::new_v4()),
                account_id: Set(None),
                slot: Set(0),
                name: Set("Foema".to_string()),
                class: Set(Class::Foema),
                evolution: Set(Evolution::Mortal),
                strength: Set(5),
                intelligence: Set(8),
                dexterity: Set(5),
                constitution: Set(5),
                current_hp: Set(60),
                current_mp: Set(65),
                last_pos: Set("(2100, 2100)".to_string()),
                ..Default::default()
            },
            entity::character::ActiveModel {
                id: Set(Uuid::new_v4()),
                account_id: Set(None),
                slot: Set(0),
                name: Set("BeastMaster".to_string()),
                class: Set(Class::BeastMaster),
                evolution: Set(Evolution::Mortal),
                strength: Set(6),
                intelligence: Set(6),
                dexterity: Set(9),
                constitution: Set(5),
                current_hp: Set(70),
                current_mp: Set(55),
                last_pos: Set("(2100, 2100)".to_string()),
                ..Default::default()
            },
            entity::character::ActiveModel {
                id: Set(Uuid::new_v4()),
                account_id: Set(None),
                slot: Set(0),
                name: Set("Huntress".to_string()),
                class: Set(Class::Huntress),
                evolution: Set(Evolution::Mortal),
                strength: Set(8),
                intelligence: Set(9),
                dexterity: Set(13),
                constitution: Set(6),
                current_hp: Set(75),
                current_mp: Set(60),
                last_pos: Set("(2100, 2100)".to_string()),
                ..Default::default()
            },
        ];

        let connection = manager.get_connection();
        entity::character::Entity::insert_many(characters)
            .exec_without_returning(connection)
            .await?;

        let items = [
            // TransKnight
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 0,
                item_id: 1,
                ef2: Some(1),
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 1,
                item_id: 1104,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 2,
                item_id: 1116,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 3,
                item_id: 1128,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 4,
                item_id: 1140,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 5,
                item_id: 1152,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::TransKnight,
                slot: 6,
                item_id: 861,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Inventory,
                class: Class::TransKnight,
                slot: 4,
                item_id: 917,
                ef2: None,
            },
            // Foema
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 0,
                item_id: 11,
                ef2: Some(11),
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 1,
                item_id: 1254,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 2,
                item_id: 1266,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 3,
                item_id: 1278,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 4,
                item_id: 1290,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 5,
                item_id: 1302,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Foema,
                slot: 6,
                item_id: 816,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Inventory,
                class: Class::Foema,
                slot: 4,
                item_id: 918,
                ef2: None,
            },
            // BeastMaster
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 0,
                item_id: 21,
                ef2: Some(21),
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 1,
                item_id: 1416,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 2,
                item_id: 1419,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 3,
                item_id: 1422,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 4,
                item_id: 1425,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 5,
                item_id: 1428,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::BeastMaster,
                slot: 6,
                item_id: 861,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Inventory,
                class: Class::BeastMaster,
                slot: 4,
                item_id: 917,
                ef2: None,
            },
            // Huntress
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 0,
                item_id: 31,
                ef2: Some(31),
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 1,
                item_id: 1553,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 2,
                item_id: 1569,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 3,
                item_id: 1572,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 4,
                item_id: 1575,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 5,
                item_id: 1578,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Equip,
                class: Class::Huntress,
                slot: 6,
                item_id: 816,
                ef2: None,
            },
            StartItemPartial {
                category: ItemCategory::Inventory,
                class: Class::Huntress,
                slot: 4,
                item_id: 923,
                ef2: None,
            },
        ];

        entity::start_item::Entity::insert_many(
            items
                .into_iter()
                .map(|item| item.into_active_model())
                .collect::<Vec<_>>(),
        )
        .exec_without_returning(connection)
        .await?;

        for item in [
            Class::TransKnight,
            Class::Foema,
            Class::BeastMaster,
            Class::Huntress,
        ]
        .map(StartItemPartial::generate_potion_for)
        {
            entity::start_item::Entity::insert_many(
                item.into_iter()
                    .map(|item| item.into_active_model())
                    .collect::<Vec<_>>(),
            )
            .exec_without_returning(connection)
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StartItem::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(StorageStartItems::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP FUNCTION IF EXISTS CreateCharacter(VARCHAR(36), VARCHAR(32), INT, INT)",
            )
            .await?;

        Ok(())
    }
}

struct StartItemPartial {
    item_id: i16,
    ef2: Option<i16>,
    category: ItemCategory,
    class: Class,
    slot: i16,
}
impl StartItemPartial {
    fn into_active_model(self) -> entity::start_item::ActiveModel {
        let ef2 = self.ef2.unwrap_or_default();

        entity::start_item::ActiveModel {
            r#type: Set(self.category),
            class: Set(self.class),
            slot: Set(self.slot),
            item_id: Set(self.item_id), // Face
            ef2: Set(ef2),
            efv2: Set(1),
            ..Default::default()
        }
    }

    fn generate_potion_for(class: Class) -> Vec<StartItemPartial> {
        (0..4)
            .map(|i| StartItemPartial {
                item_id: 400,
                ef2: None,
                category: ItemCategory::Inventory,
                class,
                slot: i,
            })
            .collect()
    }
}

#[derive(DeriveIden)]
enum StartItem {
    Table,
    Id,
    Type,
    Class,
    Slot,
    ItemId,
    Ef1,
    Efv1,
    Ef2,
    Efv2,
    Ef3,
    Efv3,
    Ef4,
    Efv4,
    Ef5,
    Efv5,
}

#[derive(DeriveIden)]
enum StorageStartItems {
    Table,
    Id,
    Slot,
    ItemId,
    Ef1,
    Efv1,
    Ef2,
    Efv2,
    Ef3,
    Efv3,
    Ef4,
    Efv4,
    Ef5,
    Efv5,
}
