use entity::item::ItemCategory;
use extension::postgres::Type;
use sea_orm::{ActiveEnum, DbBackend, Schema};
use sea_orm_migration::prelude::*;

use crate::m20241029_210508_characters::Character;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<ItemCategory>())
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Item::Table)
                    .col(
                        ColumnDef::new(Item::Id)
                            .uuid()
                            .extra("DEFAULT gen_random_uuid()")
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Item::Type)
                            .custom(ItemCategory::name())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::ItemId)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Ef1)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Efv1)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Ef2)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Efv2)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Ef3)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Efv3)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Ef4)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Efv4)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Ef5)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Item::Efv5)
                            .small_integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Item::Slot).small_integer().not_null())
                    .col(ColumnDef::new(Item::CharacterId).uuid().not_null())
                    .index(
                        Index::create()
                            .table(Item::Table)
                            .col(Item::Slot)
                            .col(Item::CharacterId)
                            .col(Item::Type)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Item::Table, Item::CharacterId)
                            .to(Character::Table, Character::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Item::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(ItemCategory::name())
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Item {
    Table,
    Id,
    Type,
    #[allow(clippy::enum_variant_names)]
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
    Slot,
    CharacterId,
}
