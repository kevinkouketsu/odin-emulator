use entity::item::ItemCategory;
use sea_orm::ActiveEnum;
use sea_orm_migration::prelude::*;

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
                            .uuid()
                            .extra("DEFAULT gen_random_uuid()")
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
                            .uuid()
                            .extra("DEFAULT gen_random_uuid()")
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

        manager.get_connection().execute_unprepared("
            -- CreateCharacter procedure
            DROP FUNCTION IF EXISTS CreateCharacter(VARCHAR(36), VARCHAR(32), INT, INT);
            CREATE OR REPLACE FUNCTION CreateCharacter(acc_guid VARCHAR(36), char_name VARCHAR(32), char_slot INT, charclass INT)
            RETURNS VARCHAR(36) AS $$
            DECLARE
                MOB_GUID UUID := uuid_generate_v4();
                new_guid VARCHAR(36);
            BEGIN
                INSERT INTO
                    character
                    (
                        account_id,
                        id,
                        slot,
                        name,
                        class,
                        coin,
                        experience,
                        last_pos,
                        level,
                        reserved,
                        strength,
                        intelligence,
                        dexterity,
                        constitution,
                        special0,
                        special1,
                        special2,
                        special3,
                        current_hp,
                        current_mp
                    )
                    SELECT
                        uuid(acc_guid),
                        MOB_GUID,
                        char_slot,
                        char_name,
                        class,
                        coin,
                        experience,
                        last_pos,
                        level,
                        reserved,
                        strength,
                        intelligence,
                        dexterity,
                        constitution,
                        special0,
                        special1,
                        special2,
                        special3,
                        current_hp,
                        current_mp
                    FROM
                        character
                    WHERE
                        account_id IS NULL AND class = charclass
                    RETURNING id INTO new_guid;

                INSERT INTO item(character_id, type, slot, item_id, ef1, efv1, ef2, efv2, ef3, efv3, ef4, efv4, ef5, efv5)
                SELECT MOB_GUID, type, slot, item_id, ef1, efv1, ef2, efv2, ef3, efv3, ef4, efv4, ef5, efv5
                FROM start_item
                WHERE start_item.class = charclass;

                RETURN new_guid;
            END;
            $$ LANGUAGE plpgsql;"
        ).await?;

        // Default starting items, the user may define afterward
        manager.get_connection().execute_unprepared("
            INSERT INTO players (slot, name, class, strength, intelligence, dexterity, constitution, current_hp, current_mp, last_pos)
            VALUES (0, 'TransKnight', 0, 8, 9, 13, 6, 75, 45, '(2100, 2100)');
            INSERT INTO players (slot, name, class, strength, intelligence, dexterity, constitution, current_hp, current_mp, last_pos)
            VALUES (0, 'Foema', 1, 5, 8, 5, 5, 60, 65, '(2100, 2100)');
            INSERT INTO players (slot, name, class, strength, intelligence, dexterity, constitution, current_hp, current_mp, last_pos)
            VALUES (0, 'BeastMaster', 2, 6, 6, 9, 5, 70, 55, '(2100, 2100)');
            INSERT INTO players (slot, name, class, strength, intelligence, dexterity, constitution, current_hp, current_mp, last_pos)
            VALUES (0, 'Huntress', 3, 8, 9, 13, 6, 75, 60, '(2100, 2100)');

            INSERT INTO start_item (type, class, slot, item_id, ef1, efv1, ef2, efv2) VALUES ('equip', 0, 0, 1, 43, 0, 1, 1);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 0, 1, 1104);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 0, 2, 1116);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 0, 3, 1128);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 0, 4, 1140);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 0, 5, 1152);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 0, 6, 861);

            INSERT INTO start_item (type, class, slot, item_id, ef1, efv1, ef2, efv2) VALUES ('equip', 1, 0, 11, 43, 0, 11, 1);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 1, 1, 1254);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 1, 2, 1266);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 1, 3, 1278);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 1, 4, 1290);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 1, 5, 1302);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 1, 6, 816);

            INSERT INTO start_item (type, class, slot, item_id, ef1, efv1, ef2, efv2) VALUES ('equip', 2, 0, 21, 43, 0, 21, 1);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 2, 1, 1416);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 2, 2, 1419);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 2, 3, 1422);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 2, 4, 1425);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 2, 5, 1428);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 2, 6, 861);

            INSERT INTO start_item (type, class, slot, item_id, ef1, efv1, ef2, efv2) VALUES ('equip', 3, 0, 31, 43, 0, 31, 1);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 3, 1, 1553);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 3, 2, 1569);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 3, 3, 1572);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 3, 4, 1575);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 3, 5, 1578);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('equip', 3, 6, 816);

            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 0, 0, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 0, 1, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 0, 2, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 0, 3, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 0, 4, 917);

            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 1, 0, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 1, 1, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 1, 2, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 1, 3, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 1, 4, 918);

            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 2, 0, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 2, 1, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 2, 2, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 2, 3, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 2, 4, 917);

            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 3, 0, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 3, 1, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 3, 2, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 3, 3, 400);
            INSERT INTO start_item (type, class, slot, item_id) VALUES ('inventory', 3, 4, 923);"
        ).await?;

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
