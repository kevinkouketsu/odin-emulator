use entity::account_ban::BanType;
use extension::postgres::TypeDropStatement;
use sea_orm::ActiveEnum;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(feature = "sqlite")]
        {
            manager
                .get_connection()
                .execute_unprepared("PRAGMA foreign_keys = ON;")
                .await?;
        }
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Account::Id).uuid().not_null().primary_key())
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
                    .col(
                        ColumnDef::new(Account::Access)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Account::StorageCoin)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Account::Token).string_len(16).null())
                    .to_owned(),
            )
            .await?;

        create_account_ban_table(manager).await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_account_username")
                    .table(Account::Table)
                    .col(Account::Username)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountBan::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await?;

        manager
            .drop_type(
                TypeDropStatement::new()
                    .if_exists()
                    .name(entity::account_ban::BanType::name())
                    .to_owned(),
            )
            .await
    }
}

async fn create_account_ban_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    #[cfg(feature = "postgresql")]
    {
        let schema = sea_orm::Schema::new(sea_orm::DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<BanType>())
            .await?;
    }

    manager
        .create_table(
            Table::create()
                .table(AccountBan::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(AccountBan::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(AccountBan::AccountId).uuid().not_null())
                .col(
                    ColumnDef::new(AccountBan::Type)
                        .custom(BanType::name())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(AccountBan::BannedAt)
                        .timestamp()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(ColumnDef::new(AccountBan::ExpiresAt).timestamp().not_null())
                .col(
                    ColumnDef::new(AccountBan::AccountBannedBy)
                        .uuid()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(AccountBan::Reason)
                        .string_len(256)
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(AccountBan::Table, AccountBan::AccountId)
                        .to(Account::Table, Account::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(AccountBan::Table, AccountBan::AccountBannedBy)
                        .to(Account::Table, Account::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_account_ban_account_id")
                .table(AccountBan::Table)
                .col(AccountBan::AccountId)
                .to_owned(),
        )
        .await
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
    Token,
}

#[derive(Iden)]
pub enum AccountBan {
    Id,
    Table,
    AccountId,
    Type,
    BannedAt,
    ExpiresAt,
    AccountBannedBy,
    Reason,
}
