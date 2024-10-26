use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Account::Id)
                            .uuid()
                            .extra("DEFAULT gen_random_uuid()")
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Account::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Account::Password).string_len(60).not_null())
                    .col(
                        ColumnDef::new(Account::Cash)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Account::Access).integer().default(0))
                    .col(
                        ColumnDef::new(Account::StorageCoin)
                            .big_integer()
                            .default(0),
                    )
                    .col(ColumnDef::new(Account::Divina).timestamp().null())
                    .col(ColumnDef::new(Account::Sephira).timestamp().null())
                    .col(ColumnDef::new(Account::Saude).timestamp().null())
                    .col(ColumnDef::new(Account::Token).string_len(16).null())
                    .col(
                        ColumnDef::new(Account::UniqueField)
                            .big_integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Account::DailyLastYearDay)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Account::DailyConsecutiveDays)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Account::WaterLastYearDay)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Account::WaterTotalEntries)
                            .integer()
                            .default(0),
                    )
                    .col(ColumnDef::new(Account::SingleGift).integer().default(0))
                    .col(ColumnDef::new(Account::TelegramToken).string_len(16).null())
                    .col(
                        ColumnDef::new(Account::ChangeServerKey)
                            .string_len(52)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .table(Account::Table)
                    .col(Account::Username)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Account {
    Table,
    Id,
    Username,
    Password,
    Cash,
    Access,
    StorageCoin,
    Divina,
    Sephira,
    Saude,
    Token,
    UniqueField,
    DailyLastYearDay,
    DailyConsecutiveDays,
    WaterLastYearDay,
    WaterTotalEntries,
    SingleGift,
    TelegramToken,
    ChangeServerKey,
}
