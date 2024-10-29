use sea_orm_migration::prelude::*;

use crate::m20241026_013531_create_accounts_table::Account;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Guild::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Guild::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Guild::Name)
                            .string_len(32)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Guild::Fame).integer().not_null().default(0))
                    .col(ColumnDef::new(Guild::Kingdom).integer().not_null())
                    .col(ColumnDef::new(Guild::Wins).integer().not_null())
                    .col(ColumnDef::new(Guild::SubGuild1).string_len(32).not_null())
                    .col(ColumnDef::new(Guild::SubGuild2).string_len(32).not_null())
                    .col(ColumnDef::new(Guild::SubGuild3).string_len(32).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Character::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Character::Id)
                            .uuid()
                            .extra("DEFAULT gen_random_uuid()")
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Character::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Character::Slot).integer().not_null())
                    .col(
                        ColumnDef::new(Character::Name)
                            .string_len(16)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Character::Merchant)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Character::GuildId).small_integer().null())
                    .col(
                        ColumnDef::new(Character::Class)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::AffectInfo)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::QuestInfo)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Gold)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Experience)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::LastPos)
                            .string_len(16)
                            .not_null()
                            .default("(0,0)"),
                    )
                    .col(
                        ColumnDef::new(Character::Level)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Reserved)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Strength)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Intelligence)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Dexterity)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Constitution)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Special0)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Special1)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Special2)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Special3)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::CurrentHp)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::CurrentMp)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Learned1)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::Learned2)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Character::GuildLevel)
                            .small_integer()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Character::Table, Character::AccountId)
                            .to(Account::Table, Account::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Character::Table, Character::GuildId)
                            .to(Guild::Table, Guild::Id),
                    )
                    .index(
                        Index::create()
                            .table(Character::Table)
                            .col(Character::AccountId)
                            .col(Character::Slot)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Guild::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Character::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Character {
    Table,
    Id,
    AccountId,
    Slot,
    Name,
    Merchant,
    GuildId,
    Class,
    AffectInfo,
    QuestInfo,
    Gold,
    Experience,
    LastPos,
    Level,
    Reserved,
    Strength,
    Intelligence,
    Dexterity,
    Constitution,
    Special0,
    Special1,
    Special2,
    Special3,
    CurrentHp,
    CurrentMp,
    Learned1,
    Learned2,
    GuildLevel,
}

#[derive(Iden)]
enum Guild {
    Table,
    Id,
    Name,
    Fame,
    Kingdom,
    Wins,
    SubGuild1,
    SubGuild2,
    SubGuild3,
}
